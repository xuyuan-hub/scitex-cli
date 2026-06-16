use std::sync::Arc;

use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::BiolabClient;
use crate::config::Config;
use crate::output::{
    print_paginated_items, print_pagination_metadata, print_result, print_stocks, OutputFormat,
};
use crate::types::{InventoryItem, Stock};

#[derive(Args)]
pub struct InventoryArgs {
    #[command(subcommand)]
    pub command: InventoryCommand,
}

#[derive(Subcommand)]
pub enum InventoryCommand {
    /// List inventory stocks.
    List {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        primer_name: Option<String>,
        #[arg(long)]
        location_id: Option<String>,
        #[arg(long)]
        low_stock: bool,
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Show inventory stock details.
    Get { id: String },
    /// Show inventory stock transactions.
    Transactions { id: String },
    /// Show legacy inventory statistics.
    Stats,
    /// List generic inventory item definitions.
    Items {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        supplier: Option<String>,
        #[arg(long)]
        filters: Option<String>,
    },
    /// Show a generic inventory item definition.
    Item { id: String },
    /// Create a generic inventory item from a JSON file.
    CreateItem { file: String },
    /// Update a generic inventory item from a JSON file.
    UpdateItem { id: String, file: String },
    /// Disable a generic inventory item.
    DisableItem { id: String },
    /// Create a stock batch from a JSON file.
    CreateStock { file: String },
    /// Show inventory summary.
    Summary {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        filters: Option<String>,
    },
    /// List all inventory transactions.
    TransactionsAll {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long = "type")]
        transaction_type: Option<String>,
        #[arg(long)]
        item_id: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        filters: Option<String>,
    },
    /// Check experiment inventory requirements using current query results.
    Check { file: String },
    /// Show inventory preferences.
    Preferences {
        #[arg(long)]
        workflow_type: Option<String>,
    },
    /// Update inventory preferences with a JSON object or JSON file.
    SetPreferences { data: String },
    /// Check in stock.
    Checkin {
        id: String,
        #[arg(long)]
        quantity: f64,
        #[arg(long)]
        purpose: Option<String>,
    },
    /// Check out stock.
    Checkout {
        id: String,
        #[arg(long)]
        quantity: f64,
        #[arg(long)]
        recipient: Option<String>,
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        experiment_ref: Option<String>,
        #[arg(long)]
        task_id: Option<String>,
        #[arg(long)]
        part_id: Option<String>,
        #[arg(long)]
        requirement_key: Option<String>,
    },
    /// Check out stock by item using backend FIFO.
    CheckoutItem {
        item_id: String,
        #[arg(long)]
        quantity: f64,
        #[arg(long)]
        recipient: Option<String>,
        #[arg(long)]
        purpose: Option<String>,
        #[arg(long)]
        experiment_ref: Option<String>,
        #[arg(long)]
        task_id: Option<String>,
        #[arg(long)]
        part_id: Option<String>,
        #[arg(long)]
        requirement_key: Option<String>,
    },
    /// Adjust stock quantity.
    Adjust {
        id: String,
        #[arg(long)]
        quantity: f64,
        #[arg(long = "type", default_value = "correction")]
        adjustment_type: String,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Transfer stock to another storage location.
    Transfer {
        id: String,
        #[arg(long)]
        location_id: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },
    /// List storage locations.
    Locations,
    /// Create a storage location.
    CreateLocation {
        name: String,
        #[arg(long)]
        parent_id: Option<String>,
    },
}

