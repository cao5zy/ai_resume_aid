use crate::errors::AppError;
use crate::models::{ApiResponse, OptimizeRequest, OptimizeResponseData, TargetGroup};
use crate::services::{pdf_parser, zhihu};
use crate::AppState;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use futures::stream;

/// POST /api/optimize
///
/// Accepts both:
/// - `multipart/form-data` with optional `file` (PDF upload), `group`, and optional `text` fields
/// - `application/json` with `{ "text": "...", "group": "disabled"|"elderly" }`
pub async fn optimize(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let (resume_text, group) = if content_type.starts_with("application/json") {
        parse_json_body(&body)?
    } else if content_type.starts_with("multipart/form-data") {
        parse_multipart_body(&body, content_type, state.config.upload_max_size_mb).await?
    } else {
        return Err(AppError::BadRequest(
            "请使用 multipart/form-data（上传 PDF）或 application/json 格式发送请求".to_string(),
        ));
    };

    // Build the appropriate prompt for the target group
    let prompt = zhihu::build_prompt(&group, &resume_text);

    tracing::info!(
        group = group.as_str(),
        text_len = resume_text.len(),
        "Calling Zhihu Direct Answer API for resume optimization"
    );

    // Call Zhihu Direct Answer API (falls back to DeepSeek if configured)
    let raw_response = zhihu::call_direct_answer(
        &state.config.zhihu_api_token,
        state.config.deepseek_api_key.as_deref(),
        &prompt,
    )
    .await?;

    // Parse the response to extract optimized text and encouragement
    let (optimized_text, encouragement) = parse_optimization_response(&raw_response);

    let response_data = OptimizeResponseData {
        optimized_text,
        encouragement,
        original_text: resume_text,
    };

    tracing::info!(group = group.as_str(), "Resume optimization completed successfully");

    Ok((StatusCode::OK, axum::Json(ApiResponse::ok(response_data))))
}

/// Parse a JSON request body to extract resume text and target group
fn parse_json_body(body: &[u8]) -> Result<(String, TargetGroup), AppError> {
    let payload: OptimizeRequest = serde_json::from_slice(body).map_err(|e| {
        AppError::BadRequest(format!("JSON 解析失败: {}", e))
    })?;

    let text = payload.text.ok_or_else(|| {
        AppError::BadRequest("缺少必填字段 'text'".to_string())
    })?;

    if text.trim().is_empty() {
        return Err(AppError::BadRequest("简历文本不能为空".to_string()));
    }

    Ok((text, payload.group))
}

/// Parse a multipart/form-data body to extract resume text and target group
async fn parse_multipart_body(
    body: &[u8],
    content_type: &str,
    upload_max_size_mb: u64,
) -> Result<(String, TargetGroup), AppError> {
    // Extract boundary from Content-Type header
    let boundary = content_type
        .split(';')
        .find_map(|part| {
            let trimmed = part.trim();
            trimmed
                .strip_prefix("boundary=")
                .map(|b| b.trim_matches('"'))
        })
        .ok_or_else(|| {
            AppError::BadRequest("multipart/form-data 缺少 boundary 参数".to_string())
        })?;

    // Create a stream from the byte slice for multer 3.x
    let body_bytes_clone = Bytes::copy_from_slice(body);
    let byte_stream = stream::once(async move { Ok::<Bytes, std::io::Error>(body_bytes_clone) });
    let mut multipart = multer::Multipart::new(byte_stream, boundary);

    let mut resume_text = String::new();
    let mut group: Option<TargetGroup> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("解析上传数据失败: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("读取文件失败: {}", e)))?;

                let max_size = upload_max_size_mb * 1024 * 1024;
                if data.len() > max_size as usize {
                    return Err(AppError::BadRequest(format!(
                        "文件大小超过限制（最大 {}MB）",
                        upload_max_size_mb,
                    )));
                }

                if data.is_empty() {
                    return Err(AppError::BadRequest("上传的 PDF 文件为空".to_string()));
                }

                file_bytes = Some(data.to_vec());
            }
            "group" => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("读取 group 字段失败: {}", e)))?;
                group = Some(match value.trim().to_lowercase().as_str() {
                    "disabled" => TargetGroup::Disabled,
                    "elderly" => TargetGroup::Elderly,
                    other => {
                        return Err(AppError::BadRequest(format!(
                            "无效的 group 值 '{}'，请使用 'disabled' 或 'elderly'",
                            other
                        )));
                    }
                });
            }
            "text" => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("读取 text 字段失败: {}", e)))?;
                resume_text = value;
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    let group = group.ok_or_else(|| {
        AppError::BadRequest("缺少必填字段 'group'".to_string())
    })?;

    // Use file text if PDF was uploaded, otherwise use the text field
    if let Some(bytes) = file_bytes {
        resume_text = pdf_parser::extract_text_from_bytes(&bytes)?;
    }

    if resume_text.trim().is_empty() {
        return Err(AppError::BadRequest(
            "请上传 PDF 文件或在 'text' 字段中提供简历文本".to_string(),
        ));
    }

    Ok((resume_text, group))
}

/// Parse the Zhihu AI response to extract the optimized resume and encouragement sections.
fn parse_optimization_response(response: &str) -> (String, String) {
    let mut optimized_text = String::new();
    let mut encouragement = String::new();
    let mut current_section = "";

    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## 优化简历") {
            current_section = "optimized";
            continue;
        } else if trimmed.starts_with("## 优化说明") {
            current_section = "notes";
            continue;
        } else if trimmed.starts_with("## 面试鼓励") {
            current_section = "encouragement";
            continue;
        }

        match current_section {
            "optimized" => {
                if !optimized_text.is_empty() {
                    optimized_text.push('\n');
                }
                optimized_text.push_str(line);
            }
            "encouragement" => {
                if !encouragement.is_empty() {
                    encouragement.push('\n');
                }
                encouragement.push_str(line);
            }
            _ => {}
        }
    }

    // Trim
    let optimized_text = optimized_text.trim().to_string();
    let encouragement = encouragement.trim().to_string();

    // Fallback if parsing fails
    if optimized_text.is_empty() {
        (
            response.to_string(),
            String::from("相信自己，你一定可以找到适合的工作！每一次尝试都是成长的机会。"),
        )
    } else {
        let encouragement = if encouragement.is_empty() {
            String::from("相信自己，你一定可以找到适合的工作！每一次尝试都是成长的机会。")
        } else {
            encouragement
        };
        (optimized_text, encouragement)
    }
}
