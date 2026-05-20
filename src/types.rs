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
}

// ============================================================
// Inventory types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub id: String,
    #[serde(default)]
    pub primer_name: Option<String>,
    #[serde(default, deserialize_with = "opt_string_or_f64")]
    pub remaining_quantity: Option<f64>,
    #[serde(default)]
    pub location_path: Option<String>,
    #[serde(default)]
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub r#type: String,
    #[serde(deserialize_with = "string_or_f64")]
    pub quantity: f64,
    pub purpose: String,
    #[serde(default)]
    pub experiment_ref: Option<String>,
    pub created_at: String,
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
    pub path: Option<String>,
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
        let order: Order = serde_json::from_str(json).expect("should parse order with string numerics");
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
        let order: Order = serde_json::from_str(json).expect("should parse order with numeric values");
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
        let tx: Transaction = serde_json::from_str(json).expect("should parse transaction with string quantity");
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
        let tx: Transaction = serde_json::from_str(json).expect("should parse transaction with numeric quantity");
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
        let stock: Stock = serde_json::from_str(json).expect("should parse stock with string remaining_quantity");
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
        let rule: ApprovalRule = serde_json::from_str(json).expect("should parse rule with string max_price");
        assert_eq!(rule.max_price, Some(500.0));
    }
}
