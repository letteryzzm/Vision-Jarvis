/// AI 提供商配置模块
///
/// 支持多个 AI 提供商的配置管理

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// AI 提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AIProvider {
    OpenAI,
    Anthropic,
    Google,
    Local,
    Custom,
}

impl Default for AIProvider {
    fn default() -> Self {
        AIProvider::OpenAI
    }
}

impl AIProvider {
    /// 获取提供商显示名称
    pub fn display_name(&self) -> &str {
        match self {
            AIProvider::OpenAI => "OpenAI",
            AIProvider::Anthropic => "Anthropic",
            AIProvider::Google => "Google AI",
            AIProvider::Local => "Local Model",
            AIProvider::Custom => "Custom Provider",
        }
    }

    /// 获取默认 API 基础 URL
    pub fn default_base_url(&self) -> &str {
        match self {
            AIProvider::OpenAI => "https://api.openai.com/v1",
            AIProvider::Anthropic => "https://api.anthropic.com",
            AIProvider::Google => "https://generativelanguage.googleapis.com/v1",
            AIProvider::Local => "http://localhost:11434/api",
            AIProvider::Custom => "",
        }
    }

    /// 获取默认模型名称
    pub fn default_model(&self) -> &str {
        match self {
            AIProvider::OpenAI => "gpt-4o",
            AIProvider::Anthropic => "claude-3-5-sonnet-20241022",
            AIProvider::Google => "gemini-1.5-flash",
            AIProvider::Local => "llama3.2",
            AIProvider::Custom => "",
        }
    }
}

/// AI 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    /// 提供商类型
    pub provider: AIProvider,
    /// API 密钥（序列化时隐藏）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// API 基础 URL
    pub base_url: String,
    /// 使用的模型
    pub model: String,
    /// 是否启用
    pub enabled: bool,
    /// 最大 tokens
    pub max_tokens: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
}

