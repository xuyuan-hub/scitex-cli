use std::sync::Arc;

use clap::{Args, Subcommand};

use crate::client::ScientexClient;
use crate::commands::confirm::require_confirmation;
use crate::config::Config;
use crate::output::{print_pagination_metadata, print_result, OutputFormat};
use crate::types::{PublicTool, TaskExactRerun, ToolRun, ToolValidation};

#[derive(Args)]
pub struct ToolsArgs {
    #[command(subcommand)]
    pub command: ToolsCommand,
}

#[derive(Subcommand)]
pub enum ToolsCommand {
    /// Discover published public tools.
    Search {
        #[arg(long, alias = "search")]
        query: Option<String>,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        family: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..=100))]
        limit: u32,
    },
    /// Show published schemas, versions, citations, and submission availability.
    Show { key: String },
    /// Validate an input JSON object without creating work.
    Validate {
        key: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        input: String,
    },
    /// Validate, confirm, then create one immutable run of a published tool.
    Run {
        key: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        lab_id: Option<String>,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    /// Confirm a new task created from one immutable prior Catalog run.
    Rerun {
        task_id: String,
        part_id: String,
        run_id: String,
        #[arg(long)]
        lab_id: Option<String>,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    /// Read a bounded preview of one declared run artifact.
    ArtifactPreview {
        task_id: String,
        part_id: String,
        run_id: String,
        artifact_index: u64,
        #[arg(long)]
        lab_id: Option<String>,
    },
}

pub async fn run(
    args: &ToolsArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;
    match &args.command {
        ToolsCommand::Search {
            query,
            domain,
            family,
            tag,
            skip,
            limit,
        } => {
            let tools = client
                .search_public_tools(
                    *skip,
                    *limit,
                    query.as_deref(),
                    domain.as_deref(),
                    family.as_deref(),
                    tag.as_deref(),
                )
                .await?;
            match format {
                OutputFormat::Json => print_result(&tools, format),
                OutputFormat::Text => print_tool_search(&tools),
            }
        }
        ToolsCommand::Show { key } => {
            let tool = client.get_public_tool(key).await?;
            print_public_tool(&tool, format);
        }
        ToolsCommand::Validate {
            key,
            version,
            input,
        } => {
            let input = read_input_object(input)?;
            let tool = client.get_public_tool(key).await?;
            validate_selected_version_and_files(&tool, version, &input)?;
            let validation = client
                .validate_public_tool_input(
                    key,
                    &serde_json::json!({
                        "tool_version_id": version,
                        "input_data": input,
                    }),
                )
                .await?;
            print_validation(&validation, format);
        }
        ToolsCommand::Run {
            key,
            version,
            input,
            title,
            description,
            lab_id,
            yes,
        } => {
            let input = read_input_object(input)?;
            let tool = client.get_public_tool(key).await?;
            let version_info = validate_selected_version_and_files(&tool, version, &input)?;
            let validation = client
                .validate_public_tool_input(
                    key,
                    &serde_json::json!({
                        "tool_version_id": version,
                        "input_data": input,
                    }),
                )
                .await?;
            if !validation.valid {
                print_validation(&validation, format);
                anyhow::bail!("Tool input validation failed; no run request was sent.");
            }
            let payload = serde_json::json!({
                "tool_version_id": version,
                "input_data": input,
                "title": title,
                "description": description,
            });
            require_confirmation(
                &tool_run_confirmation(&tool, version_info, &validation, &payload["input_data"]),
                *yes,
            )?;
            let run = client
                .run_public_tool(key, &payload, lab_id.as_deref())
                .await?;
            print_tool_run(&run, format);
        }
        ToolsCommand::Rerun {
            task_id,
            part_id,
            run_id,
            lab_id,
            yes,
        } => {
            let detail = client
                .get_lab_task_part(task_id, part_id, lab_id.as_deref())
                .await?;
            let source = detail
                .runs
                .iter()
                .find(|run| run.id == *run_id)
                .ok_or_else(|| anyhow::anyhow!("Run {run_id} is not visible on this task part"))?;
            let tool_version = source.tool_version_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Run {run_id} has no published ToolVersion provenance; it cannot be rerun"
                )
            })?;
            let manifest = source.manifest_digest.as_deref().ok_or_else(|| {
                anyhow::anyhow!("Run {run_id} has no manifest digest; it cannot be rerun")
            })?;
            let runtime = source.runtime_image_digest.as_deref().ok_or_else(|| {
                anyhow::anyhow!("Run {run_id} has no runtime digest; it cannot be rerun")
            })?;
            let profile = source.execution_profile.as_deref().ok_or_else(|| {
                anyhow::anyhow!("Run {run_id} has no execution profile; it cannot be rerun")
            })?;
            require_confirmation(
                &format!(
                    "Create an exact rerun from {run_id}.\nTool version: {tool_version}\nManifest: {manifest}\nRuntime: {runtime}\nExecution profile: {profile}\nInput artifacts: {}",
                    source.input_artifacts.len()
                ),
                *yes,
            )?;
            let rerun = client
                .rerun_lab_task_part_run(task_id, part_id, run_id, lab_id.as_deref())
                .await?;
            print_exact_rerun(&rerun, format);
        }
        ToolsCommand::ArtifactPreview {
            task_id,
            part_id,
            run_id,
            artifact_index,
            lab_id,
        } => {
            let preview = client
                .get_lab_task_run_artifact_preview(
                    task_id,
                    part_id,
                    run_id,
                    *artifact_index,
                    lab_id.as_deref(),
                )
                .await?;
            match format {
                OutputFormat::Json => print_result(&preview, format),
                OutputFormat::Text => {
                    println!(
                        "{} ({}, truncated={})",
                        preview.filename, preview.preview, preview.truncated
                    );
                    println!("{}", preview.content);
                }
            }
        }
    }
    Ok(())
}

