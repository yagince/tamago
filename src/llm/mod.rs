//! LLM テキスト生成エンジン。
//! TextGenerator trait で Local LLM と Claude CLI を切り替え。

#[cfg_attr(test, allow(dead_code))]
pub mod claude;
pub mod local;
pub mod message;

use std::path::Path;

use async_trait::async_trait;

use crate::config::Config;
#[cfg(not(test))]
use crate::config::LlmBackend;
use crate::storage::Storage;

#[async_trait]
pub trait TextGenerator: Send {
    async fn generate(&mut self, prompt: &str, system: &str, max_tokens: usize) -> Option<String>;
}

pub fn create_generator(config: &Config, storage: &Storage) -> Option<Box<dyn TextGenerator>> {
    // テスト実行時は実 LLM を起動しない（Claude CLI の 15 秒 timeout 待ちを回避）。
    #[cfg(test)]
    {
        let _ = (config, storage);
        None
    }
    #[cfg(not(test))]
    match config.llm {
        LlmBackend::Local => {
            let model_dir = storage.model_dir();
            match local::LocalLlm::load(
                &local::model_path(&model_dir),
                &local::tokenizer_path(&model_dir),
            ) {
                Ok(engine) => {
                    tracing::info!("LocalLLM ロード成功");
                    Some(Box::new(engine))
                }
                Err(e) => {
                    tracing::error!("LocalLLM ロード失敗: {e}");
                    None
                }
            }
        }
        LlmBackend::Claude => Some(Box::new(claude::ClaudeCli::new(storage.base_dir().clone()))),
        LlmBackend::None => None,
    }
}

pub async fn download_model(model_dir: &Path) -> anyhow::Result<()> {
    local::download_model(model_dir).await
}
