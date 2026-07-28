use std::sync::Arc;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::client::ScientexClient;
use crate::commands::confirm::require_confirmation;
use crate::config::Config;
use crate::output::{print_paginated_items, print_result, OutputFormat};
use crate::types::{
    ManifestImportTask, SeedIntakeBatch, SeedIntakeTask, SeedLot, SeedMovement, SeedPlacement,
    SeedReservation,
};

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
    /// Formal seed weight ledger (SeedLot), replacing stocks for new workflows.
    Lots {
        #[command(subcommand)]
        command: SeedLotsCommand,
    },
    /// Actions on seed lot reservations.
    Reservations {
        #[command(subcommand)]
        command: SeedReservationsCommand,
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
    /// Create an intake batch. Use --object-type-config for the normal flow;
    /// the JSON_OR_FILE positional form is retained for compatibility.
    Create {
        #[arg(
            value_name = "JSON_OR_FILE",
            required_unless_present = "object_type_config",
            conflicts_with = "object_type_config"
        )]
        data: Option<String>,
        #[arg(long)]
        object_type_config: Option<String>,
        #[arg(long, requires = "object_type_config")]
        batch_code: Option<String>,
    },
    /// Show one intake batch.
    Get { batch_id: String },
    /// Download the XLSX template frozen for this batch.
    DownloadTemplate {
        batch_id: String,
        #[arg(long)]
        out: Option<std::path::PathBuf>,
        #[arg(long, default_value_t = false)]
        force: bool,
    },
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
        /// Fetch all pages automatically.
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// List employee-visible intake records.
    Public {
        #[arg(long)]
        batch_id: Option<String>,
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        /// Fetch all pages automatically.
        #[arg(long, default_value_t = false)]
        all: bool,
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
        /// Fetch all pages automatically.
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// Show one seed stock.
    Get { stock_id: String },
}

