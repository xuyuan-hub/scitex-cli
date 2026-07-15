use std::io::Read;
use std::path::Path;

use anyhow::Context;
use clap::{Args, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::commands::skills;
use crate::embedded_skills::create_temp_dir;
use crate::output::{print_result, OutputFormat};

const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/xuyuan-hub/scitex-cli/releases/latest";
const USER_AGENT: &str = concat!("scitex-cli/", env!("CARGO_PKG_VERSION"));

#[derive(Args)]
pub struct UpdateArgs {
    #[command(subcommand)]
    pub command: UpdateCommand,
}

#[derive(Subcommand)]
pub enum UpdateCommand {
    /// Check GitHub Releases for a newer CLI version
    Check,
    /// Download and install the latest version from GitHub, then update skills
    Install {
        /// Install skills globally
        #[arg(long)]
        global: bool,
    },
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Serialize)]
struct UpdateReport {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_url: String,
    recommended_assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Serialize)]
struct ReleaseAsset {
    name: String,
    url: String,
}

pub async fn run(args: &UpdateArgs, format: &OutputFormat) -> anyhow::Result<()> {
    match &args.command {
        UpdateCommand::Check => {
            let report = check_report().await?;
            print_update_report(&report, format);
        }
        UpdateCommand::Install { global } => {
            install_latest(*global).await?;
        }
    }
    Ok(())
}

async fn fetch_latest_release() -> anyhow::Result<GithubRelease> {
    reqwest::Client::new()
        .get(LATEST_RELEASE_API)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json::<GithubRelease>()
        .await
        .map_err(Into::into)
}

async fn check_report() -> anyhow::Result<UpdateReport> {
    let release = fetch_latest_release().await?;
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let update_available = version_gt(&latest_version, &current_version);
    let wanted_assets = recommended_asset_names();
    let recommended_assets = release
        .assets
        .into_iter()
        .filter(|asset| wanted_assets.contains(&asset.name.as_str()))
        .map(|asset| ReleaseAsset {
            name: asset.name,
            url: asset.browser_download_url,
        })
        .collect();

    Ok(UpdateReport {
        current_version,
        latest_version,
        update_available,
        release_url: release.html_url,
        recommended_assets,
    })
}

async fn install_latest(global_skills: bool) -> anyhow::Result<()> {
    let release = fetch_latest_release().await?;
    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = release.tag_name.trim_start_matches('v').to_string();

    if !version_gt(&latest_version, current_version) {
        println!(
            "{}  v{} (latest)",
            "Already up to date.".green(),
            current_version
        );
        skills::install_with_skills_cli(global_skills)?;
        return Ok(());
    }

    println!("Updating  v{} → v{}", current_version, latest_version);

    let wanted = recommended_asset_names();
    let bin_name = wanted.iter().find(|n| !n.ends_with(".sha256")).copied();
    let sha_name = wanted.iter().find(|n| n.ends_with(".sha256")).copied();

    let bin_name =
        bin_name.ok_or_else(|| anyhow::anyhow!("no binary asset for current platform"))?;
    let sha_name =
        sha_name.ok_or_else(|| anyhow::anyhow!("no sha256 asset for current platform"))?;

    let bin_url = release
        .assets
        .iter()
        .find(|a| a.name == bin_name)
        .map(|a| &a.browser_download_url)
        .ok_or_else(|| anyhow::anyhow!("asset not found: {}", bin_name))?;

    let sha_url = release
        .assets
        .iter()
        .find(|a| a.name == sha_name)
        .map(|a| &a.browser_download_url)
        .ok_or_else(|| anyhow::anyhow!("asset not found: {}", sha_name))?;

    println!("Downloading {} ...", bin_name);
    let bin_bytes = download_bytes(bin_url).await?;

    println!("Downloading {} ...", sha_name);
    let sha_bytes = download_bytes(sha_url).await?;
    let expected_sha = String::from_utf8_lossy(&sha_bytes)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    let actual_sha = {
        let mut hasher = Sha256::new();
        hasher.update(&bin_bytes);
        hex::encode(hasher.finalize())
    };

    if actual_sha != expected_sha {
        anyhow::bail!(
            "Checksum mismatch\n  expected: {}\n  got:      {}",
            expected_sha,
            actual_sha
        );
    }
    println!("{}", "Checksum verified.".green());

    let new_binary = if bin_name.ends_with(".zip") {
        extract_from_zip(&bin_bytes)?
    } else {
        bin_bytes
    };

    println!("Installing Skills bundled with v{} ...", latest_version);
    install_skills_from_binary(&new_binary, &latest_version, global_skills)?;

    let current_exe = std::env::current_exe()?;
    if let Err(replace_error) = replace_binary(&new_binary, &current_exe) {
        let rollback = skills::install_with_skills_cli(global_skills);
        return match rollback {
            Ok(()) => Err(replace_error).context(
                "failed to replace the CLI binary; restored Skills bundled with the current CLI",
            ),
            Err(rollback_error) => Err(replace_error).context(format!(
                "failed to replace the CLI binary and failed to restore current Skills: {rollback_error}"
            )),
        };
    }

    println!();
    println!("{}", "Update complete.".green().bold());
    if cfg!(windows) {
        println!(
            "The new binary will be applied when this process exits. \
             Please restart your terminal."
        );
    }
    Ok(())
}

