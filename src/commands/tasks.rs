use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use crate::client::BiolabClient;
use crate::config::Config;
use crate::errors::BiolabError;
use crate::output::{print_paginated_items, print_pagination_metadata, print_result, OutputFormat};
use crate::types::{StaffAssignmentItem, Task, TaskPart, TaskResult, TaskType, WorkflowDetail};

#[derive(Args)]
pub struct TasksArgs {
    #[command(subcommand)]
    pub command: TasksCommand,
}

#[derive(Subcommand)]
pub enum TasksCommand {
    /// List task types available to the current lab.
    Types {
        #[arg(short, long, default_value_t = 0)]
        skip: u32,
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Create a task from a JSON file.
    Create {
        file: String,
        /// Attach input files as field=path entries for multipart task creation.
        #[arg(long = "file-field")]
        file_fields: Vec<String>,
        #[arg(long)]
        lab_id: Option<String>,
    },
    /// Create a workflow task from a JSON file.
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
    /// Show workflow detail for a task.
    Workflow { id: String },
    /// Update a task from an inline JSON object.
    Update { id: String, data: String },
    /// Update a task from a JSON file.
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
    /// My assigned staff tasks.
    My {
        #[command(subcommand)]
        command: MyTasksCommand,
    },
}

#[derive(Subcommand)]
pub enum MyTasksCommand {
    /// List assignments assigned to me.
    List {
        #[arg(short, long, default_value_t = 0)]
        skip: u32,
        #[arg(short, long, default_value_t = 100)]
        limit: u32,
    },
    /// Show one assignment assigned to me.
    Get { assignment_id: String },
    /// Update my assignment status.
    Status {
        assignment_id: String,
        status: AssignmentStatusArg,
    },
    /// Submit a result for my assignment from a JSON file.
    SubmitResult { assignment_id: String, file: String },
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
    Blocked,
}

impl AssignmentStatusArg {
    fn as_str(&self) -> &'static str {
        match self {
            AssignmentStatusArg::Pending => "pending",
            AssignmentStatusArg::InProgress => "in_progress",
            AssignmentStatusArg::Completed => "completed",
            AssignmentStatusArg::Blocked => "blocked",
        }
    }
}