#[derive(Subcommand)]
pub enum SeedLotsCommand {
    /// List formal seed lots. The old stocks command is migration compatibility only.
    List {
        #[arg(long = "type")]
        seed_type_code: Option<String>,
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// Show a formal seed lot and its current balances.
    Get { lot_id: String },
    /// List immutable movements for a lot.
    Movements { lot_id: String },
    /// List reservations for a lot.
    Reservations { lot_id: String },
    /// Reserve available lot weight in grams.
    Reserve {
        lot_id: String,
        #[arg(long)]
        weight_g: String,
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        experiment_ref: Option<String>,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    /// Checkout lot weight in grams.
    Checkout {
        lot_id: String,
        #[arg(long)]
        weight_g: String,
        #[arg(long)]
        reservation: Option<String>,
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        experiment_ref: Option<String>,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    /// Change a lot's current placement.
    Transfer {
        lot_id: String,
        #[arg(long)]
        location_id: Option<String>,
        #[arg(long)]
        site: Option<String>,
        #[arg(long)]
        location_text: Option<String>,
        #[arg(long)]
        note: Option<String>,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    /// Record an audited weight adjustment in grams.
    Adjust {
        lot_id: String,
        #[arg(long = "type")]
        movement_type: SeedAdjustmentTypeArg,
        #[arg(long, allow_hyphen_values = true)]
        weight_delta_g: String,
        #[arg(long)]
        reason: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum SeedReservationsCommand {
    /// Release a reservation.
    Release {
        reservation_id: String,
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SeedAdjustmentTypeArg {
    Adjustment,
    Loss,
    #[value(name = "migration_correction")]
    MigrationCorrection,
}

impl SeedAdjustmentTypeArg {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Adjustment => "adjustment",
            Self::Loss => "loss",
            Self::MigrationCorrection => "migration_correction",
        }
    }
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
        SeedCommand::Lots { command } => run_seed_lots(client, slug, command, format).await?,
        SeedCommand::Reservations { command } => {
            run_seed_reservations(client, slug, command, format).await?
        }
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
        SeedBatchesCommand::Create {
            data,
            object_type_config,
            batch_code,
        } => {
            let data = match data {
                Some(data) => read_json_arg_or_file(data)?,
                None => serde_json::json!({
                    "object_type_config_id": object_type_config
                        .as_ref()
                        .expect("clap requires --object-type-config when JSON is absent"),
                    "batch_code": batch_code,
                }),
            };
            let item = client.create_seed_intake_batch(slug, &data).await?;
            print_seed_batch(&item, format);
        }
        SeedBatchesCommand::Get { batch_id } => {
            let item = client.get_seed_intake_batch(slug, batch_id).await?;
            print_seed_batch(&item, format);
        }
        SeedBatchesCommand::DownloadTemplate {
            batch_id,
            out,
            force,
        } => {
            let (path, server_filename) = client
                .download_seed_manifest_template(slug, batch_id, out.as_deref(), *force)
                .await?;
            match format {
                OutputFormat::Json => print_result(
                    &serde_json::json!({
                        "path": path,
                        "server_filename": server_filename,
                    }),
                    format,
                ),
                OutputFormat::Text => {
                    println!("Downloaded frozen manifest template: {}", path.display());
                    if out.is_some() {
                        println!("Server filename: {server_filename}");
                    }
                }
            }
        }
        SeedBatchesCommand::ImportManifest { batch_id, file } => {
            validate_manifest_file(file)?;
            let item = client
                .create_seed_manifest_import_task(slug, batch_id, file)
                .await?;
            print_manifest_import(&item, format);
        }
        SeedBatchesCommand::CreateIntakeTask {
            batch_id,
            record_ids,
        } => {
            let item = client
                .create_seed_intake_task(slug, batch_id, record_ids)
                .await?;
            print_seed_intake_task(&item, format);
        }
    }
    Ok(())
}

async fn run_seed_lots(
    client: &ScientexClient,
    slug: &str,
    command: &SeedLotsCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        SeedLotsCommand::List {
            seed_type_code,
            skip,
            limit,
            all,
        } => {
            let lots = if *all {
                let code = seed_type_code.clone();
                crate::client::collect_all_pages(100, |skip, limit| {
                    client.list_seed_lots(slug, code.as_deref(), skip, limit)
                })
                .await?
            } else {
                client
                    .list_seed_lots(slug, seed_type_code.as_deref(), *skip, *limit)
                    .await?
            };
            print_seed_lots(&lots, format);
        }
        SeedLotsCommand::Get { lot_id } => {
            let lot = client.get_seed_lot(slug, lot_id).await?;
            print_seed_lot(&lot, format);
        }
        SeedLotsCommand::Movements { lot_id } => {
            let movements = client.list_seed_lot_movements(slug, lot_id).await?;
            print_seed_movements(&movements, format);
        }
        SeedLotsCommand::Reservations { lot_id } => {
            let reservations = client.list_seed_lot_reservations(slug, lot_id).await?;
            print_seed_reservations(&reservations, format);
        }
        SeedLotsCommand::Reserve {
            lot_id,
            weight_g,
            purpose,
            experiment_ref,
            yes,
        } => {
            let weight_g = validated_decimal(weight_g, false)?;
            require_confirmation(
                &format!("Reserve {weight_g} g from seed lot {lot_id}."),
                *yes,
            )?;
            let data = serde_json::json!({
                "weight_g": weight_g,
                "purpose": purpose,
                "experiment_ref": experiment_ref,
            });
            let reservation = client.reserve_seed_lot(slug, lot_id, &data).await?;
            print_seed_reservation(&reservation, format);
        }
        SeedLotsCommand::Checkout {
            lot_id,
            weight_g,
            reservation,
            purpose,
            experiment_ref,
            yes,
        } => {
            let weight_g = validated_decimal(weight_g, false)?;
            require_confirmation(
                &format!("Checkout {weight_g} g from seed lot {lot_id}."),
                *yes,
            )?;
            let data = serde_json::json!({
                "weight_g": weight_g,
                "reservation_id": reservation,
                "purpose": purpose,
                "experiment_ref": experiment_ref,
            });
            let movement = client.checkout_seed_lot(slug, lot_id, &data).await?;
            print_seed_movement(&movement, format);
        }
        SeedLotsCommand::Transfer {
            lot_id,
            location_id,
            site,
            location_text,
            note,
            yes,
        } => {
            if location_id.is_none() && site.is_none() && location_text.is_none() && note.is_none()
            {
                anyhow::bail!(
                    "Transfer requires at least one of --location-id, --site, --location-text, or --note"
                );
            }
            require_confirmation(
                &format!("Transfer seed lot {lot_id} to the requested placement."),
                *yes,
            )?;
            let data = serde_json::json!({
                "storage_location_id": location_id,
                "storage_site": site,
                "storage_location_text": location_text,
                "note": note,
            });
            let placement = client.transfer_seed_lot(slug, lot_id, &data).await?;
            print_seed_placement(&placement, format);
        }
        SeedLotsCommand::Adjust {
            lot_id,
            movement_type,
            weight_delta_g,
            reason,
            yes,
        } => {
            let weight_delta_g = validated_decimal(weight_delta_g, true)?;
            let reason = required_non_empty(reason, "--reason")?;
            require_confirmation(
                &format!(
                    "Record {} {} g for seed lot {lot_id}: {reason}",
                    movement_type.as_str(),
                    weight_delta_g
                ),
                *yes,
            )?;
            let data = serde_json::json!({
                "movement_type": movement_type.as_str(),
                "weight_delta_g": weight_delta_g,
                "reason": reason,
            });
            let movement = client.adjust_seed_lot(slug, lot_id, &data).await?;
            print_seed_movement(&movement, format);
        }
    }
    Ok(())
}

async fn run_seed_reservations(
    client: &ScientexClient,
    slug: &str,
    command: &SeedReservationsCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        SeedReservationsCommand::Release {
            reservation_id,
            yes,
        } => {
            require_confirmation(&format!("Release seed reservation {reservation_id}."), *yes)?;
            let reservation = client
                .release_seed_reservation(slug, reservation_id)
                .await?;
            print_seed_reservation(&reservation, format);
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
            all,
        } => {
            let items = if *all {
                let b = batch_id.clone();
                let st = status.clone();
                crate::client::collect_all_pages(200, |s, l| {
                    client.list_seed_intake_records(slug, b.as_deref(), st.as_deref(), s, l)
                })
                .await?
            } else {
                client
                    .list_seed_intake_records(
                        slug,
                        batch_id.as_deref(),
                        status.as_deref(),
                        *skip,
                        *limit,
                    )
                    .await?
            };
            print_list_or_json(&items, format);
        }
        SeedRecordsCommand::Public {
            batch_id,
            skip,
            limit,
            all,
        } => {
            let items = if *all {
                let b = batch_id.clone();
                crate::client::collect_all_pages(200, |s, l| {
                    client.list_public_seed_intake_records(slug, b.as_deref(), s, l)
                })
                .await?
            } else {
                client
                    .list_public_seed_intake_records(slug, batch_id.as_deref(), *skip, *limit)
                    .await?
            };
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
        SeedStocksCommand::List { skip, limit, all } => {
            let items = if *all {
                crate::client::collect_all_pages(200, |s, l| client.list_seed_stocks(slug, s, l))
                    .await?
            } else {
                client.list_seed_stocks(slug, *skip, *limit).await?
            };
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
    let items = data.as_array().map(|a| a.as_slice()).unwrap_or_default();
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
        let category = item.get("category").and_then(|v| v.as_str()).unwrap_or("-");
        println!(
            "{:<24}  {:<12}  {:<8}  {}",
            key, label, field_type, category
        );
        if let Some(desc) = item.get("description").and_then(|v| v.as_str()) {
            if !desc.is_empty() {
                println!("  {}", desc);
            }
        }
    }
}

fn print_list_or_json<T: Serialize>(
    items: &crate::api_response::PaginatedList<T>,
    format: &OutputFormat,
) {
    match format {
        OutputFormat::Json => print_result(items, format),
        OutputFormat::Text => print_paginated_items(items),
    }
}

fn print_seed_batch(batch: &SeedIntakeBatch, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(batch, format),
        OutputFormat::Text => {
            println!("Batch: {} ({})", batch.batch_code, batch.id);
            println!("Status: {}", batch.status);
            println!("Object type config: {}", batch.object_type_config_id);
            if let Some(task_id) = &batch.import_task_id {
                println!("Import task: {task_id}");
            }
            if let Some(summary) = &batch.last_import_summary {
                println!("Last import summary: {summary}");
            }
        }
    }
}

fn print_manifest_import(item: &ManifestImportTask, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(item, format),
        OutputFormat::Text => {
            println!("Manifest import task created (upload is not import completion).");
            println!("Batch: {}", item.batch_id);
            println!("Task: {}", item.task_id);
            println!("Part: {}", item.part_id);
            if let Some(document_id) = &item.source_file_document_id {
                println!("Source file document: {document_id}");
            }
            println!(
                "Next: scitex project <slug> seed batches get {}",
                item.batch_id
            );
        }
    }
}

fn print_seed_intake_task(item: &SeedIntakeTask, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(item, format),
        OutputFormat::Text => {
            println!("Seed intake workflow created: {}", item.task_id);
            println!("Records: {}", item.record_count);
            println!("Work order part: {}", item.work_order_part_id);
            println!("Physical intake part: {}", item.physical_intake_part_id);
            println!("Backfill part: {}", item.backfill_part_id);
        }
    }
}

fn print_seed_lots(lots: &crate::api_response::PaginatedList<SeedLot>, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(lots, format),
        OutputFormat::Text => {
            if lots.items.is_empty() {
                println!("No seed lots");
                return;
            }
            println!("LOT_NO  TYPE  ON_HAND(g)  RESERVED(g)  AVAILABLE(g)  STATUS");
            for lot in &lots.items {
                println!(
                    "{}  {}  {}  {}  {}  {}",
                    lot.lot_no,
                    lot.seed_type_code,
                    lot.on_hand_weight_g,
                    lot.reserved_weight_g,
                    lot.available_weight_g,
                    lot.status
                );
            }
        }
    }
}

fn print_seed_lot(lot: &SeedLot, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(lot, format),
        OutputFormat::Text => {
            println!("Lot: {} ({})", lot.lot_no, lot.id);
            println!("Type: {}  Status: {}", lot.seed_type_code, lot.status);
            println!("Initial: {} g", lot.initial_weight_g);
            println!("On hand: {} g", lot.on_hand_weight_g);
            println!("Reserved: {} g", lot.reserved_weight_g);
            println!("Available: {} g", lot.available_weight_g);
        }
    }
}

