use std::path::{Path, PathBuf};
use std::{fs, io};

use clap::Subcommand;

#[derive(Subcommand)]
pub enum SkillCommand {
    /// Claude Code 用スキルファイルをインストール
    Install {
        /// プロジェクトローカル (.claude/skills/) にインストール
        #[arg(long, short)]
        project: bool,
    },
}

const SKILL_MD: &str = r#"---
name: tamago
description: >
  Terminal pet (tamago) CLI integration.
  Use when the user mentions their pet, wants to see pet status,
  rename their pet, or manage tamago.
  Triggers: "ペットを見せて", "show my pet", "ペットの名前",
  "pet status", "tamago", "ペット"
---

# tamago - Terminal Pet CLI

tamago はターミナルで育てるCLIペット。シェルコマンドや Claude Code の利用で経験値が溜まり、成長・進化する。

## 表示ルール

`tamago` の出力はコードブロックでそのまま表示すること。
Bash ツールの結果は折りたたまれるため、必ず出力をメッセージ本文のコードブロックに含める。

## コマンド一覧

### `tamago` (引数なし)
ペットの全ステータスを AA + ステータスバー付きで表示。
未集計の activity を集計し、decay・進化判定も行う。
「ペットを見せて」「show my pet」で使う。

### `tamago status`
statusline 用の1行出力（emoji + 名前 + 種族 + Lv + mood + hunger + EXP）。

### `tamago name <name>`
ペットの名前を変更する。

### `tamago name --ai`
Claude にペットの名前を考えさせる。
「名前を考えて」「name my pet」で使う。

### `tamago init`
初期セットアップ。卵を生成してガイドを表示。
ペットが存在しない場合のみ使う。

### `tamago reset`
全データを削除して初期化しなおす。
**ユーザーの明示的な確認なしに実行しないこと。**

## ペットのライフサイクル

- 段階: Egg → Baby → Child → Teen → Adult
- 進化は累積 EXP に基づく
- Adult の archetype はカテゴリ別 EXP の最大値で決定
- hunger / mood は時間経過で減衰し、ターミナル操作で回復
"#;

pub fn run(cmd: &SkillCommand) {
    match cmd {
        SkillCommand::Install { project } => {
            let base = if *project {
                std::env::current_dir().expect("カレントディレクトリの取得に失敗")
            } else {
                dirs::home_dir().expect("ホームディレクトリが見つかりません")
            };

            match install_to(&base) {
                Ok(path) => println!("installed: {}", path.display()),
                Err(e) => {
                    eprintln!("tamago: スキルのインストールに失敗: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn install_to(base_dir: &Path) -> io::Result<PathBuf> {
    let dir = base_dir.join(".claude/skills/tamago");
    fs::create_dir_all(&dir)?;
    let path = dir.join("SKILL.md");
    fs::write(&path, SKILL_MD)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn install_creates_skill_md() {
        let dir = TempDir::new().unwrap();
        let path = install_to(dir.path()).unwrap();

        assert!(path.exists());
        assert_eq!(path.file_name().unwrap(), "SKILL.md");

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("name: tamago"));
        assert!(content.contains("tamago status"));
    }

    #[test]
    fn install_creates_nested_directories() {
        let dir = TempDir::new().unwrap();
        let base = dir.path().join("deep/nested");
        // base itself doesn't exist yet
        let path = install_to(&base).unwrap();

        assert!(path.exists());
        assert!(base.join(".claude/skills/tamago/SKILL.md").exists());
    }

    #[test]
    fn install_overwrites_existing() {
        let dir = TempDir::new().unwrap();
        let path = install_to(dir.path()).unwrap();
        fs::write(&path, "old content").unwrap();

        let path2 = install_to(dir.path()).unwrap();
        let content = fs::read_to_string(&path2).unwrap();
        assert!(content.contains("name: tamago"));
    }
}
