use crate::storage::Storage;

/// activity.jsonl がこのサイズ以上なら集計する
const AGGREGATE_THRESHOLD: u64 = 512;


pub fn run(storage: &Storage) {
    let mut pet = match storage.load_pet() {
        Ok(pet) => pet,
        Err(_) => return,
    };

    // activity が溜まっていたら集計
    let activity_size = std::fs::metadata(storage.activity_file())
        .map(|m| m.len())
        .unwrap_or(0);
    if activity_size >= AGGREGATE_THRESHOLD {
        if let Ok(_lock) = storage.lock() {
            let now = chrono::Utc::now();
            pet.apply_decay(now);

            if let Ok(activities) = storage.read_and_clear_activities() {
                let old_stage = pet.stage.clone();
                pet.apply_activities(&activities);
                while pet.try_evolve() {}
                if pet.stage != old_stage {
                    pet.just_evolved = true;
                }
            }

            let _ = storage.save_pet(&pet);
        }
    }

    let emoji = pet.emoji();
    let lv = pet.level();
    let creature = crate::pet::render::creature_type(&pet.stage, &pet.archetype, &pet.name);

    if pet.just_evolved {
        // 進化演出: halfblock AA をそのまま表示
        // CC statusline は空白を strip するため、空白 ` ` を
        // 「黒い `█`（背景に溶けて見えない非空白文字）」に置換する
        let aa = crate::pet::render::ascii_art(
            &pet.stage,
            &pet.archetype,
            &pet.name,
            pet.hunger,
            pet.mood,
        );
        let aa = aa.trim_matches('\n');
        let replaced = aa.replace(' ', "\x1b[30m\u{2588}\x1b[0m");
        print!("{replaced}");

        pet.just_evolved = false;
        let _ = storage.save_pet(&pet);

        print!(
            "\n🎉 {emoji} {name} [{creature}] Lv.{lv} ♥{mood} 🍚{hunger} EXP:{exp}",
            name = pet.name,
            mood = pet.mood,
            hunger = pet.hunger,
            exp = pet.exp,
        );
    } else {
        print!(
            "{emoji} {name} [{creature}] Lv.{lv} ♥{mood} 🍚{hunger} EXP:{exp}",
            name = pet.name,
            mood = pet.mood,
            hunger = pet.hunger,
            exp = pet.exp,
        );
    }
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
