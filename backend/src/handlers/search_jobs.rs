use crate::errors::AppError;
use crate::models::{ApiResponse, SearchJobsRequest, SearchResultItem};
use crate::services::zhihu;
use crate::AppState;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// POST /api/search-jobs
///
/// Proxies to the Zhihu Search API to find inclusive job listings
/// suitable for the target disadvantaged group.
pub async fn search_jobs(
    State(state): State<AppState>,
    Json(payload): Json<SearchJobsRequest>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!(
        group = payload.group.as_str(),
        custom_query = ?payload.query,
        "Searching inclusive job listings"
    );

    let results: Vec<SearchResultItem> =
        zhihu::call_search(&state.config.zhihu_api_token, &payload.query, &payload.group).await?;

    Ok((StatusCode::OK, Json(ApiResponse::ok(results))))
}
