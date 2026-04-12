use clap::{Parser, Subcommand};

pub mod api;
mod commands;
mod config;
pub mod security;

#[derive(Parser)]
#[command(name = "workday_cli", version, about = "Workday CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the CLI with Workday API credentials
    Init(commands::init::InitArgs),
    /// Get a worker by WID or use "me" for the current user
    Worker(commands::worker::WorkerArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => commands::init::execute(args),
        Commands::Worker(args) => commands::worker::execute(args),
    }
}
