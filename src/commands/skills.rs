use std::process::Command;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::output::{print_result, OutputFormat};

const SKILLS_REPO: &str = "xuyuan-hub/biolab-cli";
const EXPECTED_SKILLS: &[&str] = &[
    "biolab-shared",
    "biolab-orders",
    "biolab-templates",
    "biolab-inventory",
    "biolab-admin",
    "biolab-lab",
    "biolab-project",
    "biolab-users",
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
    installed: bool,
    missing: Vec<&'static str>,
    action: &'static str,
}

pub fn run(args: &SkillsArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let report = match &args.command {
        SkillsCommand::Install { global } => {
            install_with_skills_cli(*global)?;
            SkillReport {
                installer: "npx skills",
                skills: EXPECTED_SKILLS,
                installed: true,
                missing: Vec::new(),
                action: "installed",
            }
        }
        SkillsCommand::Check { global } => {
            let missing = missing_skills_with_skills_cli(*global)?;
            let installed = missing.is_empty();
            SkillReport {
                installer: "npx skills",
                skills: EXPECTED_SKILLS,
                installed,
                missing,
                action: "checked",
            }
        }
    };

    print_report(&report, format);
    Ok(())
}

pub fn install_with_skills_cli(global: bool) -> anyhow::Result<()> {
    let mut command = Command::new(npx_bin());
    command.args(["-y", "skills", "add", SKILLS_REPO, "-y"]);
    if global {
        command.arg("-g");
    }

    let status = command.status()?;
    if !status.success() {
        anyhow::bail!(
            "`npx skills add` failed. Try manually: npx -y skills add {} -y{}",
            SKILLS_REPO,
            if global { " -g" } else { "" }
        );
    }

    Ok(())
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
        let parsed = parse_skills_list("- biolab-orders@0.2.5\nlark-calendar\nOther: heading");
        assert!(parsed.contains(&"biolab-orders".to_string()));
        assert!(parsed.contains(&"lark-calendar".to_string()));
        assert!(!parsed.contains(&"Other".to_string()));
    }

    #[test]
    fn parses_colored_skills_ls_output() {
        let parsed = parse_skills_list(
            "\u{1b}[1mProject Skills\u{1b}[0m\n\n\u{1b}[36mbiolab-orders\u{1b}[0m \u{1b}[38;5;102m./.agents/skills/biolab-orders\u{1b}[0m",
        );
        assert!(parsed.contains(&"biolab-orders".to_string()));
    }
}
