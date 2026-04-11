const REPO: &str = "yagince/tamago";
const API_URL: &str = "https://api.github.com/repos/yagince/tamago/releases/latest";

pub async fn run() {
    let current = env!("CARGO_PKG_VERSION");
    println!("現在のバージョン: v{current}");

    let latest = match fetch_latest_version().await {
        Some(v) => v,
        None => {
            eprintln!("最新バージョンの取得に失敗しました");
            return;
        }
    };

    let latest_clean = latest.trim_start_matches('v');
    if latest_clean <= current {
        println!("最新バージョンです ✓");
        return;
    }

    println!("新しいバージョンがあります: {latest}");
    println!("更新中...");

    if let Err(e) = download_and_install(&latest).await {
        eprintln!("更新に失敗しました: {e}");
        eprintln!("手動でインストールしてください:");
        eprintln!("  brew upgrade tamago");
    }
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

    resp["tag_name"].as_str().map(String::from)
}

async fn download_and_install(tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let target = detect_target();
    let asset = format!("tamago-{tag}-{target}.tar.gz");
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");

    let bytes = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "tamago-updater")
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    // tar.gz を Rust 側で展開
    let decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);

    let mut found = false;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.file_name().and_then(|n| n.to_str()) == Some("tamago") {
            let dest = std::env::current_exe()?;
            let tmp = dest.with_extension("new");

            // 新バイナリを一旦 tmp に書き出す
            let mut out = std::fs::File::create(&tmp)?;
            std::io::copy(&mut entry, &mut out)?;
            drop(out);

            // 実行権限を付与
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
            }

            // 古いバイナリを置換
            std::fs::rename(&tmp, &dest)?;

            found = true;
            break;
        }
    }

    if !found {
        return Err("アーカイブ内に tamago バイナリが見つかりません".into());
    }

    println!("更新完了: {tag} ✓");
    Ok(())
}

fn detect_target() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else {
            "x86_64-apple-darwin"
        }
    } else if cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-musl"
    } else {
        "x86_64-unknown-linux-musl"
    }
}
