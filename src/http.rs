use std::{
    path::{Path as StdPath, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use reqwest::header::{HeaderName, HeaderValue};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::de::DeserializeOwned;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::api_response::{http_error_from_text, parse_response};
use crate::config::Config;
use crate::errors::ScientexError;

const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

/// Metadata returned after a streamed, atomic download.
#[derive(Debug, Clone)]
pub(crate) struct DownloadedFile {
    pub path: PathBuf,
    pub server_filename: String,
}

pub(crate) struct ScientexHttp {
    client: Client,
    config: Arc<Config>,
}

impl ScientexHttp {
    pub(crate) fn new(config: Arc<Config>) -> Result<Self, ScientexError> {
        let token = config.load_token().ok_or(ScientexError::NotAuthenticated)?;
        let client = Client::builder()
            .timeout(DEFAULT_HTTP_TIMEOUT)
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                        .map_err(|e| ScientexError::ParseError(e.to_string()))?,
                );
                h.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json"),
                );
                h
            })
            .build()
            .map_err(ScientexError::RequestError)?;
        Ok(Self { client, config })
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ScientexError> {
        let resp = self
            .client
            .get(self.url(path))
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn get_with_headers<T: DeserializeOwned>(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<T, ScientexError> {
        let mut request = self.client.get(self.url(path));
        for (name, value) in headers {
            request = request.header(
                HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
                HeaderValue::from_str(value)
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
            );
        }
        let resp = request.send().await.map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ScientexError> {
        let resp = self
            .client
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn post_form<T: DeserializeOwned>(
        &self,
        path: &str,
        fields: &[(&str, String)],
    ) -> Result<T, ScientexError> {
        let resp = self
            .client
            .post(self.url(path))
            .form(fields)
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn post_empty<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ScientexError> {
        let resp = self
            .client
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(http_error_from_text(status, path, text));
        }
        Ok(())
    }

    pub(crate) async fn post_empty_with_headers<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: &[(&str, &str)],
    ) -> Result<(), ScientexError> {
        let mut request = self.client.post(self.url(path)).json(body);
        for (name, value) in headers {
            request = request.header(
                HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
                HeaderValue::from_str(value)
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
            );
        }
        let resp = request.send().await.map_err(ScientexError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(http_error_from_text(status, path, text));
        }
        Ok(())
    }

    pub(crate) async fn post_with_headers<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: &[(&str, &str)],
    ) -> Result<T, ScientexError> {
        let mut request = self.client.post(self.url(path)).json(body);
        for (name, value) in headers {
            request = request.header(
                HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
                HeaderValue::from_str(value)
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
            );
        }
        let resp = request.send().await.map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn patch<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ScientexError> {
        let resp = self
            .client
            .patch(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn patch_with_headers<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: &[(&str, &str)],
    ) -> Result<T, ScientexError> {
        let request = apply_extra_headers(self.client.patch(self.url(path)).json(body), headers)?;
        let resp = request.send().await.map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ScientexError> {
        let resp = self
            .client
            .put(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ScientexError> {
        let resp = self
            .client
            .delete(self.url(path))
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn delete_empty(&self, path: &str) -> Result<(), ScientexError> {
        let resp = self
            .client
            .delete(self.url(path))
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(http_error_from_text(status, path, text));
        }
        Ok(())
    }

    pub(crate) async fn delete_empty_with_headers(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<(), ScientexError> {
        let mut request = self.client.delete(self.url(path));
        for (name, value) in headers {
            request = request.header(
                HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
                HeaderValue::from_str(value)
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
            );
        }
        let resp = request.send().await.map_err(ScientexError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(http_error_from_text(status, path, text));
        }
        Ok(())
    }

    pub(crate) async fn download_bytes(&self, path: &str) -> Result<Vec<u8>, ScientexError> {
        let resp = self
            .client
            .get(self.url(path))
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(http_error_from_text(status, path, text));
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(ScientexError::RequestError)
    }

    pub(crate) async fn download_bytes_with_headers(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<Vec<u8>, ScientexError> {
        let mut request = self.client.get(self.url(path));
        for (name, value) in headers {
            request = request.header(
                HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
                HeaderValue::from_str(value)
                    .map_err(|e| ScientexError::ParseError(e.to_string()))?,
            );
        }
        let resp = request.send().await.map_err(ScientexError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(http_error_from_text(status, path, text));
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(ScientexError::RequestError)
    }

    /// Stream a server-provided file to a temporary sibling and atomically move
    /// it into place only after the complete response has been written.
    pub(crate) async fn download_to_file(
        &self,
        path: &str,
        output: Option<&StdPath>,
        force: bool,
    ) -> Result<DownloadedFile, ScientexError> {
        let mut response = self
            .client
            .get(self.url(path))
            .timeout(DEFAULT_DOWNLOAD_TIMEOUT)
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(http_error_from_text(status, path, text));
        }

        let server_filename = content_disposition_filename(
            response
                .headers()
                .get(reqwest::header::CONTENT_DISPOSITION)
                .and_then(|value| value.to_str().ok()),
        )
        .unwrap_or_else(|| "manifest-template.xlsx".to_string());
        let target = match output {
            Some(output) => output.to_path_buf(),
            None => std::env::current_dir()
                .map_err(ScientexError::IoError)?
                .join(&server_filename),
        };
        if target.exists() && !force {
            return Err(ScientexError::ParseError(format!(
                "Refusing to overwrite {}. Pass --force to replace it.",
                target.display()
            )));
        }
        let parent = target.parent().unwrap_or_else(|| StdPath::new("."));
        if !parent.is_dir() {
            return Err(ScientexError::ParseError(format!(
                "Output directory does not exist: {}",
                parent.display()
            )));
        }

        let temp = temporary_download_path(parent, &server_filename)?;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .await
            .map_err(ScientexError::IoError)?;

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(ScientexError::RequestError)?
        {
            if let Err(error) = file.write_all(&chunk).await {
                let _ = tokio::fs::remove_file(&temp).await;
                return Err(ScientexError::IoError(error));
            }
        }
        if let Err(error) = file.sync_all().await {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(ScientexError::IoError(error));
        }
        drop(file);

        // Windows cannot replace an existing file with rename. The caller made
        // this deletion explicit by passing --force, and it occurs only after a
        // complete temporary download exists.
        if force && target.exists() {
            tokio::fs::remove_file(&target)
                .await
                .map_err(ScientexError::IoError)?;
        }
        if let Err(error) = tokio::fs::rename(&temp, &target).await {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(ScientexError::IoError(error));
        }

        Ok(DownloadedFile {
            path: target,
            server_filename,
        })
    }

    pub(crate) async fn download_absolute_bytes(
        &self,
        url: &str,
    ) -> Result<Vec<u8>, ScientexError> {
        let download_url = self.checked_download_url(url)?;
        let download_client = Client::builder()
            .timeout(DEFAULT_DOWNLOAD_TIMEOUT)
            .build()
            .map_err(ScientexError::RequestError)?;

        let resp = download_client
            .get(download_url.clone())
            .send()
            .await
            .map_err(ScientexError::RequestError)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(http_error_from_text(status, download_url.as_str(), text));
        }
        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(ScientexError::RequestError)
    }

    pub(crate) async fn upload_file(
        &self,
        path: &str,
        file_path: &str,
    ) -> Result<serde_json::Value, ScientexError> {
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
            .map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn upload_multipart(
        &self,
        path: &str,
        file_path: &str,
        fields: &[(&str, &str)],
        extra_headers: &[(&str, &str)],
    ) -> Result<serde_json::Value, ScientexError> {
        let mut form = Form::new();
        for (name, value) in fields {
            form = form.text((*name).to_string(), (*value).to_string());
        }
        form = form.part("file", file_part(file_path, "application/octet-stream")?);

        let request =
            apply_extra_headers(self.client.post(self.url(path)), extra_headers)?.multipart(form);

        let resp = request.send().await.map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    pub(crate) async fn post_multipart(
        &self,
        path: &str,
        fields: &[(&str, String)],
        files: &[(&str, &str)],
        extra_headers: &[(&str, &str)],
    ) -> Result<serde_json::Value, ScientexError> {
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

        let resp = request.send().await.map_err(ScientexError::RequestError)?;
        parse_response(resp, path).await
    }

    fn checked_download_url(&self, url: &str) -> Result<Url, ScientexError> {
        checked_download_url(&self.config.base_url, url)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url, path)
    }
}

fn file_part(file_path: &str, mime: &str) -> Result<Part, ScientexError> {
    let filename = StdPath::new(file_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("upload.bin")
        .to_string();
    let content = std::fs::read(file_path)
        .map_err(|e| ScientexError::ParseError(format!("Cannot read file {file_path}: {e}")))?;

    Part::bytes(content)
        .file_name(filename)
        .mime_str(mime)
        .map_err(|e| ScientexError::ParseError(e.to_string()))
}

fn content_disposition_filename(value: Option<&str>) -> Option<String> {
    let value = value?;
    let mut plain = None;
    for part in value.split(';').skip(1) {
        let (key, raw_value) = part.trim().split_once('=')?;
        let raw_value = raw_value.trim().trim_matches('"');
        if key.eq_ignore_ascii_case("filename*") {
            let encoded = raw_value
                .strip_prefix("UTF-8''")
                .or_else(|| raw_value.strip_prefix("utf-8''"))?;
            return sanitize_download_filename(&percent_decode_utf8(encoded)?);
        }
        if key.eq_ignore_ascii_case("filename") {
            plain = sanitize_download_filename(raw_value);
        }
    }
    plain
}

fn percent_decode_utf8(value: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(value.len());
    let source = value.as_bytes();
    let mut index = 0;
    while index < source.len() {
        if source[index] == b'%' {
            if index + 2 >= source.len() {
                return None;
            }
            let high = (source[index + 1] as char).to_digit(16)?;
            let low = (source[index + 2] as char).to_digit(16)?;
            bytes.push((high * 16 + low) as u8);
            index += 3;
        } else {
            bytes.push(source[index]);
            index += 1;
        }
    }
    String::from_utf8(bytes).ok()
}

fn sanitize_download_filename(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let cleaned = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | '\0' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect::<String>();
    let name = StdPath::new(&cleaned).file_name()?.to_str()?.trim();
    if name.is_empty() || name == "." || name == ".." {
        None
    } else {
        Some(name.to_string())
    }
}

fn temporary_download_path(parent: &StdPath, filename: &str) -> Result<PathBuf, ScientexError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| ScientexError::ParseError(error.to_string()))?
        .as_nanos();
    let pid = std::process::id();
    for index in 0..1000 {
        let candidate = parent.join(format!(".{filename}.scitex-{pid}-{nanos}-{index}.tmp"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(ScientexError::ParseError(
        "Could not allocate a temporary download path".to_string(),
    ))
}

fn apply_extra_headers(
    mut request: reqwest::RequestBuilder,
    headers: &[(&str, &str)],
) -> Result<reqwest::RequestBuilder, ScientexError> {
    for (name, value) in headers {
        request = request.header(
            HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| ScientexError::ParseError(e.to_string()))?,
            HeaderValue::from_str(value).map_err(|e| ScientexError::ParseError(e.to_string()))?,
        );
    }
    Ok(request)
}

fn checked_download_url(base_url: &str, input: &str) -> Result<Url, ScientexError> {
    let base = Url::parse(base_url)
        .map_err(|e| ScientexError::ParseError(format!("Invalid base URL `{base_url}`: {e}")))?;
    let url = Url::parse(input)
        .or_else(|_| base.join(input))
        .map_err(|e| ScientexError::ParseError(format!("Invalid download URL `{input}`: {e}")))?;

    if url.host_str() != base.host_str() {
        return Err(ScientexError::ParseError(format!(
            "Refusing to download from non-Scientex host `{}`",
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
        assert!(err.to_string().contains("non-Scientex host"));
    }

    #[test]
    fn reads_and_sanitizes_content_disposition_filename() {
        assert_eq!(
            content_disposition_filename(Some("attachment; filename=GM1.xlsx")),
            Some("GM1.xlsx".to_string())
        );
        assert_eq!(
            content_disposition_filename(Some(
                "attachment; filename*=UTF-8''GM1-%E6%B8%85%E5%8D%95.xlsx"
            )),
            Some("GM1-清单.xlsx".to_string())
        );
        assert_eq!(
            content_disposition_filename(Some("attachment; filename=../../secret.xlsx")),
            Some(".._.._secret.xlsx".to_string())
        );
    }
}
