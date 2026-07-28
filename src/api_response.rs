use serde::de::DeserializeOwned;

use crate::errors::ScientexError;

/// A paginated list response that preserves both the items array and the
/// backend-provided pagination metadata (count, total_pages, etc.).
#[derive(Debug, Clone, serde::Serialize)]
pub struct PaginatedList<T> {
    pub items: Vec<T>,
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_next: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_previous: Option<bool>,
}

impl<T: serde::de::DeserializeOwned> PaginatedList<T> {
    /// Build from a raw response that may be one of:
    /// - `{ data: [...], count: N, ... }` — flat array in `data`, pagination at top level
    /// - `{ data: { items: [...], count: N, ... } }` — nested in `data`
    /// - `{ items: [...], count: N, ... }` — no `data` wrapper
    /// - `[...]` — bare array, no pagination
    fn from_raw(resp: serde_json::Value) -> Result<Self, ScientexError> {
        // Case 1: bare array
        if resp.is_array() {
            let items: Vec<T> = serde_json::from_value(resp)
                .map_err(|e| ScientexError::ParseError(e.to_string()))?;
            let len = items.len() as u64;
            return Ok(PaginatedList {
                items,
                count: len,
                total_pages: None,
                current_page: None,
                has_next: None,
                has_previous: None,
            });
        }

        let obj = resp
            .as_object()
            .ok_or_else(|| ScientexError::ParseError("expected object or array".into()))?;

        // Extract pagination metadata from top level (before stripping data wrapper)
        let count = obj.get("count").and_then(|v| v.as_u64());
        let total_pages = obj.get("total_pages").and_then(|v| v.as_u64());
        let current_page = obj.get("current_page").and_then(|v| v.as_u64());
        let has_next = obj.get("has_next").and_then(|v| v.as_bool());
        let has_previous = obj.get("has_previous").and_then(|v| v.as_bool());

        // Case 2: has a `data` field
        if let Some(data) = obj.get("data") {
            if data.is_array() {
                // `{ data: [...], count: N, ... }` — flat array with top-level pagination
                let items: Vec<T> = serde_json::from_value(data.clone())
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?;
                let len = items.len() as u64;
                return Ok(PaginatedList {
                    items,
                    count: count.unwrap_or(len),
                    total_pages,
                    current_page,
                    has_next,
                    has_previous,
                });
            }
            if let Some(data_obj) = data.as_object() {
                // `{ data: { items: [...], count: N, ... } }` — nested
                let items_key = data_obj
                    .get("items")
                    .or_else(|| data_obj.get("results"))
                    .or_else(|| data_obj.get("records"))
                    .ok_or_else(|| {
                        ScientexError::ParseError(
                            "expected data object with items/results/records".into(),
                        )
                    })?;

                let items: Vec<T> = serde_json::from_value(items_key.clone())
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?;
                let len = items.len() as u64;

                // Prefer top-level pagination, fall back to nested pagination
                return Ok(PaginatedList {
                    items,
                    count: count
                        .or_else(|| data_obj.get("count").and_then(|v| v.as_u64()))
                        .unwrap_or(len),
                    total_pages: total_pages
                        .or_else(|| data_obj.get("total_pages").and_then(|v| v.as_u64())),
                    current_page: current_page
                        .or_else(|| data_obj.get("current_page").and_then(|v| v.as_u64())),
                    has_next: has_next
                        .or_else(|| data_obj.get("has_next").and_then(|v| v.as_bool())),
                    has_previous: has_previous
                        .or_else(|| data_obj.get("has_previous").and_then(|v| v.as_bool())),
                });
            }
        }

        // Case 3: no `data` wrapper — look for items/results/records at top level
        let items_key = obj
            .get("items")
            .or_else(|| obj.get("results"))
            .or_else(|| obj.get("records"))
            .ok_or_else(|| {
                ScientexError::ParseError(
                    "expected paginated object with data/items/results/records".into(),
                )
            })?;

        let items: Vec<T> = serde_json::from_value(items_key.clone())
            .map_err(|e| ScientexError::ParseError(e.to_string()))?;
        let len = items.len() as u64;

        Ok(PaginatedList {
            items,
            count: count.unwrap_or(len),
            total_pages,
            current_page,
            has_next,
            has_previous,
        })
    }
}

