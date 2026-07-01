use crate::errors::AppError;
use crate::models::{TargetGroup, ZhihuChatResponse};
use serde_json::json;

/// Prompt template for the disabled job seeker group
fn disabled_prompt(resume_text: &str) -> String {
    format!(
        r#"你是一位专业的简历优化顾问，专门帮助求职中可能需要跨越障碍的朋友优化简历。
请遵循以下原则：
1. 弱化身体局限的描述，将注意力引导到技能和成就上
2. 如果原文提到身体状况或障碍相关情况，用积极、中性的语言重新表述
3. 突出实际工作能力和项目经验
4. 使用清晰、自信的职业化语言
5. 不要主动提及障碍相关状况，除非对岗位有直接关联
6. 保持真实，不要编造经历
7. **非常重要：你必须输出完整的简历内容，不要省略任何部分。即使某些内容不需要修改，也要完整地写出来，不要写"保持原样"、"无需调整"、"同上"等省略说明。**

简历内容：
{resume_text}

请按照以下格式返回：
## 优化简历
[优化后的完整简历——必须包含全部内容，不得省略]

## 优化说明
[简要说明做了哪些优化调整]

## 面试鼓励
[一段温暖的鼓励话术，帮助求职者建立信心]"#
    )
}

/// Prompt template for the elderly job seeker group
fn elderly_prompt(resume_text: &str) -> String {
    format!(
        r#"你是一位专业的简历优化顾问，专门帮助45岁以上求职者优化简历。
请遵循以下原则：
1. 强调丰富的工作经验和实操能力
2. 弱化年龄相关表述，避免"老练"、"资深"等可能暗示年龄的词汇
3. 突出稳定性、责任心和职场智慧
4. 将多年经验转化为"行业深耕"、"经验丰富"等积极表述
5. 强调持续学习和适应能力
6. 保持真实，不要编造经历
7. **非常重要：你必须输出完整的简历内容，不要省略任何部分。即使某些内容不需要修改，也要完整地写出来，不要写"保持原样"、"无需调整"、"同上"等省略说明。**

简历内容：
{resume_text}

请按照以下格式返回：
## 优化简历
[优化后的完整简历——必须包含全部内容，不得省略]

## 优化说明
[简要说明做了哪些优化调整]

## 面试鼓励
[一段温暖的鼓励话术，帮助求职者建立信心]"#
    )
}

/// Build the prompt for the given group and resume text
pub fn build_prompt(group: &TargetGroup, resume_text: &str) -> String {
    match group {
        TargetGroup::Disabled => disabled_prompt(resume_text),
        TargetGroup::Elderly => elderly_prompt(resume_text),
    }
}

/// Build a Zhihu Direct Answer API request, send it, and return the response.
async fn send_chat_request(
    client: &reqwest::Client,
    access_secret: &str,
    timestamp: &str,
    body: &serde_json::Value,
) -> Result<reqwest::Response, reqwest::Error> {
    client
        .post("https://developer.zhihu.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", access_secret))
        .header("X-Request-Timestamp", timestamp)
        .header("Content-Type", "application/json")
        .header("User-Agent", "ai-resume-aid/0.1")
        .json(body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
}

/// Parse a Zhihu Chat Completion JSON response into the assistant message content.
async fn parse_chat_response(response: reqwest::Response) -> Result<String, AppError> {
    let chat_response: ZhihuChatResponse = response.json().await.map_err(|e| {
        AppError::ZhihuApi(format!("解析 Zhihu API 响应失败: {}", e))
    })?;

    chat_response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| AppError::ZhihuApi("Zhihu API 返回了空的响应内容".to_string()))
}

