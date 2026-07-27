use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use serde::Serialize;

use crate::client::ScientexClient;
use crate::config::Config;
use crate::errors::ScientexError;
use crate::output::{
    print_paginated_items, print_pagination_metadata, print_result,
    unique_output_path as unique_download_path, OutputFormat,
};
use crate::types::{
    LabTaskTypeDetail, LabTaskTypeListItem, StaffAssignmentItem, Task, TaskDocument, TaskPart,
    TaskResult, TaskSummary, WorkflowDetail,
};

#[derive(Args)]
pub struct TasksArgs {
    #[command(subcommand)]
    pub command: TasksCommand,
}

#[derive(Subcommand)]
pub enum TasksCommand {
    /// Search lightweight task type summaries available to the current lab.
    Types {
        #[arg(short, long, default_value_t = 0)]
        skip: u32,
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
        /// Search key, display name, description, or scope within the current lab.
        #[arg(long)]
        search: Option<String>,
        /// Restrict results to one task type category.
        #[arg(long)]
        category: Option<TaskTypeCategoryArg>,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Show submission schema and user-visible documents for one lab-available task type.
    Type {
        id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Create a task in the current lab from a JSON file.
    Create {
        file: String,
        /// Attach input files as field=path entries for multipart task creation.
        #[arg(long = "file-field")]
        file_fields: Vec<String>,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Create a workflow task in the current lab from a JSON file.
    CreateWorkflow {
        file: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// List lab tasks.
    List {
        #[arg(short, long, default_value_t = 0)]
        skip: u32,
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Show one lab task.
    Get {
        id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Show one workflow part visible to the current lab.
    Part {
        task_id: String,
        part_id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Show global workflow detail for a task (platform_admin or superuser only).
    #[command(hide = true)]
    Workflow { id: String },
    /// Update a task globally (platform_admin or superuser only).
    #[command(hide = true)]
    Update { id: String, data: String },
    /// Update a task globally from a JSON file (platform_admin or superuser only).
    #[command(hide = true)]
    UpdateFile { id: String, file: String },
    /// List lab-visible task documents.
    Documents {
        id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Download a lab-visible task document.
    DownloadDocument {
        document_id: String,
        output: Option<String>,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Upload a file to a task field (e.g. plasmid file).
    UploadField {
        id: String,
        file: String,
        field_key: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// List task results visible to the lab.
    Results {
        id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Download compute output files from a task or workflow.
    DownloadResults {
        id: String,
        output_dir: Option<String>,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Confirm a task that is waiting for lab confirmation (waiting_lab_confirm → completed).
    Confirm {
        id: String,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Reject a task that is waiting for lab confirmation (waiting_lab_confirm → in_progress).
    Reject {
        id: String,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// My assigned task stages (staff view; not a lab task list).
    My {
        #[command(subcommand)]
        command: MyTasksCommand,
    },
}

#[derive(Subcommand)]
pub enum MyTasksCommand {
    /// List task stages assigned to me.
    List {
        #[arg(short, long, default_value_t = 0)]
        skip: u32,
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
        /// Search task title, description, or part name.
        #[arg(long)]
        search: Option<String>,
        /// Exclude assignments in this status.
        #[arg(long, value_enum)]
        exclude_status: Option<AssignmentStatusArg>,
    },
    /// Show one task stage assigned to me.
    Get { assignment_id: String },
    /// Update the status of my assigned task stage.
    Status {
        assignment_id: String,
        status: AssignmentStatusArg,
    },
    /// Submit a result for my assigned task stage from a JSON file.
    SubmitResult {
        assignment_id: String,
        /// JSON output object for the assigned stage. It is sent as `output_data`.
        #[arg(value_name = "OUTPUT_JSON")]
        file: String,
        /// Optional JSON file sent as top-level document_feedback (SOP/work_order feedback).
        #[arg(long, value_name = "FILE")]
        feedback: Option<String>,
    },
    /// Atomically submit a result, complete the assignment, and unlock downstream stages.
    Complete {
        assignment_id: String,
        /// JSON output object for the assigned stage. It is sent as `output_data`.
        #[arg(value_name = "OUTPUT_JSON")]
        file: String,
        /// Optional JSON file sent as top-level document_feedback (SOP/work_order feedback).
        #[arg(long, value_name = "FILE")]
        feedback: Option<String>,
    },
    /// Upload a file field for a staff-visible task.
    UploadField {
        task_id: String,
        file: String,
        field_key: String,
        #[arg(long, value_enum, default_value_t = TaskDocumentVisibilityArg::LabAndStaff)]
        visibility: TaskDocumentVisibilityArg,
    },
    /// List staff-visible documents for a task.
    Documents { task_id: String },
    /// Download a staff-visible task document.
    DownloadDocument {
        document_id: String,
        output: Option<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum AssignmentStatusArg {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TaskTypeCategoryArg {
    Staff,
    Compute,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TaskDocumentVisibilityArg {
    LabAndStaff,
    StaffOnly,
    LabOnly,
}

impl TaskDocumentVisibilityArg {
    fn as_str(&self) -> &'static str {
        match self {
            TaskDocumentVisibilityArg::LabAndStaff => "lab_and_staff",
            TaskDocumentVisibilityArg::StaffOnly => "staff_only",
            TaskDocumentVisibilityArg::LabOnly => "lab_only",
        }
    }
}

impl TaskTypeCategoryArg {
    fn as_str(&self) -> &'static str {
        match self {
            TaskTypeCategoryArg::Staff => "STAFF",
            TaskTypeCategoryArg::Compute => "COMPUTE",
        }
    }
}

impl AssignmentStatusArg {
    fn as_str(&self) -> &'static str {
        match self {
            AssignmentStatusArg::Pending => "PENDING",
            AssignmentStatusArg::InProgress => "IN_PROGRESS",
            AssignmentStatusArg::Completed => "COMPLETED",
        }
    }
}

pub async fn run(
    args: &TasksArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;

    match &args.command {
        TasksCommand::Types {
            skip,
            limit,
            search,
            category,
            lab_id,
        } => {
            let types = client
                .list_lab_task_types(
                    *skip,
                    *limit,
                    search.as_deref(),
                    category.as_ref().map(TaskTypeCategoryArg::as_str),
                    lab_id.as_deref(),
                )
                .await?;
            match format {
                OutputFormat::Json => print_result(&types, format),
                OutputFormat::Text => print_lab_task_types(&types),
            }
        }
        TasksCommand::Type { id, lab_id } => {
            let task_type = client.get_lab_task_type(id, lab_id.as_deref()).await?;
            match format {
                OutputFormat::Json => print_result(&task_type, format),
                OutputFormat::Text => print_lab_task_type_detail(&task_type),
            }
        }
        TasksCommand::Create {
            file,
            file_fields,
            lab_id,
        } => {
            let mut data = read_json_file(file)?;
            normalize_task_create_payload(&mut data)?;
            validate_task_create_payload(&data)?;
            let task = if file_fields.is_empty() {
                client.create_lab_task(&data, lab_id.as_deref()).await?
            } else {
                let parsed_file_fields = parse_file_fields(file_fields)?;
                let file_field_refs: Vec<(&str, &str)> = parsed_file_fields
                    .iter()
                    .map(|(field, path)| (field.as_str(), path.as_str()))
                    .collect();
                client
                    .create_lab_task_multipart(&data, &file_field_refs, lab_id.as_deref())
                    .await?
            };
            print_result(&task, format);
        }
        TasksCommand::CreateWorkflow { file, lab_id } => {
            let mut data = read_json_file(file)?;
            normalize_task_create_payload(&mut data)?;
            validate_task_create_payload(&data)?;
            validate_task_workflow_payload(&data)?;
            let task = client.create_lab_task(&data, lab_id.as_deref()).await?;
            print_result(&task, format);
        }
        TasksCommand::List {
            skip,
            limit,
            lab_id,
        } => {
            let tasks = client
                .list_lab_tasks(*skip, *limit, lab_id.as_deref())
                .await?;
            match format {
                OutputFormat::Json => print_result(&tasks, format),
                OutputFormat::Text => print_tasks(&tasks),
            }
        }
        TasksCommand::Get { id, lab_id } => {
            let task = client.get_lab_task(id, lab_id.as_deref()).await?;
            let input_requirements =
                load_task_input_requirements(&client, &task, lab_id.as_deref()).await;
            match format {
                OutputFormat::Json => {
                    print_result(&task_detail_value(&task, &input_requirements), format)
                }
                OutputFormat::Text => print_task_detail(&task, &input_requirements),
            }
        }
        TasksCommand::Part {
            task_id,
            part_id,
            lab_id,
        } => {
            let part = client
                .get_lab_task_part(task_id, part_id, lab_id.as_deref())
                .await?;
            print_result(&part, format);
        }
        TasksCommand::Workflow { id } => {
            let workflow = client.get_task_workflow(id).await?;
            match client.list_lab_task_results(id, None).await {
                Ok(results) if !results.items.is_empty() => {
                    print_workflow_results(&client, &workflow, &results, None, format).await?;
                }
                _ => match format {
                    OutputFormat::Json => print_result(&workflow, format),
                    OutputFormat::Text => print_workflow_detail(&workflow),
                },
            }
        }
        TasksCommand::Update { id, data } => {
            let data: serde_json::Value = serde_json::from_str(data)?;
            let task = client.update_task(id, &data).await?;
            print_result(&task, format);
        }
        TasksCommand::UpdateFile { id, file } => {
            let data = read_json_file(file)?;
            let task = client.update_task(id, &data).await?;
            print_result(&task, format);
        }
        TasksCommand::Documents { id, lab_id } => {
            let documents = client
                .list_lab_task_documents(id, lab_id.as_deref())
                .await?;
            match format {
                OutputFormat::Json => print_result(&documents, format),
                OutputFormat::Text => print_task_documents_text(&documents.items),
            }
        }
        TasksCommand::DownloadDocument {
            document_id,
            output,
            lab_id,
        } => {
            let bytes = client
                .download_lab_task_document(document_id, lab_id.as_deref())
                .await?;
            write_download(document_id, output.as_deref(), &bytes)?;
        }
        TasksCommand::UploadField {
            id,
            file,
            field_key,
            lab_id,
        } => {
            let result = client
                .upload_lab_task_field(id, file, field_key, lab_id.as_deref())
                .await?;
            print_result(&result, format);
        }
        TasksCommand::Results { id, lab_id } => {
            let task = client.get_lab_task(id, lab_id.as_deref()).await?;
            let workflow = get_task_workflow_if_available(&client, id).await?;
            let results = client.list_lab_task_results(id, lab_id.as_deref()).await?;

            if let Some(workflow) =
                workflow.filter(|item| should_render_workflow_results(&task, item))
            {
                print_workflow_results(&client, &workflow, &results, lab_id.as_deref(), format)
                    .await?;
            } else if resolve_is_compute_task(&client, &task, lab_id.as_deref()).await {
                print_compute_results(&task, format);
            } else {
                print_experiment_results(&results, format);
            }
        }
        TasksCommand::DownloadResults {
            id,
            output_dir,
            lab_id,
        } => {
            let task = client.get_lab_task(id, lab_id.as_deref()).await?;
            let workflow = get_task_workflow_if_available(&client, id).await?;
            download_task_result_files(&client, &task, workflow.as_ref(), output_dir.as_deref())
                .await?;
        }
        TasksCommand::Confirm { id, lab_id } => {
            let result = client.confirm_lab_task(id, lab_id.as_deref()).await?;
            print_result(&result, format);
        }
        TasksCommand::Reject { id, reason, lab_id } => {
            let result = client
                .reject_lab_task(id, reason.as_deref(), lab_id.as_deref())
                .await?;
            print_result(&result, format);
        }
        TasksCommand::My { command } => run_my_tasks(&client, command, format).await?,
    }

    Ok(())
}

async fn run_my_tasks(
    client: &ScientexClient,
    command: &MyTasksCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        MyTasksCommand::List {
            skip,
            limit,
            search,
            exclude_status,
        } => {
            let assignments = client
                .list_my_task_assignments(
                    *skip,
                    *limit,
                    search.as_deref(),
                    exclude_status.as_ref().map(AssignmentStatusArg::as_str),
                )
                .await?;
            match format {
                OutputFormat::Json => print_result(&assignments, format),
                OutputFormat::Text => print_assignments(&assignments),
            }
        }
        MyTasksCommand::Get { assignment_id } => {
            let assignment = client.get_my_task_assignment(assignment_id).await?;
            print_result(&assignment, format);
        }
        MyTasksCommand::Status {
            assignment_id,
            status,
        } => {
            let result = client
                .update_my_task_assignment_status(assignment_id, status.as_str())
                .await?;
            print_result(&result, format);
        }
        MyTasksCommand::SubmitResult {
            assignment_id,
            file,
            feedback,
        } => {
            let data = read_result_payload(file, feedback.as_deref())?;
            let result = client.submit_my_task_result(assignment_id, &data).await?;
            print_result(&result, format);
        }
        MyTasksCommand::Complete {
            assignment_id,
            file,
            feedback,
        } => {
            let data = read_result_payload(file, feedback.as_deref())?;
            let result = client
                .complete_my_task_assignment(assignment_id, &data)
                .await?;
            print_result(&result, format);
        }
        MyTasksCommand::UploadField {
            task_id,
            file,
            field_key,
            visibility,
        } => {
            let result = client
                .upload_my_task_field(task_id, file, field_key, visibility.as_str())
                .await?;
            print_result(&result, format);
        }
        MyTasksCommand::Documents { task_id } => {
            let documents = client.list_my_task_documents(task_id).await?;
            match format {
                OutputFormat::Json => print_result(&documents, format),
                OutputFormat::Text => print_task_documents_text(&documents.items),
            }
        }
        MyTasksCommand::DownloadDocument {
            document_id,
            output,
        } => {
            let bytes = client.download_my_task_document(document_id).await?;
            write_download(document_id, output.as_deref(), &bytes)?;
        }
    }

    Ok(())
}

fn read_json_file(path: &str) -> anyhow::Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn read_result_payload(
    path: &str,
    feedback_path: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    let data = read_json_file(path)?;
    let feedback_data = feedback_path.map(read_json_file).transpose()?;
    normalize_result_payload(data, feedback_data)
}

/// Normalize a result file into the backend's `TaskResultCreate` request body.
///
/// A bare output object is the ergonomic default. Existing full request envelopes
/// remain accepted for compatibility; an empty object is also kept as an empty
/// request envelope rather than silently changing its semantics.
fn normalize_result_payload(
    data: serde_json::Value,
    feedback_data: Option<serde_json::Value>,
) -> anyhow::Result<serde_json::Value> {
    let is_existing_envelope = data.as_object().is_some_and(|object| {
        object.is_empty()
            || object.keys().any(|key| {
                matches!(
                    key.as_str(),
                    "output_data" | "comment" | "document_feedback"
                )
            })
    });

    let mut payload = if is_existing_envelope {
        data
    } else if data.is_object() || data.is_null() {
        serde_json::json!({ "output_data": data })
    } else {
        anyhow::bail!("Result JSON file must contain a JSON object or null");
    };

    if let Some(feedback_data) = feedback_data {
        payload
            .as_object_mut()
            .expect("result payload is always a JSON object")
            .insert("document_feedback".to_string(), feedback_data);
    }
    Ok(payload)
}

fn print_task_documents_text(docs: &[TaskDocument]) {
    if docs.is_empty() {
        println!("No documents for this task");
        return;
    }
    println!("Task documents:");
    for doc in docs {
        let sync_badge = feishu_sync_badge(doc.feishu_sync_status.as_deref());
        let source_info = doc
            .source_type_document_id
            .as_deref()
            .map(|id| format!("  source_template={}", &id[..8.min(id.len())]))
            .unwrap_or_default();
        println!(
            "  {}  {}  {}  {}  vis={}  feishu={}{}",
            doc.id,
            doc.document_type,
            doc.filename,
            doc.file_size,
            doc.visibility,
            sync_badge,
            source_info,
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

fn normalize_task_create_payload(data: &mut serde_json::Value) -> anyhow::Result<()> {
    let obj = data
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Task payload must be a JSON object"))?;

    obj.remove("lab_id");
    obj.remove("source_type");
    obj.remove("source_id");

    let root_task_type_id = obj.remove("task_type_id");
    let root_input_data = obj.get("input_data").cloned();
    let title = obj
        .get("title")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Stage 1")
        .to_string();

    if obj.get("parts").is_none() {
        if let Some(task_type_id) = root_task_type_id.clone() {
            let mut generated_part = serde_json::Map::new();
            generated_part.insert("client_key".to_string(), serde_json::json!("part_1"));
            generated_part.insert("name".to_string(), serde_json::json!(title));
            generated_part.insert("task_type_id".to_string(), task_type_id);
            if let Some(input_data) = root_input_data.clone() {
                generated_part.insert("input_data".to_string(), input_data);
            }
            obj.insert(
                "parts".to_string(),
                serde_json::Value::Array(vec![serde_json::Value::Object(generated_part)]),
            );
        }
        return Ok(());
    }

    let Some(parts) = obj.get_mut("parts").and_then(|value| value.as_array_mut()) else {
        return Ok(());
    };

    let part_count = parts.len();
    for (index, part) in parts.iter_mut().enumerate() {
        let Some(part_obj) = part.as_object_mut() else {
            continue;
        };
        if !part_obj.contains_key("client_key") {
            part_obj.insert(
                "client_key".to_string(),
                serde_json::json!(format!("part_{}", index + 1)),
            );
        }
        if part_count == 1 {
            if !part_obj.contains_key("task_type_id") {
                if let Some(task_type_id) = root_task_type_id.clone() {
                    part_obj.insert("task_type_id".to_string(), task_type_id);
                }
            }
            if !part_obj.contains_key("input_data") {
                if let Some(input_data) = root_input_data.clone() {
                    part_obj.insert("input_data".to_string(), input_data);
                }
            }
            if !part_obj.contains_key("name") {
                part_obj.insert("name".to_string(), serde_json::json!(title));
            }
        }
    }

    Ok(())
}

fn validate_task_create_payload(data: &serde_json::Value) -> anyhow::Result<()> {
    let obj = data
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Task payload must be a JSON object"))?;

    obj.get("title")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Task payload requires a non-empty `title`"))?;

    let parts = obj
        .get("parts")
        .and_then(|value| value.as_array())
        .ok_or_else(|| anyhow::anyhow!("Task payload requires a `parts` array"))?;
    if parts.is_empty() {
        anyhow::bail!("Task payload must contain at least one part");
    }

    for (index, part) in parts.iter().enumerate() {
        let part_obj = part
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Task part #{} must be a JSON object", index + 1))?;
        part_obj
            .get("client_key")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("Task part #{} requires a non-empty `client_key`", index + 1)
            })?;
    }

    Ok(())
}

fn validate_task_workflow_payload(data: &serde_json::Value) -> anyhow::Result<()> {
    let obj = data
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Workflow payload must be a JSON object"))?;
    let parts = obj
        .get("parts")
        .and_then(|value| value.as_array())
        .ok_or_else(|| anyhow::anyhow!("Workflow payload requires a `parts` array"))?;

    let mut client_keys = HashSet::new();
    for (index, part) in parts.iter().enumerate() {
        let part_obj = part
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Workflow part #{} must be a JSON object", index + 1))?;
        let client_key = part_obj
            .get("client_key")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Workflow part #{} requires a non-empty `client_key`",
                    index + 1
                )
            })?;

        if !client_keys.insert(client_key.to_string()) {
            anyhow::bail!("Duplicate workflow part client_key `{client_key}`");
        }
    }

    let mut dependency_graph: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(dependencies) = obj.get("dependencies") {
        let dependencies = dependencies
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("`dependencies` must be an array"))?;
        for (index, dependency) in dependencies.iter().enumerate() {
            let dep_obj = dependency.as_object().ok_or_else(|| {
                anyhow::anyhow!("Workflow dependency #{} must be a JSON object", index + 1)
            })?;
            let prerequisite = dep_obj
                .get("prerequisite_client_key")
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Workflow dependency #{} requires `prerequisite_client_key`",
                        index + 1
                    )
                })?;
            let dependent = dep_obj
                .get("dependent_client_key")
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Workflow dependency #{} requires `dependent_client_key`",
                        index + 1
                    )
                })?;
            if prerequisite == dependent {
                anyhow::bail!(
                    "Workflow dependency #{} cannot point a part to itself (`{prerequisite}`)",
                    index + 1
                );
            }
            if !client_keys.contains(prerequisite) {
                anyhow::bail!(
                    "Workflow dependency #{} references unknown prerequisite client_key `{prerequisite}`",
                    index + 1
                );
            }
            if !client_keys.contains(dependent) {
                anyhow::bail!(
                    "Workflow dependency #{} references unknown dependent client_key `{dependent}`",
                    index + 1
                );
            }
            dependency_graph
                .entry(prerequisite.to_string())
                .or_default()
                .push(dependent.to_string());
        }
    }

    ensure_workflow_dependencies_are_acyclic(&dependency_graph)?;

    Ok(())
}

fn ensure_workflow_dependencies_are_acyclic(
    graph: &HashMap<String, Vec<String>>,
) -> anyhow::Result<()> {
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    for node in graph.keys() {
        detect_workflow_dependency_cycle(node, graph, &mut visiting, &mut visited, &mut stack)?;
    }

    Ok(())
}

fn detect_workflow_dependency_cycle(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) -> anyhow::Result<()> {
    if visited.contains(node) {
        return Ok(());
    }
    if visiting.contains(node) {
        let start = stack.iter().position(|item| item == node).unwrap_or(0);
        let mut cycle = stack[start..].to_vec();
        cycle.push(node.to_string());
        anyhow::bail!(
            "Workflow dependencies contain a cycle: {}",
            cycle.join(" -> ")
        );
    }

    visiting.insert(node.to_string());
    stack.push(node.to_string());

    if let Some(children) = graph.get(node) {
        for child in children {
            detect_workflow_dependency_cycle(child, graph, visiting, visited, stack)?;
        }
    }

    stack.pop();
    visiting.remove(node);
    visited.insert(node.to_string());
    Ok(())
}

fn parse_file_fields(values: &[String]) -> anyhow::Result<Vec<(String, String)>> {
    values
        .iter()
        .map(|value| {
            let (field, path) = value
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("File field must use field=path format"))?;
            if field.trim().is_empty() || path.trim().is_empty() {
                anyhow::bail!("File field must use non-empty field=path values");
            }
            Ok((field.to_string(), path.to_string()))
        })
        .collect()
}

fn write_download(document_id: &str, output: Option<&str>, bytes: &[u8]) -> anyhow::Result<()> {
    let output = output
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("task_document_{document_id}"));
    let output_path = unique_download_path(output);
    std::fs::write(&output_path, bytes)?;
    println!("Downloaded to {}", output_path.display());
    Ok(())
}

async fn download_task_result_files(
    client: &ScientexClient,
    task: &Task,
    workflow: Option<&WorkflowDetail>,
    output_dir: Option<&str>,
) -> anyhow::Result<()> {
    let base_dir = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("task_results_{}", task.id)));
    fs::create_dir_all(&base_dir)?;

    let mut downloads = Vec::new();
    collect_output_file_downloads("task", task.output_data.as_ref(), &mut downloads);
    if !task.parts.is_empty() {
        for part in &task.parts {
            let label = format!("{}_{}", part.sort_order, sanitize_path_segment(&part.name));
            collect_output_file_downloads(&label, part.output_data.as_ref(), &mut downloads);
        }
    } else if let Some(workflow) = workflow {
        for part in &workflow.parts {
            let label = format!("{}_{}", part.sort_order, sanitize_path_segment(&part.name));
            collect_output_file_downloads(&label, part.output_data.as_ref(), &mut downloads);
        }
    }

    if downloads.is_empty() {
        println!("No downloadable result files found for task {}", task.id);
        return Ok(());
    }

    for download in downloads {
        let part_dir = base_dir.join(&download.group);
        fs::create_dir_all(&part_dir)?;
        let output_path = unique_output_path(&part_dir, &download.filename);
        let bytes = client.download_task_output_file(&download.url).await?;
        fs::write(&output_path, bytes)?;
        println!("Downloaded {}", output_path.display());
    }

    Ok(())
}

struct OutputFileDownload {
    group: String,
    filename: String,
    url: String,
}

fn collect_output_file_downloads(
    group: &str,
    output_data: Option<&serde_json::Value>,
    downloads: &mut Vec<OutputFileDownload>,
) {
    let Some(files) = output_data
        .and_then(|value| value.get("files"))
        .and_then(|value| value.as_array())
    else {
        return;
    };

    for file in files {
        let Some(url) = file.get("download_url").and_then(|value| value.as_str()) else {
            continue;
        };
        let filename = file
            .get("filename")
            .and_then(|value| value.as_str())
            .map(sanitize_path_segment)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "result.bin".to_string());
        downloads.push(OutputFileDownload {
            group: group.to_string(),
            filename,
            url: url.to_string(),
        });
    }
}

