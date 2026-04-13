use chrono::Utc;

use crate::config::{Config, LlmBackend};
use crate::llm;
use crate::pet::PetState;
use crate::storage::Storage;

pub async fn run(storage: &Storage) {
    if storage.pet_exists() {
        eprintln!("すでにペットが存在します。");
        std::process::exit(1);
    }

    storage
        .ensure_dir()
        .expect("ディレクトリの作成に失敗しました");

    let config = Config::load(storage.base_dir());

    if config.llm == LlmBackend::Local {
        let model_dir = storage.model_dir();
        if let Err(e) = llm::download_model(&model_dir).await {
            eprintln!("モデルのダウンロードに失敗: {e}");
            eprintln!("オフラインモードで続行します");
        }
    }

    let mut generator = llm::create_generator(&config, &storage.model_dir());
    let name = match generator {
        Some(ref mut g) => crate::pet::names::generate_name(Some(&mut **g)).await,
        None => crate::pet::names::generate_name(None).await,
    };
    let mut pet = PetState::new(&name, Utc::now());
    let personality = match generator {
        Some(ref mut g) => pet.generate_personality(Some(&mut **g)).await,
        None => pet.generate_personality(None).await,
    };
    pet.personality = personality;
    storage
        .save_pet(&pet)
        .expect("pet.json の保存に失敗しました");

    tracing::info!(
        "ペット作成: name={}, stage={:?}, llm={:?}",
        pet.name,
        pet.stage,
        config.llm
    );

    print_guide(storage, &name);
}

/// テスト用の同期版（LLM を使わない）
#[cfg(test)]
pub fn run_sync_for_test(storage: &Storage) {
    use crate::pet::names::random_name;
    storage
        .ensure_dir()
        .expect("ディレクトリの作成に失敗しました");
    let name = random_name();
    let mut pet = PetState::new(&name, Utc::now());
    pet.personality = pet.fallback_personality();
    storage
        .save_pet(&pet)
        .expect("pet.json の保存に失敗しました");
}

fn print_guide(storage: &Storage, name: &str) {
    println!("🥚 {name} が生まれました！");
    println!("📁 {}", storage.base_dir().display());
    println!();
    println!("次にフックを設定してください:");
    println!();
    println!("  tamago hook zsh  >> ~/.zshrc");
    println!("  tamago hook bash >> ~/.bashrc");
    println!();
    println!("設定後:");
    println!("  source ~/.zshrc");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        (dir, storage)
    }

    #[test]
    fn init_creates_pet_json() {
        let (_dir, storage) = setup();
        run_sync_for_test(&storage);

        assert!(storage.pet_exists());
        let pet = storage.load_pet().unwrap();
        assert!(!pet.name.is_empty());
        assert_eq!(pet.stage, crate::pet::Stage::Egg);
        assert_eq!(pet.hunger, 100);
        assert_eq!(pet.mood, 100);
    }

    #[test]
    fn init_creates_nested_directory() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("a").join("b");
        let storage = Storage::new(&nested);

        run_sync_for_test(&storage);
        assert!(storage.pet_exists());
    }
}
