# 代码文件功能映射 (CODEMAP)

> 最后更新: 2026-02-19
> 当前架构: V3 主动式 AI 记忆系统

---

## 后端 (`vision-jarvis/src-tauri/src/`)

### 入口

| 文件 | 功能 |
|------|------|
| `main.rs` | 应用入口，启动 Tauri |
| `lib.rs` | 模块注册、Tauri 插件配置、AppState 初始化 |
| `error.rs` | 统一错误类型 `AppError` |

---

### `commands/` — Tauri IPC 命令层（前后端通信接口）

| 文件 | 功能 |
|------|------|
| `mod.rs` | `AppState` 定义、`ApiResponse<T>` 通用响应结构 |
| `screenshot.rs` | 截图相关命令：触发截图、查询截图列表 |
| `memory.rs` | 记忆相关命令：查询活动、总结、项目、习惯 |
| `notification.rs` | 通知相关命令：查询通知、标记已读 |
| `settings.rs` | 设置相关命令：读写用户配置 |
| `storage.rs` | 文件存储命令：管理本地文件 |
| `ai_config.rs` | AI 配置命令：配置 AI 提供商、API Key |
| `window.rs` | 窗口管理命令：控制悬浮球/弹窗窗口 |

---

### `memory/` — 记忆系统（V3 主动式 AI 记忆）

| 文件 | 功能 |
|------|------|
| `mod.rs` | 模块声明 |
| `pipeline.rs` | **记忆管道调度器**：统一调度所有记忆任务（截图分析5min、活动分组30min、索引同步10min、习惯检测每日、日总结23:00） |
| `screenshot_analyzer.rs` | **截图 AI 分析器**：将截图发给 AI 理解，提取应用名、活动类型、标签，存入 `screenshot_analyses` 表 |
| `activity_grouper.rs` | **活动分组器**：将连续相似截图聚合为 `ActivitySession`，按应用名/活动类型/时间间隔判断归属 |
| `summary_generator.rs` | **总结生成器**：聚合活动数据，调用 AI 生成日/周/月总结，存入 `summaries` 表和 Markdown 文件 |
| `project_extractor.rs` | **项目提取器**：从活动中自动识别项目，相似度匹配现有项目或创建新项目，维护项目 Markdown 文件 |
| `habit_detector.rs` | **习惯检测器**：从活动历史识别时间模式、触发模式、序列模式，存入 `habits` 表 |
| `markdown_generator.rs` | Markdown 文件生成器：为活动/项目/总结生成结构化 Markdown |
| `index_manager.rs` | 文件索引管理器：增量索引本地 Markdown 文件 |
| `vector_store.rs` | 向量存储：管理文本嵌入向量，支持语义搜索 |
| `hybrid_search.rs` | 混合搜索：结合关键词搜索和向量语义搜索 |
| `chunker.rs` | 文本分块器：将长文本切分为适合嵌入的块 |
| `short_term.rs` | 短期记忆管理（V2 遗留） |
| `long_term.rs` | 长期记忆管理（V2 遗留） |
| `scheduler.rs` | 旧版记忆调度器（V2 遗留，已被 pipeline.rs 替代） |

---

### `notification/` — 通知系统

| 文件 | 功能 |
|------|------|
| `mod.rs` | `Notification` 结构体、`NotificationType`、`NotificationPriority` |
| `scheduler.rs` | 通知调度器：定时评估规则并推送通知 |
| `rules.rs` | 通知规则接口 `NotificationRule`、`RuleContext` |
| `context.rs` | 规则上下文构建：从 DB 聚合当前状态（工作时长、习惯匹配等） |
| `delivery.rs` | 通知投递：通过 Tauri 事件推送到前端 |
| `smart/mod.rs` | 智能提醒模块声明 |
| `smart/proactive.rs` | **主动建议规则 V3**：习惯提醒、上下文切换警告、智能休息提醒、项目进度提醒 |

---

### `capture/` — 截图采集

| 文件 | 功能 |
|------|------|
| `mod.rs` | `ScreenCapture`：截图采集、图片压缩存储 |
| `scheduler.rs` | `CaptureScheduler`：定时截图调度 |
| `storage.rs` | 截图文件存储管理 |

---

### `ai/` — AI 客户端（Provider 工厂模式）

