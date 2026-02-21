use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::ai::provider::AIProviderConfig;
use crate::ai::traits::AIProvider;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    InlineData { inline_data: GeminiInlineData },
}

#[derive(Debug, Serialize)]
struct GeminiInlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    max_output_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

pub struct GeminiProvider {
    config: AIProviderConfig,
    client: Client,
}

impl GeminiProvider {
    pub fn new(config: AIProviderConfig) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AppError::network(1, format!("创建 HTTP 客户端失败: {}", e)))?;
        Ok(Self { config, client })
    }

    fn api_url(&self) -> String {
        format!(
            "{}/v1beta/models/{}:generateContent",
            self.config.api_base_url.trim_end_matches('/'),
            self.config.model
        )
    }

    async fn send_request(&self, parts: Vec<GeminiPart>) -> AppResult<String> {
        let request_body = GeminiRequest {
            contents: vec![GeminiContent { parts }],
            generation_config: GeminiGenerationConfig {
                max_output_tokens: 4096,
                temperature: 0.7,
            },
        };

        let response = self.client
            .post(&self.api_url())
            .header("x-goog-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AppError::network(1, "请求超时")
                } else if e.is_connect() {
                    AppError::network(2, "网络连接失败")
                } else {
                    AppError::network(999, format!("请求失败: {}", e))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            return Err(match status.as_u16() {
                401 => AppError::ai(401, "API Key 无效或未授权"),
                403 => AppError::ai(403, "访问被拒绝"),
                404 => AppError::ai(404, "API 端点不存在"),
                429 => AppError::ai(429, "请求过于频繁，请稍后重试"),
                500..=599 => AppError::ai(500, format!("服务器错误: {}", error_text)),
                _ => AppError::ai(999, format!("HTTP 错误 {}: {}", status, error_text)),
            });
        }

        let gemini_response: GeminiResponse = response.json().await
            .map_err(|e| AppError::ai(1, format!("解析响应失败: {}", e)))?;

        gemini_response.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| p.text.clone())
            .ok_or_else(|| AppError::ai(2, "响应中没有内容"))
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn send_text(&self, prompt: &str) -> AppResult<String> {
        let parts = vec![GeminiPart::Text { text: prompt.to_string() }];
        self.send_request(parts).await
    }

    async fn analyze_video(&self, video_base64: &str, prompt: &str) -> AppResult<String> {
        let parts = vec![
            GeminiPart::Text { text: prompt.to_string() },
            GeminiPart::InlineData {
                inline_data: GeminiInlineData {
                    mime_type: "video/mp4".to_string(),
                    data: video_base64.to_string(),
                },
            },
        ];
        self.send_request(parts).await
    }

    async fn analyze_image(&self, image_base64: &str, prompt: &str) -> AppResult<String> {
        let parts = vec![
            GeminiPart::Text { text: prompt.to_string() },
            GeminiPart::InlineData {
                inline_data: GeminiInlineData {
                    mime_type: "image/jpeg".to_string(),
                    data: image_base64.to_string(),
                },
            },
        ];
        self.send_request(parts).await
    }

    async fn test_connection(&self) -> AppResult<String> {
        self.send_text("Hello").await?;
        Ok("连接成功".to_string())
    }

    fn config(&self) -> &AIProviderConfig {
        &self.config
    }
}