fn read_input_object(path: &str) -> anyhow::Result<serde_json::Value> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("Cannot read input JSON {path}: {error}"))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| anyhow::anyhow!("Cannot parse input JSON {path}: {error}"))?;
    if !value.is_object() {
        anyhow::bail!("Tool input JSON must contain an object");
    }
    Ok(value)
}

fn validate_selected_version_and_files<'a>(
    tool: &'a PublicTool,
    version: &str,
    input: &serde_json::Value,
) -> anyhow::Result<&'a crate::types::PublicToolVersion> {
    let version_info = tool
        .versions
        .iter()
        .find(|candidate| candidate.id == version)
        .ok_or_else(|| {
            anyhow::anyhow!("Version {version} is not published for tool {}", tool.key)
        })?;
    let properties = version_info
        .parameter_schema
        .get("properties")
        .and_then(serde_json::Value::as_object);
    let input = input
        .as_object()
        .expect("read_input_object and callers require an object");
    if let Some(properties) = properties {
        for (key, schema) in properties {
            if schema.get("format").and_then(serde_json::Value::as_str) == Some("file") {
                let Some(value) = input.get(key) else {
                    continue;
                };
                if !is_file_field_ref(value) {
                    anyhow::bail!(
                        "Tool input `{key}` must be a FileFieldRef returned by `scitex files upload`, not a local path, URL, or storage key"
                    );
                }
            }
        }
    }
    Ok(version_info)
}

fn is_file_field_ref(value: &serde_json::Value) -> bool {
    let Some(value) = value.as_object() else {
        return false;
    };
    ["storage_backend", "storage_key", "filename", "content_type"]
        .iter()
        .all(|key| {
            value
                .get(*key)
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.is_empty())
        })
        && value
            .get("size")
            .and_then(serde_json::Value::as_u64)
            .is_some()
}

fn redact_file_refs(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(object) if is_file_field_ref(value) => serde_json::json!({
            "file": object.get("filename"),
            "content_type": object.get("content_type"),
            "size": object.get("size"),
            "sha256": object.get("sha256"),
        }),
        serde_json::Value::Object(object) => serde_json::Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), redact_file_refs(value)))
                .collect(),
        ),
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(redact_file_refs).collect())
        }
        value => value.clone(),
    }
}

fn tool_run_confirmation(
    tool: &PublicTool,
    version: &crate::types::PublicToolVersion,
    validation: &ToolValidation,
    input: &serde_json::Value,
) -> String {
    format!(
        "Create a Tool Catalog run.\nTool: {} ({})\nVersion: {}\nManifest: {}\nStatic timeout: {} seconds\nExpected artifacts: {}\nInput: {}",
        tool.key,
        tool.display_name,
        version.id,
        validation.manifest_digest,
        validation.estimate.timeout_seconds,
        version.artifact_schema.len(),
        redact_file_refs(input)
    )
}

