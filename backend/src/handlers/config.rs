use crate::models::{ApiResponse, ConfigResponse};
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

/// GET /api/config
///
/// Returns runtime configuration values that the frontend needs to know.
pub async fn get_config(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let config = ConfigResponse {
        upload_max_size_mb: state.config.upload_max_size_mb,
    };

    (StatusCode::OK, Json(ApiResponse::ok(config)))
}
