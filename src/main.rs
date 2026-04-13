use clap::Parser;

mod cmd;
mod config;
mod llm;
mod logger;
mod pet;
mod storage;
mod tracker;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = cmd::Cli::parse();
    let storage = storage::Storage::default();
    logger::init(storage.base_dir());
    cmd::run(cli, storage).await;
}
