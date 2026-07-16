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
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/openapi.json");
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
    let path = path.strip_prefix("/api/v1").unwrap_or(path);
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

fn openapi_operations(doc: &serde_json::Value) -> HashSet<(String, String)> {
    let methods = ["get", "post", "put", "patch", "delete"];
    let Some(paths) = doc.pointer("/paths").and_then(|v| v.as_object()) else {
        return HashSet::new();
    };
    paths
        .iter()
        .flat_map(|(path, item)| {
            let normalized = normalize_path(path);
            methods.iter().filter_map(move |method| {
                item.get(*method)
                    .map(|_| (method.to_ascii_uppercase(), normalized.clone()))
            })
        })
        .collect()
}

fn assert_operations_exist(doc: &serde_json::Value, expected: &[(&str, &str)]) {
    let operations = openapi_operations(doc);
    for (method, path) in expected {
        assert!(
            operations.contains(&((*method).to_string(), (*path).to_string())),
            "CLI endpoint `{method} {path}` not found in OpenAPI"
        );
    }
}

fn query_parameter_names(doc: &serde_json::Value, path: &str, method: &str) -> HashSet<String> {
    doc.pointer(&format!("/paths/{path}/{method}/parameters"))
        .and_then(|value| value.as_array())
        .map(|parameters| {
            parameters
                .iter()
                .filter(|parameter| {
                    parameter.get("in").and_then(|value| value.as_str()) == Some("query")
                })
                .filter_map(|parameter| {
                    parameter
                        .get("name")
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned)
                })
                .collect()
        })
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
    for value in ["PENDING", "IN_PROGRESS", "COMPLETED"] {
        assert!(
            backend.contains(value),
            "CLI assignment status `{value}` not in backend TaskAssignmentStatus {backend:?}"
        );
    }
}

#[test]
fn cli_task_type_category_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> = enum_values(&doc, "TaskTypeCategory").into_iter().collect();
    for value in ["COMPUTE", "STAFF"] {
        assert!(
            backend.contains(value),
            "CLI task type category `{value}` not in backend TaskTypeCategory {backend:?}"
        );
    }
}

#[test]
fn task_type_catalog_search_parameters_match_backend() {
    let doc = load_openapi();
    let parameters = query_parameter_names(&doc, "~1api~1v1~1task-types", "get");
    for expected in ["skip", "limit", "search", "filters"] {
        assert!(
            parameters.contains(expected),
            "task type catalog query parameter `{expected}` not found in OpenAPI: {parameters:?}"
        );
    }
}

#[test]
fn lab_task_type_search_parameters_match_backend() {
    let doc = load_openapi();
    let parameters = query_parameter_names(&doc, "~1api~1v1~1lab~1tasks~1task-types", "get");
    for expected in ["skip", "limit", "search", "category"] {
        assert!(
            parameters.contains(expected),
            "lab task type query parameter `{expected}` not found in OpenAPI: {parameters:?}"
        );
    }
    assert!(
        !parameters.contains("filters"),
        "lab task type query must not expose administrator catalog filters: {parameters:?}"
    );

    for path in [
        "~1api~1v1~1lab~1tasks~1task-types",
        "~1api~1v1~1lab~1tasks~1task-types~1{type_id}",
    ] {
        let has_current_lab_header = doc
            .pointer(&format!("/paths/{path}/get/parameters"))
            .and_then(|value| value.as_array())
            .is_some_and(|parameters| {
                parameters.iter().any(|parameter| {
                    parameter.get("in").and_then(|value| value.as_str()) == Some("header")
                        && parameter.get("name").and_then(|value| value.as_str())
                            == Some("X-Current-Lab")
                })
            });
        assert!(
            has_current_lab_header,
            "lab task type endpoint `{path}` must preserve X-Current-Lab scoping"
        );
    }
}

#[test]
fn lab_task_type_list_and_detail_responses_match_user_safe_contract() {
    let doc = load_openapi();
    let list_response = doc.pointer(
        "/paths/~1api~1v1~1lab~1tasks~1task-types/get/responses/200/content/application~1json/schema/$ref",
    );
    assert_eq!(
        list_response.and_then(|value| value.as_str()),
        Some("#/components/schemas/LabTaskTypeListResponse")
    );

    let detail_response = doc.pointer(
        "/paths/~1api~1v1~1lab~1tasks~1task-types~1{type_id}/get/responses/200/content/application~1json/schema/$ref",
    );
    assert_eq!(
        detail_response.and_then(|value| value.as_str()),
        Some("#/components/schemas/LabTaskTypeDetailResponse")
    );

    let list_properties = doc
        .pointer("/components/schemas/LabTaskTypeListItem/properties")
        .and_then(|value| value.as_object())
        .expect("LabTaskTypeListItem properties must exist");
    for expected in [
        "id",
        "key",
        "display_name",
        "category",
        "input_summary",
        "has_sop",
        "has_work_order",
        "is_assignable",
    ] {
        assert!(
            list_properties.contains_key(expected),
            "LabTaskTypeListItem is missing `{expected}`"
        );
    }
    for forbidden in [
        "input_schema",
        "output_schema",
        "documents",
        "command_template",
        "timeout_seconds",
        "queue",
        "assigned_staff",
        "enabled",
    ] {
        assert!(
            !list_properties.contains_key(forbidden),
            "LabTaskTypeListItem must remain lightweight and omit `{forbidden}`"
        );
    }

    let detail_properties = doc
        .pointer("/components/schemas/LabTaskTypeDetailResponse/properties")
        .and_then(|value| value.as_object())
        .expect("LabTaskTypeDetailResponse properties must exist");
    for expected in ["input_schema", "output_schema", "documents"] {
        assert!(
            detail_properties.contains_key(expected),
            "LabTaskTypeDetailResponse is missing `{expected}`"
        );
    }
}

