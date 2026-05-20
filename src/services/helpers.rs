pub fn empty_body() -> serde_json::Value {
    serde_json::json!({})
}

pub fn single_field_body(field: &str, value: impl Into<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({ field: value.into() })
}

pub fn url_encode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
