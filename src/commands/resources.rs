use clap::{Args, Subcommand};

use crate::api::ApiClient;

#[derive(Args)]
pub struct ResourcesArgs {
    #[command(subcommand)]
    pub command: ResourcesCommand,
}

#[derive(Subcommand)]
pub enum ResourcesCommand {
    /// Search for Job Profiles by name (e.g. "Integration Engineer")
    JobProfiles {
        /// Name to search for (e.g. "Integration Engineer")
        #[arg(value_name = "SEARCH")]
        search: String,
    },
    /// Search for position time types by name (e.g. "Full Time", "Part Time")
    TimeTypes {
        /// Name to search for (e.g. "Full Time")
        #[arg(value_name = "SEARCH")]
        search: String,
    },
    /// Search for locations by name (e.g. "San Fran", "New York")
    Locations {
        /// Name to search for (e.g. "San Fran")
        #[arg(value_name = "SEARCH")]
        search: String,
    },
    /// Search for supervisory organizations by name (e.g. "Information Technology")
    SupervisoryOrg {
        /// Name to search for (e.g. "Information Technology")
        #[arg(value_name = "SEARCH")]
        search: String,
    },
    /// Search for employee types by name (e.g. "Regular", "Temporary")
    EmployeeTypes {
        /// Name to search for (e.g. "Regular")
        #[arg(value_name = "SEARCH")]
        search: String,
    },
}

pub fn execute(args: &ResourcesArgs) {
    let client = ApiClient::new().expect("Run 'init' first.");

    let result = match &args.command {
        ResourcesCommand::JobProfiles { search } => fetch_job_profiles(&client, search),
        ResourcesCommand::TimeTypes { search } => fetch_time_types(&client, search),
        ResourcesCommand::Locations { search } => fetch_locations(&client, search),
        ResourcesCommand::SupervisoryOrg { search } => fetch_supervisory_org(&client, search),
        ResourcesCommand::EmployeeTypes { search } => fetch_employee_types(&client, search),
    };

    match result {
        Ok(body) => println!("{body}"),
        Err(e) => eprintln!("{e}"),
    }
}

fn fetch_job_profiles(client: &ApiClient, search: &str) -> Result<String, String> {
    let encoded = urlencoding::encode(search);
    client.get(
        "staffing",
        "v7",
        &format!("values/jobChangesGroup/jobProfiles/?search={encoded}"),
    )
}

fn fetch_time_types(client: &ApiClient, search: &str) -> Result<String, String> {
    let encoded = urlencoding::encode(search);
    client.get(
        "staffing",
        "v7",
        &format!("values/jobChangesGroup/timeTypes/?search={encoded}"),
    )
}

fn fetch_locations(client: &ApiClient, search: &str) -> Result<String, String> {
    let encoded = urlencoding::encode(search);
    client.get(
        "staffing",
        "v7",
        &format!("values/jobChangesGroup/locations/?search={encoded}"),
    )
}

fn fetch_supervisory_org(client: &ApiClient, search: &str) -> Result<String, String> {
    let encoded = urlencoding::encode(search);
    client.get(
        "staffing",
        "v7",
        &format!("values/jobChangesGroup/supervisoryOrganization/?search={encoded}"),
    )
}

fn fetch_employee_types(client: &ApiClient, search: &str) -> Result<String, String> {
    let encoded = urlencoding::encode(search);
    client.get(
        "staffing",
        "v7",
        &format!("values/jobChangesGroup/employeeTypes/?search={encoded}"),
    )
}
