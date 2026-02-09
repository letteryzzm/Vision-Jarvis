# AI Providers Service

多AI提供商配置管理服务

## 概述

AI Providers Service 提供统一的多AI提供商配置管理，支持OpenAI、Anthropic、Google AI、Local (Ollama) 和 Custom 提供商。

## 位置

- **模块路径**: `src-tauri/src/ai/providers.rs`
- **行数**: 480 lines
- **测试**: 7 个单元测试

## 核心类型

### AIProvider

AI提供商枚举，支持5种提供商：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AIProvider {
    OpenAI,      // OpenAI GPT models
    Anthropic,   // Claude models
    Google,      // Gemini models
    Local,       // Ollama (local models)
    Custom,      // 自定义提供商
}
```

### AIProviderConfig

单个提供商的配置：

```rust
pub struct AIProviderConfig {
    pub provider: AIProvider,
    pub api_key: Option<SecretString>,  // 使用 secrecy crate 保护
    pub base_url: String,
    pub model: String,
    pub enabled: bool,
}
```

**安全特性**:
- `api_key` 使用 `SecretString` 类型，内存中加密存储
- 序列化时 API key 会被遮蔽显示为 "***"

### AIConfigCollection

所有提供商的配置集合：

```rust
pub struct AIConfigCollection {
    configs: HashMap<AIProvider, AIProviderConfig>,
    active_provider: Option<AIProvider>,
}
```

### AIConfigSummary

配置摘要（用于前端显示）：

```rust
pub struct AIConfigSummary {
    pub active_provider: Option<AIProvider>,
    pub providers: Vec<ProviderSummary>,
}

pub struct ProviderSummary {
    pub provider: AIProvider,
    pub display_name: String,
    pub has_api_key: bool,  // 不暴露实际key
    pub enabled: bool,
    pub model: String,
}
```

## AIProvider 方法

### display_name

```rust
pub fn display_name(&self) -> &str
```

获取提供商的显示名称。

**返回值**:
- `OpenAI` → "OpenAI"
- `Anthropic` → "Anthropic Claude"
- `Google` → "Google AI"
- `Local` → "Ollama (Local)"
- `Custom` → "Custom Provider"

### default_config

```rust
pub fn default_config(&self) -> AIProviderConfig
```

获取提供商的默认配置。

**默认配置**:

| Provider | Base URL | Default Model |
|----------|----------|---------------|
| OpenAI | https://api.openai.com/v1 | gpt-4 |
| Anthropic | https://api.anthropic.com | claude-3-5-sonnet-20241022 |
| Google | https://generativelanguage.googleapis.com/v1 | gemini-2.0-flash-exp |
| Local | http://localhost:11434/api | llama2 |
| Custom | http://localhost:8080 | custom-model |

## AIProviderConfig 方法

### validate

```rust
pub fn validate(&self) -> Result<()>
```

验证配置的有效性。

**检查项**:
- base_url 不能为空
- model 不能为空
- 如果不是 Local 或 Custom，api_key 必须存在

**示例**:
```rust
let config = AIProviderConfig {
    provider: AIProvider::OpenAI,
    api_key: Some(SecretString::new("sk-...".to_string())),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    enabled: true,
};