fn unique_output_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }

    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("result");
    let extension = path.extension().and_then(|value| value.to_str());
    for index in 2.. {
        let filename = match extension {
            Some(extension) => format!("{stem}_{index}.{extension}"),
            None => format!("{stem}_{index}"),
        };
        let candidate = dir.join(filename);
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("unbounded loop always returns")
}

fn sanitize_path_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string()
}

async fn get_task_workflow_if_available(
    client: &ScientexClient,
    task_id: &str,
) -> anyhow::Result<Option<WorkflowDetail>> {
    match client.get_task_workflow(task_id).await {
        Ok(workflow) => Ok(Some(workflow)),
        // Lab members can see task results through /lab/tasks but do not have access to
        // the global /tasks/{id}/workflow administrator view. Fall back to lab results.
        Err(ScientexError::HttpError {
            status: 401 | 403 | 404,
            ..
        }) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

async fn resolve_is_compute_task(
    client: &ScientexClient,
    task: &Task,
    lab_id: Option<&str>,
) -> bool {
    if let Some(task_type_id) = task.task_type_id.as_deref() {
        if let Ok(task_type) = client.get_lab_task_type(task_type_id, lab_id).await {
            return is_compute_task_from_category_or_output(
                task,
                Some(task_type.category.as_str()),
            );
        }
    }

    is_compute_task_from_category_or_output(task, None)
}

fn should_render_workflow_results(task: &Task, workflow: &WorkflowDetail) -> bool {
    if workflow.parts.len() > 1 || !workflow.dependencies.is_empty() {
        return true;
    }

    if task.task_type_id.is_none() && !workflow.parts.is_empty() {
        return true;
    }

    let Some(root_task_type_id) = task.task_type_id.as_deref() else {
        return false;
    };
    workflow
        .parts
        .iter()
        .filter_map(|part| part.task_type_id.as_deref())
        .any(|part_task_type_id| part_task_type_id != root_task_type_id)
}

fn is_compute_task_from_category_or_output(task: &Task, category: Option<&str>) -> bool {
    if let Some(category) = category {
        return is_compute_category(category);
    }
    output_data_has_content(task.output_data.as_ref())
}

fn is_compute_category(category: &str) -> bool {
    category.eq_ignore_ascii_case("compute")
}

fn output_data_has_content(output_data: Option<&serde_json::Value>) -> bool {
    match output_data {
        None | Some(serde_json::Value::Null) => false,
        Some(serde_json::Value::Object(obj)) => !obj.is_empty(),
        Some(serde_json::Value::Array(items)) => !items.is_empty(),
        Some(serde_json::Value::String(value)) => !value.is_empty(),
        Some(_) => true,
    }
}

fn print_compute_results(task: &Task, format: &OutputFormat) {
    let view = serde_json::json!({
        "kind": "compute",
        "task_id": task.id,
        "status": task.status,
        "output_data": task.output_data,
    });

    match format {
        OutputFormat::Json => print_result(&view, format),
        OutputFormat::Text => print_compute_results_text(task),
    }
}

fn print_compute_results_text(task: &Task) {
    println!("Task: {}", task.id);
    println!("Kind: compute");
    println!("Status: {}", task.status);
    for line in compute_output_lines(task.output_data.as_ref(), &task.status) {
        println!("{line}");
    }
}

fn print_experiment_results(
    results: &crate::api_response::PaginatedList<TaskResult>,
    format: &OutputFormat,
) {
    match format {
        OutputFormat::Json => print_result(results, format),
        OutputFormat::Text => {
            if results.items.is_empty() {
                println!("No submitted task results");
            } else {
                print_paginated_items(results);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct WorkflowPartResultView {
    part_id: String,
    name: String,
    status: String,
    sort_order: i64,
    task_type_id: Option<String>,
    category: String,
    assignment_count: usize,
    result_count: usize,
    output_data: Option<serde_json::Value>,
    results: Vec<TaskResult>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkflowResultsView {
    kind: &'static str,
    task_id: String,
    status: String,
    part_count: usize,
    dependency_count: usize,
    assignment_count: usize,
    parts: Vec<WorkflowPartResultView>,
}

async fn print_workflow_results(
    client: &ScientexClient,
    workflow: &WorkflowDetail,
    results: &crate::api_response::PaginatedList<TaskResult>,
    lab_id: Option<&str>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let part_views = build_workflow_part_views(client, workflow, &results.items, lab_id).await?;
    let view = WorkflowResultsView {
        kind: "workflow",
        task_id: workflow.task.id.clone(),
        status: workflow.task.status.clone(),
        part_count: part_views.len(),
        dependency_count: workflow.dependencies.len(),
        assignment_count: workflow.assignments.len(),
        parts: part_views,
    };

    match format {
        OutputFormat::Json => print_result(&view, format),
        OutputFormat::Text => print_workflow_results_text(&view),
    }

    Ok(())
}

async fn build_workflow_part_views(
    client: &ScientexClient,
    workflow: &WorkflowDetail,
    results: &[TaskResult],
    lab_id: Option<&str>,
) -> anyhow::Result<Vec<WorkflowPartResultView>> {
    let mut results_by_part: HashMap<&str, Vec<TaskResult>> = HashMap::new();
    for result in results {
        results_by_part
            .entry(result.part_id.as_str())
            .or_default()
            .push(result.clone());
    }

    let mut assignment_count_by_part: HashMap<&str, usize> = HashMap::new();
    for assignment in &workflow.assignments {
        *assignment_count_by_part
            .entry(assignment.part_id.as_str())
            .or_insert(0) += 1;
    }

    let mut task_type_categories = load_task_type_categories(client, lab_id).await;
    let mut parts = workflow.parts.clone();
    parts.sort_by_key(|part| part.sort_order);

    let mut views = Vec::with_capacity(parts.len());
    for part in parts {
        let category =
            resolve_part_category(client, &part, &mut task_type_categories, lab_id).await?;
        let part_results = results_by_part.remove(part.id.as_str()).unwrap_or_default();
        let result_count = part_results.len();
        let assignment_count = assignment_count_by_part
            .get(part.id.as_str())
            .copied()
            .unwrap_or(0);

        views.push(WorkflowPartResultView {
            part_id: part.id,
            name: part.name,
            status: part.status,
            sort_order: part.sort_order,
            task_type_id: part.task_type_id,
            category,
            assignment_count,
            result_count,
            output_data: part.output_data,
            results: part_results,
        });
    }

    Ok(views)
}

async fn load_task_type_categories(
    client: &ScientexClient,
    lab_id: Option<&str>,
) -> HashMap<String, String> {
    client
        .list_lab_task_types(0, 100, None, None, lab_id)
        .await
        .map(|types| {
            types
                .items
                .into_iter()
                .map(|task_type| (task_type.id, task_type.category))
                .collect()
        })
        .unwrap_or_default()
}

async fn resolve_part_category(
    client: &ScientexClient,
    part: &TaskPart,
    categories: &mut HashMap<String, String>,
    lab_id: Option<&str>,
) -> anyhow::Result<String> {
    if let Some(task_type_id) = part.task_type_id.as_deref() {
        if let Some(category) = categories.get(task_type_id) {
            return Ok(category.clone());
        }

        let task_type = client
            .get_lab_task_type(task_type_id, lab_id)
            .await
            .with_context(|| {
                format!(
                    "Failed to load task type `{task_type_id}` for workflow part `{}`",
                    part.id
                )
            })?;
        categories.insert(task_type.id.clone(), task_type.category.clone());
        return Ok(task_type.category);
    }

    if output_data_has_content(part.output_data.as_ref()) {
        Ok("COMPUTE".to_string())
    } else {
        Ok("STAFF".to_string())
    }
}

fn print_workflow_detail(workflow: &WorkflowDetail) {
    println!("Task: {}", workflow.task.id);
    println!("Title: {}", workflow.task.title);
    println!("Status: {}", workflow.task.status);
    println!("Parts: {}", workflow.parts.len());
    println!("Dependencies: {}", workflow.dependencies.len());
    println!("Assignments: {}", workflow.assignments.len());

    if !workflow.parts.is_empty() {
        println!("\nParts:");
        let mut parts = workflow.parts.clone();
        parts.sort_by_key(|part| part.sort_order);
        for part in parts {
            println!(
                "  {}. {}  [{}]  type={}",
                part.sort_order,
                part.name,
                part.status,
                part.task_type_id.as_deref().unwrap_or("-")
            );
        }
    }

    if !workflow.dependencies.is_empty() {
        println!("\nDependencies:");
        for dependency in &workflow.dependencies {
            println!(
                "  {} -> {}  ({})",
                dependency_field(dependency, "prerequisite_client_key"),
                dependency_field(dependency, "dependent_client_key"),
                dependency_field(dependency, "condition_type")
            );
        }
    }

    if !workflow.assignments.is_empty() {
        println!("\nAssignments:");
        for assignment in &workflow.assignments {
            println!(
                "  {}  part={}  assignee={}  {:10}  {}",
                assignment.id,
                assignment.part_id,
                assignment.assignee_id,
                assignment.role,
                assignment.status
            );
        }
    }
}

fn dependency_field<'a>(dependency: &'a serde_json::Value, key: &str) -> &'a str {
    dependency
        .get(key)
        .and_then(|value| value.as_str())
        .unwrap_or("-")
}

fn print_workflow_results_text(view: &WorkflowResultsView) {
    println!("Task: {}", view.task_id);
    println!("Kind: workflow");
    println!("Status: {}", view.status);
    println!(
        "Parts: {}  Dependencies: {}  Assignments: {}",
        view.part_count, view.dependency_count, view.assignment_count
    );

    for (index, part) in view.parts.iter().enumerate() {
        println!("\n[{}] {}", index + 1, part.name);
        println!("  Part ID: {}", part.part_id);
        println!("  Status: {}", part.status);
        println!(
            "  Task type: {}",
            part.task_type_id.as_deref().unwrap_or("-")
        );
        println!("  Category: {}", part.category);
        println!("  Assignments: {}", part.assignment_count);

        if is_compute_category(&part.category) {
            println!("  Compute output:");
            for line in compute_output_lines(part.output_data.as_ref(), &part.status) {
                println!("    {line}");
            }
        } else {
            println!("  Submitted results: {}", part.result_count);
            if part.results.is_empty() {
                println!("    No submitted task results");
            } else {
                for result in &part.results {
                    println!(
                        "    {}  submitted_by={}  created_at={}",
                        result.id, result.submitted_by_id, result.created_at
                    );
                    if let Some(comment) =
                        result.comment.as_deref().filter(|value| !value.is_empty())
                    {
                        println!("      comment: {comment}");
                    }
                    if let Some(output) = result
                        .output_data
                        .as_ref()
                        .and_then(|value| value.as_object())
                        .filter(|obj| !obj.is_empty())
                    {
                        println!("      output:");
                        for (key, value) in output {
                            let display = match value {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                            println!("        {key}: {display}");
                        }
                    }
                }
            }
        }
    }
}

fn compute_output_lines(output_data: Option<&serde_json::Value>, status: &str) -> Vec<String> {
    let Some(output_data) = output_data.filter(|value| output_data_has_content(Some(value))) else {
        return vec![format!("No compute output yet. Task status: {status}")];
    };

    let mut lines = Vec::new();

    if let Some(exit_code) = output_data.get("exit_code") {
        lines.push(format!("Exit code: {exit_code}"));
    }

    if output_data
        .get("exit_code")
        .and_then(|value| value.as_i64())
        .is_some_and(|code| code != 0)
    {
        lines.push(format!(
            "Compute task failed with exit code {}",
            output_data["exit_code"]
        ));
        if let Some(stderr) = output_data
            .get("stderr_log_url")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("stderr: {stderr}"));
        }
    }

    if let Some(files) = output_data.get("files").and_then(|value| value.as_array()) {
        lines.push("Files:".to_string());
        if files.is_empty() {
            lines.push("  No files".to_string());
        }
        for file in files {
            let filename = file
                .get("filename")
                .and_then(|value| value.as_str())
                .unwrap_or("-");
            let size = file
                .get("size_bytes")
                .map(ToString::to_string)
                .unwrap_or_else(|| "-".to_string());
            let relative_path = file
                .get("relative_path")
                .and_then(|value| value.as_str())
                .unwrap_or("-");
            lines.push(format!("  {filename}  {size} bytes  {relative_path}"));
            if let Some(download_url) = file.get("download_url").and_then(|value| value.as_str()) {
                lines.push(format!("    {download_url}"));
            }
        }
    }

    let stdout = output_data
        .get("stdout_log_url")
        .and_then(|value| value.as_str());
    let stderr = output_data
        .get("stderr_log_url")
        .and_then(|value| value.as_str());
    if stdout.is_some() || stderr.is_some() {
        lines.push("Logs:".to_string());
        if let Some(stdout) = stdout {
            lines.push(format!("  stdout: {stdout}"));
        }
        if let Some(stderr) = stderr {
            lines.push(format!("  stderr: {stderr}"));
        }
    }

    if lines.is_empty() {
        lines.push(format!("No compute output yet. Task status: {status}"));
    }

    lines
}

fn print_lab_task_types(list: &crate::api_response::PaginatedList<LabTaskTypeListItem>) {
    print_pagination_metadata(list);
    if list.items.is_empty() {
        println!("No task types");
        return;
    }
    for item in &list.items {
        println!(
            "{}  {:8}  fields={}/{} files={}  {}",
            item.id,
            item.category,
            item.input_summary.required_field_count,
            item.input_summary.field_count,
            item.input_summary.file_field_count,
            item.display_name
        );
    }
}

fn print_lab_task_type_detail(task_type: &LabTaskTypeDetail) {
    println!(
        "{}  {}  {}",
        task_type.id, task_type.category, task_type.display_name
    );
    if let Some(description) = &task_type.description {
        println!("  {description}");
    }
    println!(
        "  inputs: {} total, {} required, {} file fields",
        task_type.input_summary.field_count,
        task_type.input_summary.required_field_count,
        task_type.input_summary.file_field_count
    );
    println!(
        "  documents: {} (SOP: {}, work order: {})",
        task_type.documents.len(),
        task_type.has_sop,
        task_type.has_work_order
    );
}

#[derive(Debug, Clone, Serialize)]
struct TaskTypeInputRequirements {
    id: String,
    key: String,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct TaskPartInputRequirements {
    part_id: String,
    part_name: String,
    task_type_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    requirements: Option<TaskTypeInputRequirements>,
}

async fn load_task_input_requirements(
    client: &ScientexClient,
    task: &Task,
    lab_id: Option<&str>,
) -> Vec<TaskPartInputRequirements> {
    let mut task_types = HashMap::<String, Option<LabTaskTypeDetail>>::new();
    let mut requirements = Vec::new();

    for part in &task.parts {
        let Some(task_type_id) = part.task_type_id.as_deref() else {
            continue;
        };
        if !task_types.contains_key(task_type_id) {
            let task_type = client.get_lab_task_type(task_type_id, lab_id).await.ok();
            task_types.insert(task_type_id.to_string(), task_type);
        }
        let task_type = task_types
            .get(task_type_id)
            .and_then(|task_type| task_type.as_ref());
        requirements.push(TaskPartInputRequirements {
            part_id: part.id.clone(),
            part_name: part.name.clone(),
            task_type_id: task_type_id.to_string(),
            requirements: task_type.map(|task_type| TaskTypeInputRequirements {
                id: task_type.id.clone(),
                key: task_type.key.clone(),
                display_name: task_type.display_name.clone(),
                description: task_type.description.clone(),
                input_schema: task_type.input_schema.clone(),
            }),
        });
    }

    requirements
}

fn task_detail_value(
    task: &Task,
    input_requirements: &[TaskPartInputRequirements],
) -> serde_json::Value {
    let mut task_detail = serde_json::to_value(task).expect("Task should always serialize");
    if let Some(object) = task_detail.as_object_mut() {
        object.insert(
            "input_requirements".to_string(),
            serde_json::to_value(input_requirements)
                .expect("task input requirements should always serialize"),
        );
    }
    task_detail
}

fn print_task_detail(task: &Task, input_requirements: &[TaskPartInputRequirements]) {
    println!("{}  {}  {}", task.id, task.status, task.title);
    if let Some(description) = task
        .description
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        println!("  {description}");
    }

    let mut parts = task.parts.iter().collect::<Vec<_>>();
    parts.sort_by_key(|part| part.sort_order);
    for part in parts {
        println!("\n阶段 {}  {}  {}", part.id, part.status, part.name);
        if let Some(description) = part
            .description
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            println!("  {description}");
        }
        let requirement = input_requirements
            .iter()
            .find(|requirement| requirement.part_id == part.id);
        match requirement.and_then(|requirement| requirement.requirements.as_ref()) {
            Some(requirement) => {
                println!(
                    "  输入要求（{} / {}）:",
                    requirement.display_name, requirement.key
                );
                if let Some(description) = requirement
                    .description
                    .as_deref()
                    .filter(|value| !value.is_empty())
                {
                    for line in description.lines() {
                        println!("    {line}");
                    }
                }
                print_task_input_schema(requirement.input_schema.as_ref());
            }
            None if part.task_type_id.is_some() => {
                println!("  输入要求暂不可读取（任务类型不再对当前实验室可见）。");
            }
            None => {}
        }
    }
}

fn print_task_input_schema(schema: Option<&serde_json::Value>) {
    let Some(properties) = schema
        .and_then(|schema| schema.get("properties"))
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    if properties.is_empty() {
        return;
    }

    let required = schema
        .and_then(|schema| schema.get("required"))
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    println!("    输入字段:");
    let mut fields = properties.iter().collect::<Vec<_>>();
    fields.sort_by(|(left, _), (right, _)| left.cmp(right));
    for (key, field) in fields {
        let title = field
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(key);
        let field_type = field
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("-");
        let required_marker = if required.contains(key.as_str()) {
            " 必填"
        } else {
            ""
        };
        println!("      {title} ({key}, {field_type}){required_marker}");
        if let Some(description) = field
            .get("description")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
        {
            println!("        {description}");
        }
        if let Some(accept) = field
            .get("accept")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
        {
            println!("        接受文件: {accept}");
        }
    }
}

fn print_tasks(list: &crate::api_response::PaginatedList<TaskSummary>) {
    print_pagination_metadata(list);
    if list.items.is_empty() {
        println!("No tasks");
        return;
    }
    for task in &list.items {
        let parts_summary = if task.parts_summary.is_empty() {
            "-".to_string()
        } else {
            task.parts_summary.join(" | ")
        };
        println!("{}  {:20}  {}", task.id, task.status, task.title);
        println!("  parts: {parts_summary}");
    }
}

fn print_assignments(list: &crate::api_response::PaginatedList<StaffAssignmentItem>) {
    print_pagination_metadata(list);
    if list.items.is_empty() {
        println!("No assigned tasks");
        return;
    }
    for assignment in &list.items {
        println!(
            "{}  {:12}  {:8}  {}  part={}",
            assignment.assignment.id,
            assignment.assignment.status,
            assignment.assignment.role,
            assignment.task.title,
            assignment.part.name
        );
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
        Tasks(TasksArgs),
    }

    fn parse_tasks(args: &[&str]) -> TasksArgs {
        let cli = TestCli::try_parse_from(std::iter::once("scitex").chain(args.iter().copied()))
            .expect("tasks command should parse");
        match cli.command {
            TestCommand::Tasks(args) => args,
        }
    }

    fn task_with_output(output_data: Option<serde_json::Value>) -> Task {
        Task {
            id: "task-1".to_string(),
            lab_id: "lab-1".to_string(),
            title: "Task".to_string(),
            status: "completed".to_string(),
            created_by_id: "user-1".to_string(),
            created_at: "2026-06-09T00:00:00Z".to_string(),
            updated_at: "2026-06-09T00:00:00Z".to_string(),
            description: None,
            input_data: None,
            output_data,
            source_type: None,
            source_id: None,
            task_type_id: Some("type-1".to_string()),
            parts: vec![],
        }
    }

    fn task_part(
        id: &str,
        task_type_id: Option<&str>,
        output_data: Option<serde_json::Value>,
        sort_order: i64,
    ) -> TaskPart {
        TaskPart {
            id: id.to_string(),
            task_id: "task-1".to_string(),
            name: format!("Part {id}"),
            status: "pending".to_string(),
            sort_order,
            created_at: "2026-06-09T00:00:00Z".to_string(),
            updated_at: "2026-06-09T00:00:00Z".to_string(),
            description: None,
            task_type_id: task_type_id.map(ToString::to_string),
            input_data: None,
            output_schema: None,
            output_data,
        }
    }

    fn workflow_with_parts(parts: Vec<TaskPart>) -> WorkflowDetail {
        WorkflowDetail {
            task: task_with_output(None),
            parts,
            dependencies: vec![],
            assignments: vec![],
        }
    }

    fn task_type_input_requirements() -> TaskPartInputRequirements {
        TaskPartInputRequirements {
            part_id: "part-1".to_string(),
            part_name: "清单导入解析".to_string(),
            task_type_id: "type-1".to_string(),
            requirements: Some(TaskTypeInputRequirements {
                id: "type-1".to_string(),
                key: "seed_manifest_import".to_string(),
                display_name: "种子清单导入解析".to_string(),
                description: Some("样品名称为必填列".to_string()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "source_file": {"type": "object", "format": "file", "accept": ".xlsx"}
                    },
                    "required": ["source_file"]
                })),
            }),
        }
    }

    #[test]
    fn task_detail_json_includes_part_input_requirements() {
        let mut task = task_with_output(None);
        task.parts = vec![task_part("part-1", Some("type-1"), None, 0)];

        let detail = task_detail_value(&task, &[task_type_input_requirements()]);

        assert_eq!(detail["id"], "task-1");
        assert_eq!(
            detail["input_requirements"][0]["requirements"]["key"],
            "seed_manifest_import"
        );
        assert_eq!(
            detail["input_requirements"][0]["requirements"]["input_schema"]["properties"]
                ["source_file"]["accept"],
            ".xlsx"
        );
    }

    #[test]
    fn parses_task_list_options() {
        let args = parse_tasks(&[
            "tasks", "list", "--skip", "10", "--limit", "25", "--lab-id", "lab-1",
        ]);
        match args.command {
            TasksCommand::List {
                skip,
                limit,
                lab_id,
            } => {
                assert_eq!(skip, 10);
                assert_eq!(limit, 25);
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected task list command"),
        }
    }

    #[test]
    fn parses_task_types_search_options() {
        let args = parse_tasks(&[
            "tasks",
            "types",
            "--search",
            "sample qc",
            "--skip",
            "10",
            "--limit",
            "25",
        ]);
        match args.command {
            TasksCommand::Types {
                skip,
                limit,
                search,
                category,
                lab_id,
            } => {
                assert_eq!(skip, 10);
                assert_eq!(limit, 25);
                assert_eq!(search.as_deref(), Some("sample qc"));
                assert!(category.is_none());
                assert!(lab_id.is_none());
            }
            _ => panic!("expected task types command"),
        }
    }

    #[test]
    fn parses_task_types_category_and_lab_options() {
        let args = parse_tasks(&[
            "tasks",
            "types",
            "--search",
            "ngs",
            "--category",
            "compute",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            TasksCommand::Types {
                skip,
                limit,
                search,
                category,
                lab_id,
            } => {
                assert_eq!(skip, 0);
                assert_eq!(limit, 20);
                assert_eq!(search.as_deref(), Some("ngs"));
                assert!(matches!(category, Some(TaskTypeCategoryArg::Compute)));
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected task types command"),
        }
    }

    #[test]
    fn parses_lab_task_type_detail_options() {
        let args = parse_tasks(&["tasks", "type", "type-1", "--lab-id", "lab-1"]);
        match args.command {
            TasksCommand::Type { id, lab_id } => {
                assert_eq!(id, "type-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected task type detail command"),
        }
    }

    #[test]
    fn parses_task_create_with_lab_id() {
        let args = parse_tasks(&["tasks", "create", "task.json", "--lab-id", "lab-1"]);
        match args.command {
            TasksCommand::Create {
                file,
                file_fields,
                lab_id,
            } => {
                assert_eq!(file, "task.json");
                assert!(file_fields.is_empty());
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected task create command"),
        }
    }

    #[test]
    fn parses_task_create_workflow_with_lab_id() {
        let args = parse_tasks(&[
            "tasks",
            "create-workflow",
            "workflow.json",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            TasksCommand::CreateWorkflow { file, lab_id } => {
                assert_eq!(file, "workflow.json");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected task create-workflow command"),
        }
    }

    #[test]
    fn parses_task_create_file_fields() {
        let args = parse_tasks(&[
            "tasks",
            "create",
            "task.json",
            "--file-field",
            "plasmid=plasmid.dna",
            "--file-field",
            "template=template.fa",
        ]);
        match args.command {
            TasksCommand::Create {
                file, file_fields, ..
            } => {
                assert_eq!(file, "task.json");
                assert_eq!(
                    file_fields,
                    vec![
                        "plasmid=plasmid.dna".to_string(),
                        "template=template.fa".to_string()
                    ]
                );
            }
            _ => panic!("expected task create command"),
        }
    }

    #[test]
    fn parses_task_workflow() {
        let args = parse_tasks(&["tasks", "workflow", "task-1"]);
        match args.command {
            TasksCommand::Workflow { id } => assert_eq!(id, "task-1"),
            _ => panic!("expected task workflow command"),
        }
    }

    #[test]
    fn parses_file_field_pairs() {
        let values = vec![r#"plasmid=C:\data\plasmid.dna"#.to_string()];
        let parsed = parse_file_fields(&values).expect("file fields should parse");
        assert_eq!(
            parsed,
            vec![("plasmid".to_string(), r#"C:\data\plasmid.dna"#.to_string())]
        );
    }

    #[test]
    fn parses_task_upload_field() {
        let args = parse_tasks(&[
            "tasks",
            "upload-field",
            "task-1",
            "plasmid.dna",
            "plasmid",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            TasksCommand::UploadField {
                id,
                file,
                field_key,
                lab_id,
            } => {
                assert_eq!(id, "task-1");
                assert_eq!(file, "plasmid.dna");
                assert_eq!(field_key, "plasmid");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected upload-field command"),
        }
    }

    #[test]
    fn parses_task_results_with_lab_id() {
        let args = parse_tasks(&["tasks", "results", "task-1", "--lab-id", "lab-1"]);
        match args.command {
            TasksCommand::Results { id, lab_id } => {
                assert_eq!(id, "task-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected task results command"),
        }
    }

    #[test]
    fn parses_task_update_inline_json() {
        let args = parse_tasks(&["tasks", "update", "task-1", r#"{"description":"x"}"#]);
        match args.command {
            TasksCommand::Update { id, data } => {
                assert_eq!(id, "task-1");
                assert_eq!(data, r#"{"description":"x"}"#);
            }
            _ => panic!("expected task update command"),
        }
    }

    #[test]
    fn parses_my_task_status() {
        let args = parse_tasks(&["tasks", "my", "status", "assignment-1", "in-progress"]);
        match args.command {
            TasksCommand::My {
                command:
                    MyTasksCommand::Status {
                        assignment_id,
                        status,
                    },
            } => {
                assert_eq!(assignment_id, "assignment-1");
                assert!(matches!(status, AssignmentStatusArg::InProgress));
            }
            _ => panic!("expected my task status command"),
        }
    }

    #[test]
    fn parses_my_submit_result() {
        let args = parse_tasks(&[
            "tasks",
            "my",
            "submit-result",
            "assignment-1",
            "result.json",
        ]);
        match args.command {
            TasksCommand::My {
                command:
                    MyTasksCommand::SubmitResult {
                        assignment_id,
                        file,
                        feedback,
                    },
            } => {
                assert_eq!(assignment_id, "assignment-1");
                assert_eq!(file, "result.json");
                assert!(feedback.is_none());
            }
            _ => panic!("expected submit result command"),
        }
    }

    #[test]
    fn wraps_bare_result_output_in_task_result_create() {
        let payload = normalize_result_payload(json!({ "required_output": "ready" }), None)
            .expect("bare output object should be wrapped");

        assert_eq!(
            payload,
            json!({ "output_data": { "required_output": "ready" } })
        );
    }

    #[test]
    fn preserves_existing_task_result_request_envelope() {
        let envelope = json!({
            "output_data": { "required_output": "ready" },
            "comment": "finished"
        });
        assert_eq!(
            normalize_result_payload(envelope.clone(), None)
                .expect("existing request envelope should remain valid"),
            envelope
        );

        let empty_envelope = json!({});
        assert_eq!(
            normalize_result_payload(empty_envelope.clone(), None)
                .expect("empty request envelope should remain valid"),
            empty_envelope
        );
    }

    #[test]
    fn wraps_null_result_and_keeps_feedback_at_request_root() {
        let payload = normalize_result_payload(
            serde_json::Value::Null,
            Some(json!({ "sop": "needs clarification" })),
        )
        .expect("null output data is allowed by TaskResultCreate");

        assert_eq!(payload["output_data"], serde_json::Value::Null);
        assert_eq!(
            payload["document_feedback"],
            json!({ "sop": "needs clarification" })
        );
    }

    #[test]
    fn rejects_non_object_non_null_result_data() {
        for invalid in [json!(["not an output object"]), json!(42), json!("done")] {
            let error = normalize_result_payload(invalid, None)
                .expect_err("arrays and scalar result data violate TaskResultCreate");
            assert!(error
                .to_string()
                .contains("must contain a JSON object or null"));
        }
    }

    #[test]
    fn parses_lab_task_part() {
        let args = parse_tasks(&["tasks", "part", "task-1", "part-1", "--lab-id", "lab-1"]);
        match args.command {
            TasksCommand::Part {
                task_id,
                part_id,
                lab_id,
            } => {
                assert_eq!(task_id, "task-1");
                assert_eq!(part_id, "part-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected lab task part command"),
        }
    }

    #[test]
    fn parses_my_list_filters() {
        let args = parse_tasks(&[
            "tasks",
            "my",
            "list",
            "--search",
            "sample qc",
            "--exclude-status",
            "completed",
        ]);
        match args.command {
            TasksCommand::My {
                command:
                    MyTasksCommand::List {
                        search,
                        exclude_status,
                        ..
                    },
            } => {
                assert_eq!(search.as_deref(), Some("sample qc"));
                assert!(matches!(
                    exclude_status,
                    Some(AssignmentStatusArg::Completed)
                ));
            }
            _ => panic!("expected filtered my tasks list"),
        }
    }

    #[test]
    fn parses_my_complete_and_upload_field() {
        let complete = parse_tasks(&[
            "tasks",
            "my",
            "complete",
            "assignment-1",
            "result.json",
            "--feedback",
            "feedback.json",
        ]);
        assert!(matches!(
            complete.command,
            TasksCommand::My {
                command: MyTasksCommand::Complete { .. }
            }
        ));

        let upload = parse_tasks(&[
            "tasks",
            "my",
            "upload-field",
            "task-1",
            "result.bin",
            "result_file",
            "--visibility",
            "staff-only",
        ]);
        match upload.command {
            TasksCommand::My {
                command: MyTasksCommand::UploadField { visibility, .. },
            } => assert!(matches!(visibility, TaskDocumentVisibilityArg::StaffOnly)),
            _ => panic!("expected my upload-field command"),
        }
    }

    #[test]
    fn parses_task_confirm() {
        let args = parse_tasks(&["tasks", "confirm", "task-1", "--lab-id", "lab-1"]);
        match args.command {
            TasksCommand::Confirm { id, lab_id } => {
                assert_eq!(id, "task-1");
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected task confirm command"),
        }
    }

    #[test]
    fn parses_task_reject_with_reason() {
        let args = parse_tasks(&[
            "tasks",
            "reject",
            "task-1",
            "--reason",
            "incomplete results",
            "--lab-id",
            "lab-1",
        ]);
        match args.command {
            TasksCommand::Reject { id, reason, lab_id } => {
                assert_eq!(id, "task-1");
                assert_eq!(reason.as_deref(), Some("incomplete results"));
                assert_eq!(lab_id.as_deref(), Some("lab-1"));
            }
            _ => panic!("expected task reject command"),
        }
    }

    #[test]
    fn parses_task_reject_without_reason() {
        let args = parse_tasks(&["tasks", "reject", "task-2"]);
        match args.command {
            TasksCommand::Reject { id, reason, lab_id } => {
                assert_eq!(id, "task-2");
                assert!(reason.is_none());
                assert!(lab_id.is_none());
            }
            _ => panic!("expected task reject command"),
        }
    }

    #[test]
    fn validates_workflow_payload() {
        let payload = json!({
            "title": "Example workflow",
            "parts": [
                { "client_key": "extract", "task_type_id": "type-a" },
                { "client_key": "analyze", "task_type_id": "type-b" }
            ],
            "dependencies": [
                {
                    "prerequisite_client_key": "extract",
                    "dependent_client_key": "analyze",
                    "condition_type": "completed"
                }
            ]
        });
        validate_task_create_payload(&payload).expect("task payload should validate");
        validate_task_workflow_payload(&payload).expect("workflow payload should validate");
    }

    #[test]
    fn rejects_workflow_dependency_with_unknown_part() {
        let payload = json!({
            "title": "Example workflow",
            "parts": [{ "client_key": "extract" }],
            "dependencies": [{
                "prerequisite_client_key": "extract",
                "dependent_client_key": "missing"
            }]
        });
        let err = validate_task_workflow_payload(&payload).expect_err("payload should fail");
        assert!(err.to_string().contains("unknown dependent client_key"));
    }

    #[test]
    fn rejects_workflow_dependency_cycle() {
        let payload = json!({
            "title": "Cyclic workflow",
            "parts": [
                { "client_key": "a" },
                { "client_key": "b" }
            ],
            "dependencies": [
                { "prerequisite_client_key": "a", "dependent_client_key": "b" },
                { "prerequisite_client_key": "b", "dependent_client_key": "a" }
            ]
        });
        let err = validate_task_workflow_payload(&payload).expect_err("payload should fail");
        assert!(err.to_string().contains("cycle"));
    }

    #[test]
    fn normalizes_legacy_single_stage_payload() {
        let mut payload = json!({
            "title": "Tm compute",
            "task_type_id": "type-1",
            "input_data": { "sequence": "ATGC" }
        });
        normalize_task_create_payload(&mut payload).expect("payload should normalize");
        assert_eq!(
            payload,
            json!({
                "title": "Tm compute",
                "input_data": { "sequence": "ATGC" },
                "parts": [{
                    "client_key": "part_1",
                    "name": "Tm compute",
                    "task_type_id": "type-1",
                    "input_data": { "sequence": "ATGC" }
                }]
            })
        );
    }

    #[test]
    fn normalizes_existing_parts_with_generated_client_keys() {
        let mut payload = json!({
            "title": "QC task",
            "parts": [
                { "name": "Stage A" },
                { "name": "Stage B", "client_key": "custom_b" }
            ]
        });
        normalize_task_create_payload(&mut payload).expect("payload should normalize");
        assert_eq!(payload["parts"][0]["client_key"], "part_1");
        assert_eq!(payload["parts"][1]["client_key"], "custom_b");
    }

    #[test]
    fn compute_category_routes_to_compute_results() {
        let task = task_with_output(None);
        assert!(is_compute_task_from_category_or_output(
            &task,
            Some("compute")
        ));
        assert!(is_compute_task_from_category_or_output(
            &task,
            Some("COMPUTE")
        ));
    }

    #[test]
    fn non_compute_category_routes_to_experiment_results_even_with_output_data() {
        let task = task_with_output(Some(json!({ "exit_code": 0 })));
        assert!(!is_compute_task_from_category_or_output(
            &task,
            Some("experiment")
        ));
    }

    #[test]
    fn missing_category_falls_back_to_non_empty_output_data() {
        let task = task_with_output(Some(json!({
            "exit_code": 0,
            "files": []
        })));
        assert!(is_compute_task_from_category_or_output(&task, None));

        let empty_task = task_with_output(Some(json!({})));
        assert!(!is_compute_task_from_category_or_output(&empty_task, None));
    }

    #[test]
    fn workflow_results_render_for_multi_part_workflow() {
        let task = task_with_output(None);
        let workflow = workflow_with_parts(vec![
            task_part("part-1", Some("type-1"), None, 1),
            task_part("part-2", Some("type-2"), None, 2),
        ]);
        assert!(should_render_workflow_results(&task, &workflow));
    }

    #[test]
    fn single_stage_workflow_keeps_legacy_results_view() {
        let task = task_with_output(None);
        let workflow = workflow_with_parts(vec![task_part("part-1", Some("type-1"), None, 1)]);
        assert!(!should_render_workflow_results(&task, &workflow));
    }

    #[test]
    fn task_without_root_task_type_uses_workflow_view() {
        let mut task = task_with_output(None);
        task.task_type_id = None;
        let workflow = workflow_with_parts(vec![task_part("part-1", Some("type-1"), None, 1)]);
        assert!(should_render_workflow_results(&task, &workflow));
    }
}
