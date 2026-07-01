use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Unified application error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Zhihu API error: {0}")]
    ZhihuApi(String),

    #[error("PDF parse error: {0}")]
    PdfParse(String),

    #[error("PDF generate error: {0}")]
    PdfGenerate(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl AppError {
    /// Return the HTTP status code corresponding to this error
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ZhihuApi(_) => StatusCode::BAD_GATEWAY,
            AppError::PdfParse(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::PdfGenerate(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Reqwest(_) => StatusCode::BAD_GATEWAY,
            AppError::Serde(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Return the error code string
    fn error_code(&self) -> &str {
        match self {
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::ZhihuApi(_) => "ZHIHU_API_ERROR",
            AppError::PdfParse(_) => "PDF_PARSE_ERROR",
            AppError::PdfGenerate(_) => "PDF_GENERATE_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Reqwest(_) => "REQUEST_ERROR",
            AppError::Serde(_) => "SERIALIZATION_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code().to_string();
        let message = self.to_string();

        // Log the error with appropriate level
        match &self {
            AppError::Internal(_) | AppError::ZhihuApi(_) | AppError::Io(_) => {
                tracing::error!(error = %self, "Internal error occurred");
            }
            AppError::BadRequest(_) => {
                tracing::warn!(error = %self, "Bad request");
            }
            _ => {
                tracing::error!(error = %self, "Error occurred");
            }
        }

        let body = json!({
            "success": false,
            "error": {
                "message": message,
                "code": code,
            }
        });

        (status, Json(body)).into_response()
    }
}

// Allow converting anyhow::Error into AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

// Allow converting printpdf errors
impl From<printpdf::Error> for AppError {
    fn from(err: printpdf::Error) -> Self {
        AppError::PdfGenerate(err.to_string())
    }
}
