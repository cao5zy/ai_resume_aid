use crate::errors::AppError;
use std::path::Path;

/// Extract text content from a PDF file at the given path
pub fn extract_text_from_pdf(path: &Path) -> Result<String, AppError> {
    let bytes = std::fs::read(path).map_err(|e| {
        AppError::PdfParse(format!("无法读取 PDF 文件: {}", e))
    })?;

    extract_text_inner(&bytes)
}

/// Extract text from PDF bytes in memory
pub fn extract_text_from_bytes(bytes: &[u8]) -> Result<String, AppError> {
    extract_text_inner(bytes)
}

/// Internal helper for PDF text extraction
fn extract_text_inner(bytes: &[u8]) -> Result<String, AppError> {
    let text = pdf_extract::extract_text_from_mem(bytes).map_err(|e| {
        AppError::PdfParse(format!("PDF 文本提取失败: {}", e))
    })?;

    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(AppError::PdfParse(
            "PDF 文件中没有提取到文本内容，请确认文件是否为文本型 PDF".to_string(),
        ));
    }

    Ok(text)
}
