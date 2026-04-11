use clap::Parser;

mod cmd;
mod config;
mod llm;
mod pet;
mod storage;
mod tracker;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = cmd::Cli::parse();
    let storage = storage::Storage::default();
    cmd::run(cli, storage).await;
}
