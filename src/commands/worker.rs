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

    /// Get worker payslip information
    Payslips {
        /// Worker ID or "me" for the current authenticated user
        #[arg(value_name = "WID")]
        wid: String,
        /// Number of payslips to get (default: 1)
        #[arg(default_value_t = 1)]
        limit: i16,
    },

    /// Get worker history events
    History {
        /// Worker ID or "me" for the current authenticated user
        #[arg(value_name = "WID")]
        wid: String,
        /// Number of history events to get (default: 5)
        #[arg(default_value_t = 5)]
        limit: i16,
    },
}

pub fn execute(args: &WorkerArgs) {
    let client = ApiClient::new().expect("Run 'init' first.");

    let result = match &args.command {
        WorkerCommand::Details { wid } => client.get(&"common", &"v1", &format!("workers/{wid}")),
        WorkerCommand::SearchWorker { name } => {
            client.get(&"common", &"v1", &format!("workers?search={name}"))
        }
        WorkerCommand::DirectReports { wid } => {
            client.get(&"common", &"v1", &format!("workers/{wid}/directReports"))
        }
        WorkerCommand::TimeOff { wid } => {
            client.get(&"common", &"v1", &format!("workers/{wid}/timeOff"))
        }
        WorkerCommand::Payslips { wid, limit } => client.get(
            &"common",
            &"v1",
            &format!("workers/{wid}/paySlips?limit={limit}"),
        ),
        WorkerCommand::History { wid, limit } => {
            client.get(&"common", &"v1", "workers/{wid}/history?limit={limit}")
        }
    };

    match result {
        Ok(body) => println!("{body}"),
        Err(e) => eprintln!("{e}"),
    }
}
