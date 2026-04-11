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
        use std::io::Read;

        let args = vec![
            "-p".to_string(),
            prompt.to_string(),
            "--model".to_string(),
            self.model.clone(),
            "--system-prompt".to_string(),
            system.to_string(),
        ];

        let mut child = std::process::Command::new("claude")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        // stdout を先に取り出して非同期で読む
        let mut stdout = child.stdout.take()?;
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let handle = std::thread::spawn(move || {
            let mut buf = String::new();
            let _ = stdout.read_to_string(&mut buf);
            let _ = done_tx.send(());
            buf
        });

        // タイムアウト待ち
        match done_rx.recv_timeout(self.timeout) {
            Ok(()) => {}
            Err(_) => {
                let _ = child.kill();
            }
        }
        let _ = child.wait();

        let msg = handle.join().ok()?.trim().to_string();

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
