use async_trait::async_trait;
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::ai::provider::AIProviderConfig;
use crate::ai::traits::AIProvider;
use crate::ai::frame_extractor::{extract_frames, FrameExtractConfig};

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ClaudeContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ClaudeImageSource },
}

#[derive(Debug, Serialize)]
struct ClaudeImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeResponseContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponseContent {
    text: Option<String>,
}

pub struct ClaudeProvider {
    config: AIProviderConfig,
    client: Client,
}

impl ClaudeProvider {
    pub fn new(config: AIProviderConfig) -> AppResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AppError::network(1, format!("创建 HTTP 客户端失败: {}", e)))?;
        Ok(Self { config, client })
    }

    fn api_url(&self) -> String {
        format!("{}/v1/messages", self.config.api_base_url.trim_end_matches('/'))
    }

    async fn send_request(&self, messages: Vec<ClaudeMessage>, model: &str) -> AppResult<String> {
        let request_body = ClaudeRequest {
            model: model.to_string(),
            max_tokens: 4096,
            messages,
        };

        let response = self.client
            .post(&self.api_url())
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
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

        let claude_response: ClaudeResponse = response.json().await
            .map_err(|e| AppError::ai(1, format!("解析响应失败: {}", e)))?;

        claude_response.content
            .first()
            .and_then(|c| c.text.clone())
            .ok_or_else(|| AppError::ai(2, "响应中没有内容"))
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn send_text(&self, prompt: &str) -> AppResult<String> {
        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: vec![ClaudeContent::Text { text: prompt.to_string() }],
        }];
        self.send_request(messages, &self.config.model).await
    }

    async fn analyze_video(&self, video_base64: &str, prompt: &str) -> AppResult<String> {
        info!("[Claude] 不支持原生视频分析，使用帧提取预处理");
        let config = FrameExtractConfig::default();
        match extract_frames(video_base64, &config) {
            Ok(frames) => {
                info!("[Claude] 帧提取成功，提取 {} 帧，发送多图分析", frames.len());
                let mut content: Vec<ClaudeContent> = frames.into_iter()
                    .map(|frame_b64| ClaudeContent::Image {
                        source: ClaudeImageSource {
                            source_type: "base64".to_string(),
                            media_type: "image/jpeg".to_string(),
                            data: frame_b64,
                        },
                    })
                    .collect();
                content.push(ClaudeContent::Text {
                    text: format!("以下是从视频录屏中均匀提取的关键帧。请综合所有帧分析视频内容：\n\n{}", prompt),
                });
                let messages = vec![ClaudeMessage {
                    role: "user".to_string(),
                    content,
                }];
                self.send_request(messages, self.config.effective_video_model()).await
            }
            Err(e) => {
                warn!("[Claude] 帧提取失败({}), 回退到纯文本提示", e);
                let fallback_prompt = format!(
                    "用户提供了一段视频录屏，但当前无法处理视频。请根据以下分析提示尽量提供帮助：\n\n{}",
                    prompt
                );
                self.send_text(&fallback_prompt).await
            }
        }
    }

    async fn analyze_image(&self, image_base64: &str, prompt: &str) -> AppResult<String> {
        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: vec![
                ClaudeContent::Text { text: prompt.to_string() },
                ClaudeContent::Image {
                    source: ClaudeImageSource {
                        source_type: "base64".to_string(),
                        media_type: "image/jpeg".to_string(),
                        data: image_base64.to_string(),
                    },
                },
            ],
        }];
        self.send_request(messages, self.config.effective_video_model()).await
    }

    async fn test_connection(&self) -> AppResult<String> {
        self.send_text("Hello").await?;
        Ok("连接成功".to_string())
    }

    fn config(&self) -> &AIProviderConfig {
        &self.config
    }
}
