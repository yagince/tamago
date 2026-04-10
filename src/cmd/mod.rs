mod hook;
pub(crate) mod init;
mod name;
mod reset;
mod show;
mod show_tui;
mod skill;
mod status;
mod tick;
mod update;

use clap::{Parser, Subcommand};

use crate::storage::Storage;

#[derive(Parser)]
#[command(name = "tamago", about = "CLI で育てるターミナルペット", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// 初期セットアップ（卵生成 + ガイド表示）
    Init,
    /// 命名・改名
    Name {
        /// ペットの名前
        name: Option<String>,
        /// Claude に名前を考えさせる
        #[arg(long)]
        ai: bool,
    },
    /// ドラクエ風ステータス画面で表示
    Show {
        /// セリフの更新間隔（秒）
        #[arg(long, default_value = "30")]
        message_interval: u64,
    },
    /// データをリセットして初期化しなおす
    Reset,
    /// statusline 用ワンライナー
    Status,
    /// フックスクリプトを stdout に出力
    Hook {
        /// 対象シェル
        shell: hook::Shell,
    },
    /// Claude Code 用スキルを管理
    Skill {
        #[command(subcommand)]
        command: skill::SkillCommand,
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
        /// Claude の出力トークン数（経験値計算用）
        #[arg(long)]
        output_tokens: Option<u64>,
    },
    /// 最新バージョンに更新
    Update,
}

pub fn run(cli: Cli, storage: Storage) {
    match cli.command {
        None => show::run(&storage),
        Some(Command::Init) => init::run(&storage),
        Some(Command::Name { name, ai }) => name::run(&storage, name.as_deref(), ai),
        Some(Command::Show { message_interval }) => show_tui::run(&storage, message_interval),
        Some(Command::Reset) => reset::run(&storage),
        Some(Command::Status) => status::run(&storage),
        Some(Command::Hook { shell }) => hook::run(&shell),
        Some(Command::Skill { command }) => skill::run(&command),
        Some(Command::Tick {
            cmd,
            claude_turn,
            output_tokens,
        }) => tick::run(&storage, cmd.as_deref(), claude_turn, output_tokens),
        Some(Command::Update) => update::run(),
    }
}
