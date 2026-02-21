# AI Providers Service

> **最后更新**: 2026-02-21
> **版本**: v3.0 — Provider 工厂模式
> **实现状态**: 已完成

## 概述

AI Providers Service 通过 **Provider 工厂模式** 支持多种 AI 供应商的原生 API 格式。每个供应商一个独立文件，`AIClient` 作为 facade 对外提供统一接口。

## 位置

- **trait**: `src-tauri/src/ai/traits.rs`
- **工厂**: `src-tauri/src/ai/factory.rs`
- **facade**: `src-tauri/src/ai/client.rs`
- **配置**: `src-tauri/src/ai/provider.rs`
- **实现**: `src-tauri/src/ai/providers/`

## 核心类型

### ProviderType 枚举

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ProviderType {
    #[default]
    OpenAI,
    Claude,
    Gemini,
    Qwen,
    AIHubMix,
    OpenRouter,
}
```

### AIProviderConfig

```rust
pub struct AIProviderConfig {
    pub id: String,
    pub name: String,
    pub api_base_url: String,
    pub api_key: String,
    pub model: String,
    pub enabled: bool,
    pub is_active: bool,
    #[serde(default)]
    pub provider_type: ProviderType,
}
```

**向后兼容**: `provider_type` 使用 `#[serde(default)]`，旧数据没有此字段时自动默认为 `OpenAI`。

### AIProvider trait

```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn send_text(&self, prompt: &str) -> AppResult<String>;
    async fn analyze_video(&self, video_base64: &str, prompt: &str) -> AppResult<String>;
    async fn analyze_image(&self, image_base64: &str, prompt: &str) -> AppResult<String>;
    async fn test_connection(&self) -> AppResult<String>;
    fn config(&self) -> &AIProviderConfig;
}
```

## Provider 实现对比

| Provider | 端点 | 认证 | 视频处理 |
|----------|------|------|---------|
| `OpenAIProvider` | `/v1/chat/completions` | `Bearer {key}` | image_url + data URL |
| `ClaudeProvider` | `/v1/messages` | `x-api-key` + `anthropic-version` | 帧提取 → 多图 |
| `GeminiProvider` | `/v1beta/models/{m}:generateContent` | `x-goog-api-key` | 原生 inline_data |
| `QwenProvider` | `/v1/chat/completions` | `Bearer {key}` | image_url + data URL |
| `AIHubMixProvider` | `/v1/chat/completions` | `Bearer {key}` | image_url + data URL |
| `OpenRouterProvider` | `/v1/chat/completions` | `Bearer {key}` + `X-Title` + `HTTP-Referer` | image_url + data URL |

## 使用示例

### 创建客户端

```rust
use vision_jarvis_lib::ai::*;

let config = AIProviderConfig::new(
    "gemini-direct", "Gemini Direct",
    "https://generativelanguage.googleapis.com",
    "api-key", "gemini-3-flash-preview",
).with_provider_type(ProviderType::Gemini);

let client = AIClient::new(config)?;
let result = client.analyze_video(&video_b64, &prompt).await?;
```

### 通过 AIHubMix 代理

```rust
let config = AIProviderConfig::new(
    "aihubmix", "AIHubMix",
    "https://api.aihubmix.com",
    "api-key", "claude-opus-4-6",
).with_provider_type(ProviderType::AIHubMix);

let client = AIClient::new(config)?;
```

## 配置 JSON 格式

```json
{
  "id": "gemini-direct",
  "name": "Gemini Direct",
  "api_base_url": "https://generativelanguage.googleapis.com",
  "api_key": "your-key",
  "model": "gemini-3-flash-preview",
  "enabled": true,
  "is_active": true,
  "provider_type": "Gemini"
}
```

## 安全特性

- API key 仅用于 HTTP 请求头，不出现在日志中
- 配置验证: URL 格式、必填字段检查
- 错误消息不泄露敏感信息

## 相关文档

- [AI Service](ai-service.md) — 架构详解
- [AI Module](../../vision-jarvis/src-tauri/AI_MODULE.md) — 实现文档
