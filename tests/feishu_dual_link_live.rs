//! Live smoke test for the Feishu cloud-document dual-association feature.
//!
//! This test exercises the real backend:
//!   1. Read current user's Feishu settings (skip if no docs_folder_token).
//!   2. Create a task type with a SOP markdown document.
//!   3. Verify the uploaded document has feishu_sync_status="synced" and a non-empty feishu_doc_url.
//!   4. Verify list_admin_task_type_documents returns the same feishu fields.
//!   5. Clean up: delete the task type.
//!
//! Gating:
//!   - `SCIENTEX_RUN_LIVE_FEISHU_TESTS=1` must be set.
//!   - The authenticated user must have Feishu settings configured (docs_folder_token).
//!   - The authenticated user must have task_manager / platform_admin permissions.
//!
//! Run with:
//!   SCIENTEX_RUN_LIVE_FEISHU_TESTS=1 cargo test --test feishu_dual_link_live -- --ignored --nocapture

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use scitex_cli::client::ScientexClient;
use scitex_cli::config::Config;
use serde_json::json;

fn live_enabled() -> bool {
    std::env::var("SCIENTEX_RUN_LIVE_FEISHU_TESTS").as_deref() == Ok("1")
}

fn unique_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis()
}

/// Write a temporary SOP markdown file and return its path.
/// The file lives in the OS temp directory and is removed when the returned
/// `_TempFile` guard is dropped.
struct TempFile {
    path: String,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn write_temp_sop_markdown(stamp: u128) -> TempFile {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("scitex_smoke_sop_{stamp}.md"));
    let content = format!(
        "# 冒烟测试 SOP {stamp}\n\n\
         > 由 scitex-cli 飞书双关联冒烟测试自动生成。\n\
         > 若同步成功，此 Markdown 应在飞书云文档目标文件夹内出现为 docx。\n\n\
         ## 步骤\n\n\
         1. 准备试剂\n\
         2. 执行操作\n\
         3. 记录结果\n"
    );
    std::fs::write(&path, content).expect("should write temp sop markdown");
    TempFile {
        path: path.to_string_lossy().into_owned(),
    }
}

#[tokio::test]
#[ignore = "requires SCIENTEX_RUN_LIVE_FEISHU_TESTS=1 and Feishu-configured user"]
async fn live_feishu_dual_link_task_type_sop() {
    if !live_enabled() {
        eprintln!("set SCIENTEX_RUN_LIVE_FEISHU_TESTS=1 to run the live Feishu dual-link smoke test");
        return;
    }

    let client = ScientexClient::new(Arc::new(Config::new()))
        .expect("live test requires an authenticated scitex CLI token");

    // 1. Verify Feishu settings are configured
    let feishu_settings = client
        .get_feishu_settings()
        .await
        .expect("should read feishu settings");

    let folder_token = match feishu_settings.docs_folder_token {
        Some(token) if !token.is_empty() => token,
        _ => {
            eprintln!(
                "skipping: current user has no feishu docs_folder_token configured. \
                 Run: scitex me feishu-settings update --docs-folder-token <token>"
            );
            return;
        }
    };
    eprintln!("feishu docs_folder_token: {folder_token}");
    eprintln!("feishu open_id: {:?}", feishu_settings.open_id);

    // 2. Create a task type with a SOP markdown document
    let stamp = unique_stamp();
    let key = format!("smoke_feishu_dual_link_{stamp}");
    let display_name = format!("[冒烟测试] 飞书双关联 {stamp}");
    let sop = write_temp_sop_markdown(stamp);

    let payload = json!({
        "key": key,
        "display_name": display_name,
        "description": "CLI 飞书双关联冒烟测试专用，测试完成后删除。",
        "category": "staff"
    });

    let created = client
        .create_admin_task_type(&payload, None)
        .await
        .expect("task type create should succeed");
    eprintln!("created task type: {}", created.id);

    let upload_result = client
        .upload_admin_task_type_document(&created.id, &sop.path, "sop", None)
        .await;

    // 3. Verify upload result
    let uploaded = upload_result.expect("SOP upload should succeed");
    eprintln!("uploaded document: {}", uploaded.id);
    eprintln!("  feishu_sync_status: {:?}", uploaded.feishu_sync_status);
    eprintln!("  feishu_doc_url: {:?}", uploaded.feishu_doc_url);
    eprintln!("  feishu_doc_token: {:?}", uploaded.feishu_doc_token);
    eprintln!("  scientex_link_url: {:?}", uploaded.scientex_link_url);

    assert_eq!(
        uploaded.feishu_sync_status.as_deref(),
        Some("synced"),
        "feishu_sync_status should be 'synced' after SOP upload"
    );
    assert!(
        uploaded.feishu_doc_url.as_deref().is_some_and(|u| !u.is_empty()),
        "feishu_doc_url should be populated after sync"
    );
    assert!(
        uploaded
            .feishu_doc_token
            .as_deref()
            .is_some_and(|t| !t.is_empty()),
        "feishu_doc_token should be populated after sync"
    );

    // 4. Verify list-docs also returns the feishu fields (BEFORE cleanup!)
    let docs = client
        .list_admin_task_type_documents(&created.id, None)
        .await
        .expect("list-docs should succeed before cleanup");
    let found = docs
        .iter()
        .find(|d| d.id == uploaded.id)
        .expect("uploaded document should appear in list");
    assert_eq!(
        found.feishu_doc_url, uploaded.feishu_doc_url,
        "list-docs should return the same feishu_doc_url as the upload response"
    );
    assert_eq!(
        found.feishu_sync_status, uploaded.feishu_sync_status,
        "list-docs should return the same feishu_sync_status"
    );

    // 5. Cleanup
    let delete_result = client.delete_admin_task_type(&created.id, None).await;
    assert!(
        delete_result.is_ok(),
        "task type {} was created but cleanup failed: {:?}",
        created.id,
        delete_result.err()
    );
    eprintln!("cleanup ok: deleted task type {}", created.id);
}

#[tokio::test]
#[ignore = "requires SCIENTEX_RUN_LIVE_FEISHU_TESTS=1 and Feishu-configured user"]
async fn live_feishu_settings_read_update_roundtrip() {
    if !live_enabled() {
        eprintln!("set SCIENTEX_RUN_LIVE_FEISHU_TESTS=1 to run the live Feishu settings roundtrip test");
        return;
    }

    let client = ScientexClient::new(Arc::new(Config::new()))
        .expect("live test requires an authenticated scitex CLI token");

    // Read current settings
    let before = client
        .get_feishu_settings()
        .await
        .expect("should read feishu settings");
    eprintln!("before: open_id={:?}, folder={:?}", before.open_id, before.docs_folder_token);

    // Roundtrip: PATCH with the same docs_folder_token (or skip if none)
    let folder = match &before.docs_folder_token {
        Some(token) if !token.is_empty() => token.clone(),
        _ => {
            eprintln!("skipping: no docs_folder_token configured");
            return;
        }
    };

    let update = scitex_cli::types::FeishuUserSettingsUpdate {
        docs_folder_token: Some(folder.clone()),
    };
    let after = client
        .update_feishu_settings(&update)
        .await
        .expect("PATCH /feishu/settings should succeed");

    assert_eq!(
        after.docs_folder_token.as_deref(),
        Some(folder.as_str()),
        "docs_folder_token should roundtrip unchanged"
    );
    // Token fields (access_token / refresh_token) must never be exposed
    // via the public settings response. We can't check them directly since
    // they aren't in the struct, but we verify the struct parses successfully.
    eprintln!("after: open_id={:?}, folder={:?}", after.open_id, after.docs_folder_token);
}
