use std::sync::Arc;

use clap::{Args, Subcommand};

use crate::client::BiolabClient;
use crate::config::Config;
use crate::output::{print_order, print_order_brief, print_result, OutputFormat};

#[derive(Args)]
pub struct OrdersArgs {
    #[command(subcommand)]
    pub command: OrdersCommand,
}

#[derive(Subcommand)]
pub enum OrdersCommand {
    /// 订单列表
    List {
        #[arg(short, long, default_value_t = 0)]
        skip: u32,
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
    },
    /// 订单详情
    Get { id: String },
    /// 创建引物合成订单（从 JSON 文件）
    CreatePrimer { file: String },
    /// 创建测序订单（从 JSON 文件）
    CreateSequencing { file: String },
    /// 更新订单
    Update { id: String, data: String },
    /// 重发邮件（pending 状态订单）
    Resend { id: String },
    /// 下载订单 Excel
    Download {
        id: String,
        #[arg(default_value = "order.xlsx")]
        output: String,
    },
    /// 下载引物 Excel 模板
    DownloadPrimerTemplate {
        #[arg(default_value = "primer_template.xlsx")]
        output: String,
    },
    /// 下载测序 Excel 模板
    DownloadSequencingTemplate {
        #[arg(default_value = "sequencing_template.xlsx")]
        output: String,
    },
    /// 上传引物 Excel 解析
    UploadPrimerExcel { file: String },
    /// 上传测序 Excel 解析
    UploadSequencingExcel { file: String },
}

pub async fn run(
    args: &OrdersArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = BiolabClient::new(Arc::clone(config))?;

    match &args.command {
        OrdersCommand::List { skip, limit } => {
            let orders = client.list_orders(*skip, *limit).await?;
            if orders.is_empty() {
                println!("暂无订单");
                return Ok(());
            }
            match format {
                OutputFormat::Json => print_result(&orders, format),
                OutputFormat::Text => {
                    for o in &orders {
                        print_order_brief(o);
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
            print_result(&result, &OutputFormat::Json);
        }
        OrdersCommand::Download { id, output } => {
            let bytes = client.download_order(id).await?;
            std::fs::write(output, &bytes)?;
            println!("已下载到 {output}");
        }
        OrdersCommand::DownloadPrimerTemplate { output } => {
            let bytes = client.download_primer_template().await?;
            std::fs::write(output, &bytes)?;
            println!("已下载到 {output}");
        }
        OrdersCommand::DownloadSequencingTemplate { output } => {
            let bytes = client.download_sequencing_template().await?;
            std::fs::write(output, &bytes)?;
            println!("已下载到 {output}");
        }
        OrdersCommand::UploadPrimerExcel { file } => {
            let result = client.upload_primer_excel(file).await?;
            print_result(&result, &OutputFormat::Json);
        }
        OrdersCommand::UploadSequencingExcel { file } => {
            let result = client.upload_sequencing_excel(file).await?;
            print_result(&result, &OutputFormat::Json);
        }
    }
    Ok(())
}
