use std::sync::Arc;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use scitex_cli::commands::{
    admin, error_report, files, inventory, lab, orders, primers, project, projects, skills, tasks,
    templates, update, users,
};
use scitex_cli::config::Config;
use scitex_cli::error_history::ErrorHistory;
use scitex_cli::errors::ScientexError;
use scitex_cli::output::OutputFormat;
use scitex_cli::types::{ErrorCategory, ErrorReportCreate};
use scitex_cli::{check_status, login, logout};

/// Scientex lab management CLI.
#[derive(Parser)]
#[command(name = "scitex", version, about, long_about = None)]
struct Cli {
    /// Output format.
    #[arg(short, long, value_enum, default_value_t = OutputFormatArg::Text, global = true)]
    format: OutputFormatArg,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, ValueEnum)]
enum OutputFormatArg {
    Text,
    Json,
}

impl From<&OutputFormatArg> for OutputFormat {
    fn from(val: &OutputFormatArg) -> Self {
        match val {
            OutputFormatArg::Text => OutputFormat::Text,
            OutputFormatArg::Json => OutputFormat::Json,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Feishu OAuth login.
    Login,
    /// Log out and remove the local token.
    Logout,
    /// Check login status.
    Status,

    /// Current user management.
    Me(users::MeArgs),

    /// Upload reusable task input files.
    Files(files::FilesArgs),

    /// Order management.
    Orders(orders::OrdersArgs),

    /// Primer records from synthesis orders.
    Primers(primers::PrimersArgs),

    /// Template management.
    Templates(templates::TemplatesArgs),

    /// Inventory management.
    Inventory(inventory::InventoryArgs),

    /// Lab management.
    Lab(lab::LabArgs),

    /// Project-scoped workflows by slug.
    Project(project::ProjectArgs),

    /// Project management.
    Projects(projects::ProjectsArgs),

    /// Task management.
    Tasks(tasks::TasksArgs),

    /// Platform administration for tasks, task types, users, and reports.
    Admin(admin::AdminArgs),

    /// AI agent skill installation and checks.
    Skills(skills::SkillsArgs),

    /// Check CLI updates.
    Update(update::UpdateArgs),

    /// Submit an error report.
    ErrorReport(error_report::ErrorReportArgs),
}

/// Set Windows console output to UTF-8 so that Chinese and other Unicode text
/// renders correctly when the CLI prints to a terminal.
#[cfg(windows)]
fn setup_console_utf8() {
    use windows_sys::Win32::System::Console::SetConsoleOutputCP;
    unsafe { SetConsoleOutputCP(65001) };
}

#[cfg(not(windows))]
fn setup_console_utf8() {}

#[tokio::main]
async fn main() {
    setup_console_utf8();
    let cli = Cli::parse();
    let config = Arc::new(Config::new());
    let format = OutputFormat::from(&cli.format);

    let result = match cli.command {
        None => {
            println!("{}", "Scientex CLI".bold());
            println!("\nRun scitex --help to see available commands.\n");
            return;
        }
        Some(Commands::Login) => {
            if !login(&config).await {
                std::process::exit(1);
            }
            Ok(())
        }
        Some(Commands::Logout) => {
            logout(&config);
            Ok(())
        }
        Some(Commands::Status) => {
            let logged_in = check_status(&config).await;
            if !logged_in {
                std::process::exit(1);
            }
            Ok(())
        }
        Some(Commands::Me(args)) => users::run(&args, &config, &format).await,
        Some(Commands::Files(args)) => files::run(&args, &config, &format).await,
        Some(Commands::Orders(args)) => orders::run(&args, &config, &format).await,
        Some(Commands::Primers(args)) => primers::run(&args, &config, &format).await,
        Some(Commands::Templates(args)) => templates::run(&args, &config, &format).await,
        Some(Commands::Inventory(args)) => inventory::run(&args, &config, &format).await,
        Some(Commands::Lab(args)) => lab::run(&args, &config, &format).await,
        Some(Commands::Project(args)) => project::run(&args, &config, &format).await,
        Some(Commands::Projects(args)) => projects::run(&args, &config, &format).await,
        Some(Commands::Tasks(args)) => tasks::run(&args, &config, &format).await,
        Some(Commands::Admin(args)) => admin::run(&args, &config, &format).await,
        Some(Commands::Skills(args)) => skills::run(&args, &format),
        Some(Commands::Update(args)) => update::run(&args, &format).await,
        Some(Commands::ErrorReport(args)) => error_report::run(&args, &config, &format).await,
    };

    if let Err(e) = result {
        let cmd = command_context();
        let fingerprint = error_fingerprint(&e, &cmd);

        // Always record the error locally
        let mut history = ErrorHistory::load();
        let sanitized_error = sanitize_error_text(&e.to_string());
        history.record(&fingerprint, &cmd, error_type_label(&e), &sanitized_error);

        eprintln!("{}: {e}", "Error".red().bold());

        // If the same error keeps happening, offer to report it
        if history.check_threshold(&fingerprint, 10, 3) {
            if prompt_yn("检测到同类错误反复出现。是否上报错误详情帮助改进？")
            {
                match submit_error_report(&config, &e, &cmd).await {
                    Ok(report_id) => {
                        eprintln!("{} 错误已上报（ID: {}）", "✓".green(), report_id);
                    }
                    Err(report_err) => {
                        eprintln!("{}: {report_err}", "上报失败".yellow());
                    }
                }
            }
        }

        std::process::exit(1);
    }
}

fn command_context() -> String {
    let args: Vec<String> = std::env::args().collect();
    command_context_from_args(&args)
}

fn command_context_from_args(args: &[String]) -> String {
    if args.len() < 2 {
        return "scitex".to_string();
    }
    let mut ctx = String::from("scitex");
    let mut skip_next = false;
    for arg in args.iter().skip(1) {
        if skip_next {
            ctx.push_str(" ***");
            skip_next = false;
            continue;
        }
        if is_sensitive_option(arg) {
            ctx.push(' ');
            if let Some((option, _)) = arg.split_once('=') {
                ctx.push_str(option);
                ctx.push_str("=***");
            } else {
                ctx.push_str(arg);
                skip_next = true;
            }
        } else if arg.starts_with("--") || (arg.starts_with('-') && arg.len() == 2) {
            // Error reports need a command shape, not arbitrary option values. This also
            // keeps file paths passed through non-sensitive flags out of reports.
            let option = arg.split_once('=').map_or(arg.as_str(), |(name, _)| name);
            ctx.push(' ');
            ctx.push_str(option);
            if arg.contains('=') {
                ctx.push_str("=***");
            } else {
                skip_next = true;
            }
        } else if is_local_path(arg) {
            ctx.push_str(" [PATH]");
        } else {
            ctx.push(' ');
            ctx.push_str(arg);
        }
    }
    ctx
}

fn is_sensitive_option(arg: &str) -> bool {
    let option = arg.split_once('=').map_or(arg, |(name, _)| name);
    matches!(
        option,
        "--token"
            | "-t"
            | "--password"
            | "--current"
            | "--new"
            | "--access-token"
            | "--api-key"
            | "--authorization"
            | "--docs-folder-token"
            | "--secret"
    )
}

fn is_local_path(value: &str) -> bool {
    let value = value.trim_matches(|ch: char| matches!(ch, '\'' | '\"' | ',' | ':' | ';'));
    let filename_extension = value.rsplit_once('.').map(|(_, extension)| extension);
    value.starts_with("~/")
        || value.starts_with("./")
        || value.starts_with("../")
        || value.starts_with("/Users/")
        || value.starts_with("/home/")
        || value.starts_with("/private/")
        || value.starts_with("/tmp/")
        || value.starts_with("C:\\")
        || value.starts_with("file://")
        || matches!(
            filename_extension,
            Some(
                "json"
                    | "csv"
                    | "tsv"
                    | "xlsx"
                    | "xls"
                    | "fasta"
                    | "fa"
                    | "dna"
                    | "gb"
                    | "pdf"
                    | "docx"
                    | "md"
                    | "txt"
            )
        )
}

fn sanitize_error_text(text: &str) -> String {
    let args = std::iter::once("scitex".to_string())
        .chain(text.split_whitespace().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    command_context_from_args(&args)
        .strip_prefix("scitex ")
        .unwrap_or_default()
        .to_string()
}

fn error_fingerprint(e: &anyhow::Error, cmd: &str) -> String {
    let err_str = sanitize_error_text(&e.to_string());
    // Extract key error pattern: status code + path for HTTP errors,
    // or error variant name for other errors
    if let Some(scitex_err) = e.downcast_ref::<ScientexError>() {
        match scitex_err {
            ScientexError::HttpError { status, path, .. } => {
                format!("{cmd}::HttpError({status})::{path}")
            }
            ScientexError::NotAuthenticated => {
                format!("{cmd}::NotAuthenticated")
            }
            _ => {
                format!("{cmd}::{}", error_type_label(e))
            }
        }
    } else {
        // For anyhow-wrapped errors, take first 80 chars of message as fingerprint
        let short = if err_str.len() > 80 {
            &err_str[..80]
        } else {
            &err_str
        };
        format!("{cmd}::Anyhow({short})")
    }
}

fn error_type_label(e: &anyhow::Error) -> &'static str {
    if let Some(scitex_err) = e.downcast_ref::<ScientexError>() {
        match scitex_err {
            ScientexError::HttpError { .. } => "HttpError",
            ScientexError::RequestError(_) => "RequestError",
            ScientexError::NotAuthenticated => "NotAuthenticated",
            ScientexError::ParseError(_) => "ParseError",
            ScientexError::IoError(_) => "IoError",
        }
    } else {
        "Unknown"
    }
}

fn error_category(e: &anyhow::Error) -> ErrorCategory {
    if let Some(scitex_err) = e.downcast_ref::<ScientexError>() {
        match scitex_err {
            ScientexError::HttpError { status, .. } if *status == 401 || *status == 403 => {
                ErrorCategory::Permission
            }
            ScientexError::NotAuthenticated => ErrorCategory::Permission,
            ScientexError::ParseError(_) => ErrorCategory::Data,
            ScientexError::RequestError(_) => ErrorCategory::Functional,
            ScientexError::HttpError { .. } => ErrorCategory::Functional,
            ScientexError::IoError(_) => ErrorCategory::Other,
        }
    } else {
        ErrorCategory::Other
    }
}

fn prompt_yn(prompt: &str) -> bool {
    use std::io::{self, Write};
    print!("{prompt} [Y/n] ");
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim().to_lowercase();
        trimmed.is_empty() || trimmed == "y" || trimmed == "yes"
    } else {
        false
    }
}

async fn submit_error_report(
    config: &Arc<Config>,
    e: &anyhow::Error,
    cmd: &str,
) -> Result<String, anyhow::Error> {
    let client = scitex_cli::client::ScientexClient::new(Arc::clone(config))?;
    let category = error_category(e);
    let title = format!("{cmd}: {}", error_type_label(e));
    let detail = sanitize_error_text(&e.to_string());
    let description = format!(
        "命令: {cmd}\n错误类型: {}\n错误详情: {detail}\nCLI版本: {}\n平台: {}",
        error_type_label(e),
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
    );

    let report = ErrorReportCreate {
        category,
        title,
        description,
        url: None,
        user_agent: Some(error_report::crate_user_agent()),
    };

    let resp = client.post_error_report(&report).await?;
    Ok(resp.id)
}

#[cfg(test)]
mod tests {
    use super::{command_context_from_args, sanitize_error_text};

    #[test]
    fn command_context_redacts_sensitive_option_values_and_paths() {
        let args = vec![
            "scitex".to_string(),
            "me".to_string(),
            "change-password".to_string(),
            "--current".to_string(),
            "old-secret".to_string(),
            "--new=next-secret".to_string(),
            "--docs-folder-token".to_string(),
            "folder-secret".to_string(),
            "--file".to_string(),
            "/Users/alice/private.json".to_string(),
        ];

        assert_eq!(
            command_context_from_args(&args),
            "scitex me change-password --current *** --new=*** --docs-folder-token *** --file ***"
        );
    }

    #[test]
    fn error_text_redacts_known_secrets_and_local_paths() {
        let text = "failed --new hunter2 while reading /Users/alice/private.json";
        let redacted = sanitize_error_text(text);
        assert!(!redacted.contains("hunter2"));
        assert!(!redacted.contains("/Users/alice/private.json"));
        assert!(redacted.contains("--new ***"));
        assert!(redacted.contains("[PATH]"));
    }
}
