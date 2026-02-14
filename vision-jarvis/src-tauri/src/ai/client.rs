/// AI 客户端模块
///
/// 负责与 AI API 进行通信，支持图像分析和文本生成

use crate::error::{AppError, AppResult};
use crate::ai::provider::AIProviderConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// AI 请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMessage {
    /// 角色 (system, user, assistant)
    pub role: String,

    /// 消息内容
    pub content: Vec<AIContent>,
}

/// AI 消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AIContent {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

/// 图像 URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

/// AI 请求体 (OpenAI 兼容格式)
#[derive(Debug, Serialize)]
struct AIRequest {
    model: String,
    messages: Vec<AIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// AI 响应体
#[derive(Debug, Deserialize)]
struct AIResponse {
    choices: Vec<AIChoice>,
}

#[derive(Debug, Deserialize)]
struct AIChoice {
    message: AIResponseMessage,
}

#[derive(Debug, Deserialize)]
struct AIResponseMessage {
    content: String,
}

/// AI 客户端
pub struct AIClient {
    config: AIProviderConfig,
    client: Client,
}

impl AIClient {
    /// 创建新的 AI 客户端
    pub fn new(config: AIProviderConfig) -> AppResult<Self> {
        config.validate()?;

        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // 2分钟超时
            .build()
            .map_err(|e| AppError::network(1, format!("创建 HTTP 客户端失败: {}", e)))?;

        Ok(Self { config, client })
    }

    /// 分析图像
    ///
    /// # 参数
    /// - `image_base64`: Base64 编码的图像数据
    /// - `prompt`: 分析提示词
    ///
    /// # 返回
    /// AI 分析结果文本
    pub async fn analyze_image(
        &self,
        image_base64: &str,
        prompt: &str,
    ) -> AppResult<String> {
        let messages = vec![
            AIMessage {
                role: "user".to_string(),
                content: vec![
                    AIContent::Text {
                        text: prompt.to_string(),
                    },
                    AIContent::ImageUrl {
                        image_url: ImageUrl {
                            url: format!("data:image/jpeg;base64,{}", image_base64),
                        },
                    },
                ],
            },
        ];

        self.send_request(messages).await
    }

    /// 发送文本消息
    ///
    /// # 参数
    /// - `prompt`: 文本提示词
    ///
    /// # 返回
    /// AI 响应文本
    pub async fn send_text(&self, prompt: &str) -> AppResult<String> {
        let messages = vec![
            AIMessage {
                role: "user".to_string(),
                content: vec![
                    AIContent::Text {
                        text: prompt.to_string(),
                    },
                ],
            },
        ];

        self.send_request(messages).await
    }

    /// 多轮对话
    ///
    /// # 参数
    /// - `messages`: 对话历史
    ///
    /// # 返回
    /// AI 响应文本
    pub async fn chat(&self, messages: Vec<AIMessage>) -> AppResult<String> {
        self.send_request(messages).await
    }

    /// 发送请求到 AI API
    async fn send_request(&self, messages: Vec<AIMessage>) -> AppResult<String> {
        let request_body = AIRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        // 构建完整的 API URL
        let api_url = format!("{}/v1/chat/completions", self.config.api_base_url.trim_end_matches('/'));

        // 发送请求
        let response = self.client
            .post(&api_url)
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

        // 检查状态码
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

        // 解析响应
        let ai_response: AIResponse = response.json().await
            .map_err(|e| AppError::ai(1, format!("解析响应失败: {}", e)))?;

        // 提取内容
        let content = ai_response.choices
            .first()
            .ok_or_else(|| AppError::ai(2, "响应中没有内容"))?
            .message
            .content
            .clone();

        Ok(content)
    }

    /// 测试连接
    pub async fn test_connection(&self) -> AppResult<String> {
        self.send_text("Hello").await?;
        Ok("连接成功".to_string())
    }

    /// 获取当前配置
    pub fn config(&self) -> &AIProviderConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AIProviderConfig {
        AIProviderConfig::new(
            "test-provider",
            "Test Provider",
            "https://api.aihubmix.com",
            "test-api-key",
            "claude-opus-4-6",
        )
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config();
        let client = AIClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = create_test_config();
        config.api_key = String::new();

        let client = AIClient::new(config);
        assert!(client.is_err());
    }

    #[test]
    fn test_message_serialization() {
        let message = AIMessage {
            role: "user".to_string(),
            content: vec![
                AIContent::Text {
                    text: "Hello".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_image_content_serialization() {
        let content = AIContent::ImageUrl {
            image_url: ImageUrl {
                url: "data:image/jpeg;base64,abc123".to_string(),
            },
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("image_url"));
        assert!(json.contains("data:image/jpeg"));
    }

    // 注意：实际 API 调用测试需要有效的 API Key，这里跳过
    #[tokio::test]
    #[ignore]
    async fn test_send_text() {
        let config = create_test_config();
        let client = AIClient::new(config).unwrap();

        let result = client.send_text("Hello").await;
        // 在实际测试中，这应该成功或返回特定错误
        println!("Result: {:?}", result);
    }
}
