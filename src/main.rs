use clap::Parser;

mod claude;
mod cmd;
mod pet;
mod storage;
mod tracker;

fn main() {
    let cli = cmd::Cli::parse();
    let storage = storage::Storage::default();
    cmd::run(cli, storage);
}
