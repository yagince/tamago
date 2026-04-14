use chrono::Utc;

use crate::config::Config;
use crate::llm;
use crate::llm::message::build_chat_prompt;
use crate::pet::Category;
use crate::storage::{ActivityRecord, ChatFeedEntry, Storage};

/// チャット 1 回で入る経験値（ペットとの触れ合いボーナス）。
const CHAT_EXP: u64 = 50;

pub async fn run(storage: &Storage, message: &str) {
    let pet = match storage.load_pet() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("ペットが見つかりません。`tamago init` を先に実行してください。");
            std::process::exit(1);
        }
    };

    let config = Config::load(storage.base_dir());
    let mut generator = llm::create_generator(&config, storage);

    let (system, prompt) = build_chat_prompt(&pet, message);

    let reply = match generator.as_deref_mut() {
        Some(g) => g.generate(&prompt, &system, 120).await,
        None => None,
    };

    match reply {
        Some(msg) => {
            println!("{}: {msg}", pet.name);
            record_chat_activity(storage, message);
            push_to_tui(storage, &msg);
        }
        None => {
            eprintln!(
                "{} は今忙しいみたい…（LLM backend: {:?}）",
                pet.name, config.llm
            );
            std::process::exit(2);
        }
    }
}

fn push_to_tui(storage: &Storage, text: &str) {
    let entry = ChatFeedEntry {
        text: text.to_string(),
        ts: Utc::now(),
    };
    if let Err(e) = storage.append_chat_feed(&entry) {
        tracing::warn!("chat_feed への書き込みに失敗: {e}");
    }
}

fn record_chat_activity(storage: &Storage, message: &str) {
    let preview: String = message.chars().take(40).collect();
    let record = ActivityRecord {
        cmd: format!("chat: {preview}"),
        cat: Category::Ai,
        exp: CHAT_EXP,
        ts: Utc::now(),
    };
    if let Err(e) = storage.append_activity(&record) {
        tracing::warn!("chat activity の記録に失敗: {e}");
    }
}
