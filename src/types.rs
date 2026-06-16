use serde::{Deserialize, Deserializer, Serialize};

/// Accept a numeric value that may arrive as JSON number or JSON string.
fn string_or_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrF64 {
        F64(f64),
        String(String),
    }
    match StringOrF64::deserialize(deserializer)? {
        StringOrF64::F64(v) => Ok(v),
        StringOrF64::String(s) => s.parse::<f64>().map_err(Error::custom),
    }
}

/// Like `string_or_f64` but for `Option<f64>`.
fn opt_string_or_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrF64 {
        F64(f64),
        String(String),
    }
    let val = Option::<StringOrF64>::deserialize(deserializer)?;
    match val {
        None => Ok(None),
        Some(StringOrF64::F64(v)) => Ok(Some(v)),
        Some(StringOrF64::String(s)) => s.parse::<f64>().map(Some).map_err(Error::custom),
    }
}

// ============================================================
// User types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub full_name: String,
    pub email: String,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub lab_id: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub lab: Option<Lab>,
}

// ============================================================
// Order types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub status: String,
    pub supplier_name: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub total_price: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub items: Vec<OrderItem>,
    #[serde(default)]
    pub primer_items: Vec<PrimerItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub primer_name: String,
    pub sequence: String,
    #[serde(default)]
    pub base_count: Option<u32>,
    #[serde(default)]
    pub purification_method: Option<String>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub nmoles: Option<f64>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub scale_od: Option<f64>,
    #[serde(default)]
    pub tube_count: Option<u32>,
    #[serde(default)]
    pub five_modification: Option<String>,
    #[serde(default)]
    pub three_modification: Option<String>,
    // sequencing specific
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub seq_vector: Option<String>,
    #[serde(default)]
    pub universal: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimerItem {
    pub primer_name: String,
    pub sequence: String,
    #[serde(default)]
    pub scale_od: Option<String>,
    #[serde(default)]
    pub purification_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrimerOrder {
    pub r#type: String,
    pub supplier_name: String,
    pub items: Vec<OrderItem>,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub company_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub invoice_title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub principal_investigator: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub payment_method: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub recipient_address: String,
    #[serde(default)]
    pub weekend_delivery: bool,
    #[serde(default)]
    pub partial_delivery: bool,
    #[serde(default)]
    pub confidential: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSequencingOrder {
    pub r#type: String,
    pub supplier_name: String,
    pub items: Vec<OrderItem>,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub company_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub invoice_title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub principal_investigator: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub payment_method: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub recipient_address: String,
    #[serde(default)]
    pub weekend_delivery: bool,
    #[serde(default)]
    pub partial_delivery: bool,
    #[serde(default)]
    pub confidential: bool,
}

// ============================================================
// Template types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub order_type: Option<String>,
    #[serde(default)]
    pub is_default: Option<bool>,
    #[serde(default)]
    pub company_name: Option<String>,
    #[serde(default)]
    pub invoice_title: Option<String>,
    #[serde(default)]
    pub principal_investigator: Option<String>,
    #[serde(default)]
    pub payment_method: Option<String>,
    #[serde(default)]
    pub recipient_address: Option<String>,
    #[serde(default)]
    pub customer_name: Option<String>,
    #[serde(default)]
    pub customer_phone: Option<String>,
    #[serde(default)]
    pub customer_email: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

// ============================================================
// Inventory types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub id: String,
    #[serde(default)]
    pub stock_type: Option<String>,
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub source_item_id: Option<String>,
    #[serde(default)]
    pub order_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub primer_name: Option<String>,
    #[serde(default)]
    pub sequence: Option<String>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub total_quantity: Option<f64>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub remaining_quantity: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub unit_price: Option<String>,
    #[serde(default)]
    pub total_price: Option<String>,
    #[serde(default)]
    pub operator_id: Option<String>,
    #[serde(default)]
    pub storage_location_id: Option<String>,
    #[serde(default)]
    pub batch_label: Option<String>,
    #[serde(default)]
    pub location_path: Option<String>,
    #[serde(default)]
    pub item_usage_unit: Option<String>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub item_usage_quantity: Option<f64>,
    #[serde(default)]
    pub germplasm_id: Option<String>,
    #[serde(default)]
    pub plate_id: Option<String>,
    #[serde(default)]
    pub recorded_by: Option<String>,
    #[serde(default)]
    pub variety: Option<String>,
    #[serde(default)]
    pub generation: Option<String>,
    #[serde(default)]
    pub origin: Option<String>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub weight_g: Option<f64>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    #[serde(default)]
    pub stock_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    pub r#type: String,
    #[serde(deserialize_with = "string_or_f64")]
    pub quantity: f64,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub remaining_after: Option<f64>,
    #[serde(default)]
    pub unit_price: Option<String>,
    #[serde(default)]
    pub total_amount: Option<String>,
    #[serde(default)]
    pub recipient: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub experiment_ref: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub part_id: Option<String>,
    #[serde(default)]
    pub requirement_key: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub specification: Option<String>,
    #[serde(default)]
    pub supplier: Option<String>,
    #[serde(default)]
    pub catalog_number: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub usage_unit: Option<String>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub usage_unit_conversion: Option<f64>,
    #[serde(default)]
    pub storage_condition: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockOutResponse {
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub total_quantity: Option<f64>,
    #[serde(default)]
    pub total_amount: Option<String>,
    #[serde(default)]
    pub avg_unit_price: Option<String>,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockStats {
    pub total: u64,
    pub low_stock: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub full_path: Option<String>,
    #[serde(default)]
    pub children: Vec<Location>,
}

// ============================================================
// Lab types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lab {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub require_approval: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabMember {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: String,
    #[serde(default)]
    pub lab_name: String,
    pub invitee_email: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: String,
    pub invitee_email: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRule {
    pub id: String,
    #[serde(default)]
    pub order_type: Option<String>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub max_price: Option<f64>,
    pub approver_role: String,
    #[serde(default)]
    pub sort_order: Option<u32>,
}

// ============================================================
// Task scheduling types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub lab_id: String,
    pub title: String,
    pub status: String,
    pub created_by_id: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub input_data: Option<serde_json::Value>,
    #[serde(default)]
    pub output_data: Option<serde_json::Value>,
    #[serde(default)]
    pub source_type: Option<String>,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub task_type_id: Option<String>,
    #[serde(default)]
    pub parts: Vec<TaskPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: String,
    pub lab_id: String,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parts_summary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPart {
    pub id: String,
    pub task_id: String,
    pub name: String,
    pub status: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub task_type_id: Option<String>,
    #[serde(default)]
    pub input_data: Option<serde_json::Value>,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub output_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskType {
    pub id: String,
    pub key: String,
    pub display_name: String,
    pub enabled: bool,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub command_template: Option<Vec<String>>,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    #[serde(default)]
    pub documents: Vec<TaskTypeDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTypeDocument {
    pub id: String,
    pub document_type: String,
    pub filename: String,
    pub content_type: String,
    pub file_size: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffUserInfo {
    #[serde(default)]
    pub assignment_id: Option<String>,
    pub user_id: String,
    #[serde(default)]
    pub full_name: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDocument {
    pub id: String,
    pub task_id: String,
    pub document_type: String,
    pub visibility: String,
    pub filename: String,
    pub content_type: String,
    pub file_size: u64,
    pub created_at: String,
    #[serde(default)]
    pub part_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub id: String,
    pub task_id: String,
    pub part_id: String,
    pub submitted_by_id: String,
    pub created_at: String,
    #[serde(default)]
    pub assignment_id: Option<String>,
    #[serde(default)]
    pub output_data: Option<serde_json::Value>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub id: String,
    pub task_id: String,
    pub part_id: String,
    pub assignee_id: String,
    pub role: String,
    pub status: String,
    pub assigned_by_id: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDetail {
    pub task: Task,
    #[serde(default)]
    pub parts: Vec<TaskPart>,
    #[serde(default)]
    pub dependencies: Vec<serde_json::Value>,
    #[serde(default)]
    pub assignments: Vec<TaskAssignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffAssignmentBrief {
    pub id: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTypeBrief {
    pub id: String,
    pub display_name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffTaskBrief {
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "type")]
    pub task_type: Option<TaskTypeBrief>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffTaskDetail {
    pub id: String,
    pub lab_id: String,
    pub title: String,
    pub status: String,
    pub created_by_id: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "type")]
    pub task_type: Option<TaskTypeBrief>,
    #[serde(default)]
    pub input_data: Option<serde_json::Value>,
    #[serde(default)]
    pub output_data: Option<serde_json::Value>,
    #[serde(default)]
    pub source: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffPartBrief {
    pub id: String,
    pub name: String,
    pub status: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffPartDetail {
    pub id: String,
    pub task_id: String,
    pub name: String,
    pub status: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub input_data: Option<serde_json::Value>,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub output_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffAssignmentSummary {
    pub document_count: u64,
    pub has_latest_result: bool,
    #[serde(default)]
    pub latest_result_created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffDocumentBrief {
    pub id: String,
    pub filename: String,
    pub document_type: String,
    pub download_url: String,
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffAssignmentItem {
    pub assignment: StaffAssignmentBrief,
    pub task: StaffTaskBrief,
    pub part: StaffPartBrief,
    pub summary: StaffAssignmentSummary,
    #[serde(default)]
    pub documents: Vec<StaffDocumentBrief>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffAssignmentDetail {
    pub assignment: StaffAssignmentBrief,
    pub task: StaffTaskDetail,
    pub part: StaffPartDetail,
    #[serde(default)]
    pub documents: Vec<StaffDocumentBrief>,
    #[serde(default)]
    pub latest_result: Option<TaskResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Order deserialization ----

    #[test]
    fn test_order_numeric_fields_as_string() {
        let json = r#"{
            "id": "a04fd30b",
            "type": "primer_synthesis",
            "status": "ordered",
            "supplier_name": "sangon",
            "customer_name": "Test",
            "customer_phone": "13800000000",
            "customer_email": "test@test.com",
            "total_price": "86.00",
            "created_at": "2026-05-15T10:00:00Z",
            "items": [{
                "primer_name": "FWD",
                "sequence": "ATGG",
                "nmoles": "25.0",
                "scale_od": "10.00"
            }],
            "primer_items": []
        }"#;
        let order: Order =
            serde_json::from_str(json).expect("should parse order with string numerics");
        assert_eq!(order.id, "a04fd30b");
        assert_eq!(order.items[0].nmoles, Some(25.0));
        assert_eq!(order.items[0].scale_od, Some(10.0));
    }

    #[test]
    fn test_order_numeric_fields_as_number() {
        let json = r#"{
            "id": "a04fd30b",
            "type": "primer_synthesis",
            "status": "ordered",
            "supplier_name": "sangon",
            "customer_name": "Test",
            "customer_phone": "13800000000",
            "customer_email": "test@test.com",
            "total_price": "86.00",
            "created_at": "2026-05-15T10:00:00Z",
            "items": [{
                "primer_name": "FWD",
                "sequence": "ATGG",
                "nmoles": 25,
                "scale_od": 10.0
            }],
            "primer_items": []
        }"#;
        let order: Order =
            serde_json::from_str(json).expect("should parse order with numeric values");
        assert_eq!(order.items[0].nmoles, Some(25.0));
        assert_eq!(order.items[0].scale_od, Some(10.0));
    }

    // ---- Transaction deserialization ----

    #[test]
    fn test_transaction_quantity_as_string() {
        let json = r#"{
            "id": "t1",
            "type": "checkin",
            "quantity": "5.0",
            "purpose": "test",
            "created_at": "2026-05-15T10:00:00Z"
        }"#;
        let tx: Transaction =
            serde_json::from_str(json).expect("should parse transaction with string quantity");
        assert_eq!(tx.quantity, 5.0);
    }

    #[test]
    fn test_transaction_quantity_as_number() {
        let json = r#"{
            "id": "t1",
            "type": "checkin",
            "quantity": 5.0,
            "purpose": "test",
            "created_at": "2026-05-15T10:00:00Z"
        }"#;
        let tx: Transaction =
            serde_json::from_str(json).expect("should parse transaction with numeric quantity");
        assert_eq!(tx.quantity, 5.0);
    }

    // ---- Stock deserialization ----

    #[test]
    fn test_stock_remaining_quantity_as_string() {
        let json = r#"{
            "id": "s1",
            "primer_name": "FWD",
            "remaining_quantity": "3.5",
            "location_path": "Box-P2",
            "transactions": []
        }"#;
        let stock: Stock =
            serde_json::from_str(json).expect("should parse stock with string remaining_quantity");
        assert_eq!(stock.remaining_quantity, Some(3.5));
    }

    // ---- ApprovalRule deserialization ----

    #[test]
    fn test_approval_rule_max_price_as_string() {
        let json = r#"{
            "id": "r1",
            "order_type": "primer_synthesis",
            "max_price": "500.00",
            "approver_role": "finance"
        }"#;
        let rule: ApprovalRule =
            serde_json::from_str(json).expect("should parse rule with string max_price");
        assert_eq!(rule.max_price, Some(500.0));
    }

    #[test]
    fn task_type_ignores_internal_staff_bindings() {
        let json = r#"{
            "id": "tt1",
            "key": "test",
            "display_name": "Test",
            "enabled": true,
            "category": "staff",
            "created_at": "2026-06-05T00:00:00Z",
            "updated_at": "2026-06-05T00:00:00Z",
            "documents": [],
            "assigned_staff": [{
                "assignment_id": "a1",
                "user_id": "u1",
                "email": "staff@example.com",
                "full_name": "Staff"
            }]
        }"#;
        let task_type: TaskType =
            serde_json::from_str(json).expect("should ignore internal staff binding details");
        let value = serde_json::to_value(task_type).expect("should serialize task type");
        assert!(value.get("assigned_staff").is_none());
    }

    #[test]
    fn staff_user_info_parses_task_type_binding() {
        let value: StaffUserInfo = serde_json::from_value(serde_json::json!({
            "assignment_id": "assignment-1",
            "user_id": "user-1",
            "full_name": null,
            "email": "staff@example.com"
        }))
        .expect("staff binding should parse");
        assert_eq!(value.assignment_id.as_deref(), Some("assignment-1"));
        assert_eq!(value.user_id, "user-1");
        assert_eq!(value.full_name, None);
        assert_eq!(value.email, "staff@example.com");
    }

    #[test]
    fn staff_user_info_allows_missing_assignment_id() {
        let value: StaffUserInfo = serde_json::from_value(serde_json::json!({
            "user_id": "user-1",
            "full_name": "Staff",
            "email": "staff@example.com"
        }))
        .expect("staff binding without assignment_id should parse");
        assert_eq!(value.assignment_id, None);
        assert_eq!(value.user_id, "user-1");
    }
}
