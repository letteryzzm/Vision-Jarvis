use async_trait::async_trait;
use crate::error::AppResult;
use crate::ai::provider::AIProviderConfig;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn send_text(&self, prompt: &str) -> AppResult<String>;
    async fn analyze_video(&self, video_base64: &str, prompt: &str) -> AppResult<String>;
    async fn analyze_image(&self, image_base64: &str, prompt: &str) -> AppResult<String>;
    async fn test_connection(&self) -> AppResult<String>;
    fn config(&self) -> &AIProviderConfig;
}