pub async fn run(
    args: &InventoryArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = BiolabClient::new(Arc::clone(config))?;

    match &args.command {
        InventoryCommand::List {
            name,
            primer_name,
            location_id,
            low_stock,
            skip,
            limit,
            lab_id,
        } => {
            let query_name = name.as_deref().or(primer_name.as_deref());
            let stocks = if lab_id.is_some() {
                client
                    .list_lab_stocks(
                        lab_id.as_deref(),
                        query_name,
                        location_id.as_deref(),
                        *low_stock,
                        *skip,
                        *limit,
                    )
                    .await?
            } else {
                client
                    .list_stocks(
                        query_name,
                        location_id.as_deref(),
                        *low_stock,
                        *skip,
                        *limit,
                    )
                    .await?
            };
            match format {
                OutputFormat::Json => print_result(&stocks, format),
                OutputFormat::Text => {
                    print_pagination_metadata(&stocks);
                    print_stocks(&stocks.items);
                }
            }
        }
        InventoryCommand::Get { id } => {
            let stock = client.get_stock(id).await?;
            print_result(&stock, format);
        }
        InventoryCommand::Transactions { id } => {
            let transactions = client.list_stock_transactions(id).await?;
            match format {
                OutputFormat::Json => print_result(&transactions, format),
                OutputFormat::Text => print_paginated_items(&transactions),
            }
        }
        InventoryCommand::Stats => {
            let stats = client.get_stock_stats().await?;
            print_result(&stats, format);
        }
        InventoryCommand::Items {
            skip,
            limit,
            search,
            category,
            supplier,
            filters,
        } => {
            let items = client
                .list_items(
                    *skip,
                    *limit,
                    search.as_deref(),
                    category.as_deref(),
                    supplier.as_deref(),
                    filters.as_deref(),
                )
                .await?;
            match format {
                OutputFormat::Json => print_result(&items, format),
                OutputFormat::Text => print_paginated_items(&items),
            }
        }
        InventoryCommand::Item { id } => {
            let item = client.get_item(id).await?;
            print_result(&item, format);
        }
        InventoryCommand::CreateItem { file } => {
            let data = read_json_value(file)?;
            let item = client.create_item(&data).await?;
            print_result(&item, format);
        }
        InventoryCommand::UpdateItem { id, file } => {
            let data = read_json_value(file)?;
            let item = client.update_item(id, &data).await?;
            print_result(&item, format);
        }
        InventoryCommand::DisableItem { id } => {
            let response = client.disable_item(id).await?;
            print_result(&response, format);
        }
        InventoryCommand::CreateStock { file } => {
            let data = read_json_value(file)?;
            let stock = client.create_stock(&data).await?;
            print_result(&stock, format);
        }
        InventoryCommand::Summary {
            skip,
            limit,
            search,
            category,
            filters,
        } => {
            let summary = client
                .inventory_summary(
                    *skip,
                    *limit,
                    search.as_deref(),
                    category.as_deref(),
                    filters.as_deref(),
                )
                .await?;
            print_result(&summary, format);
        }
        InventoryCommand::TransactionsAll {
            skip,
            limit,
            transaction_type,
            item_id,
            search,
            filters,
        } => {
            let transactions = client
                .list_inventory_transactions(
                    *skip,
                    *limit,
                    transaction_type.as_deref(),
                    item_id.as_deref(),
                    search.as_deref(),
                    filters.as_deref(),
                )
                .await?;
            print_result(&transactions, format);
        }
        InventoryCommand::Check { file } => {
            let requirements = read_inventory_requirements(file)?;
            let report = check_inventory_requirements(&client, requirements).await?;
            print_result(&report, format);
        }
        InventoryCommand::Preferences { workflow_type } => {
            let preferences = client
                .get_inventory_preferences(workflow_type.as_deref())
                .await?;
            print_result(&preferences, format);
        }
        InventoryCommand::SetPreferences { data } => {
            let data = read_json_arg_or_file(data)?;
            let preferences = client.set_inventory_preferences(&data).await?;
            print_result(&preferences, format);
        }
        InventoryCommand::Checkin {
            id,
            quantity,
            purpose,
        } => {
            validate_positive_quantity(*quantity)?;
            let transaction = client.checkin(id, *quantity, purpose.as_deref()).await?;
            print_result(&transaction, format);
        }
        InventoryCommand::Checkout {
            id,
            quantity,
            recipient,
            purpose,
            experiment_ref,
            task_id,
            part_id,
            requirement_key,
        } => {
            validate_positive_quantity(*quantity)?;
            let transaction = client
                .checkout(
                    id,
                    *quantity,
                    recipient.as_deref(),
                    purpose.as_deref(),
                    experiment_ref.as_deref(),
                    task_id.as_deref(),
                    part_id.as_deref(),
                    requirement_key.as_deref(),
                )
                .await?;
            print_result(&transaction, format);
        }
        InventoryCommand::CheckoutItem {
            item_id,
            quantity,
            recipient,
            purpose,
            experiment_ref,
            task_id,
            part_id,
            requirement_key,
        } => {
            validate_positive_quantity(*quantity)?;
            let response = client
                .checkout_item(
                    item_id,
                    *quantity,
                    recipient.as_deref(),
                    purpose.as_deref(),
                    experiment_ref.as_deref(),
                    task_id.as_deref(),
                    part_id.as_deref(),
                    requirement_key.as_deref(),
                )
                .await?;
            print_result(&response, format);
        }
        InventoryCommand::Adjust {
            id,
            quantity,
            adjustment_type,
            reason,
        } => {
            validate_adjustment_quantity(*quantity)?;
            validate_adjustment_type(adjustment_type)?;
            let transaction = client
                .adjust_stock(id, *quantity, adjustment_type, reason.as_deref())
                .await?;
            print_result(&transaction, format);
        }
        InventoryCommand::Transfer {
            id,
            location_id,
            reason,
        } => {
            let stock = client
                .transfer_stock(id, location_id.as_deref(), reason.as_deref())
                .await?;
            print_result(&stock, format);
        }
        InventoryCommand::Locations => {
            let locations = client.list_locations().await?;
            match format {
                OutputFormat::Json => print_result(&locations, format),
                OutputFormat::Text => print_paginated_items(&locations),
            }
        }
        InventoryCommand::CreateLocation { name, parent_id } => {
            let location = client.create_location(name, parent_id.as_deref()).await?;
            print_result(&location, format);
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct InventoryRequirement {
    key: Option<String>,
    item_id: Option<String>,
    name: String,
    quantity: f64,
    unit: Option<String>,
    category: Option<String>,
    supplier: Option<String>,
}

async fn check_inventory_requirements(
    client: &BiolabClient,
    requirements: Vec<InventoryRequirement>,
) -> anyhow::Result<serde_json::Value> {
    let mut rows = Vec::with_capacity(requirements.len());
    let mut available_count = 0;
    let mut insufficient_count = 0;
    let mut missing_count = 0;
    let mut ambiguous_count = 0;

    for req in requirements {
        let candidates = resolve_candidate_items(client, &req).await?;
        if candidates.is_empty() {
            missing_count += 1;
            rows.push(requirement_row(
                &req,
                "missing_item",
                None,
                Vec::new(),
                0.0,
                vec!["No matching inventory item was found.".to_string()],
            ));
            continue;
        }
        if candidates.len() > 1 {
            ambiguous_count += 1;
            let candidate_items = candidates
                .iter()
                .map(|item| {
                    json!({
                        "id": item.id,
                        "name": item.name,
                        "category": item.category,
                        "supplier": item.supplier,
                        "unit": item.unit,
                    })
                })
                .collect();
            rows.push(requirement_row(
                &req,
                "ambiguous_item",
                None,
                candidate_items,
                0.0,
                vec!["Multiple matching inventory items were found.".to_string()],
            ));
            continue;
        }

        let item = candidates.into_iter().next().expect("candidate exists");
        let stocks = summary_stocks_for_item(client, &item, &req).await?;
        let matching_stocks = matching_stocks_for_item(&stocks, &item, &req);
        let mut available_quantity = matching_stocks
            .iter()
            .filter_map(|stock| stock.remaining_quantity)
            .filter(|quantity| *quantity > 0.0)
            .sum::<f64>();
        if matching_stocks.is_empty() {
            available_quantity = 0.0;
        }
        let mut warnings = unit_warnings(&stocks, &item, &req);
        if req.unit.is_none() {
            warnings.push(
                "Requirement has no unit; quantities were summed without unit conversion."
                    .to_string(),
            );
        }
        let stock_rows = matching_stocks
            .iter()
            .map(|stock| {
                json!({
                    "id": stock.id,
                    "name": stock_display_name(stock),
                    "batch_label": stock.batch_label,
                    "remaining_quantity": stock.remaining_quantity,
                    "unit": stock.unit,
                    "item_usage_unit": stock.item_usage_unit,
                    "item_usage_quantity": stock.item_usage_quantity,
                    "location_id": stock.storage_location_id,
                    "location_path": stock.location_path,
                })
            })
            .collect::<Vec<_>>();

        let status = if available_quantity >= req.quantity {
            available_count += 1;
            "available"
        } else {
            insufficient_count += 1;
            "insufficient_stock"
        };
        rows.push(requirement_row(
            &req,
            status,
            Some(&item),
            stock_rows,
            available_quantity,
            warnings,
        ));
    }

    Ok(json!({
        "mode": "client_aggregate_query",
        "atomic": false,
        "reservation_created": false,
        "note": "This report uses current query results only. It is not a backend reservation or atomic stock lock. Re-check before execution and use checkout/checkout-item as the final inventory mutation.",
        "summary": {
            "total": rows.len(),
            "available": available_count,
            "insufficient_stock": insufficient_count,
            "missing_item": missing_count,
            "ambiguous_item": ambiguous_count
        },
        "requirements": rows
    }))
}

async fn resolve_candidate_items(
    client: &BiolabClient,
    req: &InventoryRequirement,
) -> anyhow::Result<Vec<InventoryItem>> {
    if let Some(item_id) = &req.item_id {
        return match client.get_item(item_id).await {
            Ok(item) => Ok(vec![item]),
            Err(_) => Ok(Vec::new()),
        };
    }

    let items = client
        .list_items(
            0,
            50,
            Some(&req.name),
            req.category.as_deref(),
            req.supplier.as_deref(),
            None,
        )
        .await?
        .items;
    let exact = items
        .iter()
        .filter(|item| item.name.eq_ignore_ascii_case(&req.name))
        .cloned()
        .collect::<Vec<_>>();
    if exact.is_empty() {
        Ok(items.into_iter().take(1).collect())
    } else {
        Ok(exact)
    }
}

async fn summary_stocks_for_item(
    client: &BiolabClient,
    item: &InventoryItem,
    req: &InventoryRequirement,
) -> anyhow::Result<Vec<Stock>> {
    let summary = client
        .inventory_summary(
            0,
            200,
            Some(&item.name),
            req.category.as_deref().or(item.category.as_deref()),
            None,
        )
        .await?;
    let Some(rows) = summary.as_array() else {
        return Ok(Vec::new());
    };

    let mut stocks = Vec::new();
    for row in rows {
        let same_item = row
            .get("item_id")
            .and_then(|value| value.as_str())
            .map(|id| id == item.id)
            .unwrap_or_else(|| {
                row.get("item_name")
                    .and_then(|value| value.as_str())
                    .map(|name| name.eq_ignore_ascii_case(&item.name))
                    .unwrap_or(false)
            });
        if !same_item {
            continue;
        }
        if let Some(batches) = row.get("batches").and_then(|value| value.as_array()) {
            for batch in batches {
                if let Ok(stock) = serde_json::from_value::<Stock>(batch.clone()) {
                    stocks.push(stock);
                }
            }
        }
    }
    Ok(stocks)
}

fn matching_stocks_for_item(
    stocks: &[Stock],
    item: &InventoryItem,
    req: &InventoryRequirement,
) -> Vec<Stock> {
    stocks
        .iter()
        .filter(|stock| {
            let same_item = stock
                .item_id
                .as_deref()
                .map(|id| id == item.id)
                .unwrap_or_else(|| stock_display_name(stock).eq_ignore_ascii_case(&item.name));
            let same_unit = match (req.unit.as_deref(), stock.unit.as_deref()) {
                (Some(required), Some(actual)) => required.eq_ignore_ascii_case(actual),
                _ => true,
            };
            same_item && same_unit
        })
        .cloned()
        .collect()
}

fn unit_warnings(
    stocks: &[Stock],
    item: &InventoryItem,
    req: &InventoryRequirement,
) -> Vec<String> {
    let Some(required_unit) = req.unit.as_deref() else {
        return Vec::new();
    };
    stocks
        .iter()
        .filter(|stock| {
            let same_item = stock
                .item_id
                .as_deref()
                .map(|id| id == item.id)
                .unwrap_or_else(|| stock_display_name(stock).eq_ignore_ascii_case(&item.name));
            same_item
                && stock
                    .unit
                    .as_deref()
                    .map(|actual| !required_unit.eq_ignore_ascii_case(actual))
                    .unwrap_or(false)
        })
        .map(|stock| {
            format!(
                "Stock {} has unit {}, requirement unit is {}; no conversion was applied.",
                stock.id,
                stock.unit.as_deref().unwrap_or(""),
                required_unit
            )
        })
        .collect()
}

fn requirement_row(
    req: &InventoryRequirement,
    status: &str,
    item: Option<&InventoryItem>,
    stocks_or_candidates: Vec<serde_json::Value>,
    available_quantity: f64,
    warnings: Vec<String>,
) -> serde_json::Value {
    json!({
        "key": req.key,
        "requested": {
            "item_id": req.item_id,
            "name": req.name,
            "quantity": req.quantity,
            "unit": req.unit,
            "category": req.category,
            "supplier": req.supplier,
        },
        "status": status,
        "item": item.map(|item| {
            json!({
                "id": item.id,
                "name": item.name,
                "category": item.category,
                "supplier": item.supplier,
                "unit": item.unit,
                "usage_unit": item.usage_unit,
                "usage_unit_conversion": item.usage_unit_conversion,
            })
        }),
        "available_quantity": available_quantity,
        "stocks_or_candidates": stocks_or_candidates,
        "warnings": warnings
    })
}

fn stock_display_name(stock: &Stock) -> String {
    stock
        .name
        .as_deref()
        .or(stock.primer_name.as_deref())
        .unwrap_or("")
        .to_string()
}

fn read_inventory_requirements(path: &str) -> anyhow::Result<Vec<InventoryRequirement>> {
    let value = read_json_value(path)?;
    let array = if let Some(array) = value.as_array() {
        array.clone()
    } else if let Some(array) = value.get("requirements").and_then(|value| value.as_array()) {
        array.clone()
    } else if let Some(array) = value
        .get("inventory_requirements")
        .and_then(|value| value.as_array())
    {
        array.clone()
    } else if let Some(array) = value.get("materials").and_then(|value| value.as_array()) {
        array.clone()
    } else {
        anyhow::bail!(
            "requirements file must be an array or contain requirements/inventory_requirements/materials"
        );
    };

    array
        .iter()
        .map(parse_inventory_requirement)
        .collect::<anyhow::Result<Vec<_>>>()
}

fn parse_inventory_requirement(value: &serde_json::Value) -> anyhow::Result<InventoryRequirement> {
    let name = first_string(value, &["name", "item_name", "material", "reagent_name"]).ok_or_else(
        || anyhow::anyhow!("inventory requirement is missing name/item_name/material"),
    )?;
    let quantity = first_f64(value, &["quantity", "amount"])
        .ok_or_else(|| anyhow::anyhow!("inventory requirement `{name}` is missing quantity"))?;
    validate_positive_quantity(quantity)?;
    Ok(InventoryRequirement {
        key: first_string(value, &["requirement_key", "key", "id"]),
        item_id: first_string(value, &["item_id"]),
        name,
        quantity,
        unit: first_string(value, &["unit"]),
        category: first_string(value, &["category"]),
        supplier: first_string(value, &["supplier"]),
    })
}

fn first_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|v| v.as_str()))
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

fn first_f64(value: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| {
        let value = value.get(*key)?;
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
    })
}

