use std::sync::Arc;

use clap::{Args, Subcommand};

use crate::client::ScientexClient;
use crate::config::Config;
use crate::output::{print_paginated_items, print_result, OutputFormat};

#[derive(Args)]
pub struct ProjectArgs {
    /// Project slug, for example `tashan`.
    pub slug: String,
    #[command(subcommand)]
    pub command: ProjectCommand,
}

#[derive(Subcommand)]
pub enum ProjectCommand {
    /// Show project information by slug.
    Info,
    /// Project seed intake workflows.
    Seed {
        #[command(subcommand)]
        command: SeedCommand,
    },
}

#[derive(Subcommand)]
pub enum SeedCommand {
    /// Manage seed object type configs.
    ObjectTypes {
        #[command(subcommand)]
        command: SeedObjectTypesCommand,
    },
    /// Manage seed intake batches.
    Batches {
        #[command(subcommand)]
        command: SeedBatchesCommand,
    },
    /// Manage seed intake records.
    Records {
        #[command(subcommand)]
        command: SeedRecordsCommand,
    },
    /// Query seed stocks.
    Stocks {
        #[command(subcommand)]
        command: SeedStocksCommand,
    },
    /// Show the field catalog for seed intake records.
    FieldCatalog,
}

#[derive(Subcommand)]
pub enum SeedObjectTypesCommand {
    /// List object type configs.
    List,
    /// Create an object type config from an inline JSON object or file.
    Create { data: String },
    /// Show one object type config.
    Get { config_id: String },
    /// Update an object type config from an inline JSON object or file.
    Update { config_id: String, data: String },
}

#[derive(Subcommand)]
pub enum SeedBatchesCommand {
    /// List intake batches.
    List,
    /// Create an intake batch from an inline JSON object or file.
    Create { data: String },
    /// Show one intake batch.
    Get { batch_id: String },
    /// Upload a manifest and create an import task.
    ImportManifest {
        batch_id: String,
        #[arg(long)]
        file: String,
    },
    /// Create a physical intake task for all or selected records.
    CreateIntakeTask {
        batch_id: String,
        #[arg(long = "record-id")]
        record_ids: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum SeedRecordsCommand {
    /// List intake records.
    List {
        #[arg(long)]
        batch_id: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    /// List employee-visible intake records.
    Public {
        #[arg(long)]
        batch_id: Option<String>,
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    /// Show one intake record.
    Get { record_id: String },
    /// Update one intake record from an inline JSON object or file.
    Update { record_id: String, data: String },
    /// Complete one intake record.
    Complete { record_id: String },
}

#[derive(Subcommand)]
pub enum SeedStocksCommand {
    /// List seed stocks.
    List {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    /// Show one seed stock.
    Get { stock_id: String },
}

pub async fn run(
    args: &ProjectArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;

    match &args.command {
        ProjectCommand::Info => {
            let project = client.get_project_by_slug(&args.slug).await?;
            print_result(&project, format);
        }
        ProjectCommand::Seed { command } => {
            run_seed(&client, &args.slug, command, format).await?;
        }
    }
    Ok(())
}

async fn run_seed(
    client: &ScientexClient,
    slug: &str,
    command: &SeedCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        SeedCommand::ObjectTypes { command } => {
            run_seed_object_types(client, slug, command, format).await?
        }
        SeedCommand::Batches { command } => run_seed_batches(client, slug, command, format).await?,
        SeedCommand::Records { command } => run_seed_records(client, slug, command, format).await?,
        SeedCommand::Stocks { command } => run_seed_stocks(client, slug, command, format).await?,
        SeedCommand::FieldCatalog => {
            run_seed_field_catalog(client, slug, format).await?;
        }
    }
    Ok(())
}

async fn run_seed_object_types(
    client: &ScientexClient,
    slug: &str,
    command: &SeedObjectTypesCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        SeedObjectTypesCommand::List => {
            let items = client.list_seed_object_types(slug).await?;
            print_list_or_json(&items, format);
        }
        SeedObjectTypesCommand::Create { data } => {
            let data = read_json_arg_or_file(data)?;
            let item = client.create_seed_object_type(slug, &data).await?;
            print_result(&item, format);
        }
        SeedObjectTypesCommand::Get { config_id } => {
            let item = client.get_seed_object_type(slug, config_id).await?;
            print_result(&item, format);
        }
        SeedObjectTypesCommand::Update { config_id, data } => {
            let data = read_json_arg_or_file(data)?;
            let item = client
                .update_seed_object_type(slug, config_id, &data)
                .await?;
            print_result(&item, format);
        }
    }
    Ok(())
}

async fn run_seed_batches(
    client: &ScientexClient,
    slug: &str,
    command: &SeedBatchesCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        SeedBatchesCommand::List => {
            let items = client.list_seed_intake_batches(slug).await?;
            print_list_or_json(&items, format);
        }
        SeedBatchesCommand::Create { data } => {
            let data = read_json_arg_or_file(data)?;
            let item = client.create_seed_intake_batch(slug, &data).await?;
            print_result(&item, format);
        }
        SeedBatchesCommand::Get { batch_id } => {
            let item = client.get_seed_intake_batch(slug, batch_id).await?;
            print_result(&item, format);
        }
        SeedBatchesCommand::ImportManifest { batch_id, file } => {
            let item = client
                .create_seed_manifest_import_task(slug, batch_id, file)
                .await?;
            print_result(&item, format);
        }
        SeedBatchesCommand::CreateIntakeTask {
            batch_id,
            record_ids,
        } => {
            let item = client
                .create_seed_intake_task(slug, batch_id, record_ids)
                .await?;
            print_result(&item, format);
        }
    }
    Ok(())
}

async fn run_seed_records(
    client: &ScientexClient,
    slug: &str,
    command: &SeedRecordsCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        SeedRecordsCommand::List {
            batch_id,
            status,
            skip,
            limit,
        } => {
            let items = client
                .list_seed_intake_records(
                    slug,
                    batch_id.as_deref(),
                    status.as_deref(),
                    *skip,
                    *limit,
                )
                .await?;
            print_list_or_json(&items, format);
        }
        SeedRecordsCommand::Public {
            batch_id,
            skip,
            limit,
        } => {
            let items = client
                .list_public_seed_intake_records(slug, batch_id.as_deref(), *skip, *limit)
                .await?;
            print_list_or_json(&items, format);
        }
        SeedRecordsCommand::Get { record_id } => {
            let item = client.get_seed_intake_record(slug, record_id).await?;
            print_result(&item, format);
        }
        SeedRecordsCommand::Update { record_id, data } => {
            let data = read_json_arg_or_file(data)?;
            let item = client
                .update_seed_intake_record(slug, record_id, &data)
                .await?;
            print_result(&item, format);
        }
        SeedRecordsCommand::Complete { record_id } => {
            let item = client.complete_seed_intake_record(slug, record_id).await?;
            print_result(&item, format);
        }
    }
    Ok(())
}

async fn run_seed_stocks(
    client: &ScientexClient,
    slug: &str,
    command: &SeedStocksCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        SeedStocksCommand::List { skip, limit } => {
            let items = client.list_seed_stocks(slug, *skip, *limit).await?;
            print_list_or_json(&items, format);
        }
        SeedStocksCommand::Get { stock_id } => {
            let item = client.get_seed_stock(slug, stock_id).await?;
            print_result(&item, format);
        }
    }
    Ok(())
}

