use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::ai::provider::AIProviderConfig;
use crate::ai::traits::AIProvider;

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: Vec<OpenAIContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OpenAIContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAIImageUrl },
}

#[derive(Debug, Serialize)]
struct OpenAIImageUrl {
    url: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

pub struct OpenAIProvider {
    config: AIProviderConfig,
    client: Client,
}

impl OpenAIProvider {
    pub fn new(config: AIProviderConfig) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AppError::network(1, format!("创建 HTTP 客户端失败: {}", e)))?;
        Ok(Self { config, client })
    }

    fn api_url(&self) -> String {
        format!("{}/v1/chat/completions", self.config.api_base_url.trim_end_matches('/'))
    }

    async fn send_request(&self, messages: Vec<OpenAIMessage>) -> AppResult<String> {
        let request_body = OpenAIRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        let response = self.client
            .post(&self.api_url())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
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

        let ai_response: OpenAIResponse = response.json().await
            .map_err(|e| AppError::ai(1, format!("解析响应失败: {}", e)))?;

        ai_response.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AppError::ai(2, "响应中没有内容"))
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn send_text(&self, prompt: &str) -> AppResult<String> {
        let messages = vec![OpenAIMessage {
            role: "user".to_string(),
            content: vec![OpenAIContent::Text { text: prompt.to_string() }],
        }];
        self.send_request(messages).await
    }

    async fn analyze_video(&self, video_base64: &str, prompt: &str) -> AppResult<String> {
        let messages = vec![OpenAIMessage {
            role: "user".to_string(),
            content: vec![
                OpenAIContent::Text { text: prompt.to_string() },
                OpenAIContent::ImageUrl {
                    image_url: OpenAIImageUrl {
                        url: format!("data:video/mp4;base64,{}", video_base64),
                    },
                },
            ],
        }];
        self.send_request(messages).await
    }

    async fn analyze_image(&self, image_base64: &str, prompt: &str) -> AppResult<String> {
        let messages = vec![OpenAIMessage {
            role: "user".to_string(),
            content: vec![
                OpenAIContent::Text { text: prompt.to_string() },
                OpenAIContent::ImageUrl {
                    image_url: OpenAIImageUrl {
                        url: format!("data:image/jpeg;base64,{}", image_base64),
                    },
                },
            ],
        }];
        self.send_request(messages).await
    }

    async fn test_connection(&self) -> AppResult<String> {
        self.send_text("Hello").await?;
        Ok("连接成功".to_string())
    }

    fn config(&self) -> &AIProviderConfig {
        &self.config
    }
}
