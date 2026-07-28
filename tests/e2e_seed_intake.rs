//! End-to-end test for the seed intake flow.
//!
//! Walks the minimum happy path through the project-seed domain:
//!   1. Resolve a project by slug
//!   2. Create an intake batch
//!   3. List intake records in that batch (verifies read-after-write)
//!
//! All tests are `#[ignore]`d because they require a live backend with an
//! authenticated token and a real project slug.
//!
//! To run:
//! ```bash
//! SCIENTEX_RUN_E2E_TESTS=1 \
//! SCITEX_E2E_PROJECT_SLUG=<your-project-slug> \
//! SCITEX_E2E_OBJECT_TYPE_CONFIG_ID=<draft-compatible-config-uuid> \
//! cargo test --test e2e_seed_intake -- --ignored
//! ```

use std::sync::Arc;

use scitex_cli::client::ScientexClient;
use scitex_cli::config::Config;
use serde_json::json;

fn e2e_client() -> ScientexClient {
    ScientexClient::new(Arc::new(Config::new()))
        .expect("e2e test requires an authenticated scitex CLI token (SCIENTEX_TOKEN or keyring)")
}

fn require_e2e_env() -> String {
    if std::env::var("SCIENTEX_RUN_E2E_TESTS").as_deref() != Ok("1") {
        panic!("set SCIENTEX_RUN_E2E_TESTS=1 to run this test");
    }
    std::env::var("SCITEX_E2E_PROJECT_SLUG")
        .expect("set SCITEX_E2E_PROJECT_SLUG=<slug> to run this test")
}

fn require_object_type_config_id() -> String {
    std::env::var("SCITEX_E2E_OBJECT_TYPE_CONFIG_ID").expect(
        "set SCITEX_E2E_OBJECT_TYPE_CONFIG_ID to an object type config visible in the test project",
    )
}

#[tokio::test]
#[ignore = "requires live backend + SCIENTEX_RUN_E2E_TESTS=1 + SCITEX_E2E_PROJECT_SLUG"]
async fn e2e_seed_intake_create_and_list() {
    let slug = require_e2e_env();
    let object_type_config_id = require_object_type_config_id();
    let client = e2e_client();

    // 1. Resolve project by slug to confirm the project exists and auth works
    let project = client
        .get_project_by_slug(&slug)
        .await
        .expect("get_project_by_slug should succeed");
    let project_id = project["id"]
        .as_str()
        .expect("project should have an 'id' field")
        .to_string();
    eprintln!("project: {project_id} ({slug})");
    let _ = project_id;

    // 2. Create an intake batch
    let batch = client
        .create_seed_intake_batch(
            &slug,
            &json!({ "object_type_config_id": object_type_config_id }),
        )
        .await
        .expect("create_seed_intake_batch should succeed");
    let batch_id = batch.id.clone();
    eprintln!("created batch: {batch_id}");

    // 3. Fetch the batch back to confirm read-after-write
    let fetched = client
        .get_seed_intake_batch(&slug, &batch_id)
        .await
        .expect("get_seed_intake_batch should succeed");
    assert_eq!(
        fetched.id, batch_id,
        "fetched batch id should match created batch id"
    );
    eprintln!("fetched batch: {} — e2e flow passed", fetched.id);

    // 4. List records in the new batch (should be empty but must not error)
    let records = client
        .list_seed_intake_records(&slug, Some(&batch_id), None, 0, 10)
        .await
        .expect("list_seed_intake_records should succeed");
    eprintln!("records in batch: {}", records.count);
}
