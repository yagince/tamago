use crate::storage::Storage;

pub fn run(storage: &Storage, name: &str) {
    let mut pet = storage.load_pet().expect("ペットが見つかりません");
    let old = pet.name.clone();
    pet.name = name.to_string();
    storage.save_pet(&pet).expect("pet.json の保存に失敗しました");
    println!("{old} → {name} に改名しました！");
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
        run(&storage, "ピカチュウ");

        let pet = storage.load_pet().unwrap();
        assert_eq!(pet.name, "ピカチュウ");
    }
}
