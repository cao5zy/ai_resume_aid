use crate::errors::AppError;
use crate::models::ExportPdfRequest;
use crate::services::pdf_generator;
use axum::extract::Json;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;

/// POST /api/export-pdf
///
/// Generates a PDF document from the provided resume text.
/// Returns the PDF as a binary `application/pdf` response.
pub async fn export_pdf(
    Json(payload): Json<ExportPdfRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.text.trim().is_empty() {
        return Err(AppError::BadRequest("简历文本不能为空".to_string()));
    }

    let title = payload.title.unwrap_or_else(|| "优化简历".to_string());

    tracing::info!(
        title = %title,
        text_len = payload.text.len(),
        "Generating PDF export"
    );

    let pdf_bytes = pdf_generator::generate_pdf(&payload.text, &title)?;

    tracing::info!(
        pdf_size = pdf_bytes.len(),
        "PDF generated successfully"
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}.pdf\"", title)
            .parse()
            .unwrap(),
    );

    Ok((StatusCode::OK, headers, pdf_bytes))
}