async fn run_seed_field_catalog(
    client: &ScientexClient,
    slug: &str,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let data = client.get_seed_field_catalog(slug).await?;
    match format {
        OutputFormat::Json => print_result(&data, format),
        OutputFormat::Text => print_seed_field_catalog(&data),
    }
    Ok(())
}

fn print_seed_field_catalog(data: &serde_json::Value) {
    let items = data
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or_default();
    if items.is_empty() {
        println!("暂无字段元数据");
        return;
    }
    println!(
        "{:<24}  {:<12}  {:<8}  {}",
        "KEY", "LABEL", "TYPE", "CATEGORY"
    );
    for item in items {
        let key = item.get("key").and_then(|v| v.as_str()).unwrap_or("-");
        let label = item.get("label").and_then(|v| v.as_str()).unwrap_or("-");
        let field_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("-");
        let category = item
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        println!("{:<24}  {:<12}  {:<8}  {}", key, label, field_type, category);
        if let Some(desc) = item.get("description").and_then(|v| v.as_str()) {
            if !desc.is_empty() {
                println!("  {}", desc);
            }
        }
    }
}

fn print_list_or_json(
    items: &crate::api_response::PaginatedList<serde_json::Value>,
    format: &OutputFormat,
) {
    match format {
        OutputFormat::Json => print_result(items, format),
        OutputFormat::Text => print_paginated_items(items),
    }
}

