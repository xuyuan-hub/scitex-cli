use crate::api_response::envelope_data;
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::services::path_segment_encode;

impl ScientexClient {
    pub async fn get_project_by_slug(
        &self,
        slug: &str,
    ) -> Result<serde_json::Value, ScientexError> {
        let resp: serde_json::Value = self.http.get(&project_by_slug_path(slug)).await?;
        Ok(envelope_data(resp))
    }
}

fn project_by_slug_path(slug: &str) -> String {
    format!("/projects/by-slug/{}", path_segment_encode(slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_project_lookup_path() {
        assert_eq!(
            project_by_slug_path("ta shan"),
            "/projects/by-slug/ta%20shan"
        );
    }
}