impl Default for AIProviderConfig {
    fn default() -> Self {
        Self {
            provider: AIProvider::OpenAI,
            api_key: None,
            base_url: AIProvider::OpenAI.default_base_url().to_string(),
            model: AIProvider::OpenAI.default_model().to_string(),
            enabled: false,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }
}

impl AIProviderConfig {
    /// 创建 OpenAI 配置
    pub fn openai(api_key: Option<String>) -> Self {
        let enabled = api_key.is_some();
        Self {
            provider: AIProvider::OpenAI,
            api_key,
            base_url: AIProvider::OpenAI.default_base_url().to_string(),
            model: AIProvider::OpenAI.default_model().to_string(),
            enabled,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }

    /// 创建 Anthropic 配置
    pub fn anthropic(api_key: Option<String>) -> Self {
        let enabled = api_key.is_some();
        Self {
            provider: AIProvider::Anthropic,
            api_key,
            base_url: AIProvider::Anthropic.default_base_url().to_string(),
            model: AIProvider::Anthropic.default_model().to_string(),
            enabled,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }

    /// 创建 Google AI 配置
    pub fn google(api_key: Option<String>) -> Self {
        let enabled = api_key.is_some();
        Self {
            provider: AIProvider::Google,
            api_key,
            base_url: AIProvider::Google.default_base_url().to_string(),
            model: AIProvider::Google.default_model().to_string(),
            enabled,
            max_tokens: Some(8192),
            temperature: Some(0.7),
        }
    }

    /// 创建本地模型配置
    pub fn local() -> Self {
        Self {
            provider: AIProvider::Local,
            api_key: None,
            base_url: AIProvider::Local.default_base_url().to_string(),
            model: AIProvider::Local.default_model().to_string(),
            enabled: false,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }

    /// 创建自定义提供商配置
    pub fn custom(base_url: String, model: String, api_key: Option<String>) -> Self {
        let enabled = !base_url.is_empty() && !model.is_empty();
        Self {
            provider: AIProvider::Custom,
            api_key,
            base_url,
            model,
            enabled,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if self.enabled {
            // 非本地模型需要 API 密钥
            if self.provider != AIProvider::Local && self.api_key.is_none() {
                anyhow::bail!("{} 需要 API 密钥", self.provider.display_name());
            }

            // 验证 base_url
            if self.base_url.is_empty() {
                anyhow::bail!("API 基础 URL 不能为空");
            }

            // 验证模型
            if self.model.is_empty() {
                anyhow::bail!("模型名称不能为空");
            }
        }

        Ok(())
    }

    /// 检查是否有有效的 API 密钥
    pub fn has_api_key(&self) -> bool {
        self.api_key.as_ref().map_or(false, |k| !k.is_empty())
    }

    /// 获取掩码后的 API 密钥（用于显示）
    pub fn masked_api_key(&self) -> Option<String> {
        self.api_key.as_ref().map(|key| {
            if key.len() <= 8 {
                "*".repeat(key.len())
            } else {
                format!("{}...{}", &key[..4], &key[key.len()-4..])
            }
        })
    }
}

/// AI 配置集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfigCollection {
    /// 当前活动的提供商
    pub active_provider: AIProvider,
    /// OpenAI 配置
    pub openai: AIProviderConfig,
    /// Anthropic 配置
    pub anthropic: AIProviderConfig,
    /// Google AI 配置
    pub google: AIProviderConfig,
    /// 本地模型配置
    pub local: AIProviderConfig,
    /// 自定义提供商配置
    pub custom: AIProviderConfig,
}

impl Default for AIConfigCollection {
    fn default() -> Self {
        Self {
            active_provider: AIProvider::OpenAI,
            openai: AIProviderConfig::openai(None),
            anthropic: AIProviderConfig::anthropic(None),
            google: AIProviderConfig::google(None),
            local: AIProviderConfig::local(),
            custom: AIProviderConfig::custom(String::new(), String::new(), None),
        }
    }
}

impl AIConfigCollection {
    /// 获取活动的配置
    pub fn get_active_config(&self) -> &AIProviderConfig {
        match self.active_provider {
            AIProvider::OpenAI => &self.openai,
            AIProvider::Anthropic => &self.anthropic,
            AIProvider::Google => &self.google,
            AIProvider::Local => &self.local,
            AIProvider::Custom => &self.custom,
        }
    }

    /// 获取活动的配置（可变）
    pub fn get_active_config_mut(&mut self) -> &mut AIProviderConfig {
        match self.active_provider {
            AIProvider::OpenAI => &mut self.openai,
            AIProvider::Anthropic => &mut self.anthropic,
            AIProvider::Google => &mut self.google,
            AIProvider::Local => &mut self.local,
            AIProvider::Custom => &mut self.custom,
        }
    }

    /// 根据提供商类型获取配置
    pub fn get_config(&self, provider: &AIProvider) -> &AIProviderConfig {
        match provider {
            AIProvider::OpenAI => &self.openai,
            AIProvider::Anthropic => &self.anthropic,
            AIProvider::Google => &self.google,
            AIProvider::Local => &self.local,
            AIProvider::Custom => &self.custom,
        }
    }

    /// 根据提供商类型获取配置（可变）
    pub fn get_config_mut(&mut self, provider: &AIProvider) -> &mut AIProviderConfig {
        match provider {
            AIProvider::OpenAI => &mut self.openai,
            AIProvider::Anthropic => &mut self.anthropic,
            AIProvider::Google => &mut self.google,
            AIProvider::Local => &mut self.local,
            AIProvider::Custom => &mut self.custom,
        }
    }

    /// 设置活动提供商
    pub fn set_active_provider(&mut self, provider: AIProvider) -> Result<()> {
        let config = self.get_config(&provider);
        config.validate()?;
        self.active_provider = provider;
        Ok(())
    }

    /// 更新指定提供商的 API 密钥
    pub fn update_api_key(&mut self, provider: &AIProvider, api_key: Option<String>) {
        let config = self.get_config_mut(provider);
        config.api_key = api_key.clone();
        config.enabled = api_key.is_some();
    }

    /// 验证所有启用的配置
    pub fn validate(&self) -> Result<()> {
        if self.openai.enabled {
            self.openai.validate().context("OpenAI 配置无效")?;
        }
        if self.anthropic.enabled {
            self.anthropic.validate().context("Anthropic 配置无效")?;
        }
        if self.google.enabled {
            self.google.validate().context("Google AI 配置无效")?;
        }
        if self.local.enabled {
            self.local.validate().context("本地模型配置无效")?;
        }
        if self.custom.enabled {
            self.custom.validate().context("自定义提供商配置无效")?;
        }

        Ok(())
    }

    /// 获取所有可用的提供商列表
    pub fn get_available_providers(&self) -> Vec<AIProvider> {
        let mut providers = Vec::new();

        if self.openai.has_api_key() {
            providers.push(AIProvider::OpenAI);
        }
        if self.anthropic.has_api_key() {
            providers.push(AIProvider::Anthropic);
        }
        if self.google.has_api_key() {
            providers.push(AIProvider::Google);
        }
        if self.local.enabled {
            providers.push(AIProvider::Local);
        }
        if self.custom.enabled && !self.custom.base_url.is_empty() {
            providers.push(AIProvider::Custom);
        }

        providers
    }
}

/// 用于前端显示的配置摘要（隐藏敏感信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfigSummary {
    pub active_provider: AIProvider,
    pub providers: Vec<ProviderSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
    pub provider: AIProvider,
    pub display_name: String,
    pub enabled: bool,
    pub has_api_key: bool,
    pub masked_api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

impl From<&AIConfigCollection> for AIConfigSummary {
    fn from(config: &AIConfigCollection) -> Self {
        let providers = vec![
            ProviderSummary {
                provider: AIProvider::OpenAI,
                display_name: AIProvider::OpenAI.display_name().to_string(),
                enabled: config.openai.enabled,
                has_api_key: config.openai.has_api_key(),
                masked_api_key: config.openai.masked_api_key(),
                model: config.openai.model.clone(),
                base_url: config.openai.base_url.clone(),
            },
            ProviderSummary {
                provider: AIProvider::Anthropic,
                display_name: AIProvider::Anthropic.display_name().to_string(),
                enabled: config.anthropic.enabled,
                has_api_key: config.anthropic.has_api_key(),
                masked_api_key: config.anthropic.masked_api_key(),
                model: config.anthropic.model.clone(),
                base_url: config.anthropic.base_url.clone(),
            },
            ProviderSummary {
                provider: AIProvider::Google,
                display_name: AIProvider::Google.display_name().to_string(),
                enabled: config.google.enabled,
                has_api_key: config.google.has_api_key(),
                masked_api_key: config.google.masked_api_key(),
                model: config.google.model.clone(),
                base_url: config.google.base_url.clone(),
            },
            ProviderSummary {
                provider: AIProvider::Local,
                display_name: AIProvider::Local.display_name().to_string(),
                enabled: config.local.enabled,
                has_api_key: false,
                masked_api_key: None,
                model: config.local.model.clone(),
                base_url: config.local.base_url.clone(),
            },
            ProviderSummary {
                provider: AIProvider::Custom,
                display_name: AIProvider::Custom.display_name().to_string(),
                enabled: config.custom.enabled,
                has_api_key: config.custom.has_api_key(),
                masked_api_key: config.custom.masked_api_key(),
                model: config.custom.model.clone(),
                base_url: config.custom.base_url.clone(),
            },
        ];

        Self {
            active_provider: config.active_provider.clone(),
            providers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_default() {
        let provider = AIProvider::default();
        assert_eq!(provider, AIProvider::OpenAI);
    }

    #[test]
    fn test_provider_display_name() {
        assert_eq!(AIProvider::OpenAI.display_name(), "OpenAI");
        assert_eq!(AIProvider::Anthropic.display_name(), "Anthropic");
        assert_eq!(AIProvider::Google.display_name(), "Google AI");
    }

    #[test]
    fn test_config_validation_without_key() {
        let config = AIProviderConfig {
            enabled: true,
            api_key: None,
            ..AIProviderConfig::openai(None)
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_with_key() {
        let config = AIProviderConfig::openai(Some("test-key".to_string()));
        let mut config = config;
        config.enabled = true;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_masked_api_key() {
        let config = AIProviderConfig::openai(Some("sk-1234567890abcdefghij".to_string()));
        let masked = config.masked_api_key();
        assert!(masked.is_some());
        assert!(masked.unwrap().contains("..."));
    }

    #[test]
    fn test_config_collection_default() {
        let collection = AIConfigCollection::default();
        assert_eq!(collection.active_provider, AIProvider::OpenAI);
        assert!(!collection.openai.enabled);
    }

    #[test]
    fn test_update_api_key() {
        let mut collection = AIConfigCollection::default();
        collection.update_api_key(&AIProvider::OpenAI, Some("test-key".to_string()));
        assert!(collection.openai.enabled);
        assert!(collection.openai.has_api_key());
    }

    #[test]
    fn test_get_available_providers() {
        let mut collection = AIConfigCollection::default();
        assert!(collection.get_available_providers().is_empty());

        collection.update_api_key(&AIProvider::OpenAI, Some("test-key".to_string()));
        let providers = collection.get_available_providers();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0], AIProvider::OpenAI);
    }
}
