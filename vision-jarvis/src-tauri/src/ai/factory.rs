use crate::error::AppResult;
use crate::ai::provider::{AIProviderConfig, ProviderType};
use crate::ai::traits::AIProvider;
use crate::ai::providers::*;

pub fn create_provider(config: AIProviderConfig) -> AppResult<Box<dyn AIProvider>> {
    config.validate()?;
    match config.provider_type {
        ProviderType::OpenAI => Ok(Box::new(OpenAIProvider::new(config)?)),
        ProviderType::Claude => Ok(Box::new(ClaudeProvider::new(config)?)),
        ProviderType::Gemini => Ok(Box::new(GeminiProvider::new(config)?)),
        ProviderType::Qwen => Ok(Box::new(QwenProvider::new(config)?)),
        ProviderType::AIHubMix => Ok(Box::new(AIHubMixProvider::new(config)?)),
        ProviderType::OpenRouter => Ok(Box::new(OpenRouterProvider::new(config)?)),
    }
}
