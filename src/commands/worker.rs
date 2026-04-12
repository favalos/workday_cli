use clap::{Args, Subcommand};

use crate::api::ApiClient;

#[derive(Args)]
pub struct WorkerArgs {
    #[command(subcommand)]
    pub command: WorkerCommand,
}

#[derive(Subcommand)]
pub enum WorkerCommand {
    /// Get worker details
    Details {
        /// Worker ID or "me" for the current authenticated user
        #[arg(value_name = "WID")]
        wid: String,
    },
    /// Search worker by name
    SearchWorker {
        /// Worker partial name for search
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Get worker direct reports
    DirectReports {
        /// Worker ID or "me" for the current authenticated user
        #[arg(value_name = "WID")]
        wid: String,
    },
    /// Get worker time off
    TimeOff {
        /// Worker ID or "me" for the current authenticated user
        #[arg(value_name = "WID")]
        wid: String,
    },
}

pub fn execute(args: &WorkerArgs) {
    let client = ApiClient::new().expect("Run 'init' first.");

    let result = match &args.command {
        WorkerCommand::Details { wid } => client.get(&format!("workers/{wid}")),
        WorkerCommand::SearchWorker { name } => client.get(&format!("workers?search={name}")),
        WorkerCommand::DirectReports { wid } => client.get(&format!("workers/{wid}/directReports")),
        WorkerCommand::TimeOff { wid } => client.get(&format!("workers/{wid}/timeOff")),
    };

    match result {
        Ok(body) => println!("{body}"),
        Err(e) => eprintln!("{e}"),
    }
}
