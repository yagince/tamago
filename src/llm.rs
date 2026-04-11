//! ローカル LLM 推論エンジン。
//! candle + quantized_qwen2 で GGUF モデルを推論する。
//! GPU は optional features (metal/cuda) でコンパイル時に有効化し、
//! ランタイムで利用可否を検出して CPU にフォールバック。

use std::path::{Path, PathBuf};

use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_qwen2::ModelWeights;
use candle_transformers::utils::apply_repeat_penalty;
use tokenizers::Tokenizer;

const GGUF_REPO: &str = "Qwen/Qwen2.5-1.5B-Instruct-GGUF";
const GGUF_FILE: &str = "qwen2.5-1.5b-instruct-q4_k_m.gguf";
const TOKENIZER_REPO: &str = "Qwen/Qwen2.5-1.5B-Instruct";

const TEMPERATURE: f64 = 0.7;
const TOP_P: f64 = 0.9;
const REPEAT_PENALTY: f32 = 1.3;
const REPEAT_LAST_N: usize = 64;

/// ローカル LLM エンジン
pub struct LlmEngine {
    model: ModelWeights,
    tokenizer: Tokenizer,
    device: Device,
    eos_token_ids: Vec<u32>,
}

impl LlmEngine {
    /// GGUF モデル + tokenizer.json からロード
    pub fn load(model_path: &Path, tokenizer_path: &Path) -> anyhow::Result<Self> {
        let device = select_device();

        let mut file = std::fs::File::open(model_path)?;
        let content = gguf_file::Content::read(&mut file)?;
        let model = ModelWeights::from_gguf(content, &mut file, &device)?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("tokenizer load error: {e}"))?;

        // Qwen2.5 の EOS トークン
        let mut eos_token_ids = Vec::new();
        for special in ["<|im_end|>", "<|endoftext|>"] {
            if let Some(id) = tokenizer.token_to_id(special) {
                eos_token_ids.push(id);
            }
        }

        Ok(Self {
            model,
            tokenizer,
            device,
            eos_token_ids,
        })
    }

    /// テキスト生成（同期）
    pub fn generate(&mut self, prompt: &str, system: &str, max_tokens: usize) -> Option<String> {
        let formatted = format_prompt(system, prompt);
        self.generate_raw(&formatted, max_tokens)
    }

    fn generate_raw(&mut self, prompt: &str, max_tokens: usize) -> Option<String> {
        let encoding = self.tokenizer.encode(prompt, false).ok()?;
        let token_ids: Vec<u32> = encoding.get_ids().to_vec();
        let prompt_len = token_ids.len();

        // Prefill
        let input = Tensor::new(token_ids.as_slice(), &self.device)
            .ok()?
            .unsqueeze(0)
            .ok()?;
        let logits = self.model.forward(&input, 0).ok()?;

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let mut sampler = LogitsProcessor::new(seed, Some(TEMPERATURE), Some(TOP_P));

        // 最初のトークン
        let logits = logits.squeeze(0).ok()?;
        let logits = apply_repeat_penalty(&logits, REPEAT_PENALTY, &token_ids).ok()?;
        let next_token = sampler.sample(&logits).ok()?;
        if self.eos_token_ids.contains(&next_token) {
            return None;
        }

        let mut all_tokens = token_ids;
        all_tokens.push(next_token);
        let mut generated_ids = vec![next_token];

        // Auto-regressive 生成
        for step in 1..max_tokens {
            let input = Tensor::new(&[*all_tokens.last().unwrap()], &self.device)
                .ok()?
                .unsqueeze(0)
                .ok()?;
            let logits = self.model.forward(&input, prompt_len + step).ok()?;
            let logits = logits.squeeze(0).ok()?;
            let penalty_context = if all_tokens.len() > REPEAT_LAST_N {
                &all_tokens[all_tokens.len() - REPEAT_LAST_N..]
            } else {
                &all_tokens
            };
            let logits = apply_repeat_penalty(&logits, REPEAT_PENALTY, penalty_context).ok()?;
            let next_token = sampler.sample(&logits).ok()?;

            if self.eos_token_ids.contains(&next_token) {
                break;
            }
            all_tokens.push(next_token);
            generated_ids.push(next_token);
        }

        if generated_ids.is_empty() {
            return None;
        }

        let text = self
            .tokenizer
            .decode(&generated_ids, true)
            .ok()?
            .trim()
            .to_string();

        if text.is_empty() { None } else { Some(text) }
    }
}

