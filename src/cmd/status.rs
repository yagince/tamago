use crate::storage::Storage;

/// activity.jsonl がこのサイズ以上なら集計する
const AGGREGATE_THRESHOLD: u64 = 512;

/// CC statusline は行頭の空白を strip するため、空白 ` ` を
/// 「黒い `█`（背景に溶けて見えない非空白文字）」に置換する
const EMPTY: &str = "\x1b[30m\u{2588}\x1b[0m";
const RESET: &str = "\x1b[0m";

/// 進化/レベルアップ演出用: AA にテーマ色を適用
fn decorate_evolution_art(aa: &str, color: &str) -> String {
    let aa = aa.trim_matches('\n');
    let mut out = String::new();
    for (i, line) in aa.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        for ch in line.chars() {
            match ch {
                ' ' => out.push_str(EMPTY),
                '▀' | '▄' | '█' => out.push_str(&format!("{color}{ch}{RESET}")),
                _ => out.push(ch),
            }
        }
    }
    out
}


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
                let old_level = pet.level();
                pet.apply_activities(&activities);
                while pet.try_evolve() {}
                if pet.stage != old_stage {
                    pet.just_evolved = true;
                }
                if pet.level() > old_level {
                    pet.just_leveled_up = true;
                }
            }

            let _ = storage.save_pet(&pet);
        }
    }

    let emoji = pet.emoji();
    let lv = pet.level();
    let creature = crate::pet::render::creature_type(&pet.stage, &pet.archetype, &pet.name);

    if pet.just_evolved || pet.just_leveled_up {
        // 進化 or レベルアップ演出: halfblock AA をテーマ色 + フレーム付きで表示
        let aa = crate::pet::render::ascii_art(
            &pet.stage,
            &pet.archetype,
            &pet.name,
            pet.hunger,
            pet.mood,
        );
        let color = crate::pet::render::pet_color(&pet.stage, &pet.archetype, &pet.name);
        let framed = decorate_evolution_art(&aa, color);
        print!("{framed}");

        let label = if pet.just_evolved {
            "🎉 進化！"
        } else {
            "✨ レベルアップ！"
        };
        pet.just_evolved = false;
        pet.just_leveled_up = false;
        let _ = storage.save_pet(&pet);

        // AA とステータスの間に空行（CC statusline に strip されないよう
        // 見えない黒 `█` を 1 文字だけ置く）
        print!(
            "\n{EMPTY}\n{label} {emoji} {name} [{creature}] Lv.{lv} ♥{mood} 🍚{hunger} EXP:{exp}",
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

    #[test]
    fn status_clears_leveled_up_flag() {
        let (_dir, storage) = setup_with_pet();

        let mut pet = storage.load_pet().unwrap();
        pet.just_leveled_up = true;
        storage.save_pet(&pet).unwrap();

        run(&storage);

        let pet = storage.load_pet().unwrap();
        assert!(!pet.just_leveled_up);
    }
}
