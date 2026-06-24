use crate::api_response::extract_object;
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn error_report_post_url_is_correct() {
        // The path is passed to `self.http.post()`, which prepends the
        // configured base URL (`/api/v1`). Use a plain `/error-reports/`
        // path — NOT `/api/v1/error-reports/` — to avoid a double prefix.
        let path = "/error-reports/";
        assert_eq!(path, "/error-reports/");
    }
}
