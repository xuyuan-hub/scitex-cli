use crate::api_response::{extract_paginated, PaginatedList};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::types::PrimerRecord;

impl ScientexClient {
    pub async fn list_primers(
        &self,
        skip: u32,
        limit: u32,
    ) -> Result<PaginatedList<PrimerRecord>, ScientexError> {
        let resp: serde_json::Value = self.http.get(&primers_path(skip, limit)).await?;
        extract_paginated(resp)
    }
}

fn primers_path(skip: u32, limit: u32) -> String {
    format!("/primers/?skip={skip}&limit={limit}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_primers_path() {
        assert_eq!(primers_path(20, 50), "/primers/?skip=20&limit=50");
    }
}