/// GPU デバイス選択（コンパイル時 feature + ランタイム検出）
fn select_device() -> Device {
    #[cfg(feature = "metal")]
    if let Ok(device) = Device::new_metal(0) {
        return device;
    }
    #[cfg(feature = "cuda")]
    if let Ok(device) = Device::new_cuda(0) {
        return device;
    }
    Device::Cpu
}

/// Qwen2.5 のチャットテンプレート (ChatML)
fn format_prompt(system: &str, user: &str) -> String {
    format!(
        "<|im_start|>system\n{system}<|im_end|>\n\
         <|im_start|>user\n{user}<|im_end|>\n\
         <|im_start|>assistant\n"
    )
}

/// モデルファイルのパス
pub fn model_path(model_dir: &Path) -> PathBuf {
    model_dir.join(GGUF_FILE)
}

/// トークナイザーファイルのパス
pub fn tokenizer_path(model_dir: &Path) -> PathBuf {
    model_dir.join("tokenizer.json")
}

/// HuggingFace からモデルとトークナイザーをダウンロード
pub async fn download_model(model_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(model_dir)?;
    let api = hf_hub::api::tokio::Api::new()?;

    // GGUF モデル
    let model_dest = model_path(model_dir);
    if !model_dest.exists() {
        eprintln!("📦 AI モデルをダウンロード中... ({GGUF_REPO}/{GGUF_FILE})");
        let repo = api.model(GGUF_REPO.to_string());
        let downloaded = repo.get(GGUF_FILE).await?;
        symlink_or_copy(&downloaded, &model_dest)?;
        eprintln!("✓ モデルの準備完了");
    }

    // トークナイザー
    let tok_dest = tokenizer_path(model_dir);
    if !tok_dest.exists() {
        eprintln!("📦 トークナイザーをダウンロード中...");
        let repo = api.model(TOKENIZER_REPO.to_string());
        let downloaded = repo.get("tokenizer.json").await?;
        symlink_or_copy(&downloaded, &tok_dest)?;
        eprintln!("✓ トークナイザーの準備完了");
    }

    Ok(())
}

fn symlink_or_copy(src: &Path, dest: &Path) -> anyhow::Result<()> {
    if src == dest {
        return Ok(());
    }
    if dest.exists() {
        std::fs::remove_file(dest)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(src, dest)?;
    #[cfg(not(unix))]
    std::fs::copy(src, dest)?;
    Ok(())
}

/// テスト用: hf-hub キャッシュから直接モデルをロード（init 不要）
/// テスト用は軽量な 0.5B モデルを使用
#[cfg(test)]
const TEST_GGUF_REPO: &str = "Qwen/Qwen2.5-0.5B-Instruct-GGUF";
#[cfg(test)]
const TEST_GGUF_FILE: &str = "qwen2.5-0.5b-instruct-q4_k_m.gguf";
#[cfg(test)]
const TEST_TOKENIZER_REPO: &str = "Qwen/Qwen2.5-0.5B-Instruct";

#[cfg(test)]
fn ensure_test_model() -> (PathBuf, PathBuf) {
    use hf_hub::api::sync::Api;
    let api = Api::new().expect("HF API の初期化に失敗");

    let gguf = api
        .model(TEST_GGUF_REPO.to_string())
        .get(TEST_GGUF_FILE)
        .expect("GGUF モデルの取得に失敗。ネットワーク接続を確認してください");

    let tok = api
        .model(TEST_TOKENIZER_REPO.to_string())
        .get("tokenizer.json")
        .expect("トークナイザーの取得に失敗");

    (gguf, tok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn engine() -> &'static Mutex<LlmEngine> {
        static ENGINE: OnceLock<Mutex<LlmEngine>> = OnceLock::new();
        ENGINE.get_or_init(|| {
            let (model_path, tok_path) = ensure_test_model();
            Mutex::new(LlmEngine::load(&model_path, &tok_path).expect("モデルのロードに失敗"))
        })
    }

    #[test]
    #[ignore] // LLM テスト: cargo test -- --ignored (--features metal 推奨)
    fn load_model() {
        let _engine = engine().lock().unwrap();
    }

    #[test]
    #[ignore]
    fn generate_short_japanese() {
        let mut engine = engine().lock().unwrap();
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
    #[ignore]
    fn generate_pet_personality() {
        let mut engine = engine().lock().unwrap();
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
    #[ignore]
    fn generate_pet_name() {
        let mut engine = engine().lock().unwrap();
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
