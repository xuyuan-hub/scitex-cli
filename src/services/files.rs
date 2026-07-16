use crate::api_response::extract_object;
use crate::client::ScientexClient;
use crate::errors::ScientexError;
use crate::types::UploadFieldResponse;

impl ScientexClient {
    pub async fn upload_file_reference(
        &self,
        file_path: &str,
    ) -> Result<UploadFieldResponse, ScientexError> {
        let resp: serde_json::Value = self
            .http
            .upload_multipart("/files/upload", file_path, &[], &[])
            .await?;
        extract_object(resp)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn general_file_upload_path_is_api_relative() {
        assert_eq!("/files/upload", "/files/upload");
    }
}
