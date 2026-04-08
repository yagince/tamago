use crate::storage::Storage;

pub fn run(storage: &Storage) {
    let _lock = storage.lock().expect("ロックの取得に失敗しました");

    let mut pet = storage
        .load_pet()
        .expect("ペットが見つかりません。tamago init を実行してください。");

    let (activities, new_cursor) = storage
        .read_pending_activities(pet.activity_cursor)
        .expect("activity の読み込みに失敗しました");

    pet.apply_activities(&activities);
    pet.activity_cursor = new_cursor;

    storage
        .save_pet(&pet)
        .expect("pet.json の保存に失敗しました");

    print_status(&pet);
}

fn print_status(pet: &crate::pet::PetState) {
    let emoji = pet.emoji();
    let lv = pet.level();
    println!(
        "{emoji} {name} Lv.{lv} ♥{mood} 🍚{hunger}",
        name = pet.name,
        mood = pet.mood,
        hunger = pet.hunger,
    );
    println!("  EXP: {}", pet.exp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    use crate::pet::{Category, Stage};
    use crate::storage::ActivityRecord;

    fn setup_with_pet() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        super::super::init::run(&storage);
        (dir, storage)
    }

    #[test]
    fn show_aggregates_pending_activities() {
        let (_dir, storage) = setup_with_pet();

        // activity を追加
        for _ in 0..3 {
            let record = ActivityRecord {
                cmd: "git commit".into(),
                cat: Category::Git,
                exp: 20,
                ts: Utc::now(),
            };
            storage.append_activity(&record).unwrap();
        }

        run(&storage);

        let pet = storage.load_pet().unwrap();
        assert_eq!(pet.exp, 60);
        assert_eq!(pet.category_exp[&Category::Git], 60);
    }

    #[test]
    fn show_advances_cursor() {
        let (_dir, storage) = setup_with_pet();

        let record = ActivityRecord {
            cmd: "ls".into(),
            cat: Category::Basic,
            exp: 1,
            ts: Utc::now(),
        };
        storage.append_activity(&record).unwrap();

        run(&storage);
        let pet_after_first = storage.load_pet().unwrap();
        assert_eq!(pet_after_first.exp, 1);

        // 2回目は集計済みなので exp が増えない
        run(&storage);
        let pet_after_second = storage.load_pet().unwrap();
        assert_eq!(pet_after_second.exp, 1);
    }

    #[test]
    fn show_without_activities() {
        let (_dir, storage) = setup_with_pet();

        run(&storage);

        let pet = storage.load_pet().unwrap();
        assert_eq!(pet.exp, 0);
        assert_eq!(pet.stage, Stage::Egg);
    }

    #[test]
    fn level_calculation() {
        let now = Utc::now();
        let mut pet = crate::pet::PetState::new("test", now);

        // Egg: 0-50, Lv.1 at 0
        assert_eq!(pet.level(), 1);

        // Egg: 25/50 → ~50%
        pet.exp = 25;
        assert!(pet.level() > 1);
        assert!(pet.level() <= 100);
    }
}
