//! アプリケーション設定。
//! ~/.config/tamago/config.json で永続化。

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub llm: LlmBackend,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmBackend::Local,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmBackend {
    /// ローカル LLM (candle + Qwen2.5)
    Local,
    /// Claude CLI
    Claude,
    /// LLM なし（フォールバックのみ）
    None,
}

impl Default for LlmBackend {
    fn default() -> Self {
        Self::Local
    }
}

impl Config {
    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join("config.json");
        if let Ok(data) = std::fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, config_dir: &Path) -> std::io::Result<()> {
        let path = config_dir.join("config.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_is_local() {
        let config = Config::default();
        assert_eq!(config.llm, LlmBackend::Local);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let config = Config::load(dir.path());
        assert_eq!(config.llm, LlmBackend::Local);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let config = Config {
            llm: LlmBackend::Claude,
        };
        config.save(dir.path()).unwrap();
        let loaded = Config::load(dir.path());
        assert_eq!(loaded.llm, LlmBackend::Claude);
    }

    #[test]
    fn deserialize_none_variant() {
        let json = r#"{"llm": "none"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.llm, LlmBackend::None);
    }
}
