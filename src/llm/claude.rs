//! Claude CLI バックエンド。
//! `claude` コマンドを非同期プロセスとして呼び出す。

use std::process::Stdio;
use std::time::Duration;

use super::TextGenerator;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
const DEFAULT_MODEL: &str = "sonnet";

pub struct ClaudeCli {
    model: String,
    timeout: Duration,
}

impl ClaudeCli {
    pub fn new() -> Self {
        Self {
            model: DEFAULT_MODEL.to_string(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    fn execute(&self, prompt: &str, system: &str, max_chars: usize) -> Option<String> {
        let args = vec![
            "-p".to_string(),
            prompt.to_string(),
            "--model".to_string(),
            self.model.clone(),
            "--system-prompt".to_string(),
            system.to_string(),
        ];

        let child = std::process::Command::new("claude")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let start = std::time::Instant::now();
        let mut child = child;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if start.elapsed() >= self.timeout {
                        let _ = child.kill();
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(_) => break,
            }
        }

        let output = child.wait_with_output().ok()?;
        let msg = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if msg.is_empty() {
            None
        } else if msg.chars().count() <= max_chars {
            Some(msg)
        } else {
            Some(msg.chars().take(max_chars).collect())
        }
    }
}

impl TextGenerator for ClaudeCli {
    fn generate(&mut self, prompt: &str, system: &str, max_tokens: usize) -> Option<String> {
        self.execute(prompt, system, max_tokens)
    }
}
