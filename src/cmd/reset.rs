use std::fs;

use crate::storage::Storage;

pub async fn run(storage: &Storage) {
    let dir = storage.base_dir();
    if !dir.exists() {
        eprintln!("データが見つかりません: {}", dir.display());
        std::process::exit(1);
    }

    fs::remove_dir_all(dir).expect("データの削除に失敗しました");
    println!("🗑️  データを削除しました: {}", dir.display());

    super::init::run(storage).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_with_pet() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        super::super::init::run_sync_for_test(&storage);
        (dir, storage)
    }

    #[tokio::test]
    async fn reset_recreates_pet() {
        let (_dir, storage) = setup_with_pet();
        assert!(storage.pet_exists());

        run(&storage).await;

        assert!(storage.pet_exists());
        let pet = storage.load_pet().unwrap();
        assert_eq!(pet.exp, 0);
    }

    #[tokio::test]
    async fn reset_clears_activity() {
        let (_dir, storage) = setup_with_pet();

        // activity を書き込む
        let record = crate::storage::ActivityRecord {
            cmd: "test".into(),
            cat: crate::pet::Category::Basic,
            exp: 1,
            ts: chrono::Utc::now(),
        };
        storage.append_activity(&record).unwrap();
        assert!(storage.activity_file().exists());

        run(&storage).await;

        assert!(!storage.activity_file().exists());
    }
}
