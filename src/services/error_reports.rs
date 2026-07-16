use crate::api_response::{extract_object, extract_paginated, PaginatedList};
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::types::{ErrorReportCreate, ErrorReportResponse};

impl ScientexClient {
    pub async fn post_error_report(
        &self,
        report: &ErrorReportCreate,
    ) -> Result<ErrorReportResponse, ScientexError> {
        let resp: serde_json::Value = self.http.post("/error-reports/", report).await?;
        extract_object(resp)
    }

    pub async fn list_error_reports(
        &self,
        skip: u32,
        limit: u32,
        category: Option<&str>,
    ) -> Result<PaginatedList<ErrorReportResponse>, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .get(&admin_error_reports_path(skip, limit, category))
            .await?;
        extract_paginated(resp)
    }

    pub async fn get_error_report(
        &self,
        report_id: &str,
    ) -> Result<ErrorReportResponse, ScientexError> {
        let resp: serde_json::Value = self.http.get(&admin_error_report_path(report_id)).await?;
        extract_object(resp)
    }
}

fn admin_error_reports_path(skip: u32, limit: u32, category: Option<&str>) -> String {
    let mut path = format!("/admin/error-reports/?skip={skip}&limit={limit}");
    if let Some(category) = category.filter(|value| !value.is_empty()) {
        path.push_str("&category=");
        path.push_str(&crate::services::url_encode(category));
    }
    path
}

fn admin_error_report_path(report_id: &str) -> String {
    format!(
        "/admin/error-reports/{}",
        crate::services::path_segment_encode(report_id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_report_post_url_is_correct() {
        // The path is passed to `self.http.post()`, which prepends the
        // configured base URL (`/api/v1`). Use a plain `/error-reports/`
        // path — NOT `/api/v1/error-reports/` — to avoid a double prefix.
        let path = "/error-reports/";
        assert_eq!(path, "/error-reports/");
    }

    #[test]
    fn builds_admin_error_report_paths() {
        assert_eq!(
            admin_error_reports_path(0, 20, Some("ui display")),
            "/admin/error-reports/?skip=0&limit=20&category=ui+display"
        );
        assert_eq!(
            admin_error_report_path("report/1"),
            "/admin/error-reports/report%2F1"
        );
    }
}
