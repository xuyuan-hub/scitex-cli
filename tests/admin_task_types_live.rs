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
