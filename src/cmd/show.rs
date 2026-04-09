use crate::storage::Storage;

pub fn run(storage: &Storage) {
    let _lock = storage.lock().expect("ロックの取得に失敗しました");

    let mut pet = storage
        .load_pet()
        .expect("ペットが見つかりません。tamago init を実行してください。");

    let activities = storage
        .read_and_clear_activities()
        .expect("activity の読み込みに失敗しました");

    let now = chrono::Utc::now();
    pet.apply_decay(now);

    let old_stage = pet.stage.clone();
    pet.apply_activities(&activities);

    // 進化判定（複数段一気に上がる場合もループ）
    while pet.try_evolve() {}

    pet.just_evolved = pet.stage != old_stage;

    storage
        .save_pet(&pet)
        .expect("pet.json の保存に失敗しました");

    if pet.just_evolved {
        print_evolution(&pet);
    }
    print_status(&pet);
}

fn print_evolution(pet: &crate::pet::PetState) {
    use crate::pet::Stage;
    let msg = match &pet.stage {
        Stage::Egg => unreachable!(),
        Stage::Baby => "🎉 たまごが孵化した！",
        Stage::Child => "🎉 すくすく成長している！",
        Stage::Teen => "🎉 大きくなってきた！",
        Stage::Adult => "🎉🎉🎉 最終進化した！",
    };
    println!();
    println!("  ✨✨✨✨✨✨✨✨✨✨");
    println!("  {msg}");
    if let Some(ref archetype) = pet.archetype {
        println!("  タイプ: {archetype:?}");
    }
    println!("  ✨✨✨✨✨✨✨✨✨✨");
    println!();
}

fn print_status(pet: &crate::pet::PetState) {
    let aa = crate::pet::render::animated_art(
        &pet.stage,
        &pet.archetype,
        &pet.name,
        pet.hunger,
        pet.mood,
        pet.exp,
    );
    println!("{aa}\n");

    let emoji = pet.emoji();
    let lv = pet.level();
    println!(
        "{emoji} {name} Lv.{lv} ♥{mood} 🍚{hunger}  EXP:{exp}",
        name = pet.name,
        mood = pet.mood,
        hunger = pet.hunger,
        exp = pet.exp,
    );
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
    fn show_clears_activities_after_apply() {
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

        // 2回目はクリア済みなので exp が増えない
        run(&storage);
        let pet_after_second = storage.load_pet().unwrap();
        assert_eq!(pet_after_second.exp, 1);
    }

    #[test]
    fn show_evolves_pet_when_threshold_reached() {
        let (_dir, storage) = setup_with_pet();

        // Egg→Baby に必要な 5000 exp 分の activity
        for _ in 0..250 {
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
        assert_eq!(pet.stage, Stage::Baby);
        assert_eq!(pet.exp, 5000);
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

        // Egg: 0-5000, Lv.1 at 0
        assert_eq!(pet.level(), 1);

        // Egg: 2500/5000 → ~50%
        pet.exp = 2500;
        assert!(pet.level() > 1);
        assert!(pet.level() <= 100);
    }
}