fn read_json_arg_or_file(input: &str) -> anyhow::Result<serde_json::Value> {
    if std::path::Path::new(input).exists() {
        let content = std::fs::read_to_string(input)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(serde_json::from_str(input)?)
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
        Project(ProjectArgs),
    }

    fn parse_project(args: &[&str]) -> ProjectArgs {
        let cli = TestCli::try_parse_from(std::iter::once("scitex").chain(args.iter().copied()))
            .expect("project command should parse");
        match cli.command {
            TestCommand::Project(args) => args,
        }
    }

    #[test]
    fn parses_project_info_command() {
        let args = parse_project(&["project", "tashan", "info"]);

        assert_eq!(args.slug, "tashan");
        assert!(matches!(args.command, ProjectCommand::Info));
    }

    #[test]
    fn parses_seed_object_type_commands() {
        let args = parse_project(&["project", "tashan", "seed", "object-types", "list"]);
        assert!(matches!(
            args.command,
            ProjectCommand::Seed {
                command: SeedCommand::ObjectTypes {
                    command: SeedObjectTypesCommand::List
                }
            }
        ));

        let args = parse_project(&[
            "project",
            "tashan",
            "seed",
            "object-types",
            "update",
            "cfg-1",
            r#"{"name":"Seed"}"#,
        ]);
        match args.command {
            ProjectCommand::Seed {
                command:
                    SeedCommand::ObjectTypes {
                        command: SeedObjectTypesCommand::Update { config_id, data },
                    },
            } => {
                assert_eq!(config_id, "cfg-1");
                assert_eq!(data, r#"{"name":"Seed"}"#);
            }
            _ => panic!("expected seed object type update command"),
        }
    }

    #[test]
    fn parses_seed_batch_commands() {
        let args = parse_project(&[
            "project",
            "tashan",
            "seed",
            "batches",
            "import-manifest",
            "batch-1",
            "--file",
            "manifest.xlsx",
        ]);
        match args.command {
            ProjectCommand::Seed {
                command:
                    SeedCommand::Batches {
                        command: SeedBatchesCommand::ImportManifest { batch_id, file },
                    },
            } => {
                assert_eq!(batch_id, "batch-1");
                assert_eq!(file, "manifest.xlsx");
            }
            _ => panic!("expected seed manifest command"),
        }

        let args = parse_project(&[
            "project",
            "tashan",
            "seed",
            "batches",
            "create-intake-task",
            "batch-1",
            "--record-id",
            "rec-1",
            "--record-id",
            "rec-2",
        ]);
        match args.command {
            ProjectCommand::Seed {
                command:
                    SeedCommand::Batches {
                        command:
                            SeedBatchesCommand::CreateIntakeTask {
                                batch_id,
                                record_ids,
                            },
                    },
            } => {
                assert_eq!(batch_id, "batch-1");
                assert_eq!(record_ids, vec!["rec-1".to_string(), "rec-2".to_string()]);
            }
            _ => panic!("expected seed intake task command"),
        }
    }

    #[test]
    fn parses_seed_record_and_stock_commands() {
        let args = parse_project(&[
            "project",
            "tashan",
            "seed",
            "records",
            "list",
            "--batch-id",
            "batch-1",
            "--status",
            "pending",
            "--skip",
            "10",
            "--limit",
            "20",
        ]);
        match args.command {
            ProjectCommand::Seed {
                command:
                    SeedCommand::Records {
                        command:
                            SeedRecordsCommand::List {
                                batch_id,
                                status,
                                skip,
                                limit,
                            },
                    },
            } => {
                assert_eq!(batch_id.as_deref(), Some("batch-1"));
                assert_eq!(status.as_deref(), Some("pending"));
                assert_eq!(skip, 10);
                assert_eq!(limit, 20);
            }
            _ => panic!("expected seed records list command"),
        }

        let args = parse_project(&["project", "tashan", "seed", "stocks", "get", "stock-1"]);
        assert!(matches!(
            args.command,
            ProjectCommand::Seed {
                command: SeedCommand::Stocks {
                    command: SeedStocksCommand::Get { .. }
                }
            }
        ));
    }

    #[test]
    fn rejects_unknown_project_subcommand() {
        assert!(TestCli::try_parse_from(["scitex", "project", "tashan", "unknown"]).is_err());
    }

    #[test]
    fn rejects_removed_project_workflow_commands() {
        assert!(TestCli::try_parse_from(["scitex", "project", "tashan", "germplasm"]).is_err());
        assert!(TestCli::try_parse_from(["scitex", "project", "tashan", "planting"]).is_err());
    }

    #[test]
    fn parses_seed_field_catalog_command() {
        let args = parse_project(&["project", "tashan", "seed", "field-catalog"]);
        assert_eq!(args.slug, "tashan");
        assert!(matches!(
            args.command,
            ProjectCommand::Seed {
                command: SeedCommand::FieldCatalog
            }
        ));
    }

    #[test]
    fn print_seed_field_catalog_does_not_panic() {
        let data = serde_json::json!([
            {
                "key": "sample_name",
                "label": "样品名称",
                "type": "string",
                "category": "标识",
                "description": "样品的唯一名称"
            },
            {
                "key": "intake_date",
                "label": "入库日期",
                "type": "date",
                "category": "流程",
                "description": ""
            }
        ]);
        // The function prints to stdout; just confirm it does not panic.
        super::print_seed_field_catalog(&data);

        // Empty array should also not panic
        super::print_seed_field_catalog(&serde_json::json!([]));

        // Non-array input (falls through to empty slice) should not panic
        super::print_seed_field_catalog(&serde_json::json!({ "items": [] }));
    }
}
