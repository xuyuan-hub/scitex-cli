use std::sync::Arc;

use thiserror::Error;

use crate::config::Config;
use crate::http::BiolabHttp;

#[derive(Debug, Error)]
pub enum BiolabError {
    #[error("HTTP {status} {path}: {detail}")]
    HttpError {
        status: u16,
        path: String,
        detail: String,
    },
    #[error("Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Token not found. Run `biolab login` first.")]
    NotAuthenticated,
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct BiolabClient {
    pub(crate) http: BiolabHttp,
}

impl BiolabClient {
    pub fn new(config: Arc<Config>) -> Result<Self, BiolabError> {
        Ok(Self {
            http: BiolabHttp::new(config)?,
        })
    }

    pub fn with_token(config: Arc<Config>, token: &str) -> Result<Self, BiolabError> {
        config
            .save_token(token)
            .map_err(|e| BiolabError::ParseError(e.to_string()))?;
        Self::new(config)
    }
}
