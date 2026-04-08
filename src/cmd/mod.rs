mod init;

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
}

pub fn run(cli: Cli, storage: Storage) {
    match cli.command {
        None => show(),
        Some(Command::Init) => init::run(&storage),
    }
}

fn show() {
    todo!("show")
}
