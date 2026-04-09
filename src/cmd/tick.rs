use chrono::Utc;

use crate::storage::{ActivityRecord, Storage};
use crate::tracker;

pub fn run(storage: &Storage, cmd: Option<&str>, claude_turn: bool, output_tokens: Option<u64>) {
    let record = if claude_turn {
        // output_tokens は Claude Code の累積値なので、pet.json に記録した
        // 前回値との差分を使う
        let total = output_tokens.unwrap_or(0);
        let delta = if let Ok(_lock) = storage.lock() {
            if let Ok(mut pet) = storage.load_pet() {
                let d = total.saturating_sub(pet.last_output_tokens);
                pet.last_output_tokens = total;
                let _ = storage.save_pet(&pet);
                d
            } else {
                total
            }
        } else {
            total
        };
        let score = tracker::claude_turn_score(delta);
        ActivityRecord {
            cmd: "--claude-turn".into(),
            cat: score.category,
            exp: score.exp,
            ts: Utc::now(),
        }
    } else if let Some(cmd_str) = cmd {
        let score = tracker::score(cmd_str);
        ActivityRecord {
            cmd: cmd_str.into(),
            cat: score.category,
            exp: score.exp,
            ts: Utc::now(),
        }
    } else {
        return;
    };

    if let Err(e) = storage.append_activity(&record) {
        eprintln!("tamago: activity の記録に失敗: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pet::Category;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        storage.ensure_dir().unwrap();
        super::super::init::run(&storage);
        (dir, storage)
    }

    #[test]
    fn tick_cmd_appends_activity() {
        let (_dir, storage) = setup();
        run(&storage, Some("git commit -m fix"), false, None);

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let record: ActivityRecord = serde_json::from_str(content.trim()).unwrap();
        assert_eq!(record.cmd, "git commit -m fix");
        assert_eq!(record.cat, Category::Git);
        assert_eq!(record.exp, 20);
    }

    #[test]
    fn tick_claude_turn_records_ai() {
        let (_dir, storage) = setup();
        run(&storage, None, true, None);

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let record: ActivityRecord = serde_json::from_str(content.trim()).unwrap();
        assert_eq!(record.cmd, "--claude-turn");
        assert_eq!(record.cat, Category::Ai);
        assert_eq!(record.exp, 1); // no tokens = minimal exp
    }

    #[test]
    fn tick_no_args_does_nothing() {
        let (_dir, storage) = setup();
        run(&storage, None, false, None);

        assert!(!storage.activity_file().exists());
    }

    #[test]
    fn tick_claude_turn_with_high_tokens() {
        let (_dir, storage) = setup();
        run(&storage, None, true, Some(3000));

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let record: ActivityRecord = serde_json::from_str(content.trim()).unwrap();
        // 初回は delta = 3000 - 0 = 3000 → sqrt(3000)/12 = 4.56 → 4 + 1 = 5
        assert_eq!(record.exp, 5);
    }

    #[test]
    fn tick_claude_turn_uses_delta_not_cumulative() {
        let (_dir, storage) = setup();
        // 1 回目: 累積 10000 → delta = 10000 → sqrt(10000)/12 = 8.33 → 8+1 = 9 exp
        run(&storage, None, true, Some(10000));
        // 2 回目: 累積 12500 → delta = 2500 → sqrt(2500)/12 = 4.16 → 4+1 = 5 exp
        run(&storage, None, true, Some(12500));

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        let first: ActivityRecord = serde_json::from_str(lines[0]).unwrap();
        let second: ActivityRecord = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(first.exp, 9);
        assert_eq!(second.exp, 5);
    }

    #[test]
    fn tick_claude_turn_handles_token_reset() {
        let (_dir, storage) = setup();
        // 累積が減った場合（セッションリセット）は delta = 0 として扱う
        run(&storage, None, true, Some(50000));
        run(&storage, None, true, Some(1000));

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        let second: ActivityRecord = serde_json::from_str(lines[1]).unwrap();
        // delta = 0 → sqrt(0)/12 = 0 → 0+1 = 1 exp
        assert_eq!(second.exp, 1);
    }
}
