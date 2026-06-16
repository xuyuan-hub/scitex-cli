use std::{path::Path as StdPath, sync::Arc, time::Duration};

use reqwest::header::{HeaderName, HeaderValue};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::de::DeserializeOwned;
use url::Url;

use crate::api_response::parse_response;
use crate::config::Config;
use crate::errors::BiolabError;

const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

pub(crate) struct BiolabHttp {
    client: Client,
    config: Arc<Config>,
}

impl BiolabHttp {
    pub(crate) fn new(config: Arc<Config>) -> Result<Self, BiolabError> {
        let token = config.load_token().ok_or(BiolabError::NotAuthenticated)?;
        let client = Client::builder()
            .timeout(DEFAULT_HTTP_TIMEOUT)
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

    pub(crate) async fn get_with_headers<T: DeserializeOwned>(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<T, BiolabError> {
        let mut request = self.client.get(self.url(path));
        for (name, value) in headers {
            request = request.header(
                HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| BiolabError::ParseError(e.to_string()))?,
                HeaderValue::from_str(value).map_err(|e| BiolabError::ParseError(e.to_string()))?,
            );
        }
        let resp = request.send().await.map_err(BiolabError::RequestError)?;
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

    pub(crate) async fn post_empty<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), BiolabError> {
        let resp = self
            .client
            .post(self.url(path))
            .json(body)
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
        Ok(())
    }

    pub(crate) async fn post_with_headers<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: &[(&str, &str)],
    ) -> Result<T, BiolabError> {
        let mut request = self.client.post(self.url(path)).json(body);
        for (name, value) in headers {
            request = request.header(
                HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| BiolabError::ParseError(e.to_string()))?,
                HeaderValue::from_str(value).map_err(|e| BiolabError::ParseError(e.to_string()))?,
            );
        }
        let resp = request.send().await.map_err(BiolabError::RequestError)?;
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

    pub(crate) async fn delete_empty(&self, path: &str) -> Result<(), BiolabError> {
        let resp = self
            .client
            .delete(self.url(path))
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
        Ok(())
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

    pub(crate) async fn download_bytes_with_headers(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<Vec<u8>, BiolabError> {
        let mut request = self.client.get(self.url(path));
        for (name, value) in headers {
            request = request.header(
                HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| BiolabError::ParseError(e.to_string()))?,
                HeaderValue::from_str(value).map_err(|e| BiolabError::ParseError(e.to_string()))?,
            );
        }
        let resp = request.send().await.map_err(BiolabError::RequestError)?;
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

    pub(crate) async fn download_absolute_bytes(&self, url: &str) -> Result<Vec<u8>, BiolabError> {
        let download_url = self.checked_download_url(url)?;
        let download_client = Client::builder()
            .timeout(DEFAULT_DOWNLOAD_TIMEOUT)
            .build()
            .map_err(BiolabError::RequestError)?;

        let resp = download_client
            .get(download_url.clone())
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let detail = resp.text().await.unwrap_or_default();
            return Err(BiolabError::HttpError {
                status,
                path: download_url.to_string(),
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
        let form = Form::new().part(
            "file",
            file_part(
                file_path,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            )?,
        );

        let resp = self
            .client
            .post(self.url(path))
            .multipart(form)
            .send()
            .await
            .map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn upload_multipart(
        &self,
        path: &str,
        file_path: &str,
        fields: &[(&str, &str)],
        extra_headers: &[(&str, &str)],
    ) -> Result<serde_json::Value, BiolabError> {
        let mut form = Form::new();
        for (name, value) in fields {
            form = form.text((*name).to_string(), (*value).to_string());
        }
        form = form.part("file", file_part(file_path, "application/octet-stream")?);

        let request =
            apply_extra_headers(self.client.post(self.url(path)), extra_headers)?.multipart(form);

        let resp = request.send().await.map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn post_multipart(
        &self,
        path: &str,
        fields: &[(&str, String)],
        files: &[(&str, &str)],
        extra_headers: &[(&str, &str)],
    ) -> Result<serde_json::Value, BiolabError> {
        let mut form = Form::new();
        for (name, value) in fields {
            form = form.text((*name).to_string(), value.clone());
        }

        for (field_name, file_path) in files {
            form = form.part(
                (*field_name).to_string(),
                file_part(file_path, "application/octet-stream")?,
            );
        }

        let request =
            apply_extra_headers(self.client.post(self.url(path)), extra_headers)?.multipart(form);

        let resp = request.send().await.map_err(BiolabError::RequestError)?;
        parse_response(resp, path).await
    }

    fn checked_download_url(&self, url: &str) -> Result<Url, BiolabError> {
        checked_download_url(&self.config.base_url, url)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url, path)
    }
}

fn file_part(file_path: &str, mime: &str) -> Result<Part, BiolabError> {
    let filename = StdPath::new(file_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("upload.bin")
        .to_string();
    let content = std::fs::read(file_path)
        .map_err(|e| BiolabError::ParseError(format!("Cannot read file {file_path}: {e}")))?;

    Part::bytes(content)
        .file_name(filename)
        .mime_str(mime)
        .map_err(|e| BiolabError::ParseError(e.to_string()))
}

fn apply_extra_headers(
    mut request: reqwest::RequestBuilder,
    headers: &[(&str, &str)],
) -> Result<reqwest::RequestBuilder, BiolabError> {
    for (name, value) in headers {
        request = request.header(
            HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| BiolabError::ParseError(e.to_string()))?,
            HeaderValue::from_str(value).map_err(|e| BiolabError::ParseError(e.to_string()))?,
        );
    }
    Ok(request)
}

fn checked_download_url(base_url: &str, input: &str) -> Result<Url, BiolabError> {
    let base = Url::parse(base_url)
        .map_err(|e| BiolabError::ParseError(format!("Invalid base URL `{base_url}`: {e}")))?;
    let url = Url::parse(input)
        .or_else(|_| base.join(input))
        .map_err(|e| BiolabError::ParseError(format!("Invalid download URL `{input}`: {e}")))?;

    if url.host_str() != base.host_str() {
        return Err(BiolabError::ParseError(format!(
            "Refusing to download from non-Biolab host `{}`",
            url.host_str().unwrap_or("<none>")
        )));
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_download_url_on_same_host() {
        let url = checked_download_url(
            "http://8.136.56.203/api/v1",
            "http://8.136.56.203/static/result.txt",
        )
        .expect("same host should be allowed");
        assert_eq!(url.as_str(), "http://8.136.56.203/static/result.txt");
    }

    #[test]
    fn accepts_relative_download_url() {
        let url = checked_download_url("http://8.136.56.203/api/v1", "/static/result.txt")
            .expect("relative URL should be allowed");
        assert_eq!(url.as_str(), "http://8.136.56.203/static/result.txt");
    }

    #[test]
    fn rejects_download_url_on_external_host() {
        let err = checked_download_url(
            "http://8.136.56.203/api/v1",
            "http://example.com/static/result.txt",
        )
        .expect_err("external host should be rejected");
        assert!(err.to_string().contains("non-Biolab host"));
    }
}
