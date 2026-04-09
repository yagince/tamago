use crate::storage::Storage;

/// activity.jsonl がこのサイズ以上なら集計する
const AGGREGATE_THRESHOLD: u64 = 512;

/// 進化 / レベルアップ演出を表示し続ける秒数
const CELEBRATION_DURATION_SECS: i64 = 10;

/// CC statusline は行頭の空白を strip するため、空白 ` ` を
/// 「黒い `█`（背景に溶けて見えない非空白文字）」に置換する
const EMPTY: &str = "\x1b[30m\u{2588}\x1b[0m";

pub fn run(storage: &Storage) {
    let mut pet = match storage.load_pet() {
        Ok(pet) => pet,
        Err(_) => return,
    };

    let now = chrono::Utc::now();

    // activity が溜まっていたら集計
    let activity_size = std::fs::metadata(storage.activity_file())
        .map(|m| m.len())
        .unwrap_or(0);
    if activity_size >= AGGREGATE_THRESHOLD {
        if let Ok(_lock) = storage.lock() {
            pet.apply_decay(now);

            if let Ok(activities) = storage.read_and_clear_activities() {
                let old_stage = pet.stage.clone();
                let old_level = pet.level();
                pet.apply_activities(&activities);
                while pet.try_evolve() {}
                if pet.stage != old_stage {
                    pet.evolved_at = Some(now);
                }
                if pet.level() > old_level {
                    pet.leveled_up_at = Some(now);
                }
            }

            let _ = storage.save_pet(&pet);
        }
    }

    let emoji = pet.emoji();
    let lv = pet.level();
    let creature = crate::pet::render::creature_type(&pet.stage, &pet.archetype, &pet.name);

    let show_evolved = within_celebration_window(pet.evolved_at, now);
    let show_leveled_up = within_celebration_window(pet.leveled_up_at, now);

    if show_evolved || show_leveled_up {
        // 進化 or レベルアップ演出: animated_art (sparkle 付き) + テーマ色で表示
        // 空白は CC statusline の strip を避けるため黒 `█` に置換する
        let aa = crate::pet::render::animated_art(
            &pet.stage,
            &pet.archetype,
            &pet.name,
            pet.hunger,
            pet.mood,
            pet.exp,
        );
        let color = crate::pet::render::pet_color(&pet.stage, &pet.archetype, &pet.name);
        let colored = crate::pet::render::colorize_aa(&aa, color);
        let decorated = colored.trim_matches('\n').replace(' ', EMPTY);
        print!("{decorated}");

        let label = if show_evolved {
            "🎉 進化！"
        } else {
            "✨ レベルアップ！"
        };

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
        // 期限切れの timestamp を掃除（無駄書き込みを避けるため変化時のみ）
        let mut dirty = false;
        if pet.evolved_at.is_some() {
            pet.evolved_at = None;
            dirty = true;
        }
        if pet.leveled_up_at.is_some() {
            pet.leveled_up_at = None;
            dirty = true;
        }
        if dirty {
            let _ = storage.save_pet(&pet);
        }

        print!(
            "{emoji} {name} [{creature}] Lv.{lv} ♥{mood} 🍚{hunger} EXP:{exp}",
            name = pet.name,
            mood = pet.mood,
            hunger = pet.hunger,
            exp = pet.exp,
        );
    }
}

fn within_celebration_window(
    ts: Option<chrono::DateTime<chrono::Utc>>,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    match ts {
        Some(ts) => (now - ts).num_seconds() < CELEBRATION_DURATION_SECS,
        None => false,
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
    fn status_clears_expired_evolved_timestamp() {
        let (_dir, storage) = setup_with_pet();

        // 10 秒以上前の timestamp を立てる
        let mut pet = storage.load_pet().unwrap();
        pet.evolved_at = Some(chrono::Utc::now() - chrono::Duration::seconds(30));
        storage.save_pet(&pet).unwrap();

        run(&storage);

        let pet = storage.load_pet().unwrap();
        assert!(pet.evolved_at.is_none());
    }

    #[test]
    fn status_keeps_recent_evolved_timestamp() {
        let (_dir, storage) = setup_with_pet();

        // 3 秒前の timestamp → まだ演出中なのでクリアされない
        let ts = chrono::Utc::now() - chrono::Duration::seconds(3);
        let mut pet = storage.load_pet().unwrap();
        pet.evolved_at = Some(ts);
        storage.save_pet(&pet).unwrap();

        run(&storage);

        let pet = storage.load_pet().unwrap();
        assert_eq!(pet.evolved_at, Some(ts));
    }

    #[test]
    fn status_clears_expired_leveled_up_timestamp() {
        let (_dir, storage) = setup_with_pet();

        let mut pet = storage.load_pet().unwrap();
        pet.leveled_up_at = Some(chrono::Utc::now() - chrono::Duration::seconds(30));
        storage.save_pet(&pet).unwrap();

        run(&storage);

        let pet = storage.load_pet().unwrap();
        assert!(pet.leveled_up_at.is_none());
    }
}
