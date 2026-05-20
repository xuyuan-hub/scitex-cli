use std::{
    process::{Command, Stdio},
    time::Duration,
};

use reqwest::Client;
use serde::Deserialize;

use crate::config::Config;

const LOGIN_POLL_KEY_ENV_VAR: &str = "BIOLAB_LOGIN_POLL_KEY";

/// Response from POST /feishu/cli-auth.
#[derive(Deserialize)]
pub struct CliAuthResponse {
    pub auth_url: String,
    pub poll_key: String,
}

pub fn check_status(config: &Config) -> bool {
    let Some(token) = config.load_token() else {
        println!("未登录（未找到可用 token）");
        return false;
    };
    let url = format!("{}/users/me", config.base_url);
    let client = reqwest::blocking::Client::new();
    match client
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(user) = resp.json::<crate::types::User>() {
                println!("已登录: {} ({})", user.full_name, user.email);
                true
            } else {
                println!("Token 有效，但解析用户信息失败");
                true
            }
        }
        Ok(resp) => {
            println!("Token 无效: HTTP {}", resp.status());
            false
        }
        Err(e) => {
            println!("检查登录状态失败: {}", e);
            false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LoginMode {
    Wait,
    Background,
}

pub async fn login(config: &Config, mode: LoginMode) -> bool {
    if config.load_token().is_some() {
        println!("已有 token，尝试验证...");
        if check_status(config) {
            println!("当前 token 有效，无需重新登录。");
            println!("如需重新登录，请先执行 `biolab logout`");
            return true;
        }
        println!("Token 已过期，开始重新认证...\n");
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client");

    // Step 1: Get auth URL + poll key from backend
    match request_cli_auth(&client, config).await {
        Ok(resp) => {
            println!("\n{}", "=".repeat(55));
            println!("  请在浏览器中打开以下链接完成飞书认证：");
            println!("\n    {}\n", resp.auth_url);

            match mode {
                LoginMode::Wait => {
                    println!("  等待认证完成，每 2 秒检查一次…");
                    println!("{}\n", "=".repeat(55));

                    // Step 2: Poll for JWT
                    match poll_and_save_token(&client, config, &resp.poll_key).await {
                        Ok(()) => {
                            println!("认证成功！Token 已保存到系统凭据库");
                            check_status(config)
                        }
                        Err(e) => {
                            eprintln!("认证失败: {e}");
                            false
                        }
                    }
                }
                LoginMode::Background => {
                    println!("  已启动后台登录轮询，Agent 可先返回认证链接。");
                    println!("  用户授权后 token 会自动保存；稍后运行 `biolab status` 检查结果。");
                    println!("{}\n", "=".repeat(55));

                    if let Err(e) = spawn_login_poller(&resp.poll_key) {
                        eprintln!("启动后台登录轮询失败: {e}");
                        return false;
                    }

                    true
                }
            }
        }
        Err(e) => {
            eprintln!("请求认证失败: {e}");
            false
        }
    }
}

pub async fn poll_login_from_env(config: &Config) -> bool {
    let Ok(poll_key) = std::env::var(LOGIN_POLL_KEY_ENV_VAR) else {
        eprintln!("缺少后台登录 poll key。");
        return false;
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client");

    if let Err(e) = poll_and_save_token(&client, config, &poll_key).await {
        eprintln!("后台登录失败: {e}");
        return false;
    }

    true
}

/// Request an auth URL and poll key from the backend.
async fn request_cli_auth(
    client: &Client,
    config: &Config,
) -> Result<CliAuthResponse, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/feishu/cli-auth", config.base_url);
    let resp = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body("{}")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}").into());
    }

    let data: CliAuthResponse = resp.json().await?;
    Ok(data)
}

async fn poll_and_save_token(
    client: &Client,
    config: &Config,
    poll_key: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let token = poll_jwt(client, config, poll_key).await?;
    config.save_token(&token)?;
    Ok(())
}

/// Poll the token endpoint until the user authorizes or we time out.
async fn poll_jwt(
    client: &Client,
    config: &Config,
    poll_key: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let token_url = format!("{}/feishu/cli-token", config.base_url);
    let timeout = Duration::from_secs(300); // 5 minutes
    let deadline = std::time::Instant::now() + timeout;
    let interval = Duration::from_secs(2);

    loop {
        tokio::time::sleep(interval).await;

        if std::time::Instant::now() >= deadline {
            return Err("认证超时，用户未在 5 分钟内完成授权".into());
        }

        let resp = client
            .post(&token_url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::json!({ "poll_key": poll_key }).to_string())
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;

        // Success: token received
        if body.get("status").and_then(|v| v.as_str()) == Some("success") {
            if let Some(token) = body.get("access_token").and_then(|v| v.as_str()) {
                return Ok(token.to_string());
            }
        }

        // Error: backend returned a failure
        if body.get("status").and_then(|v| v.as_str()) == Some("error") {
            let detail = body
                .get("detail")
                .and_then(|v| v.as_str())
                .unwrap_or("未知错误");
            return Err(format!("后端返回错误: {detail}").into());
        }

        // Still waiting — keep polling
        print!(".");
        use std::io::Write;
        let _ = std::io::stdout().flush();
        continue;
    }
}

fn spawn_login_poller(poll_key: &str) -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    Command::new(exe)
        .arg("login-poll")
        .env(LOGIN_POLL_KEY_ENV_VAR, poll_key)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

pub fn logout(config: &Config) {
    if config.remove_token().is_ok() {
        println!("已登出，Token 已删除。");
    } else {
        println!("未登录。");
    }
}
