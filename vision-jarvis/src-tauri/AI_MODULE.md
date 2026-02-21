# AI 模块实现文档

**日期**: 2026-02-21
**状态**: Provider 工厂模式重构完成，支持 6 种供应商原生 API 格式

---

## 架构设计

```
AIClient (facade, 对外接口不变)
  └── Box<dyn AIProvider>  (由 factory 根据 ProviderType 创建)
        ├── OpenAIProvider       — /v1/chat/completions, Bearer auth
        ├── ClaudeProvider       — /v1/messages, x-api-key auth, 帧提取视频
        ├── GeminiProvider       — /v1beta/models/{m}:generateContent, inline_data 视频
        ├── QwenProvider         — DashScope OpenAI 兼容格式
        ├── AIHubMixProvider     — OpenAI 兼容代理, image_url data URL 视频
        └── OpenRouterProvider   — OpenAI 兼容代理, X-Title/HTTP-Referer 头
```

**调用方零改动**: `screenshot_analyzer.rs`、`summary_generator.rs`、`markdown_generator.rs`、`pipeline.rs`、`ai_config.rs` 中所有 `Arc<AIClient>` 用法不变。

---

## 核心类型

### ProviderType — 供应商类型枚举

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

### AIProviderConfig — 提供商配置

```rust
pub struct AIProviderConfig {
    pub id: String,
    pub name: String,
    pub api_base_url: String,
    pub api_key: String,
    pub model: String,
    pub enabled: bool,
    pub is_active: bool,
    pub provider_type: ProviderType,  // serde(default) 保证旧数据兼容
}
```

### AIProvider trait — 统一接口

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

### AIClient — Facade

```rust
pub struct AIClient {
    inner: Box<dyn AIProvider>,
}

impl AIClient {
    pub fn new(config: AIProviderConfig) -> AppResult<Self>;
    pub async fn analyze_image(&self, ...) -> AppResult<String>;
    pub async fn analyze_video(&self, ...) -> AppResult<String>;
    pub async fn send_text(&self, ...) -> AppResult<String>;
    pub async fn test_connection(&self) -> AppResult<String>;
    pub fn config(&self) -> &AIProviderConfig;
}
```

---

## 各 Provider 视频处理策略

| Provider | 视频处理方式 | 格式 |
|----------|------------|------|
| OpenAI | `image_url` data URL | `data:video/mp4;base64,...` |
| Claude | **帧提取** (ffmpeg 5帧 → 多图) | `image` source base64 |
| Gemini | **原生 `inline_data`** | `{ mime_type, data }` |
| Qwen | `image_url` data URL | `data:video/mp4;base64,...` |
| AIHubMix | `image_url` data URL (代理转发) | `data:video/mp4;base64,...` |
| OpenRouter | `image_url` data URL (代理转发) | `data:video/mp4;base64,...` |

### 帧提取模块 (`frame_extractor.rs`)

对不支持原生视频分析的 Provider (如 Claude)，自动:
1. 将 base64 视频写入临时文件
2. 使用 ffprobe 获取视频时长
3. 用 ffmpeg 均匀提取 N 帧 (默认 5 帧)
4. 缩放到 1280px 宽，JPEG 质量 3
5. 返回 `Vec<String>` base64 编码帧图像

---

## 文件结构

```
src/ai/
  mod.rs                    — 模块声明和导出
  provider.rs               — ProviderType 枚举 + AIProviderConfig
  client.rs                 — AIClient facade
  traits.rs                 — AIProvider trait
  factory.rs                — create_provider() 工厂函数
  prompt.rs                 — Prompt 模板系统
  frame_extractor.rs        — 视频帧提取工具
  providers/
    mod.rs                  — 导出所有 provider
    openai.rs               — OpenAI 原生实现
    claude.rs               — Anthropic Claude 实现
    gemini.rs               — Google Gemini 实现
    qwen.rs                 — 阿里云 Qwen 实现
    aihubmix.rs             — AIHubMix 代理实现
    openrouter.rs           — OpenRouter 代理实现
```

---

## 各 Provider 认证方式

| Provider | 端点 | 认证头 |
|----------|------|-------|
| OpenAI | `{base_url}/v1/chat/completions` | `Authorization: Bearer {key}` |
| Claude | `{base_url}/v1/messages` | `x-api-key: {key}` + `anthropic-version: 2023-06-01` |
| Gemini | `{base_url}/v1beta/models/{model}:generateContent` | `x-goog-api-key: {key}` |
| Qwen | `{base_url}/v1/chat/completions` | `Authorization: Bearer {key}` |
| AIHubMix | `{base_url}/v1/chat/completions` | `Authorization: Bearer {key}` |
| OpenRouter | `{base_url}/v1/chat/completions` | `Authorization: Bearer {key}` + `X-Title` + `HTTP-Referer` |

---

## 支持的模型

| 模型 ID | 名称 | 提供商 | 免费 |
|---------|------|--------|------|
| glm-5 | GLM-5 | 智谱 AI | No |
| claude-opus-4-6 | Claude Opus 4.6 | Anthropic | No |
| claude-sonnet-4-5 | Claude Sonnet 4.5 | Anthropic | No |
| gemini-3-flash-preview | Gemini 3 Flash Preview | Google | No |
| gemini-3-flash-preview-free | Gemini 3 Flash (Free) | Google | Yes |
| gpt-5.2 | GPT-5.2 | OpenAI | No |
| qwen3-max-2026-01-23 | Qwen3 Max | Alibaba | No |
| qwen3-vl-plus | Qwen3 VL Plus | Alibaba | No |
| step-3.5-flash-free | Step 3.5 Flash (Free) | Step AI | Yes |

---

## 配置示例

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

**向后兼容**: 旧配置没有 `provider_type` 字段时自动默认为 `OpenAI`（与旧行为一致）。

---

## 测试

- 126 个单元测试全部通过
- `cargo check` / `cargo build` 零警告
