//! Claude Code plugin が呼ぶ内部サブコマンド。
//! stdin から hook イベント JSON を読み取り、transcript を集計して activity に記録する。
//! 失敗しても常に exit 0 で返す（Claude Code 本体の挙動を壊さない）。

use std::io::{self, BufRead, Read};

use chrono::Utc;
use serde::Deserialize;

use crate::pet::Category;
use crate::storage::{ActivityRecord, Storage};

pub async fn run(storage: &Storage, event: &str) {
    if let Err(e) = handle(storage, event) {
        tracing::warn!("claude-hook (event={event}) 失敗: {e}");
    }
}

fn handle(storage: &Storage, event: &str) -> io::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let payload: HookPayload = serde_json::from_str(buf.trim()).unwrap_or_default();

    tracing::info!(
        "claude-hook event={event} session={:?} cwd={:?}",
        payload.session_id,
        payload.cwd
    );

    match event {
        "stop" | "session-end" => {
            let (Some(session_id), Some(path)) = (
                payload.session_id.as_deref(),
                payload.transcript_path.as_deref(),
            ) else {
                return Ok(());
            };
            ingest_transcript_incremental(storage, session_id, path)?;
            if event == "session-end" {
                let _ = std::fs::remove_file(cursor_path(storage, session_id));
            }
        }
        _ => {
            // 将来の拡張用
        }
    }
    Ok(())
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct HookPayload {
    session_id: Option<String>,
    cwd: Option<String>,
    transcript_path: Option<String>,
    #[allow(dead_code)]
    source: Option<String>,
    #[allow(dead_code)]
    reason: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct TranscriptLine {
    #[serde(rename = "type")]
    kind: Option<String>,
    message: Option<TranscriptMessage>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct TranscriptMessage {
    content: Option<serde_json::Value>,
}

#[derive(Default)]
struct Summary {
    user_msgs: u64,
    tool_counts: std::collections::HashMap<String, u64>,
}

fn cursor_path(storage: &Storage, session_id: &str) -> std::path::PathBuf {
    storage
        .base_dir()
        .join("claude_sessions")
        .join(format!("{session_id}.cursor"))
}

fn load_cursor(path: &std::path::Path) -> u64 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_cursor(path: &std::path::Path, processed_lines: u64) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, processed_lines.to_string());
}

