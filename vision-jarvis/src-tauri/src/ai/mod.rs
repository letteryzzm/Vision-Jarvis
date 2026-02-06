/// AI 集成模块
///
/// 提供 OpenAI API 集成、截图分析和向量嵌入功能

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use secrecy::{Secret, ExposeSecret};
use std::time::Duration;

pub mod analyzer;
pub mod embeddings;
pub mod providers;

/// OpenAI API 客户端
pub struct OpenAIClient {
    client: Client,
    api_key: Secret<String>,
    base_url: String,
}

impl OpenAIClient {
    /// 创建新的 OpenAI 客户端
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self {
            client,
            api_key: Secret::new(api_key),
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    /// 创建聊天完成请求
    pub async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.expose_secret()))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("发送聊天完成请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API 错误 {}: {}", status, error_text);
        }

        let completion = response.json::<ChatCompletionResponse>()
            .await
            .context("解析响应失败")?;

        Ok(completion)
    }

    /// 创建嵌入向量
    pub async fn create_embedding(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let url = format!("{}/embeddings", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.expose_secret()))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("发送嵌入请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API 错误 {}: {}", status, error_text);
        }

        let embedding = response.json::<EmbeddingResponse>()
            .await
            .context("解析响应失败")?;

        Ok(embedding)
    }
}

/// 聊天完成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: MessageContent,
}

/// 消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    MultiPart(Vec<ContentPart>),
}

/// 内容部分（用于 GPT-4 Vision）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

/// 图片 URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

/// 响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

/// 聊天完成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

/// 选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

/// 使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 嵌入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

/// 嵌入响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

/// 嵌入数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub index: u32,
    pub embedding: Vec<f32>,
}

/// 嵌入使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OpenAIClient::new("test-key".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_chat_completion_request_serialization() {
        let request = ChatCompletionRequest {
            model: "gpt-4o".to_string(),
            messages: vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: MessageContent::Text("Hello".to_string()),
                }
            ],
            temperature: Some(0.7),
            max_tokens: Some(100),
            response_format: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4o"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_embedding_request_serialization() {
        let request = EmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: "Test text".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("text-embedding-3-small"));
        assert!(json.contains("Test text"));
    }

    #[test]
    fn test_vision_message_serialization() {
        let message = ChatMessage {
            role: "user".to_string(),
            content: MessageContent::MultiPart(vec![
                ContentPart::Text {
                    text: "What's in this image?".to_string(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: "data:image/png;base64,xxx".to_string(),
                    },
                },
            ]),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("What's in this image?"));
        assert!(json.contains("data:image/png"));
    }
}
