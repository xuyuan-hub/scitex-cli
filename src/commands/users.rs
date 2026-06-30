use std::sync::Arc;

use clap::{Args, Subcommand};

use crate::client::ScientexClient;
use crate::config::Config;
use crate::output::{print_result, OutputFormat};
use crate::types::FeishuUserSettingsUpdate;

#[derive(Args)]
pub struct MeArgs {
    #[command(subcommand)]
    pub command: Option<MeCommand>,
}

#[derive(Subcommand)]
pub enum MeCommand {
    /// 更新个人信息
    Update {
        /// JSON 格式的更新字段，如 '{"phone_number":"138xxxx"}'
        #[arg(value_name = "JSON")]
        data: String,
    },
    /// 修改密码
    ChangePassword {
        /// 当前密码
        #[arg(long)]
        current: String,
        /// 新密码
        #[arg(long)]
        new: String,
    },
    /// 查看或更新飞书云文档设置
    FeishuSettings {
        #[command(subcommand)]
        command: Option<FeishuSettingsCommand>,
    },
}

#[derive(Subcommand)]
pub enum FeishuSettingsCommand {
    /// 更新飞书云文档目标文件夹 token
    Update {
        /// 飞书云文档目标文件夹 token
        #[arg(long)]
        docs_folder_token: Option<String>,
    },
}

pub async fn run(args: &MeArgs, config: &Arc<Config>, format: &OutputFormat) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;

    match &args.command {
        None => {
            let user = client.get_me().await?;
            print_result(&user, format);
        }
        Some(MeCommand::Update { data }) => {
            let data: serde_json::Value = serde_json::from_str(data)?;
            let user = client.update_me(&data).await?;
            print_result(&user, format);
        }
        Some(MeCommand::ChangePassword { current, new }) => {
            let result = client.change_password(current, new).await?;
            print_result(&result, format);
        }
        Some(MeCommand::FeishuSettings { command }) => match command {
            None => {
                let settings = client.get_feishu_settings().await?;
                print_result(&settings, format);
            }
            Some(FeishuSettingsCommand::Update { docs_folder_token }) => {
                let update = FeishuUserSettingsUpdate {
                    docs_folder_token: docs_folder_token.clone(),
                };
                let settings = client.update_feishu_settings(&update).await?;
                print_result(&settings, format);
            }
        },
    }
    Ok(())
}
