//! Claude CLI の呼び出しヘルパー。
//! tokio::process で非同期実行し、tokio::time::timeout でタイムアウトを制御する。

use std::process::Stdio;
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

    /// Claude CLI を非同期に呼び出す。タイムアウトは tokio::time::timeout で制御。
    pub async fn execute(&self) -> Option<String> {
        let mut args = vec!["-p".to_string(), self.prompt.clone()];
        args.extend(["--model".to_string(), self.model.clone()]);
        if let Some(ref sys) = self.system_prompt {
            args.extend(["--system-prompt".to_string(), sys.clone()]);
        }

        let child = tokio::process::Command::new("claude")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let output = match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            _ => return None,
        };

        let msg = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if !msg.is_empty() && msg.chars().count() <= self.max_chars {
            Some(msg)
        } else if !msg.is_empty() {
            Some(msg.chars().take(self.max_chars).collect())
        } else {
            None
        }
    }
}

/// Claude CLI が利用可能か確認（起動時に1回だけ呼ぶ）
pub async fn is_available() -> bool {
    tokio::process::Command::new("claude")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}