| 文件 | 功能 |
|------|------|
| `mod.rs` | 模块声明与公共接口导出 |
| `client.rs` | `AIClient` facade，委托给 `Box<dyn AIProvider>` |
| `traits.rs` | `AIProvider` async trait 定义（send_text / analyze_video / analyze_image / test_connection） |
| `factory.rs` | `create_provider()` 工厂函数，根据 `ProviderType` 创建具体 Provider |
| `provider.rs` | `AIProviderConfig`、`ProviderType` 枚举、`AIConfig` 配置管理 |
| `prompt.rs` | Prompt 模板管理 |
| `frame_extractor.rs` | 视频帧提取工具，使用 ffmpeg 为不支持原生视频的 Provider 预处理 |
| `providers/mod.rs` | 各供应商 Provider 导出 |
| `providers/openai.rs` | OpenAI 原生 Provider（`/v1/chat/completions`，Bearer auth） |
| `providers/claude.rs` | Anthropic Claude Provider（`/v1/messages`，`x-api-key` auth，帧提取视频处理） |
| `providers/gemini.rs` | Google Gemini Provider（`/v1beta/models/{m}:generateContent`，原生 `inline_data` 视频） |
| `providers/qwen.rs` | 阿里云 Qwen Provider（DashScope OpenAI 兼容格式） |
| `providers/aihubmix.rs` | AIHubMix 代理 Provider（OpenAI 兼容） |
| `providers/openrouter.rs` | OpenRouter 代理 Provider（OpenAI 兼容 + X-Title/HTTP-Referer 头） |

---

### `db/` — 数据库层

| 文件 | 功能 |
|------|------|
| `mod.rs` | `Database` 连接池封装 |
| `schema.rs` | 所有数据表结构体定义（`ActivitySession`、`Habit`、`Project`、`Summary` 等） |
| `migrations.rs` | SQLite 数据库迁移脚本 |

---

### `settings/` — 设置管理

| 文件 | 功能 |
|------|------|
| `mod.rs` | `SettingsManager` 公共接口 |
| `config.rs` | 配置读写、持久化（截图间隔、存储路径、AI 配置等） |

---

### `storage/` — 文件存储

| 文件 | 功能 |
|------|------|
| `mod.rs` | 本地文件存储管理（截图文件、Markdown 文件路径管理） |

---

## 前端 (`vision-jarvis/src/`)

| 文件 | 功能 |
|------|------|
| `lib/tauri-api.ts` | Tauri IPC 调用封装，前端调用后端命令的统一入口 |
| `stores/settingsStore.ts` | 设置状态管理（Nanostores） |
| `types/settings.ts` | 设置相关 TypeScript 类型定义 |

---

## 数据库表（`db/schema.rs` 中定义）

| 表名 | 用途 |
|------|------|
| `screenshots` | 截图记录（路径、时间、应用名） |
| `screenshot_analyses` | AI 分析结果（活动类型、标签、置信度） |
| `activity_sessions` | 聚合后的活动会话 |
| `projects` | 自动识别的项目 |
| `habits` | 检测到的用户习惯模式 |
| `summaries` | 日/周/月总结 |
| `notifications` | 通知历史记录 |
| `app_usage` | 应用使用统计 |
| `short_term_memory` | 短期记忆（V2 遗留） |

---

## 架构数据流（V3）

```
截图采集 (capture/)
    ↓ 每5分钟
截图AI分析 (memory/screenshot_analyzer.rs)
    ↓ 每30分钟
活动分组 (memory/activity_grouper.rs)
    ↓ 并行
├── 项目提取 (memory/project_extractor.rs)
├── 习惯检测 (memory/habit_detector.rs) [每日]
├── 日总结生成 (memory/summary_generator.rs) [23:00]
└── 文件索引 (memory/index_manager.rs) [每10分钟]
    ↓
通知规则评估 (notification/smart/proactive.rs)
    ↓
前端推送 (notification/delivery.rs)
```

---

## 文档与代码对应关系

| 文档 | 对应代码 | 状态 |
|------|---------|------|
| `docs/backend/services/memory-service.md` | `memory/pipeline.rs` + 各子模块 | ⚠️ 文档描述旧架构，需更新 |
| `docs/backend/services/notification-service.md` | `notification/` | ⚠️ 缺少 V3 主动建议规则 |
| `docs/backend/services/screenshot-service.md` | `capture/` | ✅ 基本对应 |
| `docs/backend/services/ai-service.md` | `ai/` | ✅ 已更新至 Provider 工厂模式 |
| `docs/api/endpoints/ai-config.md` | `commands/ai_config.rs` | ✅ 对应 |
| `docs/api/endpoints/storage.md` | `commands/storage.rs` | ✅ 对应 |
| `docs/api/endpoints/commands.md` | `commands/` 全部 | ⚠️ 缺少 memory、settings、window 命令 |
