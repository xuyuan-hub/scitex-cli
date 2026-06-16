use std::sync::Arc;

use anyhow::Context;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::client::BiolabClient;
use crate::config::Config;
use crate::errors::BiolabError;
use crate::output::{print_result, OutputFormat};
use crate::types::StaffUserInfo;

#[derive(Args)]
pub struct AdminArgs {
    #[command(subcommand)]
    pub command: AdminCommand,
}

#[derive(Subcommand)]
pub enum AdminCommand {
    /// Task type catalog management.
    TaskTypes {
        #[command(subcommand)]
        command: AdminTaskTypesCommand,
    },
}

#[derive(Subcommand)]
pub enum AdminTaskTypesCommand {
    /// Create a task type from a JSON file.
    Create { file: String },
    /// Delete a task type by id.
    Delete { id: String },
    /// Manage staff bindings for a task type.
    Staff {
        #[command(subcommand)]
        command: AdminTaskTypeStaffCommand,
    },
}

#[derive(Subcommand)]
pub enum AdminTaskTypeStaffCommand {
    /// List staff bound to a task type.
    List { type_id: String },
    /// Bind one staff user to a task type.
    Add { type_id: String, user_id: String },
    /// Remove one staff user from a task type.
    Remove { type_id: String, user_id: String },
}

#[derive(Debug, Serialize)]
struct DeletedTaskType<'a> {
    id: &'a str,
    deleted: bool,
}

#[derive(Debug, Serialize)]
struct StaffBindingChange<'a> {
    type_id: &'a str,
    user_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    assigned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    removed: Option<bool>,
}

pub async fn run(
    args: &AdminArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = BiolabClient::new(Arc::clone(config))?;

    match &args.command {
        AdminCommand::TaskTypes { command } => match command {
            AdminTaskTypesCommand::Create { file } => {
                let data = read_json_file(file)?;
                validate_task_type_create_payload(&data)?;
                let task_type = client
                    .create_admin_task_type(&data)
                    .await
                    .map_err(admin_operation_error)?;
                print_result(&task_type, format);
            }
            AdminTaskTypesCommand::Delete { id } => {
                client
                    .delete_admin_task_type(id)
                    .await
                    .map_err(admin_operation_error)?;
                match format {
                    OutputFormat::Json => {
                        print_result(&DeletedTaskType { id, deleted: true }, format)
                    }
                    OutputFormat::Text => println!("Deleted task type: {id}"),
                }
            }
            AdminTaskTypesCommand::Staff { command } => {
                run_task_type_staff(&client, command, format).await?;
            }
        },
    }

    Ok(())
}

async fn run_task_type_staff(
    client: &BiolabClient,
    command: &AdminTaskTypeStaffCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        AdminTaskTypeStaffCommand::List { type_id } => {
            let staff = client
                .list_admin_task_type_staff(type_id)
                .await
                .map_err(admin_operation_error)?;
            print_staff_list(&staff, format);
        }
        AdminTaskTypeStaffCommand::Add { type_id, user_id } => {
            client
                .assign_admin_task_type_staff(type_id, user_id)
                .await
                .map_err(admin_operation_error)?;
            match format {
                OutputFormat::Json => print_result(
                    &StaffBindingChange {
                        type_id,
                        user_id,
                        assigned: Some(true),
                        removed: None,
                    },
                    format,
                ),
                OutputFormat::Text => {
                    println!("Assigned staff to task type: type={type_id} user={user_id}")
                }
            }
        }
        AdminTaskTypeStaffCommand::Remove { type_id, user_id } => {
            client
                .remove_admin_task_type_staff(type_id, user_id)
                .await
                .map_err(admin_operation_error)?;
            match format {
                OutputFormat::Json => print_result(
                    &StaffBindingChange {
                        type_id,
                        user_id,
                        assigned: None,
                        removed: Some(true),
                    },
                    format,
                ),
                OutputFormat::Text => {
                    println!("Removed staff from task type: type={type_id} user={user_id}")
                }
            }
        }
    }

    Ok(())
}

fn print_staff_list(staff: &Vec<StaffUserInfo>, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_result(staff, format),
        OutputFormat::Text => {
            if staff.is_empty() {
                println!("No staff bound to this task type");
                return;
            }
            println!("Task type staff:");
            for item in staff {
                let assignment_id = item.assignment_id.as_deref().unwrap_or("-");
                let full_name = item.full_name.as_deref().unwrap_or("-");
                println!(
                    "{}  {}  {}  {}",
                    item.user_id, assignment_id, item.email, full_name
                );
            }
        }
    }
}

