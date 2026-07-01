use serde::{Deserialize, Serialize};

/// Target group for resume optimization
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TargetGroup {
    Disabled,
    Elderly,
}

impl TargetGroup {
    pub fn as_str(&self) -> &str {
        match self {
            TargetGroup::Disabled => "disabled",
            TargetGroup::Elderly => "elderly",
        }
    }
}

// ── Optimize endpoint ──────────────────────────────────────────

/// JSON request body for /api/optimize
#[derive(Debug, Deserialize)]
pub struct OptimizeRequest {
    /// Resume text to optimize (required for JSON mode)
    pub text: Option<String>,
    /// Target group: "disabled" or "elderly"
    pub group: TargetGroup,
}

/// Successful response data for /api/optimize
#[derive(Debug, Serialize)]
pub struct OptimizeResponseData {
    /// Optimized resume text in markdown
    pub optimized_text: String,
    /// Encouragement message for the job seeker
    pub encouragement: String,
    /// Original resume text that was submitted
    pub original_text: String,
}

// ── Search jobs endpoint ───────────────────────────────────────

/// JSON request body for /api/search-jobs
#[derive(Debug, Deserialize)]
pub struct SearchJobsRequest {
    /// Target group
    pub group: TargetGroup,
    /// Optional custom search query
    #[serde(default)]
    pub query: Option<String>,
}

/// A single search result item
#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

// ── Export PDF endpoint ────────────────────────────────────────

/// JSON request body for /api/export-pdf
#[derive(Debug, Deserialize)]
pub struct ExportPdfRequest {
    /// Resume text in markdown format
    pub text: String,
    /// Optional title for the PDF
    #[serde(default)]
    pub title: Option<String>,
}

// ── Config endpoint ───────────────────────────────────────────

/// Response data for GET /api/config
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    /// Maximum upload file size in MB (for PDF file validation on frontend)
    pub upload_max_size_mb: u64,
}

// ── Generic response wrappers ──────────────────────────────────

/// Standard success wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse {
            success: true,
            data,
        }
    }
}

// ── Zhihu API response types ───────────────────────────────────

/// Response from Zhihu Direct Answer API (chat completions)
#[derive(Debug, Deserialize)]
pub struct ZhihuChatResponse {
    pub choices: Vec<ZhihuChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ZhihuChatChoice {
    pub message: ZhihuChatMessage,
}

#[derive(Debug, Deserialize)]
pub struct ZhihuChatMessage {
    pub content: String,
}

/// Response from Zhihu Search API
///
/// Actual format (PascalCase at every level):
/// ```json
/// { "Code": 0, "Data": { "Items": [{ "Title": "...", "Url": "...", "Summary": "..." }] } }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZhihuSearchResponse {
    pub code: Option<i32>,
    pub data: Option<ZhihuSearchData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZhihuSearchData {
    pub items: Option<Vec<ZhihuSearchItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ZhihuSearchItem {
    pub title: String,
    pub url: Option<String>,
    pub summary: Option<String>,
}
