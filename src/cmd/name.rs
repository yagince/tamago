use crate::pet::names::generate_name;
use crate::storage::Storage;

pub fn run(storage: &Storage, name: Option<&str>, ai: bool) {
    let new_name = if ai {
        generate_name()
    } else {
        name.expect("名前を指定してください").to_string()
    };

    let mut pet = storage.load_pet().expect("ペットが見つかりません");
    let old = pet.name.clone();
    pet.name = new_name.clone();
    storage
        .save_pet(&pet)
        .expect("pet.json の保存に失敗しました");
    println!("{old} → {new_name} に改名しました！");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_with_pet() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        super::super::init::run(&storage);
        (dir, storage)
    }

    #[test]
    fn name_changes_pet_name() {
        let (_dir, storage) = setup_with_pet();
        run(&storage, Some("ピカチュウ"), false);

        let pet = storage.load_pet().unwrap();
        assert_eq!(pet.name, "ピカチュウ");
    }
}
