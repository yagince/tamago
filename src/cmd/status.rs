use crate::config::Config;
use crate::llm;
use crate::storage::Storage;

/// activity.jsonl がこのサイズ以上なら集計する
const AGGREGATE_THRESHOLD: u64 = 512;

/// 進化 / レベルアップ演出を表示し続ける秒数
const CELEBRATION_DURATION_SECS: i64 = 10;

/// CC statusline は行頭の空白を strip するため、空白 ` ` を
/// 「黒い `█`（背景に溶けて見えない非空白文字）」に置換する
const EMPTY: &str = "\x1b[30m\u{2588}\x1b[0m";

pub async fn run(storage: &Storage) {
    let mut pet = match storage.load_pet() {
        Ok(pet) => pet,
        Err(_) => return,
    };

    let now = chrono::Utc::now();

    // activity が溜まっていたら集計
    let activity_size = std::fs::metadata(storage.activity_file())
        .map(|m| m.len())
        .unwrap_or(0);
    if activity_size >= AGGREGATE_THRESHOLD
        && let Ok(_lock) = storage.lock()
    {
        if let Ok(activities) = storage.read_and_clear_activities() {
            let config = Config::load(storage.base_dir());
            let mut generator = llm::create_generator(&config, storage);
            match generator {
                Some(ref mut g) => {
                    pet.grow(now, &activities, Some(&mut **g)).await;
                }
                None => {
                    pet.grow(now, &activities, None).await;
                }
            }
        }

        let _ = storage.save_pet(&pet);
    }

    let emoji = pet.emoji();
    let lv = pet.level();
    let creature = crate::pet::render::creature_type(&pet.stage, &pet.archetype, &pet.name);

    let show_evolved = within_celebration_window(pet.evolved_at, now);
    let show_leveled_up = within_celebration_window(pet.leveled_up_at, now);

    if show_evolved || show_leveled_up {
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

        print!(
            "\n{EMPTY}\n{label} {emoji} {name} [{creature}] Lv.{lv} ♥{mood} 🍚{hunger} EXP:{exp}",
            name = pet.name,
            mood = pet.mood,
            hunger = pet.hunger,
            exp = pet.exp,
        );
    } else {
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
        super::super::init::run_sync_for_test(&storage);
        (dir, storage)
    }

    #[tokio::test]
    async fn status_without_pet_outputs_nothing() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        run(&storage).await;
    }

    #[tokio::test]
    async fn status_with_pet_outputs_statusline() {
        let (_dir, storage) = setup_with_pet();
        run(&storage).await;
    }

    #[tokio::test]
    async fn status_clears_expired_evolved_timestamp() {
        let (_dir, storage) = setup_with_pet();

        let mut pet = storage.load_pet().unwrap();
        pet.evolved_at = Some(chrono::Utc::now() - chrono::Duration::seconds(30));
        storage.save_pet(&pet).unwrap();

        run(&storage).await;

        let pet = storage.load_pet().unwrap();
        assert!(pet.evolved_at.is_none());
    }

    #[tokio::test]
    async fn status_keeps_recent_evolved_timestamp() {
        let (_dir, storage) = setup_with_pet();

        let ts = chrono::Utc::now() - chrono::Duration::seconds(3);
        let mut pet = storage.load_pet().unwrap();
        pet.evolved_at = Some(ts);
        storage.save_pet(&pet).unwrap();

        run(&storage).await;

        let pet = storage.load_pet().unwrap();
        assert_eq!(pet.evolved_at, Some(ts));
    }

    #[tokio::test]
    async fn status_clears_expired_leveled_up_timestamp() {
        let (_dir, storage) = setup_with_pet();

        let mut pet = storage.load_pet().unwrap();
        pet.leveled_up_at = Some(chrono::Utc::now() - chrono::Duration::seconds(30));
        storage.save_pet(&pet).unwrap();

        run(&storage).await;

        let pet = storage.load_pet().unwrap();
        assert!(pet.leveled_up_at.is_none());
    }
}
