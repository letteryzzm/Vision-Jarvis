/// 截图分析器
///
/// 使用 GPT-4 Vision 分析截图内容

use super::*;
use crate::db::schema::{AnalysisResult, ActivityCategory};
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};

/// 截图分析器
pub struct ScreenshotAnalyzer {
    client: OpenAIClient,
}

impl ScreenshotAnalyzer {
    /// 创建新的分析器
    pub fn new(api_key: String) -> Result<Self> {
        let client = OpenAIClient::new(api_key)?;
        Ok(Self { client })
    }

    /// 分析截图
    pub async fn analyze_screenshot(&self, image_path: &Path) -> Result<AnalysisResult> {
        // 读取图片并转换为 base64
        let image_data = std::fs::read(image_path)
            .context("读取图片文件失败")?;
        let base64_image = general_purpose::STANDARD.encode(&image_data);
        let data_url = format!("data:image/png;base64,{}", base64_image);

        // 构建分析 prompt
        let system_prompt = r#"你是一个专业的屏幕内容分析助手。分析截图并提取以下信息：
1. activity: 用户正在进行的主要活动（如"编程"、"浏览网页"、"看视频"等）
2. application: 使用的应用程序名称
3. description: 简要描述用户在做什么（1-2句话）
4. category: 活动分类（work/entertainment/communication/other）

以 JSON 格式返回结果。"#;

        let user_message = ChatMessage {
            role: "user".to_string(),
            content: MessageContent::MultiPart(vec![
                ContentPart::Text {
                    text: "请分析这张截图".to_string(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: data_url,
                    },
                },
            ]),
        };

        let request = ChatCompletionRequest {
            model: "gpt-4o".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: MessageContent::Text(system_prompt.to_string()),
                },
                user_message,
            ],
            temperature: Some(0.3),
            max_tokens: Some(300),
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
        };

        // 发送请求
        let response = self.client.chat_completion(request).await?;

        // 解析响应
        let content = match &response.choices.first()
            .context("响应中没有选择")?
            .message
            .content
        {
            MessageContent::Text(text) => text,
            _ => anyhow::bail!("期望文本响应"),
        };

        let result: AnalysisResultJson = serde_json::from_str(content)
            .context("解析 JSON 响应失败")?;

        Ok(AnalysisResult {
            activity: result.activity,
            application: result.application,
            description: result.description,
            category: match result.category.as_str() {
                "work" => ActivityCategory::Work,
                "entertainment" => ActivityCategory::Entertainment,
                "communication" => ActivityCategory::Communication,
                _ => ActivityCategory::Other,
            },
        })
    }
}

/// JSON 响应结构
#[derive(Debug, Deserialize)]
struct AnalysisResultJson {
    activity: String,
    application: String,
    description: String,
    category: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = ScreenshotAnalyzer::new("test-key".to_string());
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_analysis_result_json_deserialization() {
        let json = r#"{
            "activity": "编程",
            "application": "VS Code",
            "description": "用户正在编写 Rust 代码",
            "category": "work"
        }"#;

        let result: AnalysisResultJson = serde_json::from_str(json).unwrap();
        assert_eq!(result.activity, "编程");
        assert_eq!(result.application, "VS Code");
        assert_eq!(result.category, "work");
    }

    // 注意：实际 API 调用测试需要真实的 API key，在 CI 中应跳过
    #[tokio::test]
    #[ignore]
    async fn test_analyze_screenshot_with_real_api() {
        // 需要设置环境变量 OPENAI_API_KEY
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let analyzer = ScreenshotAnalyzer::new(api_key).unwrap();
            // 这里需要一个真实的测试图片
            // let result = analyzer.analyze_screenshot(Path::new("test.png")).await;
            // assert!(result.is_ok());
        }
    }
}