/// Call the DeepSeek Chat API as a fallback when Zhihu API fails.
///
/// Uses the same OpenAI-compatible Chat Completions format.
async fn call_deepseek(
    api_key: &str,
    prompt: &str,
) -> Result<String, AppError> {
    let client = reqwest::Client::new();

    let body = json!({
        "model": "deepseek-v4-flash",
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    tracing::info!(
        prompt_len = prompt.len(),
        "Calling DeepSeek API as fallback"
    );

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::ZhihuApi("DeepSeek 请求超时".to_string())
            } else {
                AppError::ZhihuApi(format!("DeepSeek API 请求失败: {}", e))
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let err_body = response.text().await.unwrap_or_default();
        tracing::error!(
            status = status.as_u16(),
            body = %err_body,
            "DeepSeek API returned error"
        );
        return Err(AppError::ZhihuApi(format!(
            "DeepSeek API 返回错误 ({}): {}",
            status.as_u16(),
            err_body
        )));
    }

    tracing::info!("DeepSeek API fallback succeeded");
    parse_chat_response(response).await
}

/// Call the Zhihu Direct Answer API to optimize resume text.
///
/// Uses exponential backoff to handle transient 5xx errors (commonly
/// 554 caused by server-side cold-start timeouts on `zhida-thinking-1p5`).
/// Retries up to 5 times with 1s/2s/4s/8s delays.
///
/// If all retries fail and a `deepseek_api_key` is provided, falls back to
/// DeepSeek's Chat API as a last resort.
pub async fn call_direct_answer(
    access_secret: &str,
    deepseek_api_key: Option<&str>,
    prompt: &str,
) -> Result<String, AppError> {
    let client = reqwest::Client::new();
    let timestamp = chrono::Utc::now().timestamp().to_string();

    let body = json!({
        "model": "zhida-thinking-1p5",
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let max_retries = 5;
    let mut retry_delay = 1u64;

    for attempt in 1..=max_retries {
        let is_retry = attempt > 1;

        // All attempts share the timestamp — Zhihu uses it for replay protection,
        // so it must stay the same across retries. The 1s window is long enough.
        tracing::debug!(
            attempt,
            prompt_len = prompt.len(),
            model = "zhida-thinking-1p5",
            "Sending request to Zhihu Direct Answer API"
        );

        let response = send_chat_request(&client, access_secret, &timestamp, &body).await;

        let response = match response {
            Ok(r) => r,
            Err(e) if e.is_timeout() => {
                if attempt < max_retries {
                    tracing::info!(attempt, "Request timed out, retrying in {}s", retry_delay);
                    tokio::time::sleep(std::time::Duration::from_secs(retry_delay)).await;
                    retry_delay *= 2;
                    continue;
                }
                return Err(AppError::ZhihuApi("请求超时，请稍后重试".to_string()));
            }
            Err(e) => {
                if attempt < max_retries {
                    tracing::info!(attempt, error = %e, "Request failed, retrying in {}s", retry_delay);
                    tokio::time::sleep(std::time::Duration::from_secs(retry_delay)).await;
                    retry_delay *= 2;
                    continue;
                }
                return Err(AppError::ZhihuApi(format!("Zhihu API 请求失败: {}", e)));
            }
        };

        if response.status().is_success() {
            if is_retry {
                tracing::info!("Zhihu Direct Answer API retry succeeded (attempt {})", attempt);
            }
            return parse_chat_response(response).await;
        }

        // Non-200: log and retry for 5xx, exit to fallback on last attempt
        let status_code = response.status().as_u16();
        let err_body = response.text().await.unwrap_or_default();

        if status_code >= 500 && status_code < 600 {
            if attempt < max_retries {
                tracing::warn!(
                    attempt,
                    status = status_code,
                    delay_s = retry_delay,
                    "Zhihu Direct Answer API returned server error, retrying"
                );
                tokio::time::sleep(std::time::Duration::from_secs(retry_delay)).await;
                retry_delay *= 2;
                continue;
            }
            // Last attempt with 5xx: exit loop to try fallback
            tracing::warn!(
                attempt, status = status_code,
                "Zhihu API 5xx on final attempt, exiting loop for fallback"
            );
            break;
        }

        // 4xx or other non-5xx: bail immediately (no point falling back)
        return match status_code {
            400..=499 => Err(AppError::BadRequest(format!(
                "Zhihu API 请求被拒绝 ({})", status_code
            ))),
            _ => Err(AppError::ZhihuApi(format!(
                "Zhihu API 返回错误 ({}){}",
                status_code,
                if err_body.is_empty() { String::new() } else { format!(": {}", err_body) }
            ))),
        };
    }

    // All Zhihu retries exhausted (loop ended via break or natural completion)
    // Try DeepSeek fallback if configured
    if let Some(ds_key) = deepseek_api_key {
        tracing::warn!("Zhihu API failed after {} retries, falling back to DeepSeek", max_retries);
        return call_deepseek(ds_key, prompt).await;
    }

    Err(AppError::ZhihuApi("Zhihu API 在多次重试后仍然失败，且未配置 DeepSeek 备用 API".to_string()))
}

/// Search the Zhihu Search API with a single query and return results.
async fn search_single(
    client: &reqwest::Client,
    access_secret: &str,
    timestamp: &str,
    query: &str,
) -> Result<Vec<crate::models::SearchResultItem>, AppError> {
    tracing::info!(query, "Sending search request to Zhihu Search API");

    let response = client
        .get("https://developer.zhihu.com/api/v1/content/zhihu_search")
        .header("Authorization", format!("Bearer {}", access_secret))
        .header("X-Request-Timestamp", timestamp)
        .query(&[("Query", query), ("Count", "10")])
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::ZhihuApi("搜索请求超时，请稍后重试".to_string())
            } else {
                AppError::ZhihuApi(format!("Zhihu 搜索请求失败: {}", e))
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::warn!(query, status = %status, "Zhihu Search API returned error");
        return Err(AppError::ZhihuApi(format!(
            "Zhihu 搜索 API 返回错误 ({}): {}",
            status.as_u16(),
            body
        )));
    }

    let search_response: crate::models::ZhihuSearchResponse =
        response.json().await.map_err(|e| {
            AppError::ZhihuApi(format!("解析搜索响应失败: {}", e))
        })?;

    // Check API-level error code
    if let Some(code) = search_response.code {
        if code != 0 {
            tracing::warn!(code, query, "Zhihu Search API returned non-zero code");
            return Ok(Vec::new());
        }
    }

    let items: Vec<crate::models::SearchResultItem> = search_response
        .data
        .and_then(|d| d.items)
        .unwrap_or_default()
        .into_iter()
        .map(|item| crate::models::SearchResultItem {
            title: item.title,
            url: item.url.unwrap_or_default(),
            snippet: item.summary.unwrap_or_default(),
        })
        .collect();

    Ok(items)
}

/// Default search queries per target group, tried in order until results are found.
fn default_search_queries(group: &TargetGroup) -> Vec<&'static str> {
    match group {
        TargetGroup::Disabled => vec![
            "残障人士 招聘 包容性岗位",
            "残障 就业 岗位",
            "无障碍 招聘 工作",
            "包容性 岗位 求职",
            "残疾人 招聘",
        ],
        TargetGroup::Elderly => vec![
            "中老年 再就业 招聘 45岁以上",
            "大龄 招聘 岗位",
            "中年 求职 再就业",
            "40岁以上 招聘 工作",
            "再就业 培训 岗位",
        ],
    }
}

/// Call the Zhihu Search API to find inclusive job listings.
///
/// Tries multiple queries specific to the target group until results are found.
/// Falls back through progressively broader queries.
pub async fn call_search(
    access_secret: &str,
    query: &Option<String>,
    group: &TargetGroup,
) -> Result<Vec<crate::models::SearchResultItem>, AppError> {
    let client = reqwest::Client::new();
    let timestamp = chrono::Utc::now().timestamp().to_string();

    // If user provided a custom query, try it first
    let queries: Vec<String> = match query {
        Some(q) if !q.trim().is_empty() => {
            let mut qs = vec![q.trim().to_string()];
            qs.extend(default_search_queries(group).into_iter().map(String::from));
            qs
        }
        _ => default_search_queries(group).into_iter().map(String::from).collect(),
    };

    for (i, q) in queries.iter().enumerate() {
        let result = search_single(&client, access_secret, &timestamp, q).await;

        match result {
            Ok(items) if !items.is_empty() => {
                tracing::info!(
                    query = %q,
                    result_count = items.len(),
                    attempt = i + 1,
                    group = group.as_str(),
                    "Job search found results"
                );
                return Ok(items);
            }
            Ok(_) => {
                tracing::info!(query = %q, attempt = i + 1, "Search returned 0 results, trying next query");
            }
            Err(e) => {
                tracing::warn!(query = %q, attempt = i + 1, error = %e, "Search query failed, trying next");
            }
        }
    }

    tracing::info!(
        group = group.as_str(),
        "All search queries returned 0 results"
    );
    Ok(Vec::new())
}
