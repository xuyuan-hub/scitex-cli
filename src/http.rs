use std::sync::Arc;

use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::api_response::parse_response;
use crate::client::BiolabError;
use crate::config::Config;

pub(crate) struct BiolabHttp {
    client: Client,
    config: Arc<Config>,
}

impl BiolabHttp {
    pub(crate) fn new(config: Arc<Config>) -> Result<Self, BiolabError> {
        let token = config.load_token().ok_or(BiolabError::NotAuthenticated)?;
        let client = Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                        .map_err(|e| BiolabError::ParseError(e.to_string()))?,
                );
                h.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json"),
                );
                h
            })
            .build()
            .map_err(BiolabError::RequestError)?;
        Ok(Self { client, config })
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, BiolabError> {
        let resp = self
            .client
            .get(self.url(path))
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, BiolabError> {
        let resp = self
            .client
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn patch<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, BiolabError> {
        let resp = self
            .client
            .patch(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, BiolabError> {
        let resp = self
            .client
            .put(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, BiolabError> {
        let resp = self
            .client
            .delete(self.url(path))
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn download_bytes(&self, path: &str) -> Result<Vec<u8>, BiolabError> {
        let resp = self
            .client
            .get(self.url(path))
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let detail = resp.text().await.unwrap_or_default();
            return Err(BiolabError::HttpError {
                status,
                path: path.into(),
                detail,
            });
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(BiolabError::RequestError)
    }

    pub(crate) async fn upload_file(
        &self,
        path: &str,
        file_path: &str,
    ) -> Result<serde_json::Value, BiolabError> {
        use std::path::Path as StdPath;

        let fname = StdPath::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let content = std::fs::read(file_path)
            .map_err(|e| BiolabError::ParseError(format!("Cannot read file {file_path}: {e}")))?;

        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"file\"; filename=\"{fname}\"\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(
            b"Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet\r\n\r\n",
        );
        body.extend_from_slice(&content);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let resp = self
            .client
            .post(self.url(path))
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url, path)
    }
}
