use std::sync::Arc;

use crate::config::Config;
use crate::http::BiolabHttp;

pub use crate::errors::BiolabError;

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