fn read_json_file(path: &str) -> anyhow::Result<serde_json::Value> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("Cannot read JSON file {path}"))?;
    serde_json::from_str(&content).with_context(|| format!("Cannot parse JSON file {path}"))
}

fn validate_task_type_create_payload(data: &serde_json::Value) -> anyhow::Result<()> {
    let obj = data
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Task type payload must be a JSON object"))?;

    required_non_empty_string(obj, "key")?;
    required_non_empty_string(obj, "display_name")?;

    if let Some(category) = obj.get("category").filter(|value| !value.is_null()) {
        let category = category
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("`category` must be a string"))?;
        if !matches!(category, "staff" | "compute") {
            anyhow::bail!("`category` must be either `staff` or `compute`");
        }
    }

    for schema_key in ["input_schema", "output_schema"] {
        if let Some(schema) = obj.get(schema_key).filter(|value| !value.is_null()) {
            validate_task_type_schema(schema_key, schema)?;
        }
    }

    if let Some(command_template) = obj.get("command_template").filter(|value| !value.is_null()) {
        let parts = command_template
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("`command_template` must be an array of strings"))?;
        for (index, part) in parts.iter().enumerate() {
            part.as_str().ok_or_else(|| {
                anyhow::anyhow!("`command_template` item #{} must be a string", index + 1)
            })?;
        }
    }

    if let Some(timeout) = obj.get("timeout_seconds").filter(|value| !value.is_null()) {
        let Some(timeout) = timeout.as_u64() else {
            anyhow::bail!("`timeout_seconds` must be a positive integer");
        };
        if timeout == 0 {
            anyhow::bail!("`timeout_seconds` must be a positive integer");
        }
    }

    Ok(())
}

fn validate_task_type_schema(name: &str, schema: &serde_json::Value) -> anyhow::Result<()> {
    let schema_obj = schema
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("`{name}` must be a JSON object"))?;

    if let Some(schema_type) = schema_obj.get("type").filter(|value| !value.is_null()) {
        let schema_type = schema_type
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("`{name}.type` must be a string"))?;
        if schema_type != "object" {
            anyhow::bail!("`{name}.type` must be `object`");
        }
    }

    let properties = schema_obj
        .get("properties")
        .filter(|value| !value.is_null())
        .map(|value| {
            value
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("`{name}.properties` must be a JSON object"))
        })
        .transpose()?;

    if let Some(properties) = properties {
        for (field, property) in properties {
            validate_schema_property(name, field, property)?;
        }
    }

    if let Some(required) = schema_obj.get("required").filter(|value| !value.is_null()) {
        let required = required
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("`{name}.required` must be an array of strings"))?;
        for (index, field) in required.iter().enumerate() {
            let field = field.as_str().ok_or_else(|| {
                anyhow::anyhow!("`{name}.required` item #{} must be a string", index + 1)
            })?;
            if let Some(properties) = properties {
                if !properties.contains_key(field) {
                    anyhow::bail!("`{name}.required` references unknown field `{field}`");
                }
            }
        }
    }

    Ok(())
}

fn validate_schema_property(
    schema_name: &str,
    field: &str,
    property: &serde_json::Value,
) -> anyhow::Result<()> {
    let property_obj = property
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("`{schema_name}.properties.{field}` must be an object"))?;
    let property_type = property_obj
        .get("type")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("`{schema_name}.properties.{field}.type` is required"))?;
    if !matches!(property_type, "string" | "integer" | "number" | "object") {
        anyhow::bail!(
            "`{schema_name}.properties.{field}.type` must be one of string, integer, number, object"
        );
    }

    if property_obj
        .get("format")
        .and_then(|value| value.as_str())
        .is_some_and(|format| format == "file")
        && property_type != "object"
    {
        anyhow::bail!("`{schema_name}.properties.{field}` with format=file must use type=object");
    }

    Ok(())
}

