use std::sync::Arc;

use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use serde::Serialize;

use crate::client::ScientexClient;
use crate::config::Config;
use crate::errors::ScientexError;
use crate::output::{print_result, unique_output_path, OutputFormat};
use crate::types::{StaffUserInfo, TaskResult, TaskTypeDocument};

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
    /// Platform-wide task administration.
    Tasks {
        #[command(subcommand)]
        command: AdminTasksCommand,
    },
    /// Query users eligible for task assignment.
    Users {
        #[command(subcommand)]
        command: AdminUsersCommand,
    },
    /// Inspect submitted client error reports.
    ErrorReports {
        #[command(subcommand)]
        command: AdminErrorReportsCommand,
    },
}

#[derive(Subcommand)]
pub enum AdminUsersCommand {
    /// List users eligible for task assignments.
    Staff {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
}

#[derive(Subcommand)]
pub enum AdminErrorReportsCommand {
    /// List error reports.
    List {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        category: Option<ErrorCategoryFilterArg>,
    },
    /// Show one error report.
    Get { id: String },
}

#[derive(Subcommand)]
pub enum AdminTasksCommand {
    /// List tasks across labs.
    List {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    /// Show one task across labs.
    Get { id: String },
    /// Show full workflow detail.
    Workflow { id: String },
    /// Update a task from a JSON file.
    Update { id: String, file: String },
    /// Cancel a task.
    Cancel { id: String },
    /// Manage task parts.
    Parts {
        #[command(subcommand)]
        command: AdminTaskPartsCommand,
    },
    /// Manage task assignments.
    Assignments {
        #[command(subcommand)]
        command: AdminTaskAssignmentsCommand,
    },
    /// Manage task documents.
    Documents {
        #[command(subcommand)]
        command: AdminTaskDocumentsCommand,
    },
    /// List task results.
    Results { id: String },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ErrorCategoryFilterArg {
    #[value(name = "ui-display")]
    UiDisplay,
    Functional,
    Data,
    Performance,
    Permission,
    Other,
}

impl ErrorCategoryFilterArg {
    fn as_str(&self) -> &'static str {
        match self {
            Self::UiDisplay => "ui_display",
            Self::Functional => "functional",
            Self::Data => "data",
            Self::Performance => "performance",
            Self::Permission => "permission",
            Self::Other => "other",
        }
    }
}

#[derive(Subcommand)]
pub enum AdminTaskPartsCommand {
    /// Show one task part.
    Get { task_id: String, part_id: String },
    /// Add a task part.
    Add {
        task_id: String,
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, default_value_t = 0)]
        sort_order: i64,
    },
    /// Update a task part from a JSON file.
    Update {
        task_id: String,
        part_id: String,
        file: String,
    },
    /// Delete a task part.
    Delete { task_id: String, part_id: String },
}

#[derive(Subcommand)]
pub enum AdminTaskAssignmentsCommand {
    /// Assign a user to a task part.
    Add {
        task_id: String,
        part_id: String,
        assignee_id: String,
        #[arg(long, default_value = "assignee")]
        role: String,
    },
    /// Remove an assignment.
    Remove {
        task_id: String,
        assignment_id: String,
    },
}

#[derive(Subcommand)]
pub enum AdminTaskDocumentsCommand {
    /// List documents for a task.
    List { task_id: String },
    /// Upload a task document.
    Upload {
        task_id: String,
        file: String,
        #[arg(long)]
        document_type: String,
        #[arg(long, default_value = "lab_and_staff")]
        visibility: String,
        #[arg(long)]
        part_id: Option<String>,
    },
    /// Download a task document.
    Download {
        document_id: String,
        output: Option<String>,
    },
    /// Delete a task document.
    Delete {
        task_id: String,
        document_id: String,
    },
}

