use ai_resume_aid::config::AppConfig;
use ai_resume_aid::handlers;
use ai_resume_aid::AppState;
use axum::{extract::DefaultBodyLimit, routing::{get, post}, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration from environment
    let config = AppConfig::from_env()?;

    // Validate required configuration
    if config.zhihu_api_token.is_empty() {
        anyhow::bail!("ZHIHU_API_TOKEN is set but empty. Please provide a valid API token in the .env file.");
    }
    tracing::info!("✅ Configuration validated");

    let state = AppState {
        config: Arc::new(config.clone()),
    };

    let prefix = &config.api_prefix;

    // Build router with CORS enabled for all origins (development mode)
    let app = Router::new()
        .route(&format!("{prefix}/config"), get(handlers::config::get_config))
        .route(&format!("{prefix}/optimize"), post(handlers::optimize::optimize))
        .route(&format!("{prefix}/search-jobs"), post(handlers::search_jobs::search_jobs))
        .route(&format!("{prefix}/export-pdf"), post(handlers::export_pdf::export_pdf))
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(
            config.upload_max_size_mb as usize * 1024 * 1024,
        ))
        .with_state(state);

    let bind_addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("🚀 AI Resume Aid server starting on http://{}", bind_addr);
    tracing::info!("📋 API endpoints (prefix: {prefix}):");
    tracing::info!("   POST {prefix}/optimize     - Resume optimization");
    tracing::info!("   POST {prefix}/search-jobs  - Inclusive job search");
    tracing::info!("   POST {prefix}/export-pdf   - PDF export");

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
