use crate::error::AppResult;
use crate::ai::provider::AIProviderConfig;
use crate::ai::factory::create_provider;
use crate::ai::traits::AIProvider;

/// AI 客户端（facade，委托给具体 Provider 实现）
pub struct AIClient {
    inner: Box<dyn AIProvider>,
}

impl AIClient {
    pub fn new(config: AIProviderConfig) -> AppResult<Self> {
        Ok(Self { inner: create_provider(config)? })
    }

    pub async fn analyze_image(&self, image_base64: &str, prompt: &str) -> AppResult<String> {
        self.inner.analyze_image(image_base64, prompt).await
    }

    pub async fn analyze_video(&self, video_base64: &str, prompt: &str) -> AppResult<String> {
        self.inner.analyze_video(video_base64, prompt).await
    }

    pub async fn send_text(&self, prompt: &str) -> AppResult<String> {
        self.inner.send_text(prompt).await
    }

    pub async fn test_connection(&self) -> AppResult<String> {
        self.inner.test_connection().await
    }

    pub fn config(&self) -> &AIProviderConfig {
        self.inner.config()
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
        let config = AIProviderConfig::new(
            "test-provider",
            "Test Provider",
            "https://api.aihubmix.com",
            "",
            "claude-opus-4-6",
        );
        let client = AIClient::new(config);
        assert!(client.is_err());
    }
}
