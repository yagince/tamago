//! LLM テキスト生成エンジン。
//! TextGenerator trait で Local LLM と Claude CLI を切り替え。

pub mod claude;
pub mod local;

use std::path::Path;

use crate::config::{Config, LlmBackend};

/// テキスト生成の共通インターフェース
pub trait TextGenerator: Send {
    fn generate(&mut self, prompt: &str, system: &str, max_tokens: usize) -> Option<String>;
}

/// config に基づいて適切な TextGenerator を生成
pub fn create_generator(config: &Config, model_dir: &Path) -> Option<Box<dyn TextGenerator>> {
    match config.llm {
        LlmBackend::Local => {
            let engine = local::LocalLlm::load(
                &local::model_path(model_dir),
                &local::tokenizer_path(model_dir),
            )
            .ok()?;
            Some(Box::new(engine))
        }
        LlmBackend::Claude => {
            let cli = claude::ClaudeCli::new();
            if cli.is_available() {
                Some(Box::new(cli))
            } else {
                eprintln!("Claude CLI が見つかりません。フォールバックモードで動作します");
                None
            }
        }
        LlmBackend::None => None,
    }
}

/// モデルのダウンロード（Local バックエンド用）
pub async fn download_model(model_dir: &Path) -> anyhow::Result<()> {
    local::download_model(model_dir).await
}
