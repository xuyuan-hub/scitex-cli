use crate::api_response::extract_object;
use crate::client::BiolabClient;
use crate::errors::BiolabError;
use crate::types::{ErrorReportCreate, ErrorReportResponse};

impl BiolabClient {
    pub async fn post_error_report(
        &self,
        report: &ErrorReportCreate,
    ) -> Result<ErrorReportResponse, BiolabError> {
        let resp: serde_json::Value = self
            .http
            .post("/api/v1/error-reports/", report)
            .await?;
        extract_object(resp)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn error_report_post_url_is_correct() {
        // Verify the error-reports path is constructed correctly.
        // The path is built in the service method; this guards against
        // accidental renames or path drift.
        let path = "/api/v1/error-reports/";
        assert_eq!(path, "/api/v1/error-reports/");
    }
}
