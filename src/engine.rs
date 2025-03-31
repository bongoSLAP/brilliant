use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const STOCKFISH_PATH: &str = "engines/stockfish-windows-x86-64-avx2";
const ENGINE_THREADS: &str = "4";
const ENGINE_HASH: &str = "128";

pub struct StockfishEngine {
    process: Child,
    writer: Arc<Mutex<std::process::ChildStdin>>,
    reader_thread: Option<thread::JoinHandle<()>>,
    output_buffer: Arc<Mutex<Vec<String>>>,
    running: Arc<Mutex<bool>>,
}

impl StockfishEngine {
    pub fn new() -> Result<Self, std::io::Error> {
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
                    if !line.contains("info") {
                        println!("Engine output: {}", line);
                        if let Ok(mut buffer) = reader_output_buffer.lock() {
                            buffer.push(line);
                        }
                    }
                }

                if let Ok(is_running) = reader_running.lock() {
                    if !*is_running {
                        break;
                    }
                }
            }
        });

        let engine = StockfishEngine {
            process,
            writer,
            reader_thread: Some(reader_thread),
            output_buffer,
            running,
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

    pub fn send_command(&self, command: &str) -> Result<(), std::io::Error> {
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

    pub fn wait_for_response(&self, response: &str, timeout_ms: u64) -> Result<Vec<String>, std::io::Error> {
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
                        println!("Found response: {}", line);
                        found = true;
                        break;
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        if !found {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("Timeout waiting for '{}' response", response),
            ));
        }

        Ok(self.get_output())
    }

    pub fn set_position(&self, position: &str) -> Result<(), std::io::Error> {
        self.send_command(&format!("position fen {}", position))
    }

    pub fn find_best_move(&self, depth: Option<u8>, time_ms: Option<u64>) -> Result<Vec<String>, std::io::Error> {
        let mut go_cmd = String::from("go");

        if let Some(d) = depth {
            go_cmd.push_str(&format!(" depth {}", d));
        } else if let Some(t) = time_ms {
            go_cmd.push_str(&format!(" movetime {}", t));
        } else {
            go_cmd.push_str(" depth 15");
        }

        self.send_command(&go_cmd)?;

        let timeout = time_ms.unwrap_or(30000) + 5000;
        let output = self.wait_for_response("bestmove", timeout)?;

        for line in output.iter().rev() {
            if line.contains("bestmove") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let best_move = parts[1];
                    let (to, from) = best_move.split_at(2);
                    return Ok(vec![to.to_string(), from.to_string()]);
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Best move not found in engine output",
        ))
    }

    pub fn set_option(&self, name: &str, value: &str) -> Result<(), std::io::Error> {
        self.send_command(&format!("setoption name {} value {}", name, value))
    }

    pub fn get_evaluation(&self, depth: u8) -> Result<f32, std::io::Error> {
        self.send_command(&format!("go depth {}", depth))?;
        let output = self.wait_for_response("bestmove", 30000)?;

        for line in output.iter() {
            if line.contains("score cp ") {
                let parts: Vec<&str> = line.split("score cp ").collect();
                if parts.len() >= 2 {
                    let score_parts: Vec<&str> = parts[1].split_whitespace().collect();
                    if !score_parts.is_empty() {
                        if let Ok(score) = score_parts[0].parse::<i32>() {
                            return Ok(score as f32 / 100.0);
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
                                return Ok(1000.0);
                            } else {
                                return Ok(-1000.0);
                            }
                        }
                    }
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Evaluation not found in engine output",
        ))
    }
}

impl Drop for StockfishEngine {
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