struct StagedBinary {
    root: std::path::PathBuf,
    path: std::path::PathBuf,
}

impl StagedBinary {
    fn create(bytes: &[u8]) -> anyhow::Result<Self> {
        let root = create_temp_dir("scitex-update")?;
        let filename = if cfg!(windows) {
            "scitex.exe"
        } else {
            "scitex"
        };
        let path = root.join(filename);
        let staged = Self { root, path };
        std::fs::write(&staged.path, bytes).with_context(|| {
            format!(
                "failed to stage new CLI binary at {}",
                staged.path.display()
            )
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&staged.path, std::fs::Permissions::from_mode(0o755))?;
        }

        Ok(staged)
    }
}

impl Drop for StagedBinary {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn install_skills_from_binary(
    binary: &[u8],
    expected_version: &str,
    global: bool,
) -> anyhow::Result<()> {
    let staged = StagedBinary::create(binary)?;
    let version_output = std::process::Command::new(&staged.path)
        .arg("--version")
        .output()
        .context("failed to execute the staged CLI binary")?;
    if !version_output.status.success() {
        anyhow::bail!("staged CLI binary failed its version check");
    }

    let reported_version = String::from_utf8_lossy(&version_output.stdout);
    let reported_version = reported_version.split_whitespace().last().unwrap_or("");
    if reported_version.trim_start_matches('v') != expected_version.trim_start_matches('v') {
        anyhow::bail!(
            "staged CLI version mismatch: release is v{}, binary reports {}",
            expected_version.trim_start_matches('v'),
            reported_version
        );
    }

    let mut command = std::process::Command::new(&staged.path);
    command.args(["skills", "install"]);
    if global {
        command.arg("--global");
    }
    let status = command
        .status()
        .context("failed to install Skills from the staged CLI binary")?;
    if !status.success() {
        anyhow::bail!(
            "staged scitex v{} failed to install its bundled Skills",
            expected_version.trim_start_matches('v')
        );
    }
    Ok(())
}

fn extract_from_zip(zip_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry
            .enclosed_name()
            .and_then(|n| n.file_name().map(|f| f.to_string_lossy().to_string()));
        let is_match = if cfg!(windows) {
            name.as_deref().is_some_and(|n| n.ends_with(".exe"))
        } else {
            name.as_deref() == Some("scitex")
        };
        if is_match {
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf)?;
            return Ok(buf);
        }
    }
    anyhow::bail!("no executable found in zip archive")
}

#[cfg(unix)]
fn replace_binary(new_bytes: &[u8], current_path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let tmp = current_path.with_extension("new");
    std::fs::write(&tmp, new_bytes)?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    std::fs::rename(&tmp, current_path)?;
    Ok(())
}

#[cfg(windows)]
fn replace_binary(new_bytes: &[u8], current_path: &Path) -> anyhow::Result<()> {
    let tmp = current_path.with_extension("new");
    std::fs::write(&tmp, new_bytes)?;

    let script_path = current_path.with_extension("update.bat");
    let script = format!(
        "@echo off\r\n\
         ping 127.0.0.1 -n 3 > nul\r\n\
         move /y \"{}\" \"{}\" > nul 2>&1\r\n\
         del \"{}\" > nul 2>&1\r\n\
         del \"%0\"\r\n",
        tmp.display(),
        current_path.display(),
        tmp.display()
    );
    std::fs::write(&script_path, script)?;

    std::process::Command::new("cmd.exe")
        .args(["/C", "start", "/B", script_path.to_str().unwrap()])
        .spawn()?;

    Ok(())
}

async fn download_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let resp = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.bytes().await?.to_vec())
}

fn print_update_report(report: &UpdateReport, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(report, format),
        OutputFormat::Text => {
            println!("当前版本: {}", report.current_version);
            println!("最新版本: {}", report.latest_version);

            if report.update_available {
                println!("{}", "发现新版本。".yellow().bold());
                println!("Release: {}", report.release_url);
                for asset in &report.recommended_assets {
                    println!("下载: {}  {}", asset.name, asset.url);
                }
            } else {
                println!("{}", "已是最新版本。".green());
            }
        }
    }
}

fn recommended_asset_names() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["scitex_win.zip", "scitex_win.zip.sha256"]
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        &["scitex_mac_arm64", "scitex_mac_arm64.sha256"]
    }
    #[cfg(all(target_os = "macos", not(target_arch = "aarch64")))]
    {
        &["scitex_mac_amd64", "scitex_mac_amd64.sha256"]
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        &["scitex_unix", "scitex_unix.sha256"]
    }
}

fn version_gt(left: &str, right: &str) -> bool {
    parse_version(left) > parse_version(right)
}

fn parse_version(version: &str) -> Vec<u64> {
    version
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_semver_like_versions() {
        assert!(version_gt("0.2.4", "0.2.3"));
        assert!(version_gt("v0.3.0", "0.2.99"));
        assert!(!version_gt("0.2.3", "0.2.3"));
        assert!(!version_gt("0.2.3", "0.2.4"));
    }
}
