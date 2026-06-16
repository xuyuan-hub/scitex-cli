use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use biolab::client::BiolabClient;
use biolab::config::Config;
use serde_json::json;

#[tokio::test]
#[ignore = "requires a live backend account with admin task type permissions"]
async fn live_admin_task_type_create_delete() {
    if std::env::var("BIOLAB_RUN_LIVE_ADMIN_TESTS").as_deref() != Ok("1") {
        eprintln!("set BIOLAB_RUN_LIVE_ADMIN_TESTS=1 to run the live admin task type test");
        return;
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis();
    let key = format!("codex_e2e_task_type_{stamp}");
    let display_name = format!("Codex E2E Task Type {stamp}");
    let payload = json!({
        "key": key,
        "display_name": display_name,
        "description": "Temporary task type created by biolab-cli live e2e test.",
        "category": "staff",
        "input_schema": {
            "type": "object",
            "properties": {
                "sample_id": {
                    "type": "string",
                    "title": "Sample ID"
                }
            },
            "required": ["sample_id"]
        },
        "output_schema": {
            "type": "object",
            "properties": {
                "result": {
                    "type": "string",
                    "title": "Result"
                }
            }
        }
    });

    let client = BiolabClient::new(Arc::new(Config::new()))
        .expect("live test requires an authenticated biolab CLI token");
    let created = client
        .create_admin_task_type(&payload)
        .await
        .expect("admin task type create should succeed against live backend");

    let deleted = client.delete_admin_task_type(&created.id).await;

    assert_eq!(created.key, payload["key"].as_str().unwrap());
    assert_eq!(
        created.display_name,
        payload["display_name"].as_str().unwrap()
    );
    assert_eq!(created.category, "staff");
    assert!(
        deleted.is_ok(),
        "created task type {} but cleanup failed: {:?}",
        created.id,
        deleted.err()
    );
}

#[tokio::test]
#[ignore = "requires a live backend account with admin task type permissions"]
async fn live_admin_task_type_staff_bind_unbind() {
    if std::env::var("BIOLAB_RUN_LIVE_ADMIN_TESTS").as_deref() != Ok("1") {
        eprintln!("set BIOLAB_RUN_LIVE_ADMIN_TESTS=1 to run the live admin task type staff test");
        return;
    }

    let client = BiolabClient::new(Arc::new(Config::new()))
        .expect("live test requires an authenticated biolab CLI token");
    let staff_user_id = match std::env::var("BIOLAB_LIVE_STAFF_USER_ID") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            eprintln!("BIOLAB_LIVE_STAFF_USER_ID not set; using current authenticated user");
            client
                .get_me()
                .await
                .expect("should read current user for live staff binding test")
                .id
        }
    };

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis();
    let payload = json!({
        "key": format!("codex_e2e_staff_binding_{stamp}"),
        "display_name": format!("Codex E2E Staff Binding {stamp}"),
        "description": "Temporary task type for staff binding e2e test.",
        "category": "staff",
        "input_schema": {
            "type": "object",
            "properties": {
                "sample_id": {
                    "type": "string",
                    "title": "Sample ID"
                }
            },
            "required": ["sample_id"]
        }
    });

    let created = client
        .create_admin_task_type(&payload)
        .await
        .expect("admin task type create should succeed against live backend");

    let assign_result = client
        .assign_admin_task_type_staff(&created.id, &staff_user_id)
        .await;
    let staff_after_assign_result = if assign_result.is_ok() {
        client.list_admin_task_type_staff(&created.id).await
    } else {
        Ok(Vec::new())
    };
    let remove_result = if assign_result.is_ok() {
        client
            .remove_admin_task_type_staff(&created.id, &staff_user_id)
            .await
    } else {
        Ok(())
    };
    let staff_after_remove_result = if assign_result.is_ok() && remove_result.is_ok() {
        client.list_admin_task_type_staff(&created.id).await
    } else {
        Ok(Vec::new())
    };
    let delete_result = client.delete_admin_task_type(&created.id).await;

    assert!(
        assign_result.is_ok(),
        "created task type {} but staff assignment failed: {:?}",
        created.id,
        assign_result.err()
    );
    let staff_after_assign =
        staff_after_assign_result.expect("staff list should succeed after assignment");
    assert!(
        staff_after_assign
            .iter()
            .any(|staff| staff.user_id == staff_user_id),
        "assigned user {staff_user_id} should appear in staff list"
    );
    assert!(
        remove_result.is_ok(),
        "created task type {} and assigned staff but removal failed: {:?}",
        created.id,
        remove_result.err()
    );
    let staff_after_remove =
        staff_after_remove_result.expect("staff list should succeed after removal");
    assert!(
        !staff_after_remove
            .iter()
            .any(|staff| staff.user_id == staff_user_id),
        "removed user {staff_user_id} should not appear in staff list"
    );
    assert!(
        delete_result.is_ok(),
        "created task type {} but cleanup failed: {:?}",
        created.id,
        delete_result.err()
    );
}
