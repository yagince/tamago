use clap::Subcommand;

use crate::config::{Config, LlmBackend};
use crate::storage::Storage;

#[derive(Subcommand)]
pub enum LlmCommand {
    /// 現在の LLM バックエンドを表示
    Show,
    /// ローカル LLM に切り替え (candle + Qwen2.5)
    Local,
    /// Claude CLI に切り替え
    Claude,
    /// LLM を無効化（フォールバックのみ）
    None,
    /// 推論に使うデバイス (GPU/CPU) を表示
    Device,
}

pub async fn run(storage: &Storage, command: &LlmCommand) {
    let mut config = Config::load(storage.base_dir());

    match command {
        LlmCommand::Show => {
            let backend = match config.llm {
                LlmBackend::Local => "local (candle + Qwen2.5)",
                LlmBackend::Claude => "claude (Claude CLI)",
                LlmBackend::None => "none (フォールバックのみ)",
            };
            println!("LLM バックエンド: {backend}");
        }
        LlmCommand::Local => {
            config.llm = LlmBackend::Local;
            save_and_print(storage, &config, "local");

            // モデルがなければダウンロード
            let model_dir = storage.model_dir();
            if let Err(e) = crate::llm::download_model(&model_dir).await {
                eprintln!("モデルのダウンロードに失敗: {e}");
            }
        }
        LlmCommand::Claude => {
            config.llm = LlmBackend::Claude;
            save_and_print(storage, &config, "claude");
        }
        LlmCommand::None => {
            config.llm = LlmBackend::None;
            save_and_print(storage, &config, "none");
        }
        LlmCommand::Device => {
            let info = crate::llm::local::device_info();
            let features = if info.compiled_features.is_empty() {
                "(なし)".to_string()
            } else {
                info.compiled_features.join(", ")
            };
            println!("コンパイル時 GPU feature: {features}");
            println!("推論デバイス (ランタイム): {}", info.runtime);
        }
    }
}

fn save_and_print(storage: &Storage, config: &Config, name: &str) {
    storage
        .ensure_dir()
        .expect("ディレクトリの作成に失敗しました");
    match config.save(storage.base_dir()) {
        Ok(()) => println!("LLM バックエンドを {name} に変更しました"),
        Err(e) => eprintln!("設定の保存に失敗: {e}"),
    }
}
