//! Claude CLI バックエンド。

use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;

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

    async fn execute(&self, prompt: &str, system: &str, max_chars: usize) -> Option<String> {
        let mut cmd = tokio::process::Command::new("claude");
        cmd.arg("-p")
            .arg(prompt)
            .arg("--model")
            .arg(&self.model)
            .arg("--system-prompt")
            .arg(system)
            .arg("--allowedTools")
            .arg("")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Claude CLI 起動失敗: {e}");
                return None;
            }
        };
        let output = match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Err(_) => {
                tracing::warn!("Claude CLI タイムアウト ({}s)", self.timeout.as_secs());
                return None;
            }
            Ok(Err(e)) => {
                tracing::error!("Claude CLI 実行エラー: {e}");
                return None;
            }
            Ok(Ok(o)) => o,
        };

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

#[async_trait]
impl TextGenerator for ClaudeCli {
    async fn generate(&mut self, prompt: &str, system: &str, max_tokens: usize) -> Option<String> {
        self.execute(prompt, system, max_tokens).await
    }
}
