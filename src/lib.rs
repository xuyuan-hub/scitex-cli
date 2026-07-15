mod api_response;
pub mod auth;
pub mod client;
pub mod commands;
pub mod config;
pub mod embedded_skills;
pub mod error_history;
pub mod errors;
mod http;
pub mod output;
pub mod services;
/// Scientex API client library.
pub mod types;

pub use auth::{check_status, login, logout, poll_login_from_env};
pub use client::ScientexClient;
pub use config::Config;
pub use errors::ScientexError;
pub use output::{print_result, OutputFormat};
pub use types::*;