/// Extract a paginated list from a raw API response.
/// Preserves the backend `count` field so the CLI can show totals.
pub(crate) fn extract_paginated<T: DeserializeOwned>(
    resp: serde_json::Value,
) -> Result<PaginatedList<T>, ScientexError> {
    PaginatedList::from_raw(resp)
}

pub(crate) async fn parse_response<T: DeserializeOwned>(
    resp: reqwest::Response,
    path: &str,
) -> Result<T, ScientexError> {
    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(http_error_from_text(status, path, text));
    }
    resp.json::<T>().await.map_err(ScientexError::RequestError)
}

/// Preserve the complete backend error payload while giving text-mode callers a
/// concise detail string. Backends that still return a plain-text failure are
/// represented as a JSON string rather than being discarded.
pub(crate) fn http_error_from_text(status: u16, path: &str, text: String) -> ScientexError {
    let body = serde_json::from_str::<serde_json::Value>(&text)
        .unwrap_or_else(|_| serde_json::Value::String(text.clone()));
    let object = body.as_object();
    let detail_value = object.and_then(|value| value.get("detail"));
    let detail = detail_value
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .or_else(|| detail_value.map(serde_json::Value::to_string))
        .or_else(|| {
            object
                .and_then(|value| value.get("message"))
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
        })
        .unwrap_or_else(|| text.clone());
    let error_code = object
        .and_then(|value| value.get("error_code"))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    let fields = object.and_then(|value| value.get("fields")).cloned();

    ScientexError::HttpError {
        status,
        path: path.to_string(),
        detail,
        error_code,
        fields,
        body,
    }
}

pub(crate) fn extract_array<T: DeserializeOwned>(
    resp: serde_json::Value,
) -> Result<Vec<T>, ScientexError> {
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
                ScientexError::ParseError(
                    "expected array response or data/items/results/records array".to_string(),
                )
            })?
    };

    serde_json::from_value(array_value).map_err(|e| ScientexError::ParseError(e.to_string()))
}

pub(crate) fn extract_object<T: DeserializeOwned>(
    resp: serde_json::Value,
) -> Result<T, ScientexError> {
    serde_json::from_value(envelope_data(resp))
        .map_err(|e| ScientexError::ParseError(e.to_string()))
}