fn print_seed_movements(
    movements: &crate::api_response::PaginatedList<SeedMovement>,
    format: &OutputFormat,
) {
    match format {
        OutputFormat::Json => print_result(movements, format),
        OutputFormat::Text => {
            if movements.items.is_empty() {
                println!("No lot movements");
                return;
            }
            println!("MOVEMENT  TYPE  ON_HAND Δ/AFTER(g)  RESERVED Δ/AFTER(g)  TIME");
            for movement in &movements.items {
                println!(
                    "{}  {}  {}/{}  {}/{}  {}",
                    movement.movement_no,
                    movement.movement_type,
                    movement.on_hand_delta_g,
                    movement.on_hand_after_g,
                    movement.reserved_delta_g,
                    movement.reserved_after_g,
                    movement.occurred_at
                );
            }
        }
    }
}

fn print_seed_movement(movement: &SeedMovement, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(movement, format),
        OutputFormat::Text => {
            println!(
                "Movement: {} ({})",
                movement.movement_no, movement.movement_type
            );
            println!("On hand: {} g", movement.on_hand_after_g);
            println!("Reserved: {} g", movement.reserved_after_g);
        }
    }
}

fn print_seed_reservations(
    reservations: &crate::api_response::PaginatedList<SeedReservation>,
    format: &OutputFormat,
) {
    match format {
        OutputFormat::Json => print_result(reservations, format),
        OutputFormat::Text => {
            if reservations.items.is_empty() {
                println!("No lot reservations");
                return;
            }
            println!("RESERVATION  WEIGHT(g)  STATUS  PURPOSE");
            for reservation in &reservations.items {
                println!(
                    "{}  {}  {}  {}",
                    reservation.reservation_no,
                    reservation.weight_g,
                    reservation.status,
                    reservation.purpose.as_deref().unwrap_or("-")
                );
            }
        }
    }
}

