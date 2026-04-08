mod hook;
pub(crate) mod init;
mod reset;
mod tick;

use clap::{Parser, Subcommand};

use crate::storage::Storage;

#[derive(Parser)]
#[command(name = "tamago", about = "CLI で育てるターミナルペット")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// 初期セットアップ（卵生成 + ガイド表示）
    Init,
    /// データをリセットして初期化しなおす
    Reset,
    /// フックスクリプトを stdout に出力
    Hook {
        /// 対象シェル
        shell: hook::Shell,
    },
    /// フックから呼ばれる（内部用）
    #[command(hide = true)]
    Tick {
        /// 実行されたコマンド
        #[arg(long)]
        cmd: Option<String>,
        /// Claude Code ターン記録
        #[arg(long)]
        claude_turn: bool,
    },
}

pub fn run(cli: Cli, storage: Storage) {
    match cli.command {
        None => show(),
        Some(Command::Init) => init::run(&storage),
        Some(Command::Reset) => reset::run(&storage),
        Some(Command::Hook { shell }) => hook::run(&shell),
        Some(Command::Tick { cmd, claude_turn }) => {
            tick::run(&storage, cmd.as_deref(), claude_turn)
        }
    }
}

fn show() {
    todo!("show")
}
