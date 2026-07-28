use crate::api_response::{extract_object, extract_paginated, PaginatedList};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::services::{path_segment_encode, url_encode};
use crate::types::{PublicTool, ToolRun, ToolValidation};

impl ScientexClient {
    pub async fn search_public_tools(
        &self,
        skip: u32,
        limit: u32,
        search: Option<&str>,
        domain: Option<&str>,
        family: Option<&str>,
        tag: Option<&str>,
    ) -> Result<PaginatedList<PublicTool>, ScientexError> {
        let response: serde_json::Value = self
            .http
            .get(&public_tools_path(skip, limit, search, domain, family, tag))
            .await?;
        extract_paginated(response)
    }

    pub async fn get_public_tool(&self, key: &str) -> Result<PublicTool, ScientexError> {
        let response: serde_json::Value = self.http.get(&public_tool_path(key)).await?;
        extract_object(response)
    }

    pub async fn validate_public_tool_input(
        &self,
        key: &str,
        data: &serde_json::Value,
    ) -> Result<ToolValidation, ScientexError> {
        let response: serde_json::Value = self.http.post(&tool_validate_path(key), data).await?;
        extract_object(response)
    }

    pub async fn run_public_tool(
        &self,
        key: &str,
        data: &serde_json::Value,
        lab_id: Option<&str>,
    ) -> Result<ToolRun, ScientexError> {
        let path = tool_run_path(key);
        let response: serde_json::Value = if let Some(lab_id) = lab_id {
            self.http
                .post_with_headers(&path, data, &[("X-Current-Lab", lab_id)])
                .await?
        } else {
            self.http.post(&path, data).await?
        };
        extract_object(response)
    }
}

fn public_tools_path(
    skip: u32,
    limit: u32,
    search: Option<&str>,
    domain: Option<&str>,
    family: Option<&str>,
    tag: Option<&str>,
) -> String {
    let mut path = format!("/tool-catalog/tools?skip={skip}&limit={limit}");
    for (key, value) in [
        ("search", search),
        ("domain", domain),
        ("family", family),
        ("tag", tag),
    ] {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            path.push('&');
            path.push_str(key);
            path.push('=');
            path.push_str(&url_encode(value));
        }
    }
    path
}

fn public_tool_path(key: &str) -> String {
    format!("/tool-catalog/tools/{}", path_segment_encode(key))
}

fn tool_validate_path(key: &str) -> String {
    format!("{}/validate", public_tool_path(key))
}

fn tool_run_path(key: &str) -> String {
    format!("{}/run", public_tool_path(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_public_tool_paths() {
        assert_eq!(
            public_tools_path(
                10,
                50,
                Some("primer qc"),
                Some("bio informatics"),
                Some("qc"),
                Some("stable"),
            ),
            "/tool-catalog/tools?skip=10&limit=50&search=primer+qc&domain=bio+informatics&family=qc&tag=stable"
        );
        assert_eq!(
            public_tool_path("primer/qc"),
            "/tool-catalog/tools/primer%2Fqc"
        );
        assert_eq!(
            tool_validate_path("primer/qc"),
            "/tool-catalog/tools/primer%2Fqc/validate"
        );
        assert_eq!(
            tool_run_path("primer/qc"),
            "/tool-catalog/tools/primer%2Fqc/run"
        );
    }
}