fn ingest_transcript_incremental(
    storage: &Storage,
    session_id: &str,
    path: &str,
) -> io::Result<()> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("transcript 読めず: {path}: {e}");
            return Ok(());
        }
    };
    let cursor_file = cursor_path(storage, session_id);
    let already_processed = load_cursor(&cursor_file);

    let reader = io::BufReader::new(file);
    let mut summary = Summary::default();
    let mut current_line: u64 = 0;
    for line in reader.lines().map_while(Result::ok) {
        current_line += 1;
        if current_line <= already_processed {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<TranscriptLine>(&line) else {
            continue;
        };
        match entry.kind.as_deref() {
            Some("user") => summary.user_msgs += 1,
            Some("assistant") => {
                if let Some(msg) = entry.message
                    && let Some(content) = msg.content.as_ref()
                    && let Some(items) = content.as_array()
                {
                    for item in items {
                        if item.get("type").and_then(|v| v.as_str()) == Some("tool_use")
                            && let Some(name) = item.get("name").and_then(|v| v.as_str())
                        {
                            *summary.tool_counts.entry(name.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    save_cursor(&cursor_file, current_line);
    record_summary(storage, &summary);
    Ok(())
}

fn record_summary(storage: &Storage, summary: &Summary) {
    let total_tools: u64 = summary.tool_counts.values().sum();
    tracing::info!(
        "claude transcript 集計: user_msgs={}, tool_calls={}",
        summary.user_msgs,
        total_tools
    );

    // ユーザーメッセージ: 1件 = 3 exp (Basic カテゴリ)
    if summary.user_msgs > 0 {
        append(
            storage,
            ActivityRecord {
                cmd: format!("claude-session: {} user msgs", summary.user_msgs),
                cat: Category::Basic,
                exp: summary.user_msgs * 3,
                ts: Utc::now(),
            },
        );
    }

    // tool 使用を Category ごとに集計
    let mut by_cat: std::collections::HashMap<Category, (u64, Vec<String>)> =
        std::collections::HashMap::new();
    for (name, count) in &summary.tool_counts {
        let cat = tool_category(name);
        let entry = by_cat.entry(cat).or_default();
        entry.0 += count;
        entry.1.push(format!("{name}×{count}"));
    }
    for (cat, (count, detail)) in by_cat {
        append(
            storage,
            ActivityRecord {
                cmd: format!("claude-tools {}: {}", category_label(cat), detail.join(",")),
                cat,
                exp: count * 2,
                ts: Utc::now(),
            },
        );
    }
}

fn append(storage: &Storage, record: ActivityRecord) {
    if let Err(e) = storage.append_activity(&record) {
        tracing::warn!("claude-hook activity 記録失敗: {e}");
    }
}

fn tool_category(name: &str) -> Category {
    match name {
        "Edit" | "Write" | "NotebookEdit" => Category::Dev,
        "Read" | "Grep" | "Glob" => Category::Editor,
        "Bash" => Category::Basic,
        "WebSearch" | "WebFetch" | "Agent" | "Task" | "ToolSearch" => Category::Ai,
        _ => Category::Other,
    }
}

fn category_label(cat: Category) -> &'static str {
    match cat {
        Category::Git => "git",
        Category::Ai => "ai",
        Category::Dev => "dev",
        Category::Infra => "infra",
        Category::Editor => "editor",
        Category::Basic => "basic",
        Category::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Storage) {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path());
        storage.ensure_dir().unwrap();
        (dir, storage)
    }

    #[test]
    fn tool_category_mapping() {
        assert_eq!(tool_category("Edit"), Category::Dev);
        assert_eq!(tool_category("Read"), Category::Editor);
        assert_eq!(tool_category("Bash"), Category::Basic);
        assert_eq!(tool_category("WebSearch"), Category::Ai);
        assert_eq!(tool_category("UnknownTool"), Category::Other);
    }

    fn write_transcript(dir: &TempDir, content: &str) -> std::path::PathBuf {
        let path = dir.path().join("t.jsonl");
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn ingest_counts_user_and_tool_use() {
        let (dir, storage) = setup();
        let content = r#"{"type":"user","message":{"content":"hi"}}
{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash"}]}}
{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Edit"},{"type":"tool_use","name":"Read"}]}}
{"type":"user","message":{"content":"bye"}}
"#;
        let path = write_transcript(&dir, content);
        ingest_transcript_incremental(&storage, "sess1", path.to_str().unwrap()).unwrap();

        let activities = storage.read_and_clear_activities().unwrap();
        assert_eq!(activities.len(), 4, "basic(user)+basic(Bash)+dev+editor");
        assert_eq!(
            activities
                .iter()
                .find(|a| a.cmd.contains("user msgs"))
                .unwrap()
                .exp,
            6
        );
        assert_eq!(
            activities
                .iter()
                .find(|a| a.cat == Category::Dev)
                .unwrap()
                .exp,
            2
        );
    }

    #[test]
    fn cursor_prevents_double_count() {
        let (dir, storage) = setup();
        let content = "{\"type\":\"user\",\"message\":{\"content\":\"hi\"}}\n";
        let path = write_transcript(&dir, content);
        ingest_transcript_incremental(&storage, "sess2", path.to_str().unwrap()).unwrap();
        ingest_transcript_incremental(&storage, "sess2", path.to_str().unwrap()).unwrap();
        let activities = storage.read_and_clear_activities().unwrap();
        // 2回目は増分ゼロ → record は 1回目の1件のみ
        assert_eq!(activities.len(), 1);
    }

    #[test]
    fn cursor_picks_up_appended_lines() {
        let (dir, storage) = setup();
        let path = dir.path().join("t.jsonl");
        std::fs::write(
            &path,
            "{\"type\":\"user\",\"message\":{\"content\":\"1\"}}\n",
        )
        .unwrap();
        ingest_transcript_incremental(&storage, "sess3", path.to_str().unwrap()).unwrap();
        let _ = storage.read_and_clear_activities().unwrap();

        // 行を追記
        std::fs::write(
            &path,
            "{\"type\":\"user\",\"message\":{\"content\":\"1\"}}\n{\"type\":\"user\",\"message\":{\"content\":\"2\"}}\n",
        )
        .unwrap();
        ingest_transcript_incremental(&storage, "sess3", path.to_str().unwrap()).unwrap();
        let activities = storage.read_and_clear_activities().unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].exp, 3, "新しい 1 user msg 分だけ");
    }

    #[test]
    fn missing_transcript_is_soft_error() {
        let (_dir, storage) = setup();
        let result = ingest_transcript_incremental(&storage, "sess-x", "/nonexistent/path.jsonl");
        assert!(result.is_ok());
    }
}
