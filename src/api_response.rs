use serde::de::DeserializeOwned;

use crate::errors::BiolabError;

pub(crate) async fn parse_response<T: DeserializeOwned>(
    resp: reqwest::Response,
    path: &str,
) -> Result<T, BiolabError> {
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let detail = resp.text().await.unwrap_or_default();
        return Err(BiolabError::HttpError {
            status,
            path: path.to_string(),
            detail,
        });
    }
    resp.json::<T>().await.map_err(BiolabError::RequestError)
}

pub(crate) fn extract_array<T: DeserializeOwned>(
    resp: serde_json::Value,
) -> Result<Vec<T>, BiolabError> {
    let value = envelope_data(resp);
    let array_value = if value.is_array() {
        value
    } else {
        value
            .get("items")
            .or_else(|| value.get("results"))
            .or_else(|| value.get("records"))
            .cloned()
            .ok_or_else(|| {
                BiolabError::ParseError(
                    "expected array response or data/items/results/records array".to_string(),
                )
            })?
    };

    serde_json::from_value(array_value).map_err(|e| BiolabError::ParseError(e.to_string()))
}

pub(crate) fn extract_object<T: DeserializeOwned>(
    resp: serde_json::Value,
) -> Result<T, BiolabError> {
    serde_json::from_value(envelope_data(resp)).map_err(|e| BiolabError::ParseError(e.to_string()))
}

pub(crate) fn envelope_data(resp: serde_json::Value) -> serde_json::Value {
    resp.get("data").cloned().unwrap_or(resp)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Item {
        id: String,
    }

    #[test]
    fn extracts_direct_array() {
        let items: Vec<Item> =
            extract_array(serde_json::json!([{ "id": "a" }])).expect("array should parse");
        assert_eq!(items, vec![Item { id: "a".into() }]);
    }

    #[test]
    fn extracts_data_array() {
        let items: Vec<Item> =
            extract_array(serde_json::json!({ "data": [{ "id": "a" }] })).expect("data array");
        assert_eq!(items, vec![Item { id: "a".into() }]);
    }

    #[test]
    fn extracts_paginated_items() {
        let items: Vec<Item> =
            extract_array(serde_json::json!({ "data": { "items": [{ "id": "a" }] } }))
                .expect("items array");
        assert_eq!(items, vec![Item { id: "a".into() }]);
    }

    #[test]
    fn rejects_non_array_payload() {
        let result: Result<Vec<Item>, BiolabError> =
            extract_array(serde_json::json!({ "data": { "id": "a" } }));
        assert!(result.is_err());
    }
}
