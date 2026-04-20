use crate::api::ApiClient;
use clap::{Args, Subcommand};
use std::fs;

#[derive(Args)]
pub struct RevenueArgs {
    #[command(subcommand)]
    pub command: RevenueCommand,
}

#[derive(Subcommand)]
pub enum RevenueCommand {
    /// Search for a customer by name
    Customer {
        /// Customer name to search for
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Get invoices for a customer
    Invoices {
        /// Customer ID
        #[arg(value_name = "CUSTOMER_ID")]
        customer_id: String,
        /// Number of invoices to get (default: 10)
        #[arg(default_value_t = 1)]
        limit: i16,
    },

    /// Get Print Run for Invoice
    InvoicePrint {
        /// Customer ID
        #[arg(value_name = "INVOICE_ID")]
        invoice_id: String,
        /// Number of invoices to get (default: 10)
        #[arg(default_value_t = 1)]
        limit: i16,
    },

    /// Download an invoice PDF by ID
    InvoicePdf {
        /// Invoice ID
        #[arg(value_name = "INVOICE_ID")]
        invoice_id: String,
        /// Output file path (default: invoice_<ID>.pdf)
        #[arg(short, long)]
        output: Option<String>,
    },
}

pub fn execute(args: &RevenueArgs) {
    let client = ApiClient::new().expect("Run 'init' first.");

    match &args.command {
        RevenueCommand::Customer { name } => {
            let result = client.get(
                "customerAccounts",
                "v1",
                &format!("customers?search={name}"),
            );
            match result {
                Ok(body) => println!("{body}"),
                Err(e) => eprintln!("{e}"),
            }
        }

        RevenueCommand::Invoices { customer_id, limit } => {
            let result = client.get(
                "customerAccounts",
                "v1",
                &format!("invoices?billToCustomer={customer_id}&limit={limit}"),
            );
            match result {
                Ok(body) => println!("{body}"),
                Err(e) => eprintln!("{e}"),
            }
        }

        RevenueCommand::InvoicePrint { invoice_id, limit } => {
            let result = client.get(
                "customerAccounts",
                "v1",
                &format!("invoices/{invoice_id}/printRuns?limit={limit}"),
            );
            match result {
                Ok(body) => println!("{body}"),
                Err(e) => eprintln!("{e}"),
            }
        }

        RevenueCommand::InvoicePdf { invoice_id, output } => {
            let result = client.get_bytes(
                "customerAccounts",
                "v1",
                &format!("invoicePDFs/{invoice_id}"),
            );
            match result {
                Ok(bytes) => {
                    let path = output
                        .clone()
                        .unwrap_or_else(|| format!("invoice_{invoice_id}.pdf"));
                    match fs::write(&path, bytes) {
                        Ok(_) => println!("Saved to {path}"),
                        Err(e) => eprintln!("Failed to write file: {e}"),
                    }
                }
                Err(e) => eprintln!("{e}"),
            }
        }
    }
}