#[test]
fn order_list_filter_parameters_match_backend() {
    let doc = load_openapi();
    let parameters = query_parameter_names(&doc, "~1api~1v1~1orders~1", "get");
    for expected in [
        "skip",
        "limit",
        "order_type",
        "supplier_name",
        "status",
        "price_min",
        "price_max",
        "date_from",
        "date_to",
    ] {
        assert!(
            parameters.contains(expected),
            "order list query parameter `{expected}` not found in OpenAPI: {parameters:?}"
        );
    }
}

#[test]
fn cli_error_category_values_match_backend() {
    let doc = load_openapi();
    let backend: HashSet<String> = enum_values(&doc, "ErrorCategory").into_iter().collect();
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
    let backend: HashSet<String> = enum_values(&doc, "OrderType").into_iter().collect();
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
    let backend: HashSet<String> = enum_values(&doc, "TaskDocumentType").into_iter().collect();
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
    let backend: HashSet<String> = enum_values(&doc, "TaskDocumentVisibility")
        .into_iter()
        .collect();
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
    let backend: HashSet<String> = enum_values(&doc, "TaskStatus").into_iter().collect();
    // These are values the CLI knows about and either displays or accepts.
    let known: HashSet<&str> = [
        "PENDING_ASSIGNMENT",
        "ASSIGNED",
        "IN_PROGRESS",
        "WAITING_LAB_CONFIRM",
        "COMPLETED",
        "FAILED",
        "CANCELLED",
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
    let backend: HashSet<String> = enum_values(&doc, "TaskPartStatus").into_iter().collect();
    let known: HashSet<&str> = [
        "LOCKED",
        "READY",
        "IN_PROGRESS",
        "COMPLETED",
        "FAILED",
        "CANCELLED",
        "PENDING",
        "BLOCKED",
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
    let backend: HashSet<String> = enum_values(&doc, "TaskAssignmentRole")
        .into_iter()
        .collect();
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
fn cli_implemented_endpoints_exist_with_methods() {
    let doc = load_openapi();
    let expected = [
        ("GET", "/users/me"),
        ("PATCH", "/users/me"),
        ("PATCH", "/users/me/password"),
        ("POST", "/feishu/cli-auth"),
        ("POST", "/feishu/cli-token"),
        ("GET", "/feishu/settings"),
        ("PATCH", "/feishu/settings"),
        ("GET", "/orders/"),
        ("GET", "/orders/{id}"),
        ("PATCH", "/orders/{id}"),
        ("POST", "/tasks/submit/primer-synthesis"),
        ("POST", "/tasks/submit/sanger-sequencing"),
        ("GET", "/inventory/stocks"),
        ("POST", "/inventory/stocks/batch"),
        ("POST", "/inventory/stocks/{id}/checkin"),
        ("POST", "/inventory/stocks/{id}/checkout"),
        ("POST", "/inventory/stocks/{id}/adjust"),
        ("POST", "/inventory/stocks/{id}/transfer"),
        ("GET", "/projects/by-slug/{id}"),
        ("GET", "/project/{id}/seed/object-types"),
        ("POST", "/project/{id}/seed/object-types"),
        ("GET", "/project/{id}/seed/object-types/{id}"),
        ("PATCH", "/project/{id}/seed/object-types/{id}"),
        ("GET", "/project/{id}/seed/intake-batches"),
        ("POST", "/project/{id}/seed/intake-batches"),
        ("GET", "/project/{id}/seed/intake-batches/{id}"),
        (
            "POST",
            "/project/{id}/seed/intake-batches/{id}/manifest-import-task",
        ),
        ("POST", "/project/{id}/seed/intake-batches/{id}/intake-task"),
        ("GET", "/project/{id}/seed/intake-records"),
        ("GET", "/project/{id}/seed/intake-records/public"),
        ("GET", "/project/{id}/seed/intake-records/{id}"),
        ("PATCH", "/project/{id}/seed/intake-records/{id}"),
        ("POST", "/project/{id}/seed/intake-records/{id}/complete"),
        ("GET", "/project/{id}/seed/stocks"),
        ("GET", "/project/{id}/seed/stocks/{id}"),
        ("GET", "/project/{id}/seed/field-catalog"),
    ];
    assert_operations_exist(&doc, &expected);
}

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
            paths
                .iter()
                .filter(|p| p.contains("order"))
                .collect::<Vec<_>>()
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
        "/lab/tasks/{id}/confirm",
        "/lab/tasks/{id}/reject",
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
fn cli_feishu_paths_exist_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    let expected = [
        "/feishu/settings",
        "/feishu/authorize",
        "/feishu/callback",
        "/feishu/login",
        "/feishu/status",
    ];
    for path in expected {
        assert!(
            paths.contains(path),
            "CLI feishu path `{path}` not found in OpenAPI"
        );
    }
}

#[test]
fn cli_task_type_feedback_path_exists_in_openapi() {
    let doc = load_openapi();
    let paths = openapi_paths(&doc);
    assert!(
        paths.contains("/task-types/{id}/feedback"),
        "CLI task-type feedback path `/task-types/{{id}}/feedback` not found in OpenAPI"
    );
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
        .args([
            "-sSf",
            "--connect-timeout",
            "10",
            "http://8.136.56.203/api/v1/openapi.json",
        ])
        .output()
        .expect("failed to run curl; is curl installed?");
    if !output.status.success() {
        panic!("curl failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/openapi.json");
    std::fs::write(&path, &output.stdout)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    eprintln!("Wrote {} bytes to {}", output.stdout.len(), path.display());
}
