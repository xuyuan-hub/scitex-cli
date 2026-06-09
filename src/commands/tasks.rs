use std::sync::Arc;

use clap::{Args, Subcommand, ValueEnum};

use crate::client::BiolabClient;
use crate::config::Config;
use crate::output::{print_paginated_items, print_pagination_metadata, print_result, OutputFormat};
use crate::types::{StaffAssignmentItem, Task, TaskResult, TaskType};

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
            if resolve_is_compute_task(&client, &task, lab_id.as_deref()).await {
                print_compute_results(&task, format);
            } else {
                let results = client.list_lab_task_results(id, lab_id.as_deref()).await?;
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

    let Some(output_data) = task
        .output_data
        .as_ref()
        .filter(|value| output_data_has_content(Some(value)))
    else {
        println!("No compute output yet. Task status: {}", task.status);
        return;
    };

    if let Some(exit_code) = output_data.get("exit_code") {
        println!("Exit code: {}", exit_code);
    }

    if output_data
        .get("exit_code")
        .and_then(|value| value.as_i64())
        .is_some_and(|code| code != 0)
    {
        println!(
            "Compute task failed with exit code {}",
            output_data["exit_code"]
        );
        if let Some(stderr) = output_data
            .get("stderr_log_url")
            .and_then(|value| value.as_str())
        {
            println!("stderr: {stderr}");
        }
    }

    if let Some(files) = output_data.get("files").and_then(|value| value.as_array()) {
        println!("\nFiles:");
        if files.is_empty() {
            println!("  No files");
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
            println!("  {filename}  {size} bytes  {relative_path}");
            if let Some(download_url) = file.get("download_url").and_then(|value| value.as_str()) {
                println!("    {download_url}");
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
        println!("\nLogs:");
        if let Some(stdout) = stdout {
            println!("  stdout: {stdout}");
        }
        if let Some(stderr) = stderr {
            println!("  stderr: {stderr}");
        }
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
}