fn read_json_value(path: &str) -> anyhow::Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn read_json_arg_or_file(input: &str) -> anyhow::Result<serde_json::Value> {
    if std::path::Path::new(input).exists() {
        read_json_value(input)
    } else {
        Ok(serde_json::from_str(input)?)
    }
}

fn validate_positive_quantity(quantity: f64) -> anyhow::Result<()> {
    if !quantity.is_finite() || quantity <= 0.0 {
        anyhow::bail!("quantity must be a positive finite number");
    }
    Ok(())
}

fn validate_adjustment_quantity(quantity: f64) -> anyhow::Result<()> {
    if !quantity.is_finite() || quantity == 0.0 {
        anyhow::bail!("adjust quantity must be a non-zero finite number");
    }
    Ok(())
}

fn validate_adjustment_type(adjustment_type: &str) -> anyhow::Result<()> {
    match adjustment_type {
        "correction" | "loss" | "damage" | "expire" => Ok(()),
        _ => anyhow::bail!("type must be one of correction/loss/damage/expire"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_positive_finite_quantity() {
        validate_positive_quantity(0.1).expect("positive quantity should be valid");
    }

    #[test]
    fn rejects_zero_negative_and_non_finite_positive_quantities() {
        assert!(validate_positive_quantity(0.0).is_err());
        assert!(validate_positive_quantity(-1.0).is_err());
        assert!(validate_positive_quantity(f64::INFINITY).is_err());
        assert!(validate_positive_quantity(f64::NAN).is_err());
    }

    #[test]
    fn accepts_positive_and_negative_adjustments() {
        validate_adjustment_quantity(1.0).expect("positive adjustment should be valid");
        validate_adjustment_quantity(-1.0).expect("negative adjustment should be valid");
    }

    #[test]
    fn rejects_zero_and_non_finite_adjustments() {
        assert!(validate_adjustment_quantity(0.0).is_err());
        assert!(validate_adjustment_quantity(f64::INFINITY).is_err());
        assert!(validate_adjustment_quantity(f64::NAN).is_err());
    }

    #[test]
    fn validates_adjustment_types() {
        validate_adjustment_type("correction").expect("correction is valid");
        validate_adjustment_type("loss").expect("loss is valid");
        validate_adjustment_type("damage").expect("damage is valid");
        validate_adjustment_type("expire").expect("expire is valid");
        assert!(validate_adjustment_type("other").is_err());
    }

    #[test]
    fn parses_requirement_from_common_keys() {
        let value = json!({
            "requirement_key": "pcr.dntp",
            "item_name": "dNTP Mix",
            "amount": "10",
            "unit": "uL"
        });
        let req = parse_inventory_requirement(&value).expect("requirement should parse");
        assert_eq!(req.key.as_deref(), Some("pcr.dntp"));
        assert_eq!(req.name, "dNTP Mix");
        assert_eq!(req.quantity, 10.0);
        assert_eq!(req.unit.as_deref(), Some("uL"));
    }
}
