//! OpenAPI contract tests.
//!
//! These tests ensure CLI enum values and API paths stay aligned with the
//! backend's OpenAPI specification. The fixture is a pinned snapshot of
//! `http://8.136.56.203/api/v1/openapi.json` at `tests/fixtures/openapi.json`.
//!
//! When the backend changes its API, update the fixture by running:
//!
//! ```bash
//! curl http://8.136.56.203/api/v1/openapi.json -o tests/fixtures/openapi.json
//! ```
//!
//! Then run `cargo test` — if a CLI enum value or path no longer matches the
//! backend, the relevant test will fail and point to the drift.

use std::collections::HashSet;

fn load_openapi() -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/openapi.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read OpenAPI fixture at {}: {e}", path.display()));
    serde_json::from_str(&content).expect("failed to parse OpenAPI fixture as JSON")
}

fn enum_values(doc: &serde_json::Value, name: &str) -> Vec<String> {
    doc.pointer(&format!("/components/schemas/{name}/enum"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_else(|| panic!("enum {name} not found in OpenAPI fixture"))
}

/// Normalize a path for comparison with OpenAPI path templates:
/// - strip query string
/// - strip `/api/v1` prefix (base URL already contains it)
/// - replace path parameters `{xxx}` with `{id}`
fn normalize_path(path: &str) -> String {
    let path = path.split('?').next().unwrap_or(path);
    let path = path
        .strip_prefix("/api/v1")
        .unwrap_or(path);
    let mut result = String::new();
    let mut in_brace = false;
    for ch in path.chars() {
        match ch {
            '{' => {
                in_brace = true;
                result.push('{');
            }
            '}' if in_brace => {
                in_brace = false;
                result.push_str("id}");
            }
            _ if in_brace => {}
            _ => result.push(ch),
        }
    }
    result
}

fn openapi_paths(doc: &serde_json::Value) -> HashSet<String> {
    doc.pointer("/paths")
        .and_then(|v| v.as_object())
        .map(|paths| paths.keys().map(|k| normalize_path(k)).collect())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Enum value tests
// ---------------------------------------------------------------------------

#[test]
fn cli_assignment_status_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> = enum_values(&doc, "TaskAssignmentStatus")
        .into_iter()
        .collect();
    // These are the values AssignmentStatusArg::as_str() can produce.
    for value in ["pending", "in_progress", "completed"] {
        assert!(
            backend.contains(value),
            "CLI assignment status `{value}` not in backend TaskAssignmentStatus {backend:?}"
        );
    }
}

#[test]
fn cli_task_type_category_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> =
        enum_values(&doc, "TaskTypeCategory").into_iter().collect();
    for value in ["compute", "staff"] {
        assert!(
            backend.contains(value),
            "CLI task type category `{value}` not in backend TaskTypeCategory {backend:?}"
        );
    }
}

#[test]
fn cli_error_category_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> =
        enum_values(&doc, "ErrorCategory").into_iter().collect();
    // These are the values ErrorCategory::Display produces, which is what
    // gets serialized into the JSON body via #[serde(rename_all = "snake_case")].
    for value in [
        "ui_display",
        "functional",
        "data",
        "performance",
        "permission",
        "other",
    ] {
        assert!(
            backend.contains(value),
            "CLI error category `{value}` not in backend ErrorCategory {backend:?}"
        );
    }
}

#[test]
fn cli_order_type_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> =
        enum_values(&doc, "OrderType").into_iter().collect();
    for value in ["primer_synthesis", "sequencing"] {
        assert!(
            backend.contains(value),
            "CLI order type `{value}` not in backend OrderType {backend:?}"
        );
    }
}

#[test]
fn cli_task_document_type_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> =
        enum_values(&doc, "TaskDocumentType").into_iter().collect();
    for value in ["sop", "work_order", "attachment", "result_attachment"] {
        assert!(
            backend.contains(value),
            "CLI task document type `{value}` not in backend TaskDocumentType {backend:?}"
        );
    }
}

#[test]
fn cli_task_document_visibility_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> =
        enum_values(&doc, "TaskDocumentVisibility").into_iter().collect();
    for value in ["lab_and_staff", "staff_only", "lab_only"] {
        assert!(
            backend.contains(value),
            "CLI task document visibility `{value}` not in backend TaskDocumentVisibility {backend:?}"
        );
    }
}

#[test]
fn cli_order_status_display_handles_all_backend_values() {
    // Every status the backend can return must be handled in output.rs
    // status_colored() so it gets the right terminal color.
    let doc = load_openapi();
    let backend = enum_values(&doc, "OrderStatus");
    let handled: HashSet<&str> = [
        "draft",
        "pending_approval",
        "approved",
        "pending",
        "ordered",
        "received",
        "stored",
    ]
    .into_iter()
    .collect();
    for value in &backend {
        assert!(
            handled.contains(value.as_str()),
            "backend OrderStatus `{value}` not handled in CLI status_colored()"
        );
    }
}

#[test]
fn cli_task_status_display_handles_all_backend_values() {
    // The CLI prints task status as a plain string. Verify every backend
    // value is something the CLI could legitimately display — i.e., it
    // exists in the TaskStatus enum.
    let doc = load_openapi();
    let backend: HashSet<String> =
        enum_values(&doc, "TaskStatus").into_iter().collect();
    // These are values the CLI knows about and either displays or accepts.
    let known: HashSet<&str> = [
        "pending_assignment",
        "assigned",
        "in_progress",
        "waiting_lab_confirm",
        "completed",
        "failed",
        "cancelled",
    ]
    .into_iter()
    .collect();
    for value in &backend {
        assert!(
            known.contains(value.as_str()),
            "backend TaskStatus `{value}` unknown to CLI — update output formatting"
        );
    }
}

