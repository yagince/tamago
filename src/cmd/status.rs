use crate::storage::Storage;

pub fn run(storage: &Storage) {
    let mut pet = match storage.load_pet() {
        Ok(pet) => pet,
        Err(_) => return,
    };

    if pet.just_evolved {
        // 進化演出: AA を表示
        let aa = crate::pet::render::ascii_art(
            &pet.stage,
            &pet.archetype,
            &pet.name,
            pet.hunger,
            pet.mood,
        );
        print!("{aa}");

        // フラグをクリアして保存
        pet.just_evolved = false;
        let _ = storage.save_pet(&pet);
    }

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
        run(&storage);
    }

    #[test]
    fn status_with_pet_outputs_statusline() {
        let (_dir, storage) = setup_with_pet();
        run(&storage);
    }

    #[test]
    fn status_clears_evolved_flag() {
        let (_dir, storage) = setup_with_pet();

        // フラグを立てる
        let mut pet = storage.load_pet().unwrap();
        pet.just_evolved = true;
        storage.save_pet(&pet).unwrap();

        run(&storage);

        // フラグがクリアされている
        let pet = storage.load_pet().unwrap();
        assert!(!pet.just_evolved);
    }
}
