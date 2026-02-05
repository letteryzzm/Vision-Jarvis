# 后端架构概述

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **架构模式**: 分层架构 + 服务化

---

## 目录

- [架构概述](#架构概述)
- [核心设计原则](#核心设计原则)
- [系统架构图](#系统架构图)
- [模块划分](#模块划分)
- [数据流设计](#数据流设计)
- [状态管理](#状态管理)
- [错误处理](#错误处理)

---

## 架构概述

Vision-Jarvis 后端采用**分层架构 + 服务化**的设计模式，基于 Rust 和 Tauri 框架构建。系统核心包含 4 个功能模块：

1. **屏幕截图采集模块** - 持续记录用户屏幕活动
2. **AI 内容分析模块** - 智能解析截图内容
3. **短期记忆生成模块** - 构建用户行为记忆
4. **主动推送提醒模块** - 智能提醒和建议

---

## 核心设计原则

### 1. 分层架构

```
┌─────────────────────────────────────┐
│     Presentation Layer (IPC)        │  Tauri Commands API
├─────────────────────────────────────┤
│      Service Layer (Business)       │  核心业务逻辑
├─────────────────────────────────────┤
│     Data Access Layer (DAL)         │  数据库抽象
├─────────────────────────────────────┤
│      Infrastructure Layer           │  第三方服务 (AI API, OCR)
└─────────────────────────────────────┘
```

**职责划分**:
- **Presentation Layer**: 处理前后端通信，参数验证
- **Service Layer**: 核心业务逻辑，编排服务
- **Data Access Layer**: 数据库操作抽象
- **Infrastructure Layer**: 外部依赖集成

### 2. 服务化设计

每个功能模块封装为独立服务：

| 服务 | 职责 | 依赖 |
|------|------|------|
| ScreenshotService | 截图采集、压缩、存储 | 系统 API, ImageProcessor |
| AIService | OCR、内容分析 | Claude API, OCR Service |
| MemoryService | 短期记忆生成、意图识别 | AIService, Database |
| NotificationService | 推送提醒、行为分析 | MemoryService, Database |

### 3. 异步优先

所有 I/O 操作使用异步模式：
- 数据库操作: `sqlx` async queries
- HTTP 请求: `reqwest` async client
- 文件操作: `tokio::fs`

### 4. 错误处理

统一错误类型 + Result 模式：
```rust
pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("AI service error: {0}")]
    AIService(String),

    #[error("Screenshot capture failed: {0}")]
    Screenshot(String),
}
```

---

## 系统架构图

### 整体架构

```
┌────────────────────────────────────────────────────────────────┐
│                        Frontend (Astro)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │ FloatingOrb  │  │ Memory Page  │  │ Popup Setting│        │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘        │
└─────────┼──────────────────┼──────────────────┼────────────────┘
          │                  │                  │
          └──────────────────┴──────────────────┘
                             │ IPC (invoke)
          ┌──────────────────▼──────────────────┐
          │      Tauri Commands Layer           │
          │  - screenshot_commands.rs            │
          │  - memory_commands.rs                │
          │  - notification_commands.rs          │
          └──────────────────┬──────────────────┘
                             │
          ┌──────────────────▼──────────────────┐
          │        Service Layer                │
          │  ┌────────────────────────────┐     │
          │  │   ScreenshotService        │     │
          │  │   - capture()              │     │
          │  │   - compress()             │     │
          │  │   - save()                 │     │
          │  └────────┬───────────────────┘     │
          │           │                         │
          │  ┌────────▼───────────────────┐     │
          │  │   AIService                │     │
          │  │   - analyze_screenshot()   │     │
          │  │   - extract_keywords()     │     │
          │  └────────┬───────────────────┘     │
          │           │                         │
          │  ┌────────▼───────────────────┐     │
          │  │   MemoryService            │     │
          │  │   - generate_memory()      │     │
          │  │   - identify_intent()      │     │
          │  └────────┬───────────────────┘     │
          │           │                         │
          │  ┌────────▼───────────────────┐     │
          │  │   NotificationService      │     │
          │  │   - push_reminder()        │     │
          │  │   - analyze_pattern()      │     │
          │  └────────────────────────────┘     │
          └──────────────────┬──────────────────┘
                             │
          ┌──────────────────▼──────────────────┐
          │      Data Access Layer              │
          │  - ScreenshotRepository             │
          │  - MemoryRepository                 │
          │  - NotificationRepository           │
          └──────────────────┬──────────────────┘
                             │
          ┌──────────────────▼──────────────────┐
          │        Database (SQLite)            │
          │  - screenshots (D1)                 │
          │  - short_term_memory (D3)           │
          │  - app_usage (D4)                   │
          │  - notifications                    │
          └─────────────────────────────────────┘
```

---

## 模块划分

### 目录结构

```
src-tauri/
├── src/
│   ├── main.rs                    # 应用入口
│   ├── commands/                  # Tauri Commands
│   │   ├── mod.rs
│   │   ├── screenshot_commands.rs # 截图相关命令
│   │   ├── memory_commands.rs     # 记忆相关命令
│   │   ├── ai_commands.rs         # AI 分析命令
│   │   └── notification_commands.rs # 通知命令
│   ├── services/                  # 业务服务层
│   │   ├── mod.rs
│   │   ├── screenshot_service.rs  # 截图服务
│   │   ├── ai_service.rs          # AI 服务
│   │   ├── memory_service.rs      # 记忆服务
│   │   └── notification_service.rs # 通知服务
│   ├── repositories/              # 数据访问层
│   │   ├── mod.rs
│   │   ├── screenshot_repo.rs
│   │   ├── memory_repo.rs
│   │   └── notification_repo.rs
│   ├── models/                    # 数据模型
│   │   ├── mod.rs
│   │   ├── screenshot.rs
│   │   ├── memory.rs
│   │   └── notification.rs
│   ├── utils/                     # 工具函数
│   │   ├── mod.rs
│   │   ├── image_processor.rs     # 图片处理
│   │   ├── time_window.rs         # 时间窗口管理
│   │   └── app_monitor.rs         # 应用监控
│   ├── config/                    # 配置管理
│   │   ├── mod.rs
│   │   └── app_config.rs
│   └── error.rs                   # 错误定义
└── migrations/                    # 数据库迁移
    └── 001_initial_schema.sql
```

---

## 数据流设计

### 功能 1: 屏幕截图采集

```
[定时器触发]
    ↓
[ScreenshotService::capture()]
    ├─ 检查权限
    ├─ 获取当前活跃应用
    ├─ 判断是否需要截图
    │   ├─ 应用切换 → 立即截图
    │   ├─ 内容变化 → 立即截图
    │   └─ 定时触发 → 截图
    ↓
[ImageProcessor::compress()]
    ├─ 去噪
    ├─ 压缩
    └─ 转换格式 (WebP)
    ↓
[ScreenshotRepository::save()]
    ├─ 保存到本地文件系统
    ├─ 创建 D1 数据库记录
    ├─ 创建 D4 应用使用记录
    └─ 触发 AI 分析队列
    ↓
[AIService::analyze_async()]
    (异步队列处理)
```

### 功能 2: AI 内容分析

```
[从队列获取待分析截图]
    ↓
[AIService::analyze_screenshot()]
    ├─ 更新状态: analyzing
    ├─ 图片预处理
    ├─ OCR 文字识别
    │   └─ 调用 OCR API
    ├─ AI 内容理解
    │   ├─ 调用 Claude API
    │   ├─ 提取关键词
    │   ├─ 内容分类
    │   └─ 生成摘要
    ↓
[ScreenshotRepository::update_analysis()]
    ├─ 更新 D1 记录
    │   ├─ ocr_text
    │   ├─ ai_summary
    │   ├─ keywords
    │   └─ confidence
    └─ 更新状态: completed
```

### 功能 3: 短期记忆生成

```
[触发条件检测]
    ├─ 应用切换
    ├─ 时间窗口结束
    └─ 用户主动触发
    ↓
[MemoryService::generate_memory()]
    ├─ 收集时间窗口内的数据
    │   ├─ 截图列表 (D1)
    │   ├─ 应用使用记录 (D4)
    │   └─ 用户 Todo 列表
    ├─ 意图识别
    │   ├─ 分析截图内容
    │   ├─ 分析应用使用模式
    │   └─ 识别事项类型
    ├─ 生成记忆片段
    │   ├─ 提取关键点
    │   ├─ 生成总结
    │   └─ 计算时长
    ↓
[MemoryRepository::save()]
    ├─ 创建 D3 记录
    └─ 关联截图和应用
```

### 功能 4: 主动推送提醒

```
[NotificationService::monitor()]
    ├─ 持续监控工作时长
    ├─ 分析用户行为模式
    └─ 检测打断事件
    ↓
[决策引擎]
    ├─ Case 1: 温馨提醒
    │   └─ 连续工作超过阈值
    ├─ Case 2: 合理建议
    │   └─ 检测到优化机会
    └─ Case 3: 打断衔接
        └─ 用户回归工作
    ↓
[NotificationService::push()]
    ├─ 生成个性化文案
    ├─ 检查推送时机
    └─ 推送到前端
    ↓
[NotificationRepository::log()]
    └─ 记录推送历史
```

---

## 状态管理

### 截图处理状态机

```
[Idle] ──定时器触发──> [Ready]
  ↑                        ↓
  │                   权限检查
  │                        ↓
  │                   [Capturing]
  │                        ↓
  │                   截图成功
  │                        ↓
  │                   [Processing]
  │                        ↓
  │                   保存成功
  │                        ↓
  └────────────────── [Completed]

异常流程:
[任意状态] → [Error] → [Retry] → [Idle]
                ↓ 重试3次后
           [Failed (记录日志)]
```

### AI 分析状态机

```
[Pending] ──进入队列──> [Analyzing]
                           ↓
                      OCR 完成
                           ↓
                    [OCR Completed]
                           ↓
                      AI 分析完成
                           ↓
                       [Completed]

异常流程:
[Analyzing] → [Failed] → [Retry] → [Analyzing]
                 ↓ 重试3次后
          [Permanently Failed]
```

---

## 错误处理

### 错误分类

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // 数据库错误
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // 截图错误
    #[error("Screenshot capture failed: {0}")]
    ScreenshotCapture(String),

    #[error("Image processing failed: {0}")]
    ImageProcessing(String),

    // AI 服务错误
    #[error("AI service error: {0}")]
    AIService(String),

    #[error("OCR failed: {0}")]
    OCR(String),

    // 权限错误
    #[error("Permission denied: {0}")]
    Permission(String),

    // 配置错误
    #[error("Configuration error: {0}")]
    Config(String),
}
```

### 错误处理策略

| 错误类型 | 处理策略 | 用户反馈 |
|---------|---------|---------|
| 权限错误 | 引导用户授权 | 弹窗提示 |
| 网络错误 | 自动重试 3 次 | 静默重试，失败后提示 |
| 数据库错误 | 记录日志，降级处理 | Toast 提示 |
| AI 服务错误 | 队列重试，最终放弃 | 后台静默处理 |

---

## 性能优化

### 1. 异步并发

```rust
// 并发处理多个截图分析
let tasks: Vec<_> = screenshots
    .iter()
    .map(|s| ai_service.analyze_screenshot(s.id))
    .collect();

let results = futures::future::join_all(tasks).await;
```

### 2. 数据库连接池

```rust
// SQLx 连接池配置
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
```

### 3. 图片压缩

- 目标分辨率: 1920x1080 → 压缩到 1280x720
- 压缩格式: PNG → WebP (70% 质量)
- 预期压缩比: ~5x

---

## 安全考虑

### 1. 数据隐私

- 截图数据仅本地存储
- 敏感信息自动模糊处理
- 用户可随时清除历史数据

### 2. API 密钥管理

```rust
// 环境变量存储
let api_key = env::var("CLAUDE_API_KEY")
    .expect("CLAUDE_API_KEY must be set");

// 不在日志中打印密钥
#[derive(Debug)]
pub struct AIClient {
    #[debug(skip)]
    api_key: String,
}
```

### 3. 权限控制

- 屏幕录制权限检查
- 文件系统访问限制
- 应用监控权限验证

---

## 可扩展性

### 插件化设计

未来可支持：
- 自定义 AI 模型
- 第三方 OCR 服务
- 自定义提醒规则

### 配置化

```toml
[screenshot]
interval_seconds = 5
max_resolution = "1920x1080"
compression_quality = 70

[ai]
provider = "claude"  # 支持切换: claude, openai, local
model = "claude-3-sonnet"

[memory]
time_window_minutes = 30
min_screenshots_for_memory = 3
```

---

## 相关文档

- [模块详细设计](modules.md)
- [错误处理机制](error-handling.md)
- [并发设计](concurrency.md)
- [服务层文档](../services/README.md)

---

**维护者**: 后端架构组
**审核者**: 技术负责人
**最后审核日期**: 2026-02-04
