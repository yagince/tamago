//! Claude CLI の呼び出しヘルパー。
//! timeout コマンドに依存せず、Rust 側でタイムアウトを制御する。

use std::process::{Command, Stdio};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
const DEFAULT_MODEL: &str = "sonnet";

pub struct ClaudeRequest {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub model: String,
    pub timeout: Duration,
    pub max_chars: usize,
}

impl Default for ClaudeRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            system_prompt: None,
            model: DEFAULT_MODEL.to_string(),
            timeout: DEFAULT_TIMEOUT,
            max_chars: 40,
        }
    }
}

impl ClaudeRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    pub fn system(mut self, s: impl Into<String>) -> Self {
        self.system_prompt = Some(s.into());
        self
    }

    #[allow(dead_code)]
    pub fn model(mut self, m: impl Into<String>) -> Self {
        self.model = m.into();
        self
    }

    pub fn max_chars(mut self, n: usize) -> Self {
        self.max_chars = n;
        self
    }

    /// Claude CLI を同期的に呼び出す。タイムアウトは Rust 側で制御。
    pub fn execute(&self) -> Option<String> {
        let mut args = vec!["-p".to_string(), self.prompt.clone()];
        args.extend(["--model".to_string(), self.model.clone()]);
        if let Some(ref sys) = self.system_prompt {
            args.extend(["--system-prompt".to_string(), sys.clone()]);
        }

        let mut child = Command::new("claude")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let start = std::time::Instant::now();

        // タイムアウト付きで完了を待つ
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

        if !msg.is_empty() && msg.chars().count() <= self.max_chars {
            Some(msg)
        } else if !msg.is_empty() {
            // max_chars 超えたら切り詰め
            Some(msg.chars().take(self.max_chars).collect())
        } else {
            None
        }
    }

    /// バックグラウンドスレッドで実行して結果を channel に送る
    pub fn execute_async(self, tx: std::sync::mpsc::Sender<String>) {
        std::thread::spawn(move || {
            if let Some(msg) = self.execute() {
                let _ = tx.send(msg);
            }
        });
    }
}

/// Claude CLI が利用可能か確認（起動時に1回だけ呼ぶ）
pub fn is_available() -> bool {
    Command::new("claude")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