pub async fn run(
    args: &TasksArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = BiolabClient::new(Arc::clone(config))?;

    match &args.command {
        TasksCommand::Types {
            skip,
            limit,
            search,
            lab_id,
        } => {
            let should_search_task_types = search.as_deref().is_some_and(|value| !value.is_empty())
                || *skip != 0
                || *limit != 100;
            let types = if should_search_task_types {
                client
                    .search_task_types(*skip, *limit, search.as_deref())
                    .await?
            } else {
                client.list_lab_task_types(lab_id.as_deref()).await?
            };
            match format {
                OutputFormat::Json => print_result(&types, format),
                OutputFormat::Text => print_task_types(&types),
            }
        }
        TasksCommand::Create {
            file,
            file_fields,
            lab_id,
        } => {
            let mut data = read_json_file(file)?;
            prepare_lab_task_payload(&mut data)?;
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
            prepare_lab_task_payload(&mut data)?;
            validate_lab_workflow_payload(&data)?;
            let task = client
                .create_lab_workflow_task(&data, lab_id.as_deref())
                .await?;
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
            print_result(&task, format);
        }
        TasksCommand::Workflow { id } => {
            let workflow = client.get_task_workflow(id).await?;
            match format {
                OutputFormat::Json => print_result(&workflow, format),
                OutputFormat::Text => print_workflow_detail(&workflow),
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
                OutputFormat::Text => print_paginated_items(&documents),
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
        TasksCommand::My { command } => run_my_tasks(&client, command, format).await?,
    }

    Ok(())
}

async fn run_my_tasks(
    client: &BiolabClient,
    command: &MyTasksCommand,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    match command {
        MyTasksCommand::List { skip, limit } => {
            let assignments = client.list_my_task_assignments(*skip, *limit).await?;
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
        } => {
            let data = read_json_file(file)?;
            let result = client.submit_my_task_result(assignment_id, &data).await?;
            print_result(&result, format);
        }
        MyTasksCommand::Documents { task_id } => {
            let documents = client.list_my_task_documents(task_id).await?;
            match format {
                OutputFormat::Json => print_result(&documents, format),
                OutputFormat::Text => print_paginated_items(&documents),
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

fn prepare_lab_task_payload(data: &mut serde_json::Value) -> anyhow::Result<()> {
    let obj = data
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Task payload must be a JSON object"))?;
    obj.remove("lab_id");
    Ok(())
}

fn validate_lab_workflow_payload(data: &serde_json::Value) -> anyhow::Result<()> {
    let obj = data
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Workflow payload must be a JSON object"))?;

    obj.get("title")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Workflow payload requires a non-empty `title`"))?;

    let parts = obj
        .get("parts")
        .and_then(|value| value.as_array())
        .ok_or_else(|| anyhow::anyhow!("Workflow payload requires a `parts` array"))?;
    if parts.is_empty() {
        anyhow::bail!("Workflow payload must contain at least one part");
    }

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
        }
    }

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
    std::fs::write(&output, bytes)?;
    println!("Downloaded to {output}");
    Ok(())
}

async fn get_task_workflow_if_available(
    client: &BiolabClient,
    task_id: &str,
) -> anyhow::Result<Option<WorkflowDetail>> {
    match client.get_task_workflow(task_id).await {
        Ok(workflow) => Ok(Some(workflow)),
        Err(BiolabError::HttpError { status: 404, .. }) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

async fn resolve_is_compute_task(client: &BiolabClient, task: &Task, lab_id: Option<&str>) -> bool {
    if let Some(task_type_id) = task.task_type_id.as_deref() {
        if let Ok(types) = client.list_lab_task_types(lab_id).await {
            if let Some(task_type) = types.items.iter().find(|item| item.id == task_type_id) {
                return is_compute_task_from_category_or_output(
                    task,
                    Some(task_type.category.as_str()),
                );
            }
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
    client: &BiolabClient,
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
    client: &BiolabClient,
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
        let category = resolve_part_category(client, &part, &mut task_type_categories).await?;
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
    client: &BiolabClient,
    lab_id: Option<&str>,
) -> HashMap<String, String> {
    client
        .list_lab_task_types(lab_id)
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
    client: &BiolabClient,
    part: &TaskPart,
    categories: &mut HashMap<String, String>,
) -> anyhow::Result<String> {
    if let Some(task_type_id) = part.task_type_id.as_deref() {
        if let Some(category) = categories.get(task_type_id) {
            return Ok(category.clone());
        }

        let task_type = client.get_task_type(task_type_id).await.with_context(|| {
            format!(
                "Failed to load task type `{task_type_id}` for workflow part `{}`",
                part.id
            )
        })?;
        categories.insert(task_type.id.clone(), task_type.category.clone());
        return Ok(task_type.category);
    }

    if output_data_has_content(part.output_data.as_ref()) {
        Ok("compute".to_string())
    } else {
        Ok("staff".to_string())
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

fn print_task_types(list: &crate::api_response::PaginatedList<TaskType>) {
    print_pagination_metadata(list);
    if list.items.is_empty() {
        println!("No task types");
        return;
    }
    for item in &list.items {
        println!(
            "{}  {:8}  {:7}  {}",
            item.id, item.category, item.enabled, item.display_name
        );
    }
}

fn print_tasks(list: &crate::api_response::PaginatedList<Task>) {
    print_pagination_metadata(list);
    if list.items.is_empty() {
        println!("No tasks");
        return;
    }
    for task in &list.items {
        println!(
            "{}  {:20}  {:24}  {}",
            task.id,
            task.status,
            task.task_type_id.as_deref().unwrap_or("-"),
            task.title
        );
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
        let cli = TestCli::try_parse_from(std::iter::once("biolab").chain(args.iter().copied()))
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
                lab_id,
            } => {
                assert_eq!(skip, 10);
                assert_eq!(limit, 25);
                assert_eq!(search.as_deref(), Some("sample qc"));
                assert!(lab_id.is_none());
            }
            _ => panic!("expected task types command"),
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
                    },
            } => {
                assert_eq!(assignment_id, "assignment-1");
                assert_eq!(file, "result.json");
            }
            _ => panic!("expected submit result command"),
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
        validate_lab_workflow_payload(&payload).expect("workflow payload should validate");
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
        let err = validate_lab_workflow_payload(&payload).expect_err("payload should fail");
        assert!(err.to_string().contains("unknown dependent client_key"));
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
