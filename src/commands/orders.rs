use std::sync::Arc;

use clap::{Args, Subcommand};

use crate::client::ScientexClient;
use crate::config::Config;
use crate::output::{
    print_order, print_order_brief, print_pagination_metadata, print_result, unique_output_path,
    OutputFormat,
};

#[derive(Args)]
pub struct OrdersArgs {
    #[command(subcommand)]
    pub command: OrdersCommand,
}

#[derive(Subcommand)]
pub enum OrdersCommand {
    /// Show order statistics.
    Stats {
        #[arg(long)]
        start_date: Option<String>,
        #[arg(long)]
        end_date: Option<String>,
    },
    /// List orders.
    List {
        #[arg(short, long, default_value_t = 0)]
        skip: u32,
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        order_type: Option<String>,
        #[arg(long)]
        supplier_name: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        price_min: Option<String>,
        #[arg(long)]
        price_max: Option<String>,
        /// Inclusive ISO 8601 date or datetime lower bound.
        #[arg(long)]
        date_from: Option<String>,
        /// Inclusive ISO 8601 date or datetime upper bound.
        #[arg(long)]
        date_to: Option<String>,
    },
    /// List orders waiting for my approval.
    PendingApprovals,
    /// Show order details.
    Get { id: String },
    /// Create a primer synthesis order from a JSON file.
    CreatePrimer { file: String },
    /// Create a sequencing order from a JSON file.
    CreateSequencing { file: String },
    /// Update an order with a JSON object.
    Update { id: String, data: String },
    /// Resend order email for pending orders.
    Resend { id: String },
    /// Send order email.
    Send { id: String },
    /// Approve an order.
    Approve { id: String },
    /// Reject an order.
    Reject { id: String },
    /// Download order Excel.
    Download {
        id: String,
        #[arg(default_value = "order.xlsx")]
        output: String,
    },
    /// Download primer Excel template.
    DownloadPrimerTemplate {
        #[arg(default_value = "primer_template.xlsx")]
        output: String,
    },
    /// Download sequencing Excel template.
    DownloadSequencingTemplate {
        #[arg(default_value = "sequencing_template.xlsx")]
        output: String,
    },
    /// Upload and parse primer Excel.
    UploadPrimerExcel { file: String },
    /// Upload and parse sequencing Excel.
    UploadSequencingExcel { file: String },
}

pub async fn run(
    args: &OrdersArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;

    match &args.command {
        OrdersCommand::Stats {
            start_date,
            end_date,
        } => {
            let stats = client
                .get_order_stats(start_date.as_deref(), end_date.as_deref())
                .await?;
            print_result(&stats, format);
        }
        OrdersCommand::List {
            skip,
            limit,
            order_type,
            supplier_name,
            status,
            price_min,
            price_max,
            date_from,
            date_to,
        } => {
            let orders = client
                .list_orders(
                    *skip,
                    *limit,
                    order_type.as_deref(),
                    supplier_name.as_deref(),
                    status.as_deref(),
                    price_min.as_deref(),
                    price_max.as_deref(),
                    date_from.as_deref(),
                    date_to.as_deref(),
                )
                .await?;
            match format {
                OutputFormat::Json => print_result(&orders, format),
                OutputFormat::Text => {
                    print_pagination_metadata(&orders);
                    if orders.items.is_empty() {
                        println!("No orders");
                    } else {
                        for o in &orders.items {
                            print_order_brief(o);
                        }
                    }
                }
            }
        }
        OrdersCommand::PendingApprovals => {
            let orders = client.list_pending_approvals().await?;
            match format {
                OutputFormat::Json => print_result(&orders, format),
                OutputFormat::Text => {
                    print_pagination_metadata(&orders);
                    if orders.items.is_empty() {
                        println!("No pending approvals");
                    } else {
                        for o in &orders.items {
                            print_order_brief(o);
                        }
                    }
                }
            }
        }
        OrdersCommand::Get { id } => {
            let order = client.get_order(id).await?;
            match format {
                OutputFormat::Json => print_result(&order, format),
                OutputFormat::Text => print_order(&order),
            }
        }
        OrdersCommand::CreatePrimer { file } => {
            let content = std::fs::read_to_string(file)?;
            let order: serde_json::Value = serde_json::from_str(&content)?;
            let result = client.create_primer_order(&order).await?;
            match format {
                OutputFormat::Json => print_result(&result, format),
                OutputFormat::Text => print_order(&result),
            }
        }
        OrdersCommand::CreateSequencing { file } => {
            let content = std::fs::read_to_string(file)?;
            let order: serde_json::Value = serde_json::from_str(&content)?;
            let result = client.create_sequencing_order(&order).await?;
            match format {
                OutputFormat::Json => print_result(&result, format),
                OutputFormat::Text => print_order(&result),
            }
        }
        OrdersCommand::Update { id, data } => {
            let data: serde_json::Value = serde_json::from_str(data)?;
            let result = client.update_order(id, &data).await?;
            match format {
                OutputFormat::Json => print_result(&result, format),
                OutputFormat::Text => print_order(&result),
            }
        }
        OrdersCommand::Resend { id } => {
            let result = client.resend_order(id).await?;
            print_result(&result, format);
        }
        OrdersCommand::Send { id } => {
            let result = client.send_order(id).await?;
            print_result(&result, format);
        }
        OrdersCommand::Approve { id } => {
            let result = client.approve_order(id).await?;
            print_result(&result, format);
        }
        OrdersCommand::Reject { id } => {
            let result = client.reject_order(id).await?;
            print_result(&result, format);
        }
        OrdersCommand::Download { id, output } => {
            let bytes = client.download_order(id).await?;
            let output_path = unique_output_path(output);
            std::fs::write(&output_path, &bytes)?;
            println!("Downloaded to {}", output_path.display());
        }
        OrdersCommand::DownloadPrimerTemplate { output } => {
            let bytes = client.download_primer_template().await?;
            let output_path = unique_output_path(output);
            std::fs::write(&output_path, &bytes)?;
            println!("Downloaded to {}", output_path.display());
        }
        OrdersCommand::DownloadSequencingTemplate { output } => {
            let bytes = client.download_sequencing_template().await?;
            let output_path = unique_output_path(output);
            std::fs::write(&output_path, &bytes)?;
            println!("Downloaded to {}", output_path.display());
        }
        OrdersCommand::UploadPrimerExcel { file } => {
            let result = client.upload_primer_excel(file).await?;
            print_result(&result, format);
        }
        OrdersCommand::UploadSequencingExcel { file } => {
            let result = client.upload_sequencing_excel(file).await?;
            print_result(&result, format);
        }
    }
    Ok(())
}