config.validate()?; // 验证通过
```

### has_api_key

```rust
pub fn has_api_key(&self) -> bool
```

检查是否已设置API密钥。

### masked_api_key

```rust
pub fn masked_api_key(&self) -> String
```

获取遮蔽后的API密钥（用于显示）。

**返回**:
- 如果有 API key: `"sk-***"`（保留前缀）
- 如果没有: `"未设置"`

## AIConfigCollection 方法

### default

```rust
impl Default for AIConfigCollection
```

创建默认配置集合：
- 包含所有5个提供商的默认配置
- OpenAI 设为默认活动提供商
- 所有提供商默认 enabled=true

### get_config / get_config_mut

```rust
pub fn get_config(&self, provider: &AIProvider) -> &AIProviderConfig
pub fn get_config_mut(&mut self, provider: &AIProvider) -> &mut AIProviderConfig
```

获取指定提供商的配置（可变/不可变）。

### update_api_key

```rust
pub fn update_api_key(&mut self, provider: &AIProvider, api_key: Option<String>)
```

更新指定提供商的API密钥。

**参数**:
- `provider`: 提供商
- `api_key`: 新的API密钥（None表示清空）

**示例**:
```rust
let mut config = AIConfigCollection::default();
config.update_api_key(
    &AIProvider::OpenAI,
    Some("sk-new-key".to_string())
);
```

### set_active_provider

```rust
pub fn set_active_provider(&mut self, provider: AIProvider) -> Result<()>
```

设置活动的提供商。

**验证**:
- 提供商必须已启用（enabled=true）
- 提供商配置必须有效（validate通过）

**示例**:
```rust
config.set_active_provider(AIProvider::Anthropic)?;
```

### get_available_providers

```rust
pub fn get_available_providers(&self) -> Vec<AIProvider>
```

获取所有已启用的提供商列表。

**返回**: 所有 `enabled=true` 的提供商

### validate

```rust
pub fn validate(&self) -> Result<()>
```

验证整个配置集合。

**检查项**:
- 至少有一个提供商已启用
- 如果设置了活动提供商，该提供商必须已启用

## AIConfigSummary 方法

### from

```rust
impl From<&AIConfigCollection> for AIConfigSummary
```

从 AIConfigCollection 创建摘要（用于前端显示）。

**特点**:
- 不暴露实际的API密钥
- 只显示 `has_api_key` 布尔值
- 包含所有提供商的基本信息

## 安全特性

### API 密钥保护

1. **内存保护**: 使用 `secrecy::SecretString` 存储API密钥
   ```rust
   pub api_key: Option<SecretString>
   ```

2. **序列化保护**: 序列化时自动遮蔽
   ```rust
   #[serde(serialize_with = "serialize_secret_option")]
   pub api_key: Option<SecretString>
   ```

3. **显示保护**: `masked_api_key()` 方法只显示前缀
   ```rust
   "sk-***"  // 而不是完整密钥
   ```

### 配置验证

所有配置更新都经过验证：
- `validate()` 方法检查配置完整性
- `set_active_provider()` 验证提供商可用性
- 防止设置无效配置

## 单元测试

测试覆盖：

1. `test_provider_display_name` - 提供商显示名称
2. `test_provider_default` - 默认配置
3. `test_config_validation_with_key` - 配置验证（有key）
4. `test_config_validation_without_key` - 配置验证（无key）
5. `test_masked_api_key` - API密钥遮蔽
6. `test_update_api_key` - 更新API密钥
7. `test_config_collection_default` - 配置集合默认值
8. `test_get_available_providers` - 获取可用提供商

## 使用示例

### 基本配置管理

```rust
use vision_jarvis::ai::providers::*;
use secrecy::SecretString;

// 创建默认配置
let mut config = AIConfigCollection::default();

// 更新 OpenAI API key
config.update_api_key(
    &AIProvider::OpenAI,
    Some("sk-your-key".to_string())
);

// 切换到 Anthropic
config.set_active_provider(AIProvider::Anthropic)?;

// 获取活动配置
let active = config.get_active_config();
println!("Using: {}", active.provider.display_name());
println!("Model: {}", active.model);

// 获取摘要（用于前端显示）
let summary = AIConfigSummary::from(&config);
for provider in summary.providers {
    println!("{}: {} (key: {})",
        provider.display_name,
        if provider.enabled { "enabled" } else { "disabled" },
        if provider.has_api_key { "set" } else { "not set" }
    );
}
```

### 配置验证

```rust
// 验证单个配置
let config = AIProviderConfig {
    provider: AIProvider::OpenAI,
    api_key: Some(SecretString::new("sk-key".to_string())),
    base_url: "https://api.openai.com/v1".to_string(),
    model: "gpt-4".to_string(),
    enabled: true,
};

match config.validate() {
    Ok(_) => println!("配置有效"),
    Err(e) => println!("配置无效: {}", e),
}

// 验证整个集合
let collection = AIConfigCollection::default();
collection.validate()?;
```

## 最佳实践

1. **API密钥管理**:
   - 使用环境变量或安全存储
   - 不要在代码中硬编码
   - 定期轮换密钥

2. **配置验证**:
   - 更新配置后立即调用 `validate()`
   - 设置活动提供商前检查可用性

3. **错误处理**:
   - 正确处理 `set_active_provider` 的错误
   - 提供用户友好的错误信息

4. **安全显示**:
   - 使用 `AIConfigSummary` 向前端传递配置
   - 永远不要直接暴露 `SecretString`

## 相关文档

- [AI Config Commands](../../api/endpoints/ai-config.md) - Tauri commands for AI config
- [AI Config API](../../api/endpoints/ai-config.md) - Frontend API reference
- [Security Guidelines](../security.md) - API key security best practices
