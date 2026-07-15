use std::path::Path;
use std::process::Command;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::embedded_skills::{BundledRelease, MaterializedSkills, INSTALL_MARKER};
use crate::output::{print_result, OutputFormat};

const EXPECTED_SKILLS: &[&str] = &[
    "scitex-shared",
    "scitex-orders",
    "scitex-templates",
    "scitex-inventory",
    "scitex-admin",
    "scitex-lab",
    "scitex-project",
    "scitex-users",
    "scitex-evo",
    "scitex-experiment",
    "scitex-task",
    "scitex-tashan",
    "scitex-error-report",
];

#[derive(Args)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Subcommand)]
pub enum SkillsCommand {
    /// Install AI agent skills through the standard skills installer
    Install {
        /// Install globally for all supported agents
        #[arg(long)]
        global: bool,
    },
    /// Check whether the skill is installed through the standard skills installer
    Check {
        /// Check globally installed skills
        #[arg(long)]
        global: bool,
    },
}

#[derive(Debug, Serialize)]
struct SkillReport {
    installer: &'static str,
    skills: &'static [&'static str],
    bundled: bool,
    release: BundledRelease,
    installed: bool,
    missing: Vec<&'static str>,
    incompatible: Vec<&'static str>,
    action: &'static str,
}

pub fn run(args: &SkillsArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let release = BundledRelease::current()?;
    let report = match &args.command {
        SkillsCommand::Install { global } => {
            install_with_skills_cli(*global)?;
            SkillReport {
                installer: "npx skills",
                skills: EXPECTED_SKILLS,
                bundled: true,
                release,
                installed: true,
                missing: Vec::new(),
                incompatible: Vec::new(),
                action: "installed",
            }
        }
        SkillsCommand::Check { global } => {
            let missing = missing_skills_with_skills_cli(*global)?;
            let incompatible = incompatible_installed_skills(*global, &release)?;
            let installed = missing.is_empty() && incompatible.is_empty();
            SkillReport {
                installer: "npx skills",
                skills: EXPECTED_SKILLS,
                bundled: true,
                release,
                installed,
                missing,
                incompatible,
                action: "checked",
            }
        }
    };

    print_report(&report, format);
    Ok(())
}

pub fn install_with_skills_cli(global: bool) -> anyhow::Result<()> {
    let materialized = MaterializedSkills::create()?;
    let mut command = skills_add_command(&materialized.skills_path(), global);

    let status = command.status()?;
    if !status.success() {
        anyhow::bail!(
            "`npx skills add` failed while installing Skills bundled with scitex v{}",
            env!("CARGO_PKG_VERSION")
        );
    }

    Ok(())
}

fn skills_add_command(source: &Path, global: bool) -> Command {
    let mut command = Command::new(npx_bin());
    command.args(["-y", "skills", "add"]);
    command.arg(source);
    command.args(["--copy", "-y"]);
    if global {
        command.arg("-g");
    }
    command
}

fn missing_skills_with_skills_cli(global: bool) -> anyhow::Result<Vec<&'static str>> {
    let mut command = Command::new(npx_bin());
    command.args(["-y", "skills", "ls"]);
    if global {
        command.arg("-g");
    }

    let output = command.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("`npx skills ls` failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let installed = parse_skills_list(&stdout);
    Ok(EXPECTED_SKILLS
        .iter()
        .copied()
        .filter(|expected| !installed.iter().any(|skill| skill == expected))
        .collect())
}

fn incompatible_installed_skills(
    global: bool,
    expected: &BundledRelease,
) -> anyhow::Result<Vec<&'static str>> {
    let root = if global {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?
            .join(".agents/skills")
    } else {
        std::env::current_dir()?.join(".agents/skills")
    };
    Ok(incompatible_skills_under(&root, expected))
}

fn incompatible_skills_under(root: &Path, expected: &BundledRelease) -> Vec<&'static str> {
    EXPECTED_SKILLS
        .iter()
        .copied()
        .filter(|skill| {
            let marker = root.join(skill).join(INSTALL_MARKER);
            let Ok(contents) = std::fs::read(marker) else {
                return true;
            };
            let Ok(installed) = serde_json::from_slice::<BundledRelease>(&contents) else {
                return true;
            };
            installed.cli_version != expected.cli_version
                || installed.skills_sha256 != expected.skills_sha256
                || installed.compatibility != expected.compatibility
        })
        .collect()
}

fn parse_skills_list(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let clean = strip_ansi(line);
            let token = clean.trim().trim_start_matches('-').trim();
            if token.is_empty() || token.ends_with(':') {
                return None;
            }

            let first_column = token.split_whitespace().next().unwrap_or(token);
            let name = first_column.split('@').next().unwrap_or(first_column);
            if name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | ':' | '-'))
            {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn print_report(report: &SkillReport, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(report, format),
        OutputFormat::Text => {
            let status = if report.installed {
                "installed"
            } else {
                "not installed"
            };
            println!(
                "{}  {}  {}  {}",
                report.installer,
                report.action,
                status,
                report.skills.join(", ")
            );
            if !report.missing.is_empty() {
                println!("missing: {}", report.missing.join(", "));
            }
            if !report.incompatible.is_empty() {
                println!(
                    "missing or incompatible release marker: {}",
                    report.incompatible.join(", ")
                );
                println!("run `scitex skills install` to install the bundled release");
            }
            println!(
                "bundled release: v{}  skills sha256: {}",
                report.release.cli_version, report.release.skills_sha256
            );
            let openapi = &report.release.compatibility["openapi"];
            println!(
                "OpenAPI baseline: {}  fixture sha256: {}",
                openapi["baseline_date"].as_str().unwrap_or("unknown"),
                openapi["fixture_sha256"].as_str().unwrap_or("unknown")
            );
        }
    }
}

fn npx_bin() -> &'static str {
    if cfg!(windows) {
        "npx.cmd"
    } else {
        "npx"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_skills_ls_output() {
        let parsed = parse_skills_list("- scitex-orders@0.2.5\nlark-calendar\nOther: heading");
        assert!(parsed.contains(&"scitex-orders".to_string()));
        assert!(parsed.contains(&"lark-calendar".to_string()));
        assert!(!parsed.contains(&"Other".to_string()));
    }

    #[test]
    fn parses_colored_skills_ls_output() {
        let parsed = parse_skills_list(
            "\u{1b}[1mProject Skills\u{1b}[0m\n\n\u{1b}[36mscitex-orders\u{1b}[0m \u{1b}[38;5;102m./.agents/skills/scitex-orders\u{1b}[0m",
        );
        assert!(parsed.contains(&"scitex-orders".to_string()));
    }

    #[test]
    fn installs_from_local_bundled_source_in_copy_mode() {
        let command = skills_add_command(Path::new("/tmp/scitex-bundled-skills"), true);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            args,
            [
                "-y",
                "skills",
                "add",
                "/tmp/scitex-bundled-skills",
                "--copy",
                "-y",
                "-g"
            ]
        );
        assert!(!args.iter().any(|arg| arg.contains("xuyuan-hub")));
    }

    #[test]
    fn checks_installed_release_markers() {
        let materialized = MaterializedSkills::create().unwrap();
        let expected = BundledRelease::current().unwrap();
        assert!(incompatible_skills_under(&materialized.skills_path(), &expected).is_empty());

        std::fs::write(
            materialized
                .skills_path()
                .join("scitex-orders")
                .join(INSTALL_MARKER),
            b"{}",
        )
        .unwrap();
        assert_eq!(
            incompatible_skills_under(&materialized.skills_path(), &expected),
            ["scitex-orders"]
        );
    }
}