#[derive(Subcommand)]
pub enum AdminTaskTypesCommand {
    /// List and search the global task type catalog.
    List {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        search: Option<String>,
        /// Administrator filters encoded as a JSON array.
        #[arg(long)]
        filters: Option<String>,
    },
    /// Show one global task type definition.
    Get { id: String },
    /// Update a global task type definition from a JSON file.
    Update { id: String, file: String },
    /// Create a task type from a JSON file.
    Create {
        file: String,
        /// SOP document file to attach after creation.
        #[arg(long, value_name = "FILE")]
        sop: Option<String>,
        /// Work order document file to attach after creation.
        #[arg(long, value_name = "FILE")]
        work_order: Option<String>,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Delete a task type by id.
    Delete {
        id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Manage staff bindings for a task type.
    Staff {
        #[command(subcommand)]
        command: AdminTaskTypeStaffCommand,
    },
    /// List documents for a task type.
    ListDocs {
        type_id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Upload a document (SOP, work_order, or attachment) to a task type.
    UploadDoc {
        type_id: String,
        file: String,
        /// Document type: sop, work_order, attachment
        #[arg(short = 'T', long, default_value = "sop")]
        doc_type: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Delete a document from a task type.
    DeleteDoc {
        type_id: String,
        doc_id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// List document feedback submitted by staff for a task type (task_manager only).
    Feedback {
        type_id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AdminTaskTypeStaffCommand {
    /// List staff bound to a task type.
    List {
        type_id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Bind one staff user to a task type.
    Add {
        type_id: String,
        user_id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Remove one staff user from a task type.
    Remove {
        type_id: String,
        user_id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
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

const VALID_DOCUMENT_TYPES: &[&str] = &["sop", "work_order", "attachment"];
const VALID_TASK_DOCUMENT_TYPES: &[&str] =
    &["sop", "work_order", "attachment", "result_attachment"];
const VALID_DOCUMENT_VISIBILITIES: &[&str] = &["lab_and_staff", "staff_only", "lab_only"];
const VALID_ASSIGNMENT_ROLES: &[&str] = &["assignee", "reviewer", "helper"];

pub async fn run(
    args: &AdminArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;

    match &args.command {
        AdminCommand::TaskTypes { command } => match command {
            AdminTaskTypesCommand::List {
                skip,
                limit,
                search,
                filters,
            } => {
                let task_types = client
                    .list_admin_task_types(*skip, *limit, search.as_deref(), filters.as_deref())
                    .await
                    .map_err(admin_operation_error)?;
                print_result(&task_types, format);
            }
            AdminTaskTypesCommand::Get { id } => {
                let task_type = client
                    .get_admin_task_type(id)
                    .await
                    .map_err(admin_operation_error)?;
                print_result(&task_type, format);
            }
            AdminTaskTypesCommand::Update { id, file } => {
                let data = read_json_file(file)?;
                let task_type = client
                    .update_admin_task_type(id, &data)
                    .await
                    .map_err(admin_operation_error)?;
                print_result(&task_type, format);
            }
            AdminTaskTypesCommand::Create {
                file,
                sop,
                work_order,
                lab_id,
            } => {
                let mut data = read_json_file(file)?;
                validate_task_type_create_payload(&mut data)?;
                let task_type = client
                    .create_admin_task_type(&data, lab_id.as_deref())
                    .await
                    .map_err(admin_operation_error)?;
                let mut uploaded_docs: Vec<TaskTypeDocument> = Vec::new();
                if let Some(sop_path) = sop {
                    let doc = client
                        .upload_admin_task_type_document(
                            &task_type.id,
                            sop_path,
                            "sop",
                            lab_id.as_deref(),
                        )
                        .await
                        .map_err(admin_operation_error)?;
                    uploaded_docs.push(doc);
                }
                if let Some(wo_path) = work_order {
                    let doc = client
                        .upload_admin_task_type_document(
                            &task_type.id,
                            wo_path,
                            "work_order",
                            lab_id.as_deref(),
                        )
                        .await
                        .map_err(admin_operation_error)?;
                    uploaded_docs.push(doc);
                }
                match format {
                    OutputFormat::Json => {
                        let combined = serde_json::json!({
                            "task_type": task_type,
                            "uploaded_documents": uploaded_docs,
                        });
                        print_result(&combined, format);
                    }
                    OutputFormat::Text => {
                        println!("Created task type: {}", task_type.id);
                        for doc in &uploaded_docs {
                            let sync_badge = feishu_sync_badge(doc.feishu_sync_status.as_deref());
                            println!(
                                "  Uploaded {}: {}  feishu={}",
                                doc.document_type, doc.filename, sync_badge,
                            );
                            if let Some(url) = &doc.feishu_doc_url {
                                println!("    feishu_url: {url}");
                            }
                        }
                    }
                }
            }
            AdminTaskTypesCommand::Delete { id, lab_id } => {
                client
                    .delete_admin_task_type(id, lab_id.as_deref())
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
            AdminTaskTypesCommand::ListDocs { type_id, lab_id } => {
                let docs = client
                    .list_admin_task_type_documents(type_id, lab_id.as_deref())
                    .await
                    .map_err(admin_operation_error)?;
                match format {
                    OutputFormat::Json => print_result(&docs, format),
                    OutputFormat::Text => print_type_documents_text(&docs),
                }
            }
            AdminTaskTypesCommand::UploadDoc {
                type_id,
                file,
                doc_type,
                lab_id,
            } => {
                validate_document_type(doc_type)?;
                let doc = client
                    .upload_admin_task_type_document(type_id, file, doc_type, lab_id.as_deref())
                    .await
                    .map_err(admin_operation_error)?;
                match format {
                    OutputFormat::Json => print_result(&doc, format),
                    OutputFormat::Text => {
                        let sync_badge = feishu_sync_badge(doc.feishu_sync_status.as_deref());
                        println!(
                            "Uploaded document: {}  type={}  feishu={}",
                            doc.filename, doc.document_type, sync_badge,
                        );
                        if let Some(url) = &doc.feishu_doc_url {
                            println!("  feishu_url: {url}");
                        }
                    }
                }
            }
            AdminTaskTypesCommand::DeleteDoc {
                type_id,
                doc_id,
                lab_id,
            } => {
                client
                    .delete_admin_task_type_document(type_id, doc_id, lab_id.as_deref())
                    .await
                    .map_err(admin_operation_error)?;
                match format {
                    OutputFormat::Json => print_result(
                        &serde_json::json!({"type_id": type_id, "doc_id": doc_id, "deleted": true}),
                        format,
                    ),
                    OutputFormat::Text => {
                        println!("Deleted document {doc_id} from task type {type_id}")
                    }
                }
            }
            AdminTaskTypesCommand::Feedback { type_id, lab_id: _ } => {
                let results = client
                    .list_task_type_feedback(type_id)
                    .await
                    .map_err(admin_operation_error)?;
                match format {
                    OutputFormat::Json => print_result(&results, format),
                    OutputFormat::Text => print_feedback_text(&results),
                }
            }
        },
        AdminCommand::Tasks { command } => {
            run_admin_tasks(&client, command, format).await?;
        }
        AdminCommand::Users { command } => match command {
            AdminUsersCommand::Staff { skip, limit } => {
                let users = client
                    .list_staff_users(*skip, *limit)
                    .await
                    .map_err(admin_operation_error)?;
                print_result(&users, format);
            }
        },
        AdminCommand::ErrorReports { command } => match command {
            AdminErrorReportsCommand::List {
                skip,
                limit,
                category,
            } => {
                let reports = client
                    .list_error_reports(
                        *skip,
                        *limit,
                        category.as_ref().map(|value| value.as_str()),
                    )
                    .await
                    .map_err(admin_operation_error)?;
                print_result(&reports, format);
            }
            AdminErrorReportsCommand::Get { id } => {
                let report = client
                    .get_error_report(id)
                    .await
                    .map_err(admin_operation_error)?;
                print_result(&report, format);
            }
        },
    }

    Ok(())
}

async fn run_admin_tasks(
    client: &ScientexClient,
    command: &AdminTasksCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        AdminTasksCommand::List { skip, limit } => {
            let tasks = client
                .list_tasks(*skip, *limit)
                .await
                .map_err(admin_operation_error)?;
            print_result(&tasks, format);
        }
        AdminTasksCommand::Get { id } => {
            let task = client.get_task(id).await.map_err(admin_operation_error)?;
            print_result(&task, format);
        }
        AdminTasksCommand::Workflow { id } => {
            let workflow = client
                .get_task_workflow(id)
                .await
                .map_err(admin_operation_error)?;
            print_result(&workflow, format);
        }
        AdminTasksCommand::Update { id, file } => {
            let data = read_json_file(file)?;
            let task = client
                .update_task(id, &data)
                .await
                .map_err(admin_operation_error)?;
            print_result(&task, format);
        }
        AdminTasksCommand::Cancel { id } => {
            let result = client
                .cancel_task(id)
                .await
                .map_err(admin_operation_error)?;
            print_result(&result, format);
        }
        AdminTasksCommand::Parts { command } => {
            run_admin_task_parts(client, command, format).await?;
        }
        AdminTasksCommand::Assignments { command } => {
            run_admin_task_assignments(client, command, format).await?;
        }
        AdminTasksCommand::Documents { command } => {
            run_admin_task_documents(client, command, format).await?;
        }
        AdminTasksCommand::Results { id } => {
            let results = client
                .list_task_results(id)
                .await
                .map_err(admin_operation_error)?;
            print_result(&results, format);
        }
    }
    Ok(())
}

async fn run_admin_task_parts(
    client: &ScientexClient,
    command: &AdminTaskPartsCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        AdminTaskPartsCommand::Get { task_id, part_id } => {
            let part = client
                .get_task_part(task_id, part_id)
                .await
                .map_err(admin_operation_error)?;
            print_result(&part, format);
        }
        AdminTaskPartsCommand::Add {
            task_id,
            name,
            description,
            sort_order,
        } => {
            let part = client
                .add_task_part(task_id, name, description.as_deref(), *sort_order)
                .await
                .map_err(admin_operation_error)?;
            print_result(&part, format);
        }
        AdminTaskPartsCommand::Update {
            task_id,
            part_id,
            file,
        } => {
            let data = read_json_file(file)?;
            let part = client
                .update_task_part(task_id, part_id, &data)
                .await
                .map_err(admin_operation_error)?;
            print_result(&part, format);
        }
        AdminTaskPartsCommand::Delete { task_id, part_id } => {
            client
                .delete_task_part(task_id, part_id)
                .await
                .map_err(admin_operation_error)?;
            print_result(
                &serde_json::json!({"task_id": task_id, "part_id": part_id, "deleted": true}),
                format,
            );
        }
    }
    Ok(())
}

async fn run_admin_task_assignments(
    client: &ScientexClient,
    command: &AdminTaskAssignmentsCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        AdminTaskAssignmentsCommand::Add {
            task_id,
            part_id,
            assignee_id,
            role,
        } => {
            validate_assignment_role(role)?;
            let assignment = client
                .create_task_assignment(task_id, part_id, assignee_id, role)
                .await
                .map_err(admin_operation_error)?;
            print_result(&assignment, format);
        }
        AdminTaskAssignmentsCommand::Remove {
            task_id,
            assignment_id,
        } => {
            client
                .delete_task_assignment(task_id, assignment_id)
                .await
                .map_err(admin_operation_error)?;
            print_result(
                &serde_json::json!({
                    "task_id": task_id,
                    "assignment_id": assignment_id,
                    "removed": true
                }),
                format,
            );
        }
    }
    Ok(())
}

async fn run_admin_task_documents(
    client: &ScientexClient,
    command: &AdminTaskDocumentsCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        AdminTaskDocumentsCommand::List { task_id } => {
            let documents = client
                .list_task_documents(task_id)
                .await
                .map_err(admin_operation_error)?;
            print_result(&documents, format);
        }
        AdminTaskDocumentsCommand::Upload {
            task_id,
            file,
            document_type,
            visibility,
            part_id,
        } => {
            validate_task_document_type(document_type)?;
            validate_document_visibility(visibility)?;
            let document = client
                .upload_task_document(
                    task_id,
                    file,
                    document_type,
                    Some(visibility),
                    part_id.as_deref(),
                )
                .await
                .map_err(admin_operation_error)?;
            print_result(&document, format);
        }
        AdminTaskDocumentsCommand::Download {
            document_id,
            output,
        } => {
            let bytes = client
                .download_task_document(document_id)
                .await
                .map_err(admin_operation_error)?;
            let output = output
                .clone()
                .unwrap_or_else(|| format!("task_document_{document_id}"));
            let output_path = unique_output_path(output);
            std::fs::write(&output_path, bytes)?;
            println!("Downloaded to {}", output_path.display());
        }
        AdminTaskDocumentsCommand::Delete {
            task_id,
            document_id,
        } => {
            client
                .delete_task_document(task_id, document_id)
                .await
                .map_err(admin_operation_error)?;
            print_result(
                &serde_json::json!({
                    "task_id": task_id,
                    "document_id": document_id,
                    "deleted": true
                }),
                format,
            );
        }
    }
    Ok(())
}

async fn run_task_type_staff(
    client: &ScientexClient,
    command: &AdminTaskTypeStaffCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        AdminTaskTypeStaffCommand::List { type_id, lab_id } => {
            let staff = client
                .list_admin_task_type_staff(type_id, lab_id.as_deref())
                .await
                .map_err(admin_operation_error)?;
            print_staff_list(&staff, format);
        }
        AdminTaskTypeStaffCommand::Add {
            type_id,
            user_id,
            lab_id,
        } => {
            client
                .assign_admin_task_type_staff(type_id, user_id, lab_id.as_deref())
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
        AdminTaskTypeStaffCommand::Remove {
            type_id,
            user_id,
            lab_id,
        } => {
            client
                .remove_admin_task_type_staff(type_id, user_id, lab_id.as_deref())
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

fn print_type_documents_text(docs: &[TaskTypeDocument]) {
    if docs.is_empty() {
        println!("No documents for this task type");
        return;
    }
    println!("Task type documents:");
    for doc in docs {
        let sync_badge = feishu_sync_badge(doc.feishu_sync_status.as_deref());
        println!(
            "  {}  {}  {}  {}  {}  feishu={}",
            doc.id, doc.document_type, doc.filename, doc.content_type, doc.file_size, sync_badge,
        );
        if let Some(url) = &doc.feishu_doc_url {
            println!("    feishu_url: {url}");
        }
        if let Some(url) = &doc.scientex_link_url {
            println!("    scientex_link: {url}");
        }
    }
}

fn feishu_sync_badge(status: Option<&str>) -> String {
    match status {
        Some("synced") => "✓ synced".green().to_string(),
        Some("pending") => "◌ pending".yellow().to_string(),
        Some("failed") => "✗ failed".red().to_string(),
        Some("skipped") => "⊘ skipped".dimmed().to_string(),
        Some(other) => format!("? {other}"),
        None => "—".dimmed().to_string(),
    }
}

fn print_feedback_text(results: &[TaskResult]) {
    if results.is_empty() {
        println!("No document feedback for this task type");
        return;
    }
    println!(
        "Document feedback ({} results with feedback):",
        results.len()
    );
    for result in results {
        let feedback = match &result.document_feedback {
            Some(fb) => fb,
            None => continue,
        };
        println!(
            "  result={}  task={}  part={}  submitted_by={}",
            result.id, result.task_id, result.part_id, result.submitted_by_id,
        );
        if let Some(items) = feedback.get("items").and_then(|v| v.as_array()) {
            for (i, item) in items.iter().enumerate() {
                let target = item
                    .get("target_document_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let text = item
                    .get("feedback_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let modified_url = item
                    .get("modified_feishu_doc_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                println!("    #{}  target={}  feedback={}", i + 1, target, text,);
                if !modified_url.is_empty() {
                    println!("      modified_doc: {modified_url}");
                }
            }
        } else {
            // Fallback: show raw feedback JSON
            println!("    feedback: {feedback}");
        }
    }
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

fn validate_task_type_create_payload(data: &mut serde_json::Value) -> anyhow::Result<()> {
    let obj = data
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Task type payload must be a JSON object"))?;

    required_non_empty_string(obj, "key")?;
    required_non_empty_string(obj, "display_name")?;

    if let Some(category) = obj.get("category").filter(|value| !value.is_null()) {
        let category = category
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("`category` must be a string"))?;
        // Backend TaskTypeCategory expects uppercase values; accept either case
        // from the payload file and normalize before sending.
        let normalized = category.to_ascii_uppercase();
        if !matches!(normalized.as_str(), "STAFF" | "COMPUTE") {
            anyhow::bail!("`category` must be either `staff` or `compute`");
        }
        obj.insert(
            "category".to_string(),
            serde_json::Value::String(normalized),
        );
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

fn validate_document_type(doc_type: &str) -> anyhow::Result<()> {
    if VALID_DOCUMENT_TYPES.contains(&doc_type) {
        Ok(())
    } else {
        anyhow::bail!(
            "Invalid document type '{}'. Must be one of: {}",
            doc_type,
            VALID_DOCUMENT_TYPES.join(", ")
        )
    }
}

fn validate_task_document_type(doc_type: &str) -> anyhow::Result<()> {
    if VALID_TASK_DOCUMENT_TYPES.contains(&doc_type) {
        Ok(())
    } else {
        anyhow::bail!(
            "Invalid task document type '{}'. Must be one of: {}",
            doc_type,
            VALID_TASK_DOCUMENT_TYPES.join(", ")
        )
    }
}

fn validate_document_visibility(visibility: &str) -> anyhow::Result<()> {
    if VALID_DOCUMENT_VISIBILITIES.contains(&visibility) {
        Ok(())
    } else {
        anyhow::bail!(
            "Invalid document visibility '{}'. Must be one of: {}",
            visibility,
            VALID_DOCUMENT_VISIBILITIES.join(", ")
        )
    }
}

fn validate_assignment_role(role: &str) -> anyhow::Result<()> {
    if VALID_ASSIGNMENT_ROLES.contains(&role) {
        Ok(())
    } else {
        anyhow::bail!(
            "Invalid assignment role '{}'. Must be one of: {}",
            role,
            VALID_ASSIGNMENT_ROLES.join(", ")
        )
    }
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

fn admin_operation_error(error: ScientexError) -> anyhow::Error {
    if is_permission_error(&error) {
        anyhow::anyhow!("当前账号权限不足，无法执行该 admin 操作: {error}")
    } else {
        error.into()
    }
}

fn is_permission_error(error: &ScientexError) -> bool {
    match error {
        ScientexError::HttpError { status, detail, .. } => {
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
        let cli = TestCli::try_parse_from(std::iter::once("scitex").chain(args.iter().copied()))
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
                command: AdminTaskTypesCommand::Create { file, .. },
            } => assert_eq!(file, "task-type.json"),
            _ => panic!("expected admin task-types create command"),
        }
    }

    #[test]
    fn parses_task_type_list_get_and_update() {
        let list = parse_admin(&["admin", "task-types", "list", "--search", "sample qc"]);
        assert!(matches!(
            list.command,
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::List { .. }
            }
        ));

        let get = parse_admin(&["admin", "task-types", "get", "type-1"]);
        assert!(matches!(
            get.command,
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::Get { .. }
            }
        ));

        let update = parse_admin(&["admin", "task-types", "update", "type-1", "update.json"]);
        assert!(matches!(
            update.command,
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::Update { .. }
            }
        ));
    }

    #[test]
    fn parses_admin_task_management_commands() {
        let cancel = parse_admin(&["admin", "tasks", "cancel", "task-1"]);
        assert!(matches!(
            cancel.command,
            AdminCommand::Tasks {
                command: AdminTasksCommand::Cancel { .. }
            }
        ));

        let part = parse_admin(&[
            "admin",
            "tasks",
            "parts",
            "add",
            "task-1",
            "QC",
            "--sort-order",
            "20",
        ]);
        assert!(matches!(
            part.command,
            AdminCommand::Tasks {
                command: AdminTasksCommand::Parts { .. }
            }
        ));

        let assignment = parse_admin(&[
            "admin",
            "tasks",
            "assignments",
            "add",
            "task-1",
            "part-1",
            "user-1",
            "--role",
            "reviewer",
        ]);
        assert!(matches!(
            assignment.command,
            AdminCommand::Tasks {
                command: AdminTasksCommand::Assignments { .. }
            }
        ));
    }

    #[test]
    fn parses_admin_staff_and_error_report_queries() {
        let staff = parse_admin(&["admin", "users", "staff", "--limit", "50"]);
        assert!(matches!(
            staff.command,
            AdminCommand::Users {
                command: AdminUsersCommand::Staff { limit: 50, .. }
            }
        ));

        let reports = parse_admin(&["admin", "error-reports", "list", "--category", "permission"]);
        assert!(matches!(
            reports.command,
            AdminCommand::ErrorReports {
                command: AdminErrorReportsCommand::List { .. }
            }
        ));
    }

    #[test]
    fn error_report_category_filters_use_api_enum_values() {
        let cases = [
            (ErrorCategoryFilterArg::UiDisplay, "ui_display"),
            (ErrorCategoryFilterArg::Functional, "functional"),
            (ErrorCategoryFilterArg::Data, "data"),
            (ErrorCategoryFilterArg::Performance, "performance"),
            (ErrorCategoryFilterArg::Permission, "permission"),
            (ErrorCategoryFilterArg::Other, "other"),
        ];

        for (category, expected) in cases {
            assert_eq!(category.as_str(), expected);
        }
    }

    #[test]
    fn parses_task_type_create_with_lab_id() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "create",
            "task-type.json",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::Create {
                        file,
                        sop,
                        work_order,
                        lab_id,
                    },
            } => {
                assert_eq!(file, "task-type.json");
                assert!(sop.is_none());
                assert!(work_order.is_none());
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected create command"),
        }
    }

    #[test]
    fn parses_task_type_create_with_sop_and_work_order() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "create",
            "task-type.json",
            "--sop",
            "sop.md",
            "--work-order",
            "wo.pdf",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::Create {
                        file,
                        sop,
                        work_order,
                        lab_id,
                    },
            } => {
                assert_eq!(file, "task-type.json");
                assert_eq!(sop.as_deref(), Some("sop.md"));
                assert_eq!(work_order.as_deref(), Some("wo.pdf"));
                assert!(lab_id.is_none());
            }
            _ => panic!("expected create command"),
        }
    }

    #[test]
    fn parses_task_type_delete() {
        let args = parse_admin(&["admin", "task-types", "delete", "type-1"]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::Delete { id, .. },
            } => assert_eq!(id, "type-1"),
            _ => panic!("expected admin task-types delete command"),
        }
    }

    #[test]
    fn parses_task_type_delete_with_lab_id() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "delete",
            "type-1",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::Delete { id, lab_id },
            } => {
                assert_eq!(id, "type-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
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
                        command: AdminTaskTypeStaffCommand::List { type_id, .. },
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
                        command:
                            AdminTaskTypeStaffCommand::Add {
                                type_id, user_id, ..
                            },
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
                        command:
                            AdminTaskTypeStaffCommand::Remove {
                                type_id, user_id, ..
                            },
                    },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(user_id, "user-1");
            }
            _ => panic!("expected admin task-types staff remove command"),
        }
    }

    #[test]
    fn parses_task_type_staff_list_with_lab_id() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "staff",
            "list",
            "type-1",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::Staff {
                        command: AdminTaskTypeStaffCommand::List { type_id, lab_id },
                    },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected staff list command"),
        }
    }

    #[test]
    fn parses_task_type_list_docs() {
        let args = parse_admin(&["admin", "task-types", "list-docs", "type-1"]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::ListDocs { type_id, .. },
            } => assert_eq!(type_id, "type-1"),
            _ => panic!("expected admin task-types list-docs command"),
        }
    }

    #[test]
    fn parses_task_type_list_docs_with_lab_id() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "list-docs",
            "type-1",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::ListDocs { type_id, lab_id },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected list-docs command"),
        }
    }

    #[test]
    fn parses_task_type_upload_doc() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "upload-doc",
            "type-1",
            "file.md",
            "--doc-type",
            "sop",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::UploadDoc {
                        type_id,
                        file,
                        doc_type,
                        lab_id,
                    },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(file, "file.md");
                assert_eq!(doc_type, "sop");
                assert!(lab_id.is_none());
            }
            _ => panic!("expected upload-doc command"),
        }
    }

    #[test]
    fn parses_task_type_upload_doc_short_flag() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "upload-doc",
            "type-1",
            "file.md",
            "-T",
            "work_order",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::UploadDoc { doc_type, .. },
            } => assert_eq!(doc_type, "work_order"),
            _ => panic!("expected upload-doc command"),
        }
    }

    #[test]
    fn parses_task_type_delete_doc() {
        let args = parse_admin(&["admin", "task-types", "delete-doc", "type-1", "doc-1"]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::DeleteDoc {
                        type_id,
                        doc_id,
                        lab_id,
                    },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(doc_id, "doc-1");
                assert!(lab_id.is_none());
            }
            _ => panic!("expected delete-doc command"),
        }
    }

    #[test]
    fn parses_task_type_delete_doc_with_lab_id() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "delete-doc",
            "type-1",
            "doc-1",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command:
                    AdminTaskTypesCommand::DeleteDoc {
                        type_id,
                        doc_id,
                        lab_id,
                    },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(doc_id, "doc-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected delete-doc command"),
        }
    }

    #[test]
    fn parses_task_type_feedback() {
        let args = parse_admin(&["admin", "task-types", "feedback", "type-1"]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::Feedback { type_id, lab_id },
            } => {
                assert_eq!(type_id, "type-1");
                assert!(lab_id.is_none());
            }
            _ => panic!("expected feedback command"),
        }
    }

    #[test]
    fn parses_task_type_feedback_with_lab_id() {
        let args = parse_admin(&[
            "admin",
            "task-types",
            "feedback",
            "type-1",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            AdminCommand::TaskTypes {
                command: AdminTaskTypesCommand::Feedback { type_id, lab_id },
            } => {
                assert_eq!(type_id, "type-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected feedback command"),
        }
    }

    #[test]
    fn validates_minimal_task_type_payload() {
        let mut payload = json!({
            "key": "sample_qc",
            "display_name": "Sample QC"
        });
        validate_task_type_create_payload(&mut payload).expect("payload should validate");
    }

    #[test]
    fn validates_task_type_payload_with_file_field() {
        let mut payload = json!({
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
        validate_task_type_create_payload(&mut payload).expect("payload should validate");
        assert_eq!(payload["category"], "STAFF");
    }

    #[test]
    fn rejects_missing_key() {
        let mut payload = json!({ "display_name": "Sample QC" });
        let err = validate_task_type_create_payload(&mut payload)
            .expect_err("payload should be rejected");
        assert!(err.to_string().contains("`key`"));
    }

    #[test]
    fn rejects_invalid_category() {
        let mut payload = json!({
            "key": "sample_qc",
            "display_name": "Sample QC",
            "category": "experiment"
        });
        let err = validate_task_type_create_payload(&mut payload)
            .expect_err("payload should be rejected");
        assert!(err.to_string().contains("`category`"));
    }

    #[test]
    fn rejects_required_unknown_field() {
        let mut payload = json!({
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
        let err = validate_task_type_create_payload(&mut payload)
            .expect_err("payload should be rejected");
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn validate_document_type_accepts_valid() {
        assert!(validate_document_type("sop").is_ok());
        assert!(validate_document_type("work_order").is_ok());
        assert!(validate_document_type("attachment").is_ok());
    }

    #[test]
    fn validate_document_type_rejects_invalid() {
        let err = validate_document_type("report").expect_err("should be rejected");
        assert!(err.to_string().contains("report"));
    }
}
