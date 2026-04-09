use crate::storage::Storage;

pub fn run(storage: &Storage) {
    let pet = match storage.load_pet() {
        Ok(pet) => pet,
        Err(_) => return, // ペットがなければ何も出力しない
    };

    let emoji = pet.emoji();
    let lv = pet.level();
    print!(
        "{emoji} {name} Lv.{lv} ♥{mood} 🍚{hunger}",
        name = pet.name,
        mood = pet.mood,
        hunger = pet.hunger,
    );
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
    fn status_without_pet_outputs_nothing() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        // should not panic
        run(&storage);
    }

    #[test]
    fn status_with_pet_outputs_statusline() {
        let (_dir, storage) = setup_with_pet();
        // should not panic
        run(&storage);
    }
}
