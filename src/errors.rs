use thiserror::Error;

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
