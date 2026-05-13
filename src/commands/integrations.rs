use chrono::{Datelike, Duration, NaiveDate, TimeZone, Utc};
use clap::{Args, Subcommand};

use crate::api::ApiClient;

#[derive(Args)]
pub struct IntegrationsArgs {
    #[command(subcommand)]
    pub command: IntegrationsCommand,
}

#[derive(Subcommand)]
pub enum IntegrationsCommand {
    /// Get all events from the last X days
    Events {
        /// Number of days to look back, defaults to 0 (today)
        #[arg(value_name = "DAYS", default_value_t = 0)]
        days: i32,
    },
    /// Get summarized events from the last X days by status
    EventsByStatus {
        /// Number of days to look back, defaults to 0 (today)
        #[arg(value_name = "DAYS", default_value_t = 0)]
        days: i32,
    },
    /// Get summarized events by month X months ago, grouped by integration system and status.
    /// To get the last N months, use the --range flag which will make N separate requests.
    EventsByMonth {
        /// Number of months ago to fetch data from (0 = current month, 1 = last month, etc.)
        #[arg(value_name = "MONTHS", default_value_t = 0)]
        months: i32,
        /// Fetch data from the last N months (makes N separate requests, overrides months param if set)
        #[arg(long)]
        range: Option<i32>,
        /// Optional status filter
        #[arg(long)]
        status: Option<String>,
    },
}

pub fn execute(args: &IntegrationsArgs) {
    let client = ApiClient::new().expect("Run 'init' first.");

    let result = match &args.command {
        IntegrationsCommand::Events { days } => fetch_events(&client, *days),
        IntegrationsCommand::EventsByStatus { days } => fetch_events_by_status(&client, *days),
        IntegrationsCommand::EventsByMonth {
            months,
            range,
            status,
        } => fetch_events_by_month(&client, *months, range.as_ref(), status.as_deref()),
    };

    match result {
        Ok(body) => println!("{body}"),
        Err(e) => eprintln!("{e}"),
    }
}

fn fetch_events(client: &ApiClient, days: i32) -> Result<String, String> {
    let start_date = get_date_minus_days(days);

    // Build WQL query with proper formatting
    let wql_query = format!(
        "SELECT actualStartDateAndTime, integrationSystem.descriptor, status.descriptor, submittedByUser.descriptor \
         FROM allIntegrationEvents WHERE actualStartDateAndTime >= '{}'",
        start_date
    );

    client.get(&"wql", &"v1", &format!("data?query={wql_query}"))
}

fn fetch_events_by_status(client: &ApiClient, days: i32) -> Result<String, String> {
    let start_date = get_date_minus_days(days);

    // Build WQL query with GROUP BY for summarized events by status
    let wql_query = format!(
        "SELECT integrationSystem, status, count() FROM allIntegrationEvents \
         WHERE actualStartDateAndTime >= '{}' GROUP BY integrationSystem, status",
        start_date
    );

    // URL encode the query parameter
    let encoded_query = urlencoding::encode(&wql_query);

    client.get(&"wql", &"v1", &format!("data?query={encoded_query}"))
}

fn fetch_events_by_month(
    client: &ApiClient,
    months: i32,
    range: Option<&i32>,
    status: Option<&str>,
) -> Result<String, String> {
    // If range is specified, fetch data for the last N months
    if let Some(range_months) = range {
        let mut all_results = Vec::new();
        for i in 0..*range_months {
            let (start_date, end_date, year, month) = get_month_date_range_with_metadata(i);
            let result = build_and_fetch_monthly_query(client, &start_date, &end_date, status);
            match result {
                Ok(body) => {
                    // Wrap response with month and year metadata
                    let wrapped = format!(
                        "{{\"month\":{},\"year\":{},\"data\":{}}}",
                        month, year, body
                    );
                    all_results.push(wrapped);
                }
                Err(e) => {
                    eprintln!("Error fetching month {i}: {e}");
                    return Err(e);
                }
            }
        }
        Ok(format!("[{}]", all_results.join(",")))
    } else {
        // Fetch data for a single month
        let (start_date, end_date, year, month) = get_month_date_range_with_metadata(months);
        let result = build_and_fetch_monthly_query(client, &start_date, &end_date, status);
        match result {
            Ok(body) => {
                // Wrap response with month and year metadata
                Ok(format!(
                    "{{\"month\":{},\"year\":{},\"data\"{}}}",
                    month, year, body
                ))
            }
            Err(e) => Err(e),
        }
    }
}

/// Calculate date range for a specific month using chrono
/// If month is 0: start of current month to current moment
/// Otherwise: first day of month to last day of month
/// Returns: (start_date, end_date, year, month)
fn get_month_date_range_with_metadata(months_ago: i32) -> (String, String, u32, u32) {
    let now = Utc::now();
    let target_date = now - Duration::days((months_ago as i64) * 30);

    let year = target_date.year() as u32;
    let month = target_date.month();

    // First day of month
    let start = format!("{:04}-{:02}-01", year, month);

    if months_ago == 0 {
        // Current month: from start to current moment
        let end = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        (start, end, year, month)
    } else {
        // Past month: from first to last day of month
        // Calculate last day of month using chrono's NaiveDate
        let first_of_next_month = if month == 12 {
            NaiveDate::from_ymd_opt(year as i32 + 1, 1, 1).expect("Invalid date")
        } else {
            NaiveDate::from_ymd_opt(year as i32, month + 1, 1).expect("Invalid date")
        };
        let last_of_month = first_of_next_month - Duration::days(1);

        let end = format!("{}T23:59:59", last_of_month.format("%Y-%m-%d"));
        (start, end, year, month)
    }
}

/// Calculate date range for a specific month using chrono
/// If month is 0: start of current month to current moment
/// Otherwise: first day of month to last day of month
/// Build and execute the monthly WQL query
fn build_and_fetch_monthly_query(
    client: &ApiClient,
    start_date: &str,
    end_date: &str,
    status: Option<&str>,
) -> Result<String, String> {
    let mut wql_query = format!(
        "SELECT integrationSystem, status, count() FROM allIntegrationEvents \
         WHERE actualStartDateAndTime >= '{}' AND actualStartDateAndTime <= '{}'",
        start_date, end_date
    );

    // Add optional status filter using IN clause
    if let Some(s) = status {
        wql_query.push_str(&format!(" AND status IN ({})", s));
    }

    wql_query.push_str(" GROUP BY integrationSystem, status");

    // URL encode the query parameter
    let encoded_query = urlencoding::encode(&wql_query);

    client.get(&"wql", &"v1", &format!("data?query={encoded_query}"))
}

/// Calculate current date minus number of days at 00:00:00 UTC
/// Returns format: YYYY-MM-DDTHH:MM:SSZ
pub fn get_date_minus_days(days: i32) -> String {
    let now = Utc::now();
    let target_date = now - Duration::days(days as i64);
    let date_at_midnight = target_date
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("Invalid date");
    let utc_datetime = Utc.from_utc_datetime(&date_at_midnight);
    utc_datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
