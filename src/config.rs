use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use keyring::{Entry, Error as KeyringError};

pub const DEFAULT_BASE_URL: &str = "https://scientex.cn/api/v1";
const TOKEN_ENV_VAR: &str = "SCIENTEX_TOKEN";
const BASE_URL_ENV_VAR: &str = "SCIENTEX_BASE_URL";
const INSECURE_TOKEN_FILE_ENV_VAR: &str = "SCIENTEX_INSECURE_TOKEN_FILE";
const DISABLE_CONTAINER_TOKEN_FILE_ENV_VAR: &str = "SCIENTEX_DISABLE_CONTAINER_TOKEN_FILE";
const TOKEN_FILE_NAME: &str = ".scitex_token";
const KEYRING_SERVICE: &str = "scitex-cli";
const KEYRING_USERNAME: &str = "default";

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub token_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let base_url =
            std::env::var(BASE_URL_ENV_VAR).unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let token_path = token_file_path();
        Self {
            base_url,
            token_path,
        }
    }

    pub fn load_token(&self) -> Option<String> {
        // 1. Environment variable (highest priority)
        if let Ok(token) = std::env::var(TOKEN_ENV_VAR).map(|t| t.trim().to_string()) {
            if !token.is_empty() {
                return Some(token);
            }
        }

        // 2. Container agents usually have no OS keychain, so use a local
        // container-only token file without requiring a restart or mount.
        if container_file_storage_enabled() {
            if let Some(token) = self.load_token_from_file() {
                return Some(token);
            }
        }

        // 3. OS keychain
        if let Some(token) = keyring_load_token() {
            return Some(token);
        }

        // 4. Optional plaintext file storage for explicitly insecure/headless host setups.
        if insecure_token_file_enabled() {
            return self.load_token_from_file();
        }

        None
    }

    pub fn save_token(&self, token: &str) -> io::Result<()> {
        match self.save_token_to_keyring(token) {
            Ok(()) => Ok(()),
            Err(keyring_error) if container_file_storage_enabled() => {
                eprintln!("系统凭据库不可用 ({keyring_error})，已保存到容器内本地 token 文件。");
                self.save_token_to_file(token)
            }
            Err(keyring_error) if insecure_token_file_enabled() => {
                eprintln!("警告：系统凭据库存储失败 ({keyring_error})，已按 {INSECURE_TOKEN_FILE_ENV_VAR}=1 回退到明文 token 文件。");
                self.save_token_to_file(token)
            }
            Err(keyring_error) => Err(io::Error::other(
                format!(
                    "系统凭据库存储失败: {keyring_error}. \
                     默认不会在宿主机写入明文 token 文件；如确需在可信 headless 环境使用可写文件存储，\
                     请显式设置 {INSECURE_TOKEN_FILE_ENV_VAR}=1。"
                ),
            )),
        }
    }

    pub fn remove_token(&self) -> io::Result<()> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
            .map_err(|e| io::Error::other(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => {}
            Err(e) => return Err(io::Error::other(e.to_string())),
        }
        if self.token_path.exists() {
            fs::remove_file(&self.token_path)
        } else {
            Ok(())
        }
    }

    fn save_token_to_keyring(&self, token: &str) -> io::Result<()> {
        Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
            .and_then(|e| e.set_password(token))
            .map_err(|e| io::Error::other(e.to_string()))
    }

    fn load_token_from_file(&self) -> Option<String> {
        load_token_from_path(&self.token_path)
    }

    fn save_token_to_file(&self, token: &str) -> io::Result<()> {
        let mut file = token_file_options().open(&self.token_path)?;
        restrict_file_permissions(&file)?;
        file.write_all(token.as_bytes())
    }
}

fn keyring_load_token() -> Option<String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USERNAME).ok()?;
    let token = entry.get_password().ok()?.trim().to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn load_token_from_path(path: &PathBuf) -> Option<String> {
    if !path.exists() {
        return None;
    }

    let token = fs::read_to_string(path).ok()?.trim().to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

fn token_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(TOKEN_FILE_NAME)
}

fn insecure_token_file_enabled() -> bool {
    env_flag_enabled(INSECURE_TOKEN_FILE_ENV_VAR)
}

fn container_file_storage_enabled() -> bool {
    running_in_container() && !disable_container_file_storage()
}

fn running_in_container() -> bool {
    std::env::var("container").is_ok()
        || std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
        || PathBuf::from("/.dockerenv").exists()
        || PathBuf::from("/run/.containerenv").exists()
}

fn disable_container_file_storage() -> bool {
    env_flag_enabled(DISABLE_CONTAINER_TOKEN_FILE_ENV_VAR)
}

fn env_flag_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn token_file_options() -> fs::OpenOptions {
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    options
}

#[cfg(unix)]
fn restrict_file_permissions(file: &fs::File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(0o600);
    file.set_permissions(permissions)
}

#[cfg(not(unix))]
fn restrict_file_permissions(_file: &fs::File) -> io::Result<()> {
    Ok(())
}