pub(crate) fn envelope_data(resp: serde_json::Value) -> serde_json::Value {
    resp.get("data").cloned().unwrap_or(resp)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Clone, Deserialize, PartialEq, serde::Serialize)]
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
        let result: Result<Vec<Item>, ScientexError> =
            extract_array(serde_json::json!({ "data": { "id": "a" } }));
        assert!(result.is_err());
    }

    #[test]
    fn preserves_structured_http_error_payload() {
        let error = http_error_from_text(
            422,
            "/example",
            r#"{"detail":"invalid row","error_code":"SED_006","fields":{"row":2}}"#.to_string(),
        );
        match error {
            ScientexError::HttpError {
                detail,
                error_code,
                fields,
                body,
                ..
            } => {
                assert_eq!(detail, "invalid row");
                assert_eq!(error_code.as_deref(), Some("SED_006"));
                assert_eq!(fields.expect("fields")["row"], 2);
                assert_eq!(body["error_code"], "SED_006");
            }
            _ => panic!("expected HTTP error"),
        }
    }

    // ---- Paginated extraction ----

    #[test]
    fn extracts_paginated_list_with_count() {
        // Backend nested format: { data: { items: [...] } }
        let resp = serde_json::json!({
            "data": {
                "items": [{ "id": "a" }, { "id": "b" }],
                "count": 111,
                "total_pages": 2,
                "current_page": 1,
                "has_next": true,
                "has_previous": false
            }
        });
        let list: PaginatedList<Item> =
            extract_paginated(resp).expect("paginated list should parse");
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].id, "a");
        assert_eq!(list.count, 111);
        assert_eq!(list.total_pages, Some(2));
        assert_eq!(list.current_page, Some(1));
        assert_eq!(list.has_next, Some(true));
        assert_eq!(list.has_previous, Some(false));
    }

    /// Backend flat array format: { data: [...], count: N, current_page: ... }
    #[test]
    fn extracts_flat_data_array_with_top_level_pagination() {
        let resp = serde_json::json!({
            "count": 111,
            "current_page": 1,
            "data": [{ "id": "a" }, { "id": "b" }],
            "has_next": true,
            "has_previous": false,
            "total_pages": 2
        });
        let list: PaginatedList<Item> =
            extract_paginated(resp).expect("flat data array should parse");
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].id, "a");
        assert_eq!(list.count, 111);
        assert_eq!(list.total_pages, Some(2));
        assert_eq!(list.current_page, Some(1));
        assert_eq!(list.has_next, Some(true));
        assert_eq!(list.has_previous, Some(false));
    }

    #[test]
    fn extracts_paginated_list_without_data_wrapper() {
        let resp = serde_json::json!({
            "items": [{ "id": "x" }],
            "count": 1,
            "total_pages": 1,
            "current_page": 1,
            "has_next": false,
            "has_previous": false
        });
        let list: PaginatedList<Item> =
            extract_paginated(resp).expect("should parse without wrapper");
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.count, 1);
    }

    #[test]
    fn flat_array_defaults_count_to_length() {
        let resp = serde_json::json!({ "data": [{ "id": "a" }, { "id": "b" }, { "id": "c" }] });
        let list: PaginatedList<Item> = extract_paginated(resp).expect("flat array should parse");
        assert_eq!(list.items.len(), 3);
        assert_eq!(list.count, 3);
        assert_eq!(list.total_pages, None);
        assert_eq!(list.has_next, None);
    }

    #[test]
    fn paginated_list_serializes_for_cli() {
        let list = PaginatedList {
            items: vec![Item { id: "a".into() }],
            count: 111,
            total_pages: Some(2),
            current_page: Some(1),
            has_next: Some(true),
            has_previous: Some(false),
        };
        let out = serde_json::to_string_pretty(&list).unwrap();
        assert!(out.contains("\"count\": 111"));
        assert!(out.contains("\"items\""));
    }

    // ── collect_all_pages tests ──────────────────────────────────

    fn make_page(items: Vec<Item>, count: u64, has_next: Option<bool>) -> PaginatedList<Item> {
        PaginatedList {
            items,
            count,
            total_pages: None,
            current_page: None,
            has_next,
            has_previous: None,
        }
    }

    #[tokio::test]
    async fn collect_all_single_page() {
        let list = crate::client::collect_all_pages(200, |_skip, _limit| {
            std::future::ready(Ok(make_page(
                vec![Item { id: "a".into() }, Item { id: "b".into() }],
                2,
                Some(false),
            )))
        })
        .await
        .expect("should succeed");
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.count, 2);
        assert_eq!(list.has_next, Some(false));
    }

    #[tokio::test]
    async fn collect_all_two_pages() {
        let mut call_count = 0u32;
        let list = crate::client::collect_all_pages(200, |_skip, _limit| {
            call_count += 1;
            let (items, has_next) = if call_count == 1 {
                (
                    vec![Item {
                        id: format!("page1-{}", call_count),
                    }],
                    Some(true),
                )
            } else {
                (
                    vec![Item {
                        id: format!("page2-{}", call_count),
                    }],
                    Some(false),
                )
            };
            std::future::ready(Ok(make_page(items, 2, has_next)))
        })
        .await
        .expect("should succeed");
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.count, 2);
        assert_eq!(call_count, 2);
    }

    #[tokio::test]
    async fn collect_all_empty() {
        let list = crate::client::collect_all_pages(200, |_skip, _limit| {
            std::future::ready(Ok(make_page(vec![], 0, Some(false))))
        })
        .await
        .expect("should succeed");
        assert!(list.items.is_empty());
        assert_eq!(list.count, 0);
    }

    #[tokio::test]
    async fn collect_all_safety_bound_stops_loop() {
        let mut calls: u32 = 0;
        let list = crate::client::collect_all_pages(1, |_skip, _limit| {
            calls += 1;
            std::future::ready(Ok(make_page(
                vec![Item {
                    id: format!("x-{}", calls),
                }],
                1,
                Some(true), // always claims more pages — safety bound must kick in
            )))
        })
        .await
        .expect("should succeed");
        // With max_limit=1 and 10_000 iterations, the safety bound should stop the loop.
        assert_eq!(calls, 10_000);
        assert_eq!(list.items.len(), 10_000);
    }
}
