use std::future::Future;
use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::api_response::PaginatedList;
use crate::config::Config;
use crate::http::ScientexHttp;

pub use crate::errors::ScientexError;

pub struct ScientexClient {
    pub(crate) http: ScientexHttp,
}

impl ScientexClient {
    pub fn new(config: Arc<Config>) -> Result<Self, ScientexError> {
        Ok(Self {
            http: ScientexHttp::new(config)?,
        })
    }

    pub fn with_token(config: Arc<Config>, token: &str) -> Result<Self, ScientexError> {
        config
            .save_token(token)
            .map_err(|e| ScientexError::ParseError(e.to_string()))?;
        Self::new(config)
    }
}

/// Auto-fetch all pages of an offset-paginated endpoint.
///
/// Keeps calling `fetch_page(skip, max_limit)` until `has_next` is not `Some(true)`
/// or the returned items slice is empty. A safety bound stops the loop after 10 000
/// iterations to guard against a misbehaving server.
pub(crate) async fn collect_all_pages<T, Fut>(
    max_limit: u32,
    mut fetch_page: impl FnMut(u32, u32) -> Fut,
) -> Result<PaginatedList<T>, ScientexError>
where
    T: DeserializeOwned,
    Fut: Future<Output = Result<PaginatedList<T>, ScientexError>> + Send,
{
    let mut skip: u32 = 0;
    let mut all_items: Vec<T> = Vec::new();
    let mut total_count: u64;
    let mut iterations: u32 = 0;

    loop {
        let page = fetch_page(skip, max_limit).await?;
        total_count = page.count;
        let page_len = page.items.len();
        all_items.extend(page.items);

        iterations += 1;

        // Stop when has_next is explicitly false or the returned page is not full.
        // We continue when has_next is Some(true) OR when the page is full
        // (the endpoint may not return has_next at all, e.g. seed stocks).
        let page_was_full = page_len as u32 >= max_limit;
        if page.has_next != Some(true) && !page_was_full {
            break;
        }
        if iterations >= 10_000 {
            break;
        }
        skip += max_limit;
    }

    Ok(PaginatedList {
        items: all_items,
        count: total_count,
        total_pages: None,
        current_page: None,
        has_next: Some(false),
        has_previous: Some(false),
    })
}
