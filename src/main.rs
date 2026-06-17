use std::sync::Arc;

use biolab::commands::{
    admin, error_report, inventory, lab, orders, project, projects, skills, tasks, templates,
    update, users,
};
use biolab::config::Config;
use biolab::error_history::ErrorHistory;
use biolab::errors::BiolabError;
use biolab::output::OutputFormat;
use biolab::types::{ErrorCategory, ErrorReportCreate};
use biolab::{check_status, login, logout, poll_login_from_env};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;

/// Biolab lab management CLI.
#[derive(Parser)]
#[command(name = "biolab", version, about, long_about = None)]
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
    /// Finish background login polling.
    #[command(hide = true)]
    LoginPoll,
    /// Log out and remove the local token.
    Logout,
    /// Check login status.
    Status,

    /// Current user management.
    Me(users::MeArgs),

    /// Order management.
    Orders(orders::OrdersArgs),

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

    /// Admin-only catalog management.
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
            println!("{}", "Biolab CLI".bold());
            println!("\nRun biolab --help to see available commands.\n");
            return;
        }
        Some(Commands::Login) => {
            login(&config).await;
            Ok(())
        }
        Some(Commands::LoginPoll) => {
            if !poll_login_from_env(&config).await {
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
        Some(Commands::Orders(args)) => orders::run(&args, &config, &format).await,
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
        history.record(
            &fingerprint,
            &cmd,
            error_type_label(&e),
            &e.to_string(),
        );

        eprintln!("{}: {e}", "Error".red().bold());

        // If the same error keeps happening, offer to report it
        if history.check_threshold(&fingerprint, 10, 3) {
            if prompt_yn("检测到同类错误反复出现。是否上报错误详情帮助改进？") {
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
    if args.len() < 2 {
        return "biolab".to_string();
    }
    let mut ctx = String::from("biolab");
    let mut skip_next = false;
    for (_i, arg) in args.iter().enumerate().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if arg == "--token" || arg == "-t" || arg == "--password" {
            ctx.push_str(" ***");
            skip_next = true;
        } else if arg.starts_with("--token=") || arg.starts_with("-t=") || arg.starts_with("--password=") {
            ctx.push_str(" ***");
        } else {
            ctx.push(' ');
            ctx.push_str(arg);
        }
    }
    ctx
}

fn error_fingerprint(e: &anyhow::Error, cmd: &str) -> String {
    let err_str = e.to_string();
    // Extract key error pattern: status code + path for HTTP errors,
    // or error variant name for other errors
    if let Some(biolab_err) = e.downcast_ref::<BiolabError>() {
        match biolab_err {
            BiolabError::HttpError { status, path, .. } => {
                format!("{cmd}::HttpError({status})::{path}")
            }
            BiolabError::NotAuthenticated => {
                format!("{cmd}::NotAuthenticated")
            }
            _ => {
                format!("{cmd}::{}", error_type_label(e))
            }
        }
    } else {
        // For anyhow-wrapped errors, take first 80 chars of message as fingerprint
        let short = if err_str.len() > 80 { &err_str[..80] } else { &err_str };
        format!("{cmd}::Anyhow({short})")
    }
}

fn error_type_label(e: &anyhow::Error) -> &'static str {
    if let Some(biolab_err) = e.downcast_ref::<BiolabError>() {
        match biolab_err {
            BiolabError::HttpError { .. } => "HttpError",
            BiolabError::RequestError(_) => "RequestError",
            BiolabError::NotAuthenticated => "NotAuthenticated",
            BiolabError::ParseError(_) => "ParseError",
            BiolabError::IoError(_) => "IoError",
        }
    } else {
        "Unknown"
    }
}

fn error_category(e: &anyhow::Error) -> ErrorCategory {
    if let Some(biolab_err) = e.downcast_ref::<BiolabError>() {
        match biolab_err {
            BiolabError::HttpError { status, .. } if *status == 401 || *status == 403 => {
                ErrorCategory::Permission
            }
            BiolabError::NotAuthenticated => ErrorCategory::Permission,
            BiolabError::ParseError(_) => ErrorCategory::Data,
            BiolabError::RequestError(_) => ErrorCategory::Functional,
            BiolabError::HttpError { .. } => ErrorCategory::Functional,
            BiolabError::IoError(_) => ErrorCategory::Other,
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
    let client = biolab::client::BiolabClient::new(Arc::clone(config))?;
    let category = error_category(e);
    let title = format!("{cmd}: {}", error_type_label(e));
    let description = format!(
        "命令: {cmd}\n错误类型: {}\n错误详情: {e}\nCLI版本: {}\n平台: {}",
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