fn required_non_empty_string(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> anyhow::Result<()> {
    obj.get(field)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Task type payload requires a non-empty `{field}`"))?;
    Ok(())
}

fn admin_operation_error(error: BiolabError) -> anyhow::Error {
    if is_permission_error(&error) {
        anyhow::anyhow!("当前账号权限不足，无法执行该 admin 操作: {error}")
    } else {
        error.into()
    }
}

fn is_permission_error(error: &BiolabError) -> bool {
    match error {
        BiolabError::HttpError { status, detail, .. } => {
            matches!(status, 401 | 403)
                || detail.to_ascii_lowercase().contains("permission")
                || detail.to_ascii_lowercase().contains("forbidden")
                || detail.to_ascii_lowercase().contains("not authorized")
                || detail.to_ascii_lowercase().contains("platform_admin")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use serde_json::json;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommand,
    }

    #[derive(Subcommand)]
    enum TestCommand {
        Admin(AdminArgs),
    }

    fn parse_admin(args: &[&str]) -> AdminArgs {
        let cli = TestCli::try_parse_from(std::iter::once("biolab").chain(args.iter().copied()))
            .expect("admin command should parse");
        match cli.command {
            TestCommand::Admin(args) => args,
        }
    }

    #[test]
    fn parses_task_type_create() {
        let args = parse_admin(&["admin", "task-types", "create", "task-type.json"]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::Create { file },
            } => assert_eq!(file, "task-type.json"),
            _ => panic!("expected admin task-types create command"),
        }
    }

    #[test]
    fn parses_task_type_delete() {
        let args = parse_admin(&["admin", "task-types", "delete", "type-1"]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::Delete { id },
            } => assert_eq!(id, "type-1"),
            _ => panic!("expected admin task-types delete command"),
        }
    }

    #[test]
    fn parses_task_type_staff_list() {
        let args = parse_admin(&["admin", "task-types", "staff", "list", "type-1"]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::Staff {
                        command: AdminTaskTypeStaffCommand::List { type_id },
                    },
            } => assert_eq!(type_id, "type-1"),
            _ => panic!("expected admin task-types staff list command"),
        }
    }

    #[test]
    fn parses_task_type_staff_add() {
        let args = parse_admin(&["admin", "task-types", "staff", "add", "type-1", "user-1"]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::Staff {
                        command: AdminTaskTypeStaffCommand::Add { type_id, user_id },
                    },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(user_id, "user-1");
            }
            _ => panic!("expected admin task-types staff add command"),
        }
    }

    #[test]
    fn parses_task_type_staff_remove() {
        let args = parse_admin(&["admin", "task-types", "staff", "remove", "type-1", "user-1"]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::Staff {
                        command: AdminTaskTypeStaffCommand::Remove { type_id, user_id },
                    },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(user_id, "user-1");
            }
            _ => panic!("expected admin task-types staff remove command"),
        }
    }

    #[test]
    fn validates_minimal_task_type_payload() {
        let payload = json!({
            "key": "sample_qc",
            "display_name": "Sample QC"
        });
        validate_task_type_create_payload(&payload).expect("payload should validate");
    }

    #[test]
    fn validates_task_type_payload_with_file_field() {
        let payload = json!({
            "key": "plasmid_review",
            "display_name": "Plasmid Review",
            "category": "staff",
            "input_schema": {
                "type": "object",
                "properties": {
                    "plasmid_file": {
                        "type": "object",
                        "format": "file",
                        "title": "Plasmid File"
                    }
                },
                "required": ["plasmid_file"]
            }
        });
        validate_task_type_create_payload(&payload).expect("payload should validate");
    }

    #[test]
    fn rejects_missing_key() {
        let payload = json!({ "display_name": "Sample QC" });
        let err =
            validate_task_type_create_payload(&payload).expect_err("payload should be rejected");
        assert!(err.to_string().contains("`key`"));
    }

    #[test]
    fn rejects_invalid_category() {
        let payload = json!({
            "key": "sample_qc",
            "display_name": "Sample QC",
            "category": "experiment"
        });
        let err =
            validate_task_type_create_payload(&payload).expect_err("payload should be rejected");
        assert!(err.to_string().contains("`category`"));
    }

    #[test]
    fn rejects_required_unknown_field() {
        let payload = json!({
            "key": "sample_qc",
            "display_name": "Sample QC",
            "input_schema": {
                "type": "object",
                "properties": {
                    "sample": { "type": "string" }
                },
                "required": ["missing"]
            }
        });
        let err =
            validate_task_type_create_payload(&payload).expect_err("payload should be rejected");
        assert!(err.to_string().contains("unknown field"));
    }
}