fn print_seed_reservation(reservation: &SeedReservation, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(reservation, format),
        OutputFormat::Text => {
            println!(
                "Reservation: {}  {} g  {}",
                reservation.reservation_no, reservation.weight_g, reservation.status
            );
        }
    }
}

fn print_seed_placement(placement: &SeedPlacement, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(placement, format),
        OutputFormat::Text => {
            println!("Placement: {}", placement.id);
            println!("Lot: {}", placement.lot_id);
            println!("Current: {}", placement.is_current);
            println!(
                "Location: {} {}",
                placement.storage_site.as_deref().unwrap_or("-"),
                placement.storage_location_text.as_deref().unwrap_or("-")
            );
        }
    }
}

fn validated_decimal(value: &str, allow_negative: bool) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        anyhow::bail!("Weight must be a non-empty decimal string");
    }
    let (negative, unsigned) = match value.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, value),
    };
    if negative && !allow_negative {
        anyhow::bail!("Weight must be greater than zero");
    }
    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if parts.next().is_some()
        || (whole.is_empty() && fraction.unwrap_or_default().is_empty())
        || !whole.chars().all(|ch| ch.is_ascii_digit())
        || fraction.is_some_and(|part| {
            part.is_empty() || part.len() > 4 || !part.chars().all(|ch| ch.is_ascii_digit())
        })
    {
        anyhow::bail!("Weight must be a decimal string with at most four decimal places");
    }
    let nonzero = whole.chars().any(|ch| ch != '0')
        || fraction.is_some_and(|part| part.chars().any(|ch| ch != '0'));
    if !nonzero {
        anyhow::bail!("Weight must be non-zero");
    }
    Ok(value.to_string())
}

