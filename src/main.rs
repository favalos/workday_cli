use clap::{Parser, Subcommand};

pub mod api;
mod commands;
mod config;
pub mod security;

#[derive(Parser)]
#[command(
    name = "workday_cli",
    version,
    about = "Workday CLI tool",
    after_help = "Need help? Open an issue: https://github.com/favalos/workday_cli/issues\n\
                  Or reach out on LinkedIn: https://www.linkedin.com/in/favalosg/"
)]
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
    /// Revenue commands: customers, invoices, and invoice PDFs
    Revenue(commands::revenue::RevenueArgs),
    /// Integration commands: list, details, status, and logs
    Integrations(commands::integrations::IntegrationsArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => commands::init::execute(args),
        Commands::Worker(args) => commands::worker::execute(args),
        Commands::Revenue(args) => commands::revenue::execute(args),
        Commands::Integrations(args) => commands::integrations::execute(args),
    }
}
