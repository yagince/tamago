use chrono::Utc;

use crate::pet::Category;
use crate::storage::{ActivityRecord, Storage};
use crate::tracker;

pub fn run(storage: &Storage, cmd: Option<&str>, claude_turn: bool, output_tokens: Option<u64>) {
    let record = if claude_turn {
        // output_tokens に応じて経験値を変える
        // 0-100: 1, 100-500: 3, 500-2000: 5, 2000+: 10
        let exp = match output_tokens.unwrap_or(0) {
            0..=100 => 1,
            101..=500 => 3,
            501..=2000 => 5,
            _ => 10,
        };
        ActivityRecord {
            cmd: "--claude-turn".into(),
            cat: Category::Ai,
            exp,
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
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        storage.ensure_dir().unwrap();
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
        assert_eq!(record.exp, 10); // 2000+ tokens = 10 exp
    }
}
