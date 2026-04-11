//! ローカル LLM 推論エンジン。
//! candle + GGUF で Gemma 3 等のモデルを CPU 推論する。

use std::path::{Path, PathBuf};

use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_gemma3::ModelWeights;
use tokenizers::models::unigram::Unigram;
use tokenizers::Tokenizer;

const DEFAULT_REPO_ID: &str = "bartowski/google_gemma-3-1b-it-GGUF";
const DEFAULT_FILENAME: &str = "google_gemma-3-1b-it-Q4_K_M.gguf";

const TEMPERATURE: f64 = 0.8;
const TOP_P: f64 = 0.9;

/// ローカル LLM エンジン
pub struct LlmEngine {
    model: ModelWeights,
    tokenizer: Tokenizer,
    device: Device,
    stop_token_ids: Vec<u32>,
}

impl LlmEngine {
    /// GGUF ファイルからモデルとトークナイザーを一括ロード
    pub fn load_from_gguf(gguf_path: &Path) -> anyhow::Result<Self> {
        let device = Device::Cpu;

        let mut file = std::fs::File::open(gguf_path)?;
        let content = gguf_file::Content::read(&mut file)?;

        // GGUF 内蔵トークナイザーを構築
        let (tokenizer, stop_token_ids) = build_tokenizer_from_gguf(&content)?;

        let model = ModelWeights::from_gguf(content, &mut file, &device)?;

        Ok(Self {
            model,
            tokenizer,
            device,
            stop_token_ids,
        })
    }

    /// テキスト生成（同期、CPU バウンド）
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

        let mut generated_ids: Vec<u32> = Vec::new();

        // 最初のトークン
        let logits = logits.squeeze(0).ok()?;
        let next_token = sampler.sample(&logits).ok()?;
        if self.stop_token_ids.contains(&next_token) {
            return None;
        }
        generated_ids.push(next_token);

        // Auto-regressive 生成
        let mut prev_token = next_token;
        for step in 1..max_tokens {
            let input = Tensor::new(&[prev_token], &self.device)
                .ok()?
                .unsqueeze(0)
                .ok()?;
            let logits = self.model.forward(&input, prompt_len + step).ok()?;
            let logits = logits.squeeze(0).ok()?;
            let next_token = sampler.sample(&logits).ok()?;

            if self.stop_token_ids.contains(&next_token) {
                break;
            }
            generated_ids.push(next_token);
            prev_token = next_token;
        }

        if generated_ids.is_empty() {
            return None;
        }

        let text = self
            .tokenizer
            .decode(&generated_ids, true)
            .ok()?;

        // 特殊トークン文字列を除去
        let text = text
            .replace("<end_of_turn>", "")
            .replace("<start_of_turn>", "")
            .replace("<eos>", "")
            .replace("<bos>", "")
            .trim()
            .to_string();

        if text.is_empty() { None } else { Some(text) }
    }
}

/// GGUF メタデータの tokenizer.ggml.* からトークナイザーを構築
fn build_tokenizer_from_gguf(
    content: &gguf_file::Content,
) -> anyhow::Result<(Tokenizer, Vec<u32>)> {
    // トークン文字列リスト
    let tokens = content
        .metadata
        .get("tokenizer.ggml.tokens")
        .ok_or_else(|| anyhow::anyhow!("tokenizer.ggml.tokens が見つかりません"))?;
    let tokens: Vec<String> = match tokens {
        gguf_file::Value::Array(arr) => arr
            .iter()
            .map(|v| match v {
                gguf_file::Value::String(s) => Ok(s.clone()),
                _ => Err(anyhow::anyhow!("token is not string")),
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        _ => anyhow::bail!("tokenizer.ggml.tokens is not an array"),
    };

    // スコアリスト
    let scores = content
        .metadata
        .get("tokenizer.ggml.scores")
        .ok_or_else(|| anyhow::anyhow!("tokenizer.ggml.scores が見つかりません"))?;
    let scores: Vec<f64> = match scores {
        gguf_file::Value::Array(arr) => arr
            .iter()
            .map(|v| match v {
                gguf_file::Value::F32(f) => Ok(*f as f64),
                _ => Err(anyhow::anyhow!("score is not f32")),
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        _ => anyhow::bail!("tokenizer.ggml.scores is not an array"),
    };

    // UNK token ID
    let unk_id = content
        .metadata
        .get("tokenizer.ggml.unknown_token_id")
        .and_then(|v| v.to_u32().ok())
        .map(|v| v as usize);

    // EOS token ID
    let eos_token_id = content
        .metadata
        .get("tokenizer.ggml.eos_token_id")
        .and_then(|v| v.to_u32().ok())
        .unwrap_or(1);

    // vocab: Vec<(String, f64)>
    let vocab: Vec<(String, f64)> = tokens
        .into_iter()
        .zip(scores)
        .collect();

    let unigram = Unigram::from(vocab, unk_id, true)
        .map_err(|e| anyhow::anyhow!("Unigram 構築エラー: {e}"))?;

    let tokenizer = Tokenizer::new(unigram);

    // EOS + <end_of_turn> 等のストップトークン
    let mut stop_ids = vec![eos_token_id];
    // <end_of_turn> のトークン ID を検索して追加
    if let Some(eot_id) = tokenizer.token_to_id("<end_of_turn>") {
        if !stop_ids.contains(&eot_id) {
            stop_ids.push(eot_id);
        }
    }

    Ok((tokenizer, stop_ids))
}

/// Gemma 3 のチャットテンプレート
fn format_prompt(system: &str, user: &str) -> String {
    format!("<start_of_turn>user\n{system}\n{user}<end_of_turn>\n<start_of_turn>model\n")
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

    // hf-hub はキャッシュにダウンロードするので、シンボリックリンクで参照
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

/// テスト用: デフォルトのモデルパスを返す（~/.config/tamago/models/）
#[cfg(test)]
pub fn default_model_dir() -> PathBuf {
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
    fn load_model_and_tokenizer() {
        let _engine = load_engine();
        // ロード成功 = GGUF 内蔵トークナイザー構築も成功
    }

    #[test]
    fn generate_short_japanese() {
        let mut engine = load_engine();
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
        let mut engine = load_engine();
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
        let mut engine = load_engine();
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
