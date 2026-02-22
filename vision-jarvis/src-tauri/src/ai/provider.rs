/// AI 提供商配置模块
///
/// 管理 AI 模型提供商、API 配置和模型选择

use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

/// AI 供应商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ProviderType {
    #[default]
    OpenAI,
    Claude,
    Gemini,
    Qwen,
    AIHubMix,
    OpenRouter,
    SiliconFlow,
}

/// AI 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AIProviderConfig {
    /// 提供商 ID (唯一标识)
    pub id: String,

    /// 提供商名称
    pub name: String,

    /// API 基础地址
    pub api_base_url: String,

    /// API Key
    pub api_key: String,

    /// 当前选择的模型
    pub model: String,

    /// 是否启用
    pub enabled: bool,

    /// 是否为当前激活的提供商
    pub is_active: bool,

    /// 供应商类型（决定使用哪种 API 格式）
    #[serde(default)]
    pub provider_type: ProviderType,
}

impl AIProviderConfig {
    /// 创建新的提供商配置
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        api_base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            api_base_url: api_base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
            enabled: true,
            is_active: false,
            provider_type: ProviderType::default(),
        }
    }

    /// 设置供应商类型（builder 模式）
    pub fn with_provider_type(self, provider_type: ProviderType) -> Self {
        Self {
            provider_type,
            ..self
        }
    }

    /// 验证配置
    pub fn validate(&self) -> AppResult<()> {
        if self.id.is_empty() {
            return Err(AppError::validation(10, "提供商 ID 不能为空"));
        }

        if self.name.is_empty() {
            return Err(AppError::validation(11, "提供商名称不能为空"));
        }

        if self.api_base_url.is_empty() {
            return Err(AppError::validation(12, "API 地址不能为空"));
        }

        if !self.api_base_url.starts_with("http://") && !self.api_base_url.starts_with("https://") {
            return Err(AppError::validation(13, "API 地址必须以 http:// 或 https:// 开头"));
        }

        if self.api_key.is_empty() {
            return Err(AppError::validation(14, "API Key 不能为空"));
        }

        if self.model.is_empty() {
            return Err(AppError::validation(15, "模型名称不能为空"));
        }

        Ok(())
    }

    /// 设置为激活状态
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}

/// 预定义的模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 模型 ID
    pub id: String,

    /// 模型显示名称
    pub name: String,

    /// 模型提供商
    pub provider: String,

    /// 是否免费
    pub is_free: bool,

    /// 模型描述
    pub description: String,
}

