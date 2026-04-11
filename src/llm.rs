//! ローカル LLM 推論エンジン。
//! llama-cpp-2 で GGUF モデルを CPU/GPU 推論する。

use std::path::{Path, PathBuf};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::sampling::LlamaSampler;

const DEFAULT_REPO_ID: &str = "bartowski/google_gemma-3-1b-it-GGUF";
const DEFAULT_FILENAME: &str = "google_gemma-3-1b-it-Q4_K_M.gguf";

const TEMPERATURE: f32 = 0.8;
const TOP_P: f32 = 0.9;
const TOP_K: i32 = 40;

/// ローカル LLM エンジン
pub struct LlmEngine {
    #[allow(dead_code)]
    backend: LlamaBackend,
    model: LlamaModel,
}

impl LlmEngine {
    /// GGUF ファイルからモデルをロード
    pub fn load_from_gguf(gguf_path: &Path) -> anyhow::Result<Self> {
        let backend = LlamaBackend::init()?;
        let params = LlamaModelParams::default();
        let model = LlamaModel::load_from_file(&backend, gguf_path, &params)?;

        Ok(Self { backend, model })
    }

    /// テキスト生成（同期、CPU/GPU バウンド）
    pub fn generate(&self, prompt: &str, system: &str, max_tokens: usize) -> Option<String> {
        let formatted = format_prompt(system, prompt);
        self.generate_raw(&formatted, max_tokens)
    }

    fn generate_raw(&self, prompt: &str, max_tokens: usize) -> Option<String> {
        let tokens = self
            .model
            .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
            .ok()?;

        let ctx_params = LlamaContextParams::default().with_n_ctx(std::num::NonZeroU32::new(
            (tokens.len() + max_tokens + 16) as u32,
        ));
        let mut ctx = self.model.new_context(&self.backend, ctx_params).ok()?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::top_k(TOP_K),
            LlamaSampler::top_p(TOP_P, 1),
            LlamaSampler::temp(TEMPERATURE),
            LlamaSampler::dist(rand_seed()),
        ]);

        // Prefill
        let mut batch = LlamaBatch::new(tokens.len(), 1);
        for (i, &token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch.add(token, i as i32, &[0], is_last).ok()?;
        }
        ctx.decode(&mut batch).ok()?;

        // Auto-regressive 生成
        let mut generated: Vec<u8> = Vec::new();
        let mut n_cur = tokens.len();

        for _ in 0..max_tokens {
            let token = sampler.sample(&ctx, -1);
            sampler.accept(token);

            if self.model.is_eog_token(token) {
                break;
            }

            let bytes = self
                .model
                .token_to_piece_bytes(token, 64, true, None)
                .ok()?;
            generated.extend_from_slice(&bytes);

            batch.clear();
            batch.add(token, n_cur as i32, &[0], true).ok()?;
            ctx.decode(&mut batch).ok()?;
            n_cur += 1;
        }

        let text = String::from_utf8_lossy(&generated)
            .replace("<end_of_turn>", "")
            .replace("<start_of_turn>", "")
            .trim()
            .to_string();

        if text.is_empty() { None } else { Some(text) }
    }
}

/// Gemma 3 のチャットテンプレート
fn format_prompt(system: &str, user: &str) -> String {
    format!("<start_of_turn>user\n{system}\n{user}<end_of_turn>\n<start_of_turn>model\n")
}

fn rand_seed() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}

/// モデルファイルのパス
pub fn model_path(model_dir: &Path) -> PathBuf {
    model_dir.join(DEFAULT_FILENAME)
}

/// HuggingFace からモデルをダウンロード
pub async fn download_model(model_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(model_dir)?;

    let model_dest = model_path(model_dir);
    if model_dest.exists() {
        return Ok(());
    }

    eprintln!("📦 AI モデルをダウンロード中... ({DEFAULT_REPO_ID}/{DEFAULT_FILENAME})");

    let api = hf_hub::api::tokio::Api::new()?;
    let repo = api.model(DEFAULT_REPO_ID.to_string());
    let downloaded = repo.get(DEFAULT_FILENAME).await?;

    if downloaded != model_dest {
        #[cfg(unix)]
        {
            if model_dest.exists() {
                std::fs::remove_file(&model_dest)?;
            }
            std::os::unix::fs::symlink(&downloaded, &model_dest)?;
        }
        #[cfg(not(unix))]
        {
            std::fs::copy(&downloaded, &model_dest)?;
        }
    }

    eprintln!("✓ モデルの準備完了");
    Ok(())
}

#[cfg(test)]
fn default_model_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("home directory not found")
                .join(".config")
        });
    base.join("tamago").join("models")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_engine() -> LlmEngine {
        let dir = default_model_dir();
        let path = model_path(&dir);
        if !path.exists() {
            panic!(
                "テスト用モデルが見つかりません: {}\n\
                `cargo run -- init` でモデルをダウンロードしてください",
                path.display()
            );
        }
        LlmEngine::load_from_gguf(&path).expect("モデルのロードに失敗しました")
    }

    #[test]
    fn load_model() {
        let _engine = load_engine();
    }

    #[test]
    fn generate_short_japanese() {
        let engine = load_engine();
        let result = engine.generate(
            "一言挨拶して。",
            "あなたはターミナルペットです。短いセリフだけ出力してください。",
            20,
        );
        assert!(result.is_some(), "生成結果が None");
        let text = result.unwrap();
        assert!(!text.is_empty(), "空文字列が返された");
        println!("生成結果: {text}");
    }

    #[test]
    fn generate_pet_personality() {
        let engine = load_engine();
        let result = engine.generate(
            "名前:ピカドン Lv.10 開発力:5 賢さ:3 おもしろさ:8 カオスさ:2\nこのペットの性格を30文字以内で。",
            "あなたはターミナルペットの性格設定を生成するAIです。性格テキストだけを出力してください。",
            30,
        );
        assert!(result.is_some(), "性格生成が None");
        let text = result.unwrap();
        assert!(!text.is_empty(), "空文字列が返された");
        println!("性格: {text}");
    }

    #[test]
    fn generate_pet_name() {
        let engine = load_engine();
        let result = engine.generate(
            "ターミナルペットの名前を1つだけ考えて。ポケモンっぽいカタカナの名前で、名前だけを出力して。",
            "あなたはターミナルペットの名前を考えるAIです。名前だけを出力してください。",
            10,
        );
        assert!(result.is_some(), "名前生成が None");
        let text = result.unwrap();
        assert!(!text.is_empty(), "空文字列が返された");
        println!("名前: {text}");
    }
}
