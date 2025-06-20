use std::io::{BufRead, BufReader, Error, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

const STOCKFISH_PATH: &str = "engines/stockfish-windows-x86-64-avx2";
const ENGINE_THREADS: &str = "4";
const ENGINE_HASH: &str = "128";

pub struct StockfishEngineInternal {
    process: Child,
    writer: Arc<Mutex<std::process::ChildStdin>>,
    reader_thread: Option<thread::JoinHandle<()>>,
    output_buffer: Arc<Mutex<Vec<String>>>,
    running: Arc<Mutex<bool>>,
    cancel_search: Arc<AtomicBool>,
    current_best_move: Arc<Mutex<Vec<String>>>,
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

    pub fn find_best_move(&self, depth: Option<u8>, time_ms: Option<u64>, is_white_move: bool) -> Option<BestMoveInfo> {
        let mut go_cmd = String::from("go");

        if let Some(d) = depth {
            go_cmd.push_str(&format!(" depth {}", d));
        } else if let Some(t) = time_ms {
            go_cmd.push_str(&format!(" movetime {}", t));
        } else {
            go_cmd.push_str(" depth 16");
        }

        self.send_command(&go_cmd).unwrap();

        let timeout = time_ms.unwrap_or(30000) + 5000;
        let output = self.wait_for_response("bestmove", timeout).unwrap();

        let mut latest_score: Option<f32> = None;
        let mut latest_best_move: Option<Vec<String>> = None;

        for line in output.iter() {
            if line.contains("score cp ") {
                let parts: Vec<&str> = line.split("score cp ").collect();
                if parts.len() >= 2 {
                    let score_parts: Vec<&str> = parts[1].split_whitespace().collect();
                    if !score_parts.is_empty() {
                        if let Ok(score) = score_parts[0].parse::<i32>() {
                            latest_score = if is_white_move { Some(score as f32) } else { Some(-score as f32) };
                        }
                    }
                }
            } else if line.contains("score mate ") {
                let parts: Vec<&str> = line.split("score mate ").collect();
                if parts.len() >= 2 {
                    let score_parts: Vec<&str> = parts[1].split_whitespace().collect();
                    if !score_parts.is_empty() {
                        if let Ok(moves) = score_parts[0].parse::<i32>() {
                            if moves > 0 {
                                latest_score = Some(1000.0);
                            } else {
                                latest_score = Some(-1000.0);
                            }
                        }
                    }
                }
            }
        }

        for line in output.iter().rev() {
            if line.contains("bestmove") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let best_move = parts[1];
                    if best_move.contains("(none)") {
                        latest_best_move = None;
                    }

                    let from = &best_move[0..2];
                    let to = &best_move[2..4];
                    latest_best_move = Some(vec![from.to_string(), to.to_string()]);
                }
            }
        }

        Some(BestMoveInfo::new(latest_best_move, latest_score))
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
}

pub struct BestMoveInfo {
    pub best_move: Option<Vec<String>>,
    pub evaluation_score: Option<f32>
}

impl BestMoveInfo {
    fn new(best_move: Option<Vec<String>>, evaluation_score: Option<f32>) -> Self {
        Self { best_move, evaluation_score }
    }
}

