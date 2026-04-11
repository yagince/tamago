use clap::Parser;

mod claude;
mod cmd;
mod pet;
mod storage;
mod tracker;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = cmd::Cli::parse();
    let storage = storage::Storage::default();
    cmd::run(cli, storage).await;
}