#[test]
fn cli_task_part_status_display_handles_all_backend_values() {
    let doc = load_openapi();
    let backend: HashSet<String> =
        enum_values(&doc, "TaskPartStatus").into_iter().collect();
    let known: HashSet<&str> = [
        "LOCKED",
        "READY",
        "in_progress",
        "completed",
        "failed",
        "CANCELLED",
        "pending",
    ]
    .into_iter()
    .collect();
    for value in &backend {
        assert!(
            known.contains(value.as_str()),
            "backend TaskPartStatus `{value}` unknown to CLI — update output formatting"
        );
    }
}

#[test]
fn cli_task_assignment_role_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> =
        enum_values(&doc, "TaskAssignmentRole").into_iter().collect();
    for value in ["assignee", "reviewer", "helper"] {
        assert!(
            backend.contains(value),
            "CLI task assignment role `{value}` not in backend TaskAssignmentRole {backend:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Path existence tests
// ---------------------------------------------------------------------------

#[test]
fn cli_order_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/orders/",
        "/orders/{id}",
        "/orders/{id}/send",
        "/orders/{id}/download",
        "/orders/{id}/approve",
        "/orders/{id}/reject",
        "/orders/stats",
        "/orders/approvals/pending",
        "/orders/primer/template",
        "/orders/primer/upload-excel",
        "/orders/sequencing/template",
        "/orders/sequencing/upload-excel",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI order path `{path}` not found in OpenAPI. Available: {:?}",
            paths.iter().filter(|p| p.contains("order")).collect::<Vec<_>>()
        );
    }
}

#[test]
fn cli_template_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/order-info-templates/",
        "/order-info-templates/default",
        "/order-info-templates/{id}",
        "/order-info-templates/{id}/set-default",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI template path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_task_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/tasks",
        "/tasks/{id}",
        "/tasks/{id}/workflow",
        "/tasks/{id}/upload-field",
        "/tasks/{id}/documents",
        "/tasks/{id}/results",
        "/tasks/documents/{id}/download",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI task path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_lab_task_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/lab/tasks",
        "/lab/tasks/task-types",
        "/lab/tasks/{id}",
        "/lab/tasks/{id}/documents",
        "/lab/tasks/{id}/results",
        "/lab/tasks/{id}/upload-field",
        "/lab/tasks/documents/{id}/download",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI lab task path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_staff_task_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/staff/tasks/assignments",
        "/staff/tasks/assignments/{id}",
        "/staff/tasks/assignments/{id}/status",
        "/staff/tasks/assignments/{id}/results",
        "/staff/tasks/{id}/documents",
        "/staff/tasks/{id}/upload-field",
        "/staff/tasks/documents/{id}/download",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI staff task path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_task_type_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/task-types",
        "/task-types/{id}",
        "/task-types/{id}/staff",
        "/task-types/{id}/staff/{id}",
        "/task-types/{id}/documents",
        "/task-types/{id}/documents/{id}",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI task type path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_lab_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/lab",
        "/lab/create",
        "/lab/members",
        "/lab/members/{id}",
        "/lab/invite",
        "/lab/invitations",
        "/lab/invitations/{id}/accept",
        "/lab/invitations/{id}/decline",
        "/lab/applications",
        "/lab/applications/{id}/approve",
        "/lab/applications/{id}/reject",
        "/lab/approval-rules",
        "/lab/approval-rules/{id}",
        "/lab/join/{id}",
        "/lab/orders",
        "/lab/orders/stats",
        "/lab/inventory/stocks",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI lab path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_inventory_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/inventory/stocks",
        "/inventory/stocks/{id}",
        "/inventory/stocks/{id}/checkin",
        "/inventory/stocks/{id}/checkout",
        "/inventory/stocks/{id}/adjust",
        "/inventory/stocks/{id}/transfer",
        "/inventory/stocks/{id}/transactions",
        "/inventory/items",
        "/inventory/items/{id}",
        "/inventory/items/{id}/checkout",
        "/inventory/items/{id}/disable",
        "/inventory/summary",
        "/inventory/transactions",
        "/inventory/preferences",
        "/inventory/locations",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI inventory path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_user_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/users/me",
        "/users/me/password",
        "/users/",
        "/users/signup",
        "/users/staff",
        "/users/{id}",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI user path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_project_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/projects",
        "/projects/by-slug/{id}",
        "/projects/{id}",
        "/projects/{id}/members",
        "/projects/{id}/members/{id}",
        "/project/{id}/germplasm",
        "/project/{id}/germplasm/{id}",
        "/project/{id}/germplasm/{id}/sequencing-files",
        "/project/{id}/germplasm/{id}/stocks",
        "/project/{id}/planting",
        "/project/{id}/planting/{id}",
        "/project/{id}/planting/{id}/harvests",
        "/project/{id}/planting/{id}/items",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI project path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_error_report_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = ["/error-reports/"];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI error report path `{path}` not found in OpenAPI"
        );
    }
}

// ---------------------------------------------------------------------------
// Refresh helper
// ---------------------------------------------------------------------------

/// Run `cargo test refresh_openapi_fixture -- --ignored` to update the
/// fixture from the live backend. This test is ignored by default so it
/// only runs when explicitly requested.
#[test]
#[ignore]
fn refresh_openapi_fixture() {
    use std::process::Command;

    let output = Command::new("curl")
        .args(["-sSf", "--connect-timeout", "10",
               "http://8.136.56.203/api/v1/openapi.json"])
        .output()
        .expect("failed to run curl; is curl installed?");
    if !output.status.success() {
        panic!(
            "curl failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/openapi.json");
    std::fs::write(&path, &output.stdout)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    eprintln!(
        "Wrote {} bytes to {}",
        output.stdout.len(),
        path.display()
    );
}
