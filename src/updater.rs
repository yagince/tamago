//! GitHub Releases から最新バージョンをバックグラウンドで取得し、
//! 結果を設定ディレクトリにキャッシュして次回起動時に通知する fire-and-forget 方式。

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const CACHE_FILE: &str = "update_check.json";
const API_URL: &str = "https://api.github.com/repos/yagince/tamago/releases/latest";
/// チェック間隔（秒）。24時間。
const CHECK_INTERVAL_SECS: i64 = 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCache {
    pub last_checked: DateTime<Utc>,
    pub latest_version: Option<String>,
}

pub fn cache_path(base_dir: &Path) -> PathBuf {
    base_dir.join(CACHE_FILE)
}

pub fn load_cache(base_dir: &Path) -> Option<UpdateCache> {
    let data = std::fs::read_to_string(cache_path(base_dir)).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_cache(base_dir: &Path, cache: &UpdateCache) {
    let path = cache_path(base_dir);
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = std::fs::write(&path, json);
    }
}

/// キャッシュにある最新バージョンが current より新しければ返す。
pub fn pending_update(base_dir: &Path, current: &str) -> Option<String> {
    let cache = load_cache(base_dir)?;
    let latest = cache.latest_version?;
    if is_newer(&latest, current) {
        Some(latest)
    } else {
        None
    }
}

/// 最後のチェックから CHECK_INTERVAL_SECS 以上経っていれば true。
pub fn should_check(base_dir: &Path, now: DateTime<Utc>) -> bool {
    match load_cache(base_dir) {
        Some(c) => (now - c.last_checked).num_seconds() >= CHECK_INTERVAL_SECS,
        None => true,
    }
}

/// 別プロセスでバックグラウンドチェックを起動して即座に返る（fire-and-forget）。
/// 既に 24h 以内にチェック済みなら何もしない。
pub fn schedule_check(base_dir: PathBuf) {
    use std::process::{Command, Stdio};

    let now = Utc::now();
    if !should_check(&base_dir, now) {
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    // デタッチされた子プロセスで実行。親が先に exit しても init に reparent される。
    let _ = Command::new(exe)
        .arg("__update-check")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// hidden サブコマンド `__update-check` から呼ばれる。
/// 同期的にチェック → キャッシュ保存して exit する。
pub async fn run_background_check(base_dir: &Path) {
    let latest = fetch_latest_version().await;
    tracing::info!("background update check: latest={latest:?}");
    save_cache(
        base_dir,
        &UpdateCache {
            last_checked: Utc::now(),
            latest_version: latest,
        },
    );
}

async fn fetch_latest_version() -> Option<String> {
    let resp: serde_json::Value = reqwest::Client::new()
        .get(API_URL)
        .header("User-Agent", "tamago-updater")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    resp["tag_name"]
        .as_str()
        .map(|s| s.trim_start_matches('v').to_string())
}

/// `a > b` を semver 風に（数値セグメントで）比較する。失敗時は文字列比較にフォールバック。
fn is_newer(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.')
            .map(|p| p.split('-').next().unwrap_or(""))
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    let va = parse(a);
    let vb = parse(b);
    if !va.is_empty() && !vb.is_empty() {
        va > vb
    } else {
        a > b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn is_newer_compares_semver() {
        assert!(is_newer("0.7.2", "0.7.1"));
        assert!(is_newer("0.8.0", "0.7.99"));
        assert!(is_newer("1.0.0", "0.99.0"));
        assert!(!is_newer("0.7.1", "0.7.1"));
        assert!(!is_newer("0.7.0", "0.7.1"));
    }

    #[test]
    fn pending_update_reports_newer_version() {
        let dir = TempDir::new().unwrap();
        save_cache(
            dir.path(),
            &UpdateCache {
                last_checked: Utc::now(),
                latest_version: Some("0.7.2".into()),
            },
        );
        assert_eq!(pending_update(dir.path(), "0.7.1"), Some("0.7.2".into()));
        assert_eq!(pending_update(dir.path(), "0.7.2"), None);
        assert_eq!(pending_update(dir.path(), "0.8.0"), None);
    }

    #[test]
    fn should_check_true_when_no_cache() {
        let dir = TempDir::new().unwrap();
        assert!(should_check(dir.path(), Utc::now()));
    }

    #[test]
    fn should_check_false_within_24h() {
        let dir = TempDir::new().unwrap();
        save_cache(
            dir.path(),
            &UpdateCache {
                last_checked: Utc::now() - chrono::Duration::hours(1),
                latest_version: None,
            },
        );
        assert!(!should_check(dir.path(), Utc::now()));
    }

    #[test]
    fn should_check_true_after_24h() {
        let dir = TempDir::new().unwrap();
        save_cache(
            dir.path(),
            &UpdateCache {
                last_checked: Utc::now() - chrono::Duration::hours(25),
                latest_version: None,
            },
        );
        assert!(should_check(dir.path(), Utc::now()));
    }
}
