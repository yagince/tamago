use chrono::Utc;

use crate::storage::{ActivityRecord, Storage};
use crate::tracker;

pub fn run(storage: &Storage, cmd: Option<&str>, claude_turn: bool, output_tokens: Option<u64>) {
    let record: Option<ActivityRecord> = if claude_turn {
        claude_turn_record(storage, output_tokens.unwrap_or(0))
    } else if let Some(cmd_str) = cmd {
        let score = tracker::score(cmd_str);
        Some(ActivityRecord {
            cmd: cmd_str.into(),
            cat: score.category,
            exp: score.exp,
            ts: Utc::now(),
        })
    } else {
        None
    };

    if let Some(record) = record
        && let Err(e) = storage.append_activity(&record)
    {
        eprintln!("tamago: activity の記録に失敗: {e}");
    }
}

/// claude turn の exp を算出して ActivityRecord を返す。
/// output_tokens は Claude Code の累積値なので、pet.json に記録した前回値との
/// 差分を exp 計算に使う。初回 (pet.last_output_tokens == 0) は
/// ベースラインとして値だけ保存し、activity は記録しない。
fn claude_turn_record(storage: &Storage, total: u64) -> Option<ActivityRecord> {
    let _lock = storage.lock().ok()?;
    let mut pet = storage.load_pet().ok()?;

    let previous = pet.last_output_tokens;
    if total == previous {
        return None;
    }

    pet.last_output_tokens = total;
    let _ = storage.save_pet(&pet);

    if previous == 0 {
        return None;
    }

    let delta = total.saturating_sub(previous);
    let score = tracker::claude_turn_score(delta);
    Some(ActivityRecord {
        cmd: "--claude-turn".into(),
        cat: score.category,
        exp: score.exp,
        ts: Utc::now(),
    })
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
    fn tick_no_args_does_nothing() {
        let (_dir, storage) = setup();
        run(&storage, None, false, None);

        assert!(!storage.activity_file().exists());
    }

    /// 初回の claude turn は last_output_tokens == 0 のため
    /// activity を記録せず、ベースラインとして値を保存するだけにする。
    /// これによりツール初起動時の「累積値そのままが delta になる問題」を防ぐ。
    #[test]
    fn tick_claude_turn_first_call_is_baseline_only() {
        let (_dir, storage) = setup();
        run(&storage, None, true, Some(10000));

        // activity は記録されていない
        let content = fs::read_to_string(storage.activity_file()).unwrap_or_default();
        assert!(
            content.trim().is_empty(),
            "初回呼び出しでは activity を記録しないはず: {content:?}"
        );

        // last_output_tokens だけ更新されている
        let pet = storage.load_pet().unwrap();
        assert_eq!(pet.last_output_tokens, 10000);
    }

    #[test]
    fn tick_claude_turn_records_delta_after_baseline() {
        let (_dir, storage) = setup();
        // 1 回目: ベースライン確立のみ
        run(&storage, None, true, Some(1000));
        // 2 回目: delta = 4000 - 1000 = 3000 → sqrt(3000)/12 = 4.56 → 4+1 = 5
        run(&storage, None, true, Some(4000));

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
        let record: ActivityRecord = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(record.cmd, "--claude-turn");
        assert_eq!(record.cat, Category::Ai);
        assert_eq!(record.exp, 5);
    }

    #[test]
    fn tick_claude_turn_uses_delta_not_cumulative() {
        let (_dir, storage) = setup();
        // 1 回目: ベースライン (10000)
        run(&storage, None, true, Some(10000));
        // 2 回目: delta = 12500 - 10000 = 2500 → sqrt(2500)/12 = 4.16 → 4+1 = 5
        run(&storage, None, true, Some(12500));
        // 3 回目: delta = 15000 - 12500 = 2500 → 5
        run(&storage, None, true, Some(15000));

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        let first: ActivityRecord = serde_json::from_str(lines[0]).unwrap();
        let second: ActivityRecord = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(first.exp, 5);
        assert_eq!(second.exp, 5);
    }

    #[test]
    fn tick_claude_turn_handles_token_reset() {
        let (_dir, storage) = setup();
        // 1 回目: ベースライン
        run(&storage, None, true, Some(50000));
        // 2 回目: 累積が減った場合（セッションリセット）は delta = 0
        run(&storage, None, true, Some(1000));

        let content = fs::read_to_string(storage.activity_file()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
        let record: ActivityRecord = serde_json::from_str(lines[0]).unwrap();
        // delta = 0 → sqrt(0)/12 = 0 → 0+1 = 1 exp
        assert_eq!(record.exp, 1);
    }
}