fn print_tool_search(tools: &crate::api_response::PaginatedList<PublicTool>) {
    print_pagination_metadata(tools);
    if tools.items.is_empty() {
        println!("No published tools");
        return;
    }
    for tool in &tools.items {
        let availability = tool
            .submission_availability
            .as_ref()
            .map(|availability| availability.status.as_str())
            .unwrap_or("unknown");
        println!(
            "{}  {}  {}/{}  versions={}  {}",
            tool.key,
            tool.display_name,
            tool.domain,
            tool.family,
            tool.versions.len(),
            availability
        );
    }
}

fn print_public_tool(tool: &PublicTool, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(tool, format),
        OutputFormat::Text => {
            println!("{}  {}", tool.key, tool.display_name);
            if let Some(summary) = &tool.summary {
                println!("{summary}");
            }
            println!("Domain: {}  Family: {}", tool.domain, tool.family);
            println!("License: {}", tool.license_spdx);
            println!("Citation: {}", tool.citation);
            if let Some(availability) = &tool.submission_availability {
                println!(
                    "Submission: {} — {}",
                    availability.status, availability.detail
                );
            }
            println!("Published versions:");
            for version in &tool.versions {
                println!(
                    "  {}  {}  {}",
                    version.id, version.version, version.manifest_digest
                );
            }
        }
    }
}

fn print_validation(validation: &ToolValidation, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(validation, format),
        OutputFormat::Text => {
            println!(
                "Validation: {}  version={}  manifest={}",
                if validation.valid { "valid" } else { "invalid" },
                validation.tool_version_id,
                validation.manifest_digest
            );
            println!(
                "Static timeout: {} seconds",
                validation.estimate.timeout_seconds
            );
            for issue in &validation.issues {
                println!("  {}: {} ({})", issue.path, issue.message, issue.keyword);
            }
        }
    }
}

fn print_tool_run(run: &ToolRun, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(run, format),
        OutputFormat::Text => {
            println!("Tool run task created (execution is asynchronous).");
            println!("Task: {}  Part: {}", run.task_id, run.part_id);
            println!("Tool: {}  Version: {}", run.tool_key, run.tool_version_id);
            println!("Manifest: {}", run.manifest_digest);
            println!(
                "Task status: {}  Part status: {}",
                run.task_status, run.part_status
            );
        }
    }
}

fn print_exact_rerun(rerun: &TaskExactRerun, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(rerun, format),
        OutputFormat::Text => {
            println!("Exact rerun task created (execution is asynchronous).");
            println!("Source run: {}", rerun.source_run_id);
            println!("Task: {}  Part: {}", rerun.task_id, rerun.part_id);
            println!("Tool version: {}", rerun.tool_version_id);
            println!("Manifest: {}", rerun.manifest_digest);
            println!(
                "Task status: {}  Part status: {}",
                rerun.task_status, rerun.part_status
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommand,
    }

    #[derive(Subcommand)]
    enum TestCommand {
        Tool(ToolsArgs),
    }

    #[test]
    fn parses_tool_run_with_confirmation() {
        let cli = TestCli::try_parse_from([
            "scitex",
            "tool",
            "run",
            "primer-qc",
            "--version",
            "version-1",
            "--input",
            "input.json",
            "--yes",
        ])
        .expect("tool run should parse");
        assert!(matches!(
            cli.command,
            TestCommand::Tool(ToolsArgs {
                command: ToolsCommand::Run { yes: true, .. }
            })
        ));
    }

    #[test]
    fn redacts_storage_keys_in_confirmation() {
        let input = serde_json::json!({
            "source": {
                "storage_backend": "minio",
                "storage_key": "private/raw-key",
                "filename": "input.fasta",
                "content_type": "text/plain",
                "size": 42,
                "sha256": "abc"
            }
        });
        let redacted = redact_file_refs(&input);
        assert!(redacted.to_string().contains("input.fasta"));
        assert!(!redacted.to_string().contains("private/raw-key"));
    }

    #[test]
    fn file_field_ref_requires_the_complete_upload_shape() {
        assert!(!is_file_field_ref(&serde_json::json!("source.fasta")));
        assert!(!is_file_field_ref(&serde_json::json!({
            "storage_backend": "minio",
            "storage_key": "private/raw-key",
            "filename": "input.fasta",
            "content_type": "text/plain",
            "size": "42"
        })));
    }
}
