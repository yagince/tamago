//! LLM テキスト生成エンジン。
//! TextGenerator trait で Local LLM と Claude CLI を切り替え。

pub mod claude;
pub mod local;

use std::path::Path;

use async_trait::async_trait;

use crate::config::{Config, LlmBackend};

#[async_trait]
pub trait TextGenerator: Send {
    async fn generate(&mut self, prompt: &str, system: &str, max_tokens: usize) -> Option<String>;
}

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

pub async fn download_model(model_dir: &Path) -> anyhow::Result<()> {
    local::download_model(model_dir).await
}
