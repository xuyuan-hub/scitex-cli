use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::config::Config;

/// Response from POST /feishu/cli-auth.
#[derive(Deserialize)]
pub struct CliAuthResponse {
    pub auth_url: String,
    pub poll_key: String,
}

pub async fn check_status(config: &Config) -> bool {
    let Some(token) = config.load_token() else {
        println!("未登录（未找到可用 token）");
        return false;
    };
    let url = format!("{}/users/me", config.base_url);
    let client = Client::new();
    match client
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(user) = resp.json::<crate::types::User>().await {
                println!(
                    "已登录: {} ({})",
                    user.full_name.as_deref().unwrap_or("-"),
                    user.email
                );
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

pub async fn login(config: &Config) -> bool {
    if config.load_token().is_some() {
        println!("已有 token，尝试验证...");
        if check_status(config).await {
            println!("当前 token 有效，无需重新登录。");
            println!("如需重新登录，请先执行 `scitex logout`");
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

            println!("  正在前台等待授权完成；请保持此命令运行。");
            println!("  用户授权后 token 会自动保存。");
            println!("{}\n", "=".repeat(55));

            match poll_and_save_token(&client, config, &resp.poll_key).await {
                Ok(()) => {
                    println!("\n认证成功，token 已保存。");
                    true
                }
                Err(e) => {
                    eprintln!("\n认证失败: {e}");
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("请求认证失败: {e}");
            false
        }
    }
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
        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        if !status.is_success() {
            return Err(format!("轮询 token 失败: HTTP {status}: {body}").into());
        }

        match token_from_poll_response(&body)? {
            Some(token) => return Ok(token),
            None => {}
        }

        // Still waiting — keep polling
        print!(".");
        use std::io::Write;
        let _ = std::io::stdout().flush();
        continue;
    }
}

fn token_from_poll_response(body: &serde_json::Value) -> Result<Option<String>, String> {
    match body.get("status").and_then(|value| value.as_str()) {
        Some("waiting") => Ok(None),
        Some("success") => body
            .get("access_token")
            .and_then(|value| value.as_str())
            .filter(|token| !token.is_empty())
            .map(|token| Some(token.to_string()))
            .ok_or_else(|| "后端返回 success 但没有 access_token".to_string()),
        Some("error") => {
            let detail = body
                .get("detail")
                .and_then(|value| value.as_str())
                .unwrap_or("未知错误");
            Err(format!("后端返回错误: {detail}"))
        }
        Some(status) => Err(format!("后端返回未知认证状态: {status}")),
        None => Err("后端响应缺少认证状态".to_string()),
    }
}

pub fn logout(config: &Config) {
    if config.remove_token().is_ok() {
        println!("已登出，Token 已删除。");
    } else {
        println!("未登录。");
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::token_from_poll_response;

    #[test]
    fn poll_response_waiting_has_no_token() {
        assert_eq!(
            token_from_poll_response(&json!({ "status": "waiting" })),
            Ok(None)
        );
    }

    #[test]
    fn poll_response_success_returns_token() {
        assert_eq!(
            token_from_poll_response(&json!({ "status": "success", "access_token": "jwt" })),
            Ok(Some("jwt".to_string()))
        );
    }

    #[test]
    fn poll_response_success_without_token_is_an_error() {
        let error = token_from_poll_response(&json!({ "status": "success" }))
            .expect_err("a successful response must contain a token");

        assert!(error.contains("access_token"));
    }

    #[test]
    fn poll_response_error_surfaces_backend_detail() {
        let error = token_from_poll_response(&json!({ "status": "error", "detail": "denied" }))
            .expect_err("an error response must fail the poll");

        assert!(error.contains("denied"));
    }
}