/// 获取所有支持的模型
pub fn get_supported_models() -> Vec<ModelInfo> {
    vec![
        // GLM 系列
        ModelInfo {
            id: "glm-5".to_string(),
            name: "GLM-5".to_string(),
            provider: "智谱 AI".to_string(),
            is_free: false,
            description: "智谱最新旗舰模型".to_string(),
        },
        ModelInfo {
            id: "glm-4.7".to_string(),
            name: "GLM-4.7".to_string(),
            provider: "智谱 AI".to_string(),
            is_free: false,
            description: "智谱高性能模型".to_string(),
        },

        // Claude 系列
        ModelInfo {
            id: "claude-opus-4-6".to_string(),
            name: "Claude Opus 4.6".to_string(),
            provider: "Anthropic".to_string(),
            is_free: false,
            description: "Claude 最强推理模型".to_string(),
        },
        ModelInfo {
            id: "claude-opus-4-6-think".to_string(),
            name: "Claude Opus 4.6 Think".to_string(),
            provider: "Anthropic".to_string(),
            is_free: false,
            description: "Claude 深度思考模式".to_string(),
        },
        ModelInfo {
            id: "claude-opus-4-5-think".to_string(),
            name: "Claude Opus 4.5 Think".to_string(),
            provider: "Anthropic".to_string(),
            is_free: false,
            description: "Claude 4.5 深度思考".to_string(),
        },
        ModelInfo {
            id: "claude-sonnet-4-5".to_string(),
            name: "Claude Sonnet 4.5".to_string(),
            provider: "Anthropic".to_string(),
            is_free: false,
            description: "Claude 平衡性能模型".to_string(),
        },

        // Gemini 系列
        ModelInfo {
            id: "gemini-3-flash-preview".to_string(),
            name: "Gemini 3 Flash Preview".to_string(),
            provider: "Google".to_string(),
            is_free: false,
            description: "Google 快速响应模型".to_string(),
        },
        ModelInfo {
            id: "gemini-3-flash-preview-free".to_string(),
            name: "Gemini 3 Flash Preview (Free)".to_string(),
            provider: "Google".to_string(),
            is_free: true,
            description: "Google 免费快速模型".to_string(),
        },

        // Kimi 系列
        ModelInfo {
            id: "kimi-k2.5".to_string(),
            name: "Kimi K2.5".to_string(),
            provider: "Moonshot AI".to_string(),
            is_free: false,
            description: "Kimi 长文本模型".to_string(),
        },

        // GPT 系列
        ModelInfo {
            id: "gpt-5.2".to_string(),
            name: "GPT-5.2".to_string(),
            provider: "OpenAI".to_string(),
            is_free: false,
            description: "OpenAI 最新模型".to_string(),
        },

        // Qwen 系列
        ModelInfo {
            id: "qwen3-max-2026-01-23".to_string(),
            name: "Qwen3 Max".to_string(),
            provider: "阿里云".to_string(),
            is_free: false,
            description: "通义千问最强模型".to_string(),
        },
        ModelInfo {
            id: "qwen3-vl-plus".to_string(),
            name: "Qwen3 VL Plus".to_string(),
            provider: "阿里云".to_string(),
            is_free: false,
            description: "通义千问视觉增强".to_string(),
        },
        ModelInfo {
            id: "qwen3-vl-flash-2026-01-22".to_string(),
            name: "Qwen3 VL Flash".to_string(),
            provider: "阿里云".to_string(),
            is_free: false,
            description: "通义千问视觉快速版".to_string(),
        },

        // Step 系列
        ModelInfo {
            id: "step-3.5-flash-free".to_string(),
            name: "Step 3.5 Flash (Free)".to_string(),
            provider: "Step AI".to_string(),
            is_free: true,
            description: "Step AI 免费模型".to_string(),
        },

        // SiliconFlow 系列
        ModelInfo {
            id: "Pro/zai-org/GLM-4.7".to_string(),
            name: "GLM-4.7 (SiliconFlow)".to_string(),
            provider: "SiliconFlow".to_string(),
            is_free: false,
            description: "SiliconFlow 托管 GLM-4.7".to_string(),
        },
    ]
}

/// AI 配置管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// 所有提供商配置
    pub providers: Vec<AIProviderConfig>,

    /// 当前激活的提供商 ID
    pub active_provider_id: Option<String>,
}

impl AIConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            active_provider_id: None,
        }
    }

    /// 添加提供商
    pub fn add_provider(&mut self, provider: AIProviderConfig) -> AppResult<()> {
        provider.validate()?;

        // 检查 ID 是否已存在
        if self.providers.iter().any(|p| p.id == provider.id) {
            return Err(AppError::validation(16, format!("提供商 ID '{}' 已存在", provider.id)));
        }

        self.providers.push(provider);
        Ok(())
    }

    /// 更新提供商
    pub fn update_provider(&mut self, provider: AIProviderConfig) -> AppResult<()> {
        provider.validate()?;

        let index = self.providers.iter().position(|p| p.id == provider.id)
            .ok_or_else(|| AppError::validation(17, format!("提供商 ID '{}' 不存在", provider.id)))?;

        self.providers[index] = provider;
        Ok(())
    }

    /// 删除提供商
    pub fn remove_provider(&mut self, provider_id: &str) -> AppResult<()> {
        let index = self.providers.iter().position(|p| p.id == provider_id)
            .ok_or_else(|| AppError::validation(18, format!("提供商 ID '{}' 不存在", provider_id)))?;

        self.providers.remove(index);

        // 如果删除的是激活的提供商，清除激活状态
        if self.active_provider_id.as_deref() == Some(provider_id) {
            self.active_provider_id = None;
        }

        Ok(())
    }

    /// 设置激活的提供商
    pub fn set_active_provider(&mut self, provider_id: &str) -> AppResult<()> {
        // 验证提供商存在
        if !self.providers.iter().any(|p| p.id == provider_id) {
            return Err(AppError::validation(19, format!("提供商 ID '{}' 不存在", provider_id)));
        }

        // 取消所有提供商的激活状态
        for provider in &mut self.providers {
            provider.is_active = false;
        }

        // 设置新的激活提供商
        if let Some(provider) = self.providers.iter_mut().find(|p| p.id == provider_id) {
            provider.is_active = true;
        }

        self.active_provider_id = Some(provider_id.to_string());
        Ok(())
    }

    /// 获取激活的提供商
    pub fn get_active_provider(&self) -> Option<&AIProviderConfig> {
        self.active_provider_id.as_ref()
            .and_then(|id| self.providers.iter().find(|p| &p.id == id))
    }

    /// 获取提供商
    pub fn get_provider(&self, provider_id: &str) -> Option<&AIProviderConfig> {
        self.providers.iter().find(|p| p.id == provider_id)
    }
}

