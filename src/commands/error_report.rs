use std::sync::Arc;

use clap::Args;
use colored::Colorize;

use crate::client::BiolabClient;
use crate::config::Config;
use crate::output::{print_result, OutputFormat};
use crate::types::{ErrorCategory, ErrorReportCreate};

#[derive(Args)]
pub struct ErrorReportArgs {
    /// Error category.
    #[arg(short, long, value_enum)]
    category: ErrorCategoryArg,
    /// Short title.
    #[arg(short, long)]
    title: String,
    /// Detailed description.
    #[arg(short, long)]
    description: String,
    /// Optional related URL.
    #[arg(short, long)]
    url: Option<String>,
}

#[derive(clap::ValueEnum, Clone)]
enum ErrorCategoryArg {
    #[value(name = "ui-display")]
    UiDisplay,
    #[value(name = "functional")]
    Functional,
    #[value(name = "data")]
    Data,
    #[value(name = "performance")]
    Performance,
    #[value(name = "permission")]
    Permission,
    #[value(name = "other")]
    Other,
}

impl From<&ErrorCategoryArg> for ErrorCategory {
    fn from(val: &ErrorCategoryArg) -> Self {
        match val {
            ErrorCategoryArg::UiDisplay => ErrorCategory::UiDisplay,
            ErrorCategoryArg::Functional => ErrorCategory::Functional,
            ErrorCategoryArg::Data => ErrorCategory::Data,
            ErrorCategoryArg::Performance => ErrorCategory::Performance,
            ErrorCategoryArg::Permission => ErrorCategory::Permission,
            ErrorCategoryArg::Other => ErrorCategory::Other,
        }
    }
}

pub async fn run(
    args: &ErrorReportArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = BiolabClient::new(Arc::clone(config))?;
    let user_agent = crate_user_agent();

    let report = ErrorReportCreate {
        category: ErrorCategory::from(&args.category),
        title: args.title.clone(),
        description: args.description.clone(),
        url: args.url.clone(),
        user_agent: Some(user_agent),
    };

    let resp = client.post_error_report(&report).await?;

    match format {
        OutputFormat::Json => print_result(&resp, format),
        OutputFormat::Text => {
            println!(
                "{}  错误报告已提交（ID: {}）",
                "✓".green(),
                resp.id
            );
        }
    }

    Ok(())
}

pub fn crate_user_agent() -> String {
    let os = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "unknown"
    };
    format!("biolab-cli/{} ({})", env!("CARGO_PKG_VERSION"), os)
}