fn required_non_empty(value: &str, argument: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        anyhow::bail!("{argument} must be non-empty");
    }
    Ok(value.to_string())
}

fn validate_manifest_file(path: &str) -> anyhow::Result<()> {
    let path = std::path::Path::new(path);
    let metadata = std::fs::metadata(path)
        .map_err(|error| anyhow::anyhow!("Cannot read manifest {}: {error}", path.display()))?;
    if !metadata.is_file() {
        anyhow::bail!("Manifest must be a regular file: {}", path.display());
    }
    if metadata.len() == 0 {
        anyhow::bail!("Manifest must not be empty: {}", path.display());
    }
    let is_xlsx = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xlsx"));
    if !is_xlsx {
        anyhow::bail!("Manifest must use the .xlsx extension: {}", path.display());
    }
    std::fs::File::open(path)
        .map_err(|error| anyhow::anyhow!("Manifest is not readable {}: {error}", path.display()))?;
    Ok(())
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
            "create",
            "--object-type-config",
            "config-1",
            "--batch-code",
            "BATCH-001",
        ]);
        assert!(matches!(
            args.command,
            ProjectCommand::Seed {
                command: SeedCommand::Batches {
                    command: SeedBatchesCommand::Create {
                        data: None,
                        object_type_config: Some(_),
                        batch_code: Some(_),
                    }
                }
            }
        ));

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
                                ..
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

    #[test]
    fn parses_seed_stocks_list_with_all_flag() {
        let args = parse_project(&["project", "tashan", "seed", "stocks", "list", "--all"]);
        assert_eq!(args.slug, "tashan");
        assert!(matches!(
            args.command,
            ProjectCommand::Seed {
                command: SeedCommand::Stocks {
                    command: SeedStocksCommand::List { all: true, .. }
                }
            }
        ));
    }

    #[test]
    fn seed_stocks_list_all_flag_defaults_to_false() {
        let args = parse_project(&["project", "tashan", "seed", "stocks", "list"]);
        assert!(matches!(
            args.command,
            ProjectCommand::Seed {
                command: SeedCommand::Stocks {
                    command: SeedStocksCommand::List { all: false, .. }
                }
            }
        ));
    }

    #[test]
    fn parses_seed_records_list_with_all_flag() {
        let args = parse_project(&["project", "tashan", "seed", "records", "list", "--all"]);
        assert!(matches!(
            args.command,
            ProjectCommand::Seed {
                command: SeedCommand::Records {
                    command: SeedRecordsCommand::List { all: true, .. }
                }
            }
        ));
    }

    #[test]
    fn parses_seed_records_public_with_all_flag() {
        let args = parse_project(&["project", "tashan", "seed", "records", "public", "--all"]);
        assert!(matches!(
            args.command,
            ProjectCommand::Seed {
                command: SeedCommand::Records {
                    command: SeedRecordsCommand::Public { all: true, .. }
                }
            }
        ));
    }

    #[test]
    fn parses_seed_lot_write_commands_with_yes() {
        let reserve = parse_project(&[
            "project",
            "tashan",
            "seed",
            "lots",
            "reserve",
            "lot-1",
            "--weight-g",
            "1.2500",
            "--yes",
        ]);
        assert!(matches!(
            reserve.command,
            ProjectCommand::Seed {
                command: SeedCommand::Lots {
                    command: SeedLotsCommand::Reserve { yes: true, .. }
                }
            }
        ));

        let adjustment = parse_project(&[
            "project",
            "tashan",
            "seed",
            "lots",
            "adjust",
            "lot-1",
            "--type",
            "loss",
            "--weight-delta-g",
            "-0.125",
            "--reason",
            "tube leaked",
            "--yes",
        ]);
        assert!(matches!(
            adjustment.command,
            ProjectCommand::Seed {
                command: SeedCommand::Lots {
                    command: SeedLotsCommand::Adjust { yes: true, .. }
                }
            }
        ));
    }

    #[test]
    fn validates_decimal_weight_strings_without_floating_point() {
        assert_eq!(validated_decimal("1.2345", false).unwrap(), "1.2345");
        assert_eq!(validated_decimal("-0.25", true).unwrap(), "-0.25");
        for invalid in ["0", "-1", "1.23456", "1e3", "+1"] {
            assert!(
                validated_decimal(invalid, false).is_err(),
                "{invalid} should fail"
            );
        }
    }
}