impl Default for AIConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_creation() {
        let config = AIProviderConfig::new(
            "test-1",
            "Test Provider",
            "https://api.example.com",
            "test-key",
            "test-model",
        );

        assert_eq!(config.id, "test-1");
        assert_eq!(config.name, "Test Provider");
        assert!(config.enabled);
        assert!(!config.is_active);
    }

    #[test]
    fn test_provider_validation() {
        let mut config = AIProviderConfig::new(
            "test-1",
            "Test Provider",
            "https://api.example.com",
            "test-key",
            "test-model",
        );

        assert!(config.validate().is_ok());

        // 测试空 ID
        config.id = String::new();
        assert!(config.validate().is_err());

        // 测试无效 URL
        config.id = "test-1".to_string();
        config.api_base_url = "invalid-url".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ai_config_add_provider() {
        let mut config = AIConfig::new();
        let provider = AIProviderConfig::new(
            "test-1",
            "Test Provider",
            "https://api.example.com",
            "test-key",
            "test-model",
        );

        assert!(config.add_provider(provider.clone()).is_ok());
        assert_eq!(config.providers.len(), 1);

        // 测试重复 ID
        assert!(config.add_provider(provider).is_err());
    }

    #[test]
    fn test_ai_config_set_active() {
        let mut config = AIConfig::new();
        let provider = AIProviderConfig::new(
            "test-1",
            "Test Provider",
            "https://api.example.com",
            "test-key",
            "test-model",
        );

        config.add_provider(provider).unwrap();
        assert!(config.set_active_provider("test-1").is_ok());
        assert_eq!(config.active_provider_id, Some("test-1".to_string()));

        let active = config.get_active_provider().unwrap();
        assert!(active.is_active);
    }

    #[test]
    fn test_ai_config_remove_provider() {
        let mut config = AIConfig::new();
        let provider = AIProviderConfig::new(
            "test-1",
            "Test Provider",
            "https://api.example.com",
            "test-key",
            "test-model",
        );

        config.add_provider(provider).unwrap();
        config.set_active_provider("test-1").unwrap();

        assert!(config.remove_provider("test-1").is_ok());
        assert_eq!(config.providers.len(), 0);
        assert!(config.active_provider_id.is_none());
    }

    #[test]
    fn test_get_supported_models() {
        let models = get_supported_models();
        assert!(!models.is_empty());

        // 验证包含关键模型
        assert!(models.iter().any(|m| m.id == "claude-opus-4-6"));
        assert!(models.iter().any(|m| m.id == "glm-5"));
        assert!(models.iter().any(|m| m.is_free));
    }

    #[test]
    fn test_serialization() {
        let config = AIProviderConfig::new(
            "test-1",
            "Test Provider",
            "https://api.example.com",
            "test-key",
            "test-model",
        );

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AIProviderConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }
}
