# AI 服务

> **最后更新**: 2026-02-22
> **版本**: v3.1 — 新增 SiliconFlow + video_model
> **实现状态**: Provider 工厂模式重构完成

**实现文件**:
- `src-tauri/src/ai/mod.rs` — 模块声明与导出
- `src-tauri/src/ai/traits.rs` — `AIProvider` async trait
- `src-tauri/src/ai/factory.rs` — `create_provider()` 工厂函数
- `src-tauri/src/ai/client.rs` — `AIClient` facade
- `src-tauri/src/ai/provider.rs` — `AIProviderConfig`、`ProviderType`、`AIConfig`
- `src-tauri/src/ai/prompt.rs` — Prompt 模板系统
- `src-tauri/src/ai/frame_extractor.rs` — 视频帧提取（ffmpeg）
- `src-tauri/src/ai/providers/` — 7 个供应商独立实现

---

## 架构概述

```
AIClient (facade)
  └── Box<dyn AIProvider>  ← factory 根据 ProviderType 创建
        ├── OpenAIProvider       — /v1/chat/completions, Bearer auth
        ├── ClaudeProvider       — /v1/messages, x-api-key auth
        ├── GeminiProvider       — generateContent, inline_data
        ├── QwenProvider         — OpenAI 兼容
        ├── AIHubMixProvider     — OpenAI 兼容代理
        ├── OpenRouterProvider   — OpenAI 兼容 + 额外头
        └── SiliconFlowProvider  — OpenAI 兼容 + 原生 video_url
```

**设计原则**: 调用方（`screenshot_analyzer.rs`、`summary_generator.rs` 等）只依赖 `AIClient` 公共 API，不感知底层 Provider 差异。

---

## Provider 对比

| Provider | 端点格式 | 认证方式 | 视频处理 |
|----------|---------|---------|---------|
| OpenAI | `/v1/chat/completions` | Bearer token | image_url data URL |
| Claude | `/v1/messages` | x-api-key + anthropic-version | 帧提取 → 多图 |
| Gemini | `/v1beta/models/{m}:generateContent` | x-goog-api-key | 原生 inline_data |
| Qwen | `/v1/chat/completions` | Bearer token | image_url data URL |
| AIHubMix | `/v1/chat/completions` | Bearer token | image_url data URL |
| OpenRouter | `/v1/chat/completions` | Bearer + X-Title + HTTP-Referer | image_url data URL |
| SiliconFlow | `/v1/chat/completions` | Bearer token | video_url (原生) / image_url |

---

## 视频分析策略

### 原生视频支持 (Gemini)

Gemini 直接接受 `inline_data` 格式的视频数据:

```json
{
  "contents": [{
    "parts": [
      { "text": "分析视频" },
      { "inline_data": { "mime_type": "video/mp4", "data": "<base64>" } }
    ]
  }]
}
```

### OpenAI 兼容代理 (AIHubMix / OpenRouter / Qwen)

通过 `image_url` 类型 + data URL 传递视频:

```json
{
  "content": [
    { "type": "text", "text": "分析视频" },
    { "type": "image_url", "image_url": { "url": "data:video/mp4;base64,..." } }
  ]
}
```

### 原生 video_url (SiliconFlow)

SiliconFlow 支持 `video_url` 类型直接传递视频:

```json
{
  "content": [
    { "type": "text", "text": "分析视频" },
    { "type": "video_url", "video_url": { "url": "data:video/mp4;base64,...", "max_frames": 10, "fps": 2.0 } }
  ]
}
```

### 帧提取 (Claude)

Claude API 不支持原生视频。自动使用 ffmpeg 帧提取:

1. base64 视频 → 临时 mp4 文件
2. ffprobe 获取时长
3. ffmpeg 均匀提取 5 帧 (1280px, JPEG)
4. 多帧作为独立图片发送给 Claude Vision API

---

## 工厂函数

```rust
pub fn create_provider(config: AIProviderConfig) -> AppResult<Box<dyn AIProvider>> {
    config.validate()?;
    match config.provider_type {
        ProviderType::OpenAI     => Ok(Box::new(OpenAIProvider::new(config)?)),
        ProviderType::Claude     => Ok(Box::new(ClaudeProvider::new(config)?)),
        ProviderType::Gemini     => Ok(Box::new(GeminiProvider::new(config)?)),
        ProviderType::Qwen       => Ok(Box::new(QwenProvider::new(config)?)),
        ProviderType::AIHubMix   => Ok(Box::new(AIHubMixProvider::new(config)?)),
        ProviderType::OpenRouter => Ok(Box::new(OpenRouterProvider::new(config)?)),
        ProviderType::SiliconFlow => Ok(Box::new(SiliconFlowProvider::new(config)?)),
    }
}
```

---

## 错误处理

所有 Provider 统一的 HTTP 错误映射:

| HTTP 状态码 | 错误类型 |
|------------|---------|
| 401 | API Key 无效或未授权 |
| 403 | 访问被拒绝 |
| 404 | API 端点不存在 |
| 429 | 请求过于频繁 |
| 500-599 | 服务器错误 |
| 超时 | 请求超时 (120s) |
| 连接失败 | 网络连接失败 |

---

## 测试覆盖

- Provider 配置: 8 tests
- Client facade: 2 tests
- Prompt 模板: 8 tests
- 集成测试: 4 tests

**总计**: 22 个测试全部通过

---

## 相关文档

- [AI 模块实现文档](../../vision-jarvis/src-tauri/AI_MODULE.md)
- [Memory Service](memory-service.md) — 使用 `AIClient.analyze_video()` 的 `ScreenshotAnalyzer`
