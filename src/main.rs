use std::sync::Arc;

use biolab::commands::{inventory, lab, orders, skills, templates, users};
use biolab::config::Config;
use biolab::output::OutputFormat;
use biolab::{check_status, login, logout, poll_login_from_env, LoginMode};
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;

/// Biolab CLI — 实验管理系统客户端
#[derive(Parser)]
#[command(name = "biolab", version, about, long_about = None)]
struct Cli {
    /// 输出格式
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
    /// 飞书 OAuth 登录
    Login(LoginArgs),
    /// 后台完成登录轮询（内部命令）
    #[command(hide = true)]
    LoginPoll,
    /// 登出（删除本地 token）
    Logout,
    /// 检查登录状态
    Status,

    /// 用户管理
    Me(users::MeArgs),

    /// 订单管理
    Orders(orders::OrdersArgs),

    /// 模板管理
    Templates(templates::TemplatesArgs),

    /// 库存管理
    Inventory(inventory::InventoryArgs),

    /// 课题组管理
    Lab(lab::LabArgs),

    /// AI agent skill installation and checks
    Skills(skills::SkillsArgs),
}

#[derive(Args)]
struct LoginArgs {
    /// 后台等待用户授权，适合 AI Agent 使用
    #[arg(long, alias = "no-wait")]
    background: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = Arc::new(Config::new());
    let format = OutputFormat::from(&cli.format);

    let result = match cli.command {
        None => {
            println!("{}", "Biolab 实验管理系统 CLI".bold());
            println!("\n使用 biolab --help 查看可用命令\n");
            return;
        }
        Some(Commands::Login(args)) => {
            let mode = if args.background {
                LoginMode::Background
            } else {
                LoginMode::Wait
            };
            login(&config, mode).await;
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
            let logged_in = check_status(&config);
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
        Some(Commands::Skills(args)) => skills::run(&args, &format),
    };

    if let Err(e) = result {
        eprintln!("{}: {e}", "错误".red().bold());
        std::process::exit(1);
    }
}
