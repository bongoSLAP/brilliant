use std::io::{BufRead, BufReader, Error, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const STOCKFISH_PATH: &str = "engines/stockfish-windows-x86-64-avx2";
const ENGINE_THREADS: &str = "4";
const ENGINE_HASH: &str = "128";

#[derive(Clone, Debug)]
pub struct EngineUpdate {
    pub best_move: Option<Vec<String>>,
    pub evaluation: Option<f32>,
    pub depth: Option<u8>,
    pub is_final: bool,
}


pub struct StockfishEngineInternal {
    process: Child,
    writer: Arc<Mutex<std::process::ChildStdin>>,
    reader_thread: Option<thread::JoinHandle<()>>,
    output_buffer: Arc<Mutex<Vec<String>>>,
    running: Arc<Mutex<bool>>,
    cancel_search: Arc<AtomicBool>,
    current_best_move: Arc<Mutex<Option<Vec<String>>>>,
    current_evaluation: Arc<Mutex<Option<f32>>>,
}

impl StockfishEngineInternal {
    pub fn new(debug_mode: bool) -> Result<Self, Error> {
        let mut process = Command::new(STOCKFISH_PATH)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let writer = Arc::new(Mutex::new(process.stdin.take().unwrap()));
        let output_buffer = Arc::new(Mutex::new(Vec::new()));
        let running = Arc::new(Mutex::new(true));

        let reader_output_buffer = output_buffer.clone();
        let reader_running = running.clone();
        let stdout = process.stdout.take().unwrap();

        let reader_thread = thread::spawn(move || {
            let buf_reader = BufReader::new(stdout);
            for line in buf_reader.lines() {
                if let Ok(line) = line {
                    if !line.contains("info") || debug_mode {
                        println!("Engine output: {}", line);
                    }

                    if let Ok(mut buffer) = reader_output_buffer.lock() {
                        buffer.push(line);
                    }
                }

                if let Ok(is_running) = reader_running.lock() {
                    if !*is_running {
                        break;
                    }
                }
            }
        });

        let engine = StockfishEngineInternal {
            process,
            writer,
            reader_thread: Some(reader_thread),
            output_buffer,
            running,
            cancel_search: Arc::new(AtomicBool::new(false)),
            current_best_move: Arc::new(Mutex::new(None)),
            current_evaluation: Arc::new(Mutex::new(None)),
        };

        engine.send_command("uci")?;
        engine.wait_for_response("uciok", 5000)?;
        engine.set_option("Threads", ENGINE_THREADS)?;
        engine.set_option("Hash", ENGINE_HASH)?;
        engine.set_option("MultiPV", "1")?;

        engine.send_command("ucinewgame")?;
        engine.send_command("isready")?;
        engine.wait_for_response("readyok", 5000)?;

        engine.send_command("position startpos")?;
        engine.send_command("isready")?;
        engine.wait_for_response("readyok", 5000)?;

        Ok(engine)
    }

    pub fn send_command(&self, command: &str) -> Result<(), Error> {
        if let Ok(mut stdin) = self.writer.lock() {
            writeln!(stdin, "{}", command)?;
            stdin.flush()?;
        }
        Ok(())
    }

    pub fn get_output(&self) -> Vec<String> {
        let mut result = Vec::new();
        if let Ok(mut buffer) = self.output_buffer.lock() {
            result.append(&mut buffer);
        }
        result
    }

    pub fn wait_for_response(&self, response: &str, timeout_ms: u64) -> Result<Vec<String>, Error> {
        let start = std::time::Instant::now();
        let mut found = false;

        if let Ok(buffer) = self.output_buffer.lock() {
            if buffer.iter().any(|line| line.contains(response)) {
                found = true;
            }
        }

        while !found && start.elapsed().as_millis() < timeout_ms as u128 {
            if let Ok(buffer) = self.output_buffer.lock() {
                for line in buffer.iter() {
                    if line.contains(response) {
                        found = true;
                        break;
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        if !found {
            return Err(Error::new(
                std::io::ErrorKind::TimedOut,
                format!("Timeout waiting for '{}' response", response),
            ));
        }

        Ok(self.get_output())
    }

    pub fn find_best_move(&self, depth: Option<u8>, time_ms: Option<u64>, is_white_move: bool, update_sender: mpsc::Sender<EngineUpdate>) {
        self.cancel_search.store(false, Ordering::Relaxed);

        let mut go_cmd = String::from("go");
        if let Some(d) = depth {
            go_cmd.push_str(&format!(" depth {}", d));
        } else if let Some(t) = time_ms {
            go_cmd.push_str(&format!(" movetime {}", t));
        } else {
            go_cmd.push_str(" depth 20");
        }

        self.send_command(&go_cmd).unwrap();

        {
            *self.current_best_move.lock().unwrap() = None;
            *self.current_evaluation.lock().unwrap() = None;
        }

        {
            self.output_buffer.lock().unwrap().clear();
        }

        let output_buffer = self.output_buffer.clone();
        let cancel_search = self.cancel_search.clone();
        let current_best_move = self.current_best_move.clone();
        let current_evaluation = self.current_evaluation.clone();

        thread::spawn(move || {
            let mut last_sent_move: Option<Vec<String>> = None;
            let mut last_sent_eval: Option<f32> = None;
            let mut processed_lines = 0;

            loop {
                if cancel_search.load(Ordering::Relaxed) {
                    break;
                }

                thread::sleep(Duration::from_millis(100));

                let mut found_update = false;
                let mut current_move = None;
                let mut current_eval = None;
                let mut current_depth = None;
                let mut is_final = false;

                if let Ok(buffer) = output_buffer.lock() {
                    if processed_lines > buffer.len() {
                        processed_lines = 0;
                    }

                    let new_lines = if processed_lines < buffer.len() {
                        &buffer[processed_lines..]
                    } else {
                        &[]
                    };

                    for line in new_lines.iter() {
                        if cancel_search.load(Ordering::Relaxed) {
                            return;
                        }

                        if line.contains("info depth") && line.contains("score") && line.contains("pv ") {
                            if let Some(depth_start) = line.find("depth ") {
                                let depth_str = &line[depth_start + 6..];
                                if let Some(space_pos) = depth_str.find(' ') {
                                    if let Ok(d) = depth_str[..space_pos].parse::<u8>() {
                                        current_depth = Some(d);
                                    }
                                }
                            }

                            if line.contains("score cp ") {
                                let parts: Vec<&str> = line.split("score cp ").collect();
                                if parts.len() >= 2 {
                                    let score_parts: Vec<&str> = parts[1].split_whitespace().collect();
                                    if !score_parts.is_empty() {
                                        if let Ok(score) = score_parts[0].parse::<i32>() {
                                            let adjusted_score = if is_white_move { score } else { -score };
                                            current_eval = Some(adjusted_score as f32);
                                        }
                                    }
                                }
                            } else if line.contains("score mate ") {
                                let parts: Vec<&str> = line.split("score mate ").collect();
                                if parts.len() >= 2 {
                                    let score_parts: Vec<&str> = parts[1].split_whitespace().collect();
                                    if !score_parts.is_empty() {
                                        if let Ok(moves) = score_parts[0].parse::<i32>() {
                                            let mate_score = if moves > 0 { 1000.0 } else { -1000.0 };
                                            current_eval = Some(if is_white_move { mate_score } else { -mate_score });
                                        }
                                    }
                                }
                            }

                            if let Some(pv_start) = line.find(" pv ") {
                                let pv_str = &line[pv_start + 3..];
                                let moves: Vec<&str> = pv_str.split_whitespace().collect();

                                println!("PV line: {}", pv_str);

                                if !moves.is_empty() {
                                    let best_move = moves[0];
                                    println!("Extracted best move: '{}'", best_move);

                                    if best_move.len() >= 4 && best_move.len() <= 5 {
                                        let from = &best_move[0..2];
                                        let to = &best_move[2..4];
                                        current_move = Some(vec![from.to_string(), to.to_string()]);
                                        println!("Valid move parsed: {} -> {}", from, to);

                                    } else {
                                        println!("Invalid move length: '{}' (len: {})", best_move, best_move.len());
                                    }
                                }
                            }

                            found_update = true;
                        }

                        if line.contains("bestmove") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let best_move = parts[1];
                                if !best_move.contains("(none)") && best_move.len() >= 4 {
                                    let from = &best_move[0..2];
                                    let to = &best_move[2..4];
                                    current_move = Some(vec![from.to_string(), to.to_string()]);
                                    is_final = true;
                                    found_update = true;
                                }
                            }
                            break;
                        }
                    }

                    processed_lines = buffer.len();
                }

                if found_update {
                    let move_changed = current_move != last_sent_move;
                    let eval_changed = current_eval != last_sent_eval;
                    let depth_changed = current_depth.is_some();

                    if move_changed || eval_changed || depth_changed || is_final {
                        if let Some(ref mv) = current_move {
                            *current_best_move.lock().unwrap() = Some(mv.clone());
                        }
                        if let Some(eval) = current_eval {
                            *current_evaluation.lock().unwrap() = Some(eval);
                        }

                        let update = EngineUpdate {
                            best_move: current_move.clone(),
                            evaluation: current_eval,
                            depth: current_depth,
                            is_final,
                        };

                        println!("Sending update: move={:?}, eval={:?}, depth={:?}, final={}",
                                 update.best_move, update.evaluation, update.depth, update.is_final);

                        if update_sender.send(update).is_err() {
                            println!("Failed to send update - receiver dropped");
                            break;
                        }

                        last_sent_move = current_move.clone();
                        last_sent_eval = current_eval;

                        if is_final {
                            break;
                        }
                    }
                }
            }
        });
    }

    pub fn cancel_search(&self) {
        self.cancel_search.store(true, Ordering::Relaxed);
        let _ = self.send_command("stop");
    }

    pub fn set_position(&self, position: &str) -> Result<(), Error> {
        self.send_command(&format!("position fen {}", position))
    }

    pub fn set_option(&self, name: &str, value: &str) -> Result<(), Error> {
        self.send_command(&format!("setoption name {} value {}", name, value))
    }
}

impl Drop for StockfishEngineInternal {
    fn drop(&mut self) {
        if let Ok(mut is_running) = self.running.lock() {
            *is_running = false;
        }

        let _ = self.send_command("quit");

        if let Some(thread) = self.reader_thread.take() {
            let _ = thread.join();
        }

        let _ = self.process.kill();
    }
}

#[derive(Clone)]
pub struct StockfishEngine {
    pub(crate) internal: Arc<Mutex<StockfishEngineInternal>>,
}

impl StockfishEngine {
    pub fn new(debug_mode: bool) -> Self {
        let engine_internal = StockfishEngineInternal::new(debug_mode).unwrap();
        let arc_mutex_internal = Arc::new(Mutex::new(engine_internal));

        Self { internal: arc_mutex_internal }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<StockfishEngineInternal> {
        self.internal.lock().unwrap()
    }

    pub fn cancel_search(&self) {
        let engine = self.lock();
        engine.cancel_search();
    }
}


