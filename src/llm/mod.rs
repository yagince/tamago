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
        LlmBackend::Claude => Some(Box::new(claude::ClaudeCli::new())),
        LlmBackend::None => None,
    }
}

/// generator を借用して関数に渡すヘルパー
pub fn with_generator<T>(
    generator: &mut Option<Box<dyn TextGenerator>>,
    f: impl FnOnce(Option<&mut dyn TextGenerator>) -> T,
) -> T {
    match generator {
        Some(g) => f(Some(&mut **g)),
        None => f(None),
    }
}

/// モデルのダウンロード（Local バックエンド用）
pub async fn download_model(model_dir: &Path) -> anyhow::Result<()> {
    local::download_model(model_dir).await
}
