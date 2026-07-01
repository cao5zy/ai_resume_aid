use serde::Deserialize;

/// Application configuration loaded from environment variables
#[derive(Clone, Deserialize)]
pub struct AppConfig {
    /// Zhihu API token for authentication
    pub zhihu_api_token: String,
    /// DeepSeek API key for fallback when Zhihu API fails
    pub deepseek_api_key: Option<String>,
    /// Server host to bind
    pub server_host: String,
    /// Server port to bind
    pub server_port: u16,
    /// Log level filter
    pub rust_log: String,
    /// Maximum upload file size in MB (used for PDF file validation)
    pub upload_max_size_mb: u64,
    /// API path prefix for all endpoints
    pub api_prefix: String,
}

impl AppConfig {
    /// Load configuration from environment variables.
    /// Attempts to load `.env` file first via `dotenvy`.
    pub fn from_env() -> Result<Self, anyhow::Error> {
        // Try to load .env file – ignore if not found
        let _ = dotenvy::dotenv();

        Ok(AppConfig {
            zhihu_api_token: std::env::var("ZHIHU_API_TOKEN")
                .map_err(|_| anyhow::anyhow!("ZHIHU_API_TOKEN is not set"))?,
            deepseek_api_key: std::env::var("DEEPSEEK_API_KEY").ok(),
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("SERVER_PORT must be a valid u16"))?,
            rust_log: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            upload_max_size_mb: std::env::var("UPLOAD_MAX_SIZE_MB")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| anyhow::anyhow!("UPLOAD_MAX_SIZE_MB must be a valid u64"))?,
            api_prefix: std::env::var("API_PREFIX").unwrap_or_else(|_| "/api".to_string()),
        })
    }
}
