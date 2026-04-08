use chrono::Utc;

use crate::pet::names::random_name;
use crate::pet::PetState;
use crate::storage::Storage;

pub fn run(storage: &Storage) {
    if storage.pet_exists() {
        eprintln!("すでにペットが存在します。");
        std::process::exit(1);
    }

    storage
        .ensure_dir()
        .expect("ディレクトリの作成に失敗しました");

    let name = random_name();
    let pet = PetState::new(&name, Utc::now());
    storage
        .save_pet(&pet)
        .expect("pet.json の保存に失敗しました");

    print_guide(storage, &name);
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
        run(&storage);

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

        run(&storage);
        assert!(storage.pet_exists());
    }
}
