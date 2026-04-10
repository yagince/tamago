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
    let old_level = pet.level();
    pet.apply_activities(&activities);

    while pet.try_evolve() {}

    let evolved = pet.stage != old_stage;
    if evolved {
        pet.evolved_at = Some(now);
    }

    let new_level = pet.level();
    if new_level > old_level {
        pet.apply_level_up_stats(new_level - old_level);
        if crate::pet::PetState::should_regenerate_personality(old_level, new_level, evolved) {
            pet.personality = pet.generate_personality();
        }
    }

    storage
        .save_pet(&pet)
        .expect("pet.json の保存に失敗しました");

    if evolved {
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
    let color = crate::pet::render::pet_color(&pet.stage, &pet.archetype, &pet.name);
    let colored = crate::pet::render::colorize_aa(&aa, color);
    println!("\n{colored}\n");

    let emoji = pet.emoji();
    let lv = pet.level();
    let creature = crate::pet::render::creature_type(&pet.stage, &pet.archetype, &pet.name);
    println!(
        "{emoji} {name} [{creature}] Lv.{lv} ♥{mood} 🍚{hunger}  EXP:{exp}",
        name = pet.name,
        mood = pet.mood,
        hunger = pet.hunger,
        exp = pet.exp,
    );
    println!(
        "  ⚡{} 📚{} 😆{} 🌀{}",
        pet.dev_power, pet.wisdom, pet.humor, pet.chaos
    );
    if !pet.personality.is_empty() {
        println!("  💬 {}", pet.personality);
    }

    print_category_bars(pet);
}

/// category_exp を棒グラフ形式で表示。exp 多い順にソートして、最大値を 20 cols に割り当てる。
fn print_category_bars(pet: &crate::pet::PetState) {
    use crate::pet::Category;

    const BAR_WIDTH: usize = 20;
    const RESET: &str = "\x1b[0m";

    let cats: &[(Category, &str, &str, &str)] = &[
        (Category::Git, "Git  ", "🔀", "\x1b[96m"), // bright cyan
        (Category::Ai, "AI   ", "🧠", "\x1b[95m"),  // bright magenta
        (Category::Dev, "Dev  ", "🔧", "\x1b[92m"), // bright green
        (Category::Infra, "Infra", "🌐", "\x1b[94m"), // bright blue
        (Category::Editor, "Edit ", "📝", "\x1b[93m"), // bright yellow
        (Category::Basic, "Basic", "🐚", "\x1b[97m"), // bright white
        (Category::Other, "Other", "✨", "\x1b[90m"), // bright black (gray)
    ];

    let max = pet.category_exp.values().copied().max().unwrap_or(0).max(1);

    // 値で降順ソート
    let mut sorted: Vec<_> = cats.iter().collect();
    sorted
        .sort_by_key(|(cat, _, _, _)| std::cmp::Reverse(*pet.category_exp.get(cat).unwrap_or(&0)));

    println!();
    for (cat, label, icon, color) in sorted {
        let value = *pet.category_exp.get(cat).unwrap_or(&0);
        // 非ゼロ値は必ず 1 文字以上表示
        let bar_len = if value == 0 {
            0
        } else {
            ((value * BAR_WIDTH as u64 / max) as usize).max(1)
        };
        let bar: String = "█".repeat(bar_len);
        let pad: String = " ".repeat(BAR_WIDTH - bar_len);
        println!("  {icon} {label} {color}{bar}{RESET}{pad} {value}");
    }
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
