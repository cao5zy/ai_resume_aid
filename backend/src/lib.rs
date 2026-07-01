pub mod config;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod services;

use crate::config::AppConfig;
use std::sync::Arc;

/// Shared application state accessible across all handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}
