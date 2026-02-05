# 服务层概述

> **最后更新**: 2026-02-04
> **版本**: v1.0

---

## 服务层架构

服务层是 Vision-Jarvis 后端的核心业务逻辑层，负责编排和实现各个功能模块。

### 设计原则

1. **单一职责**: 每个服务专注于一个业务领域
2. **依赖注入**: 服务间通过接口依赖，便于测试
3. **异步优先**: 所有 I/O 操作使用异步模式
4. **错误透明**: 统一的错误类型和处理

---

## 核心服务

### 1. 截屏服务 (ScreenshotService)

**职责**: 屏幕截图采集、压缩、存储

**核心方法**:
```rust
pub struct ScreenshotService {
    screenshot_repo: Arc<ScreenshotRepository>,
    image_processor: Arc<ImageProcessor>,
}

impl ScreenshotService {
    // 捕获屏幕截图
    pub async fn capture(&self) -> Result<Screenshot>;

    // 获取当前活跃应用
    pub async fn get_active_app(&self) -> Result<AppInfo>;

    // 判断是否需要截图
    pub async fn should_capture(&self) -> Result<bool>;
}
```

**详细文档**: [screenshot-service.md](screenshot-service.md)

---

### 2. AI 服务 (AIService)

**职责**: OCR 识别、AI 内容分析、关键词提取

**核心方法**:
```rust
pub struct AIService {
    client: Arc<ClaudeClient>,
    ocr_service: Arc<OCRService>,
}

impl AIService {
    // 分析截图内容
    pub async fn analyze_screenshot(&self, screenshot_id: i64) -> Result<AnalysisResult>;

    // OCR 文字识别
    pub async fn extract_text(&self, image_path: &str) -> Result<String>;

    // 提取关键词
    pub async fn extract_keywords(&self, text: &str) -> Result<Vec<String>>;

    // 内容分类
    pub async fn classify_content(&self, text: &str) -> Result<ContentCategory>;
}
```

**详细文档**: [ai-service.md](ai-service.md)

---

### 3. 记忆服务 (MemoryService)

**职责**: 短期记忆生成、意图识别、事项提取

**核心方法**:
```rust
pub struct MemoryService {
    memory_repo: Arc<MemoryRepository>,
    screenshot_repo: Arc<ScreenshotRepository>,
    ai_service: Arc<AIService>,
}

impl MemoryService {
    // 生成短期记忆
    pub async fn generate_memory(&self, time_window: TimeWindow) -> Result<ShortTermMemory>;

    // 识别用户意图
    pub async fn identify_intent(&self, screenshots: &[Screenshot]) -> Result<Intent>;

    // 查询记忆
    pub async fn search_memory(&self, query: &str) -> Result<Vec<ShortTermMemory>>;
}
```

**详细文档**: [memory-service.md](memory-service.md)

---

### 4. 通知服务 (NotificationService)

**职责**: 主动推送提醒、行为分析、打断衔接

**核心方法**:
```rust
pub struct NotificationService {
    notification_repo: Arc<NotificationRepository>,
    memory_service: Arc<MemoryService>,
}

impl NotificationService {
    // 推送温馨提醒
    pub async fn push_work_reminder(&self) -> Result<()>;

    // 推送合理建议
    pub async fn push_suggestion(&self, suggestion: Suggestion) -> Result<()>;

    // 打断衔接提醒
    pub async fn push_resume_reminder(&self, context: WorkContext) -> Result<()>;

    // 分析工作模式
    pub async fn analyze_work_pattern(&self) -> Result<WorkPattern>;
}
```

**详细文档**: [notification-service.md](notification-service.md)

---

## 服务依赖关系

```
┌─────────────────────┐
│ NotificationService │
└──────────┬──────────┘
           │ depends on
┌──────────▼──────────┐
│   MemoryService     │
└──────────┬──────────┘
           │ depends on
┌──────────▼──────────┐     ┌─────────────────┐
│ ScreenshotService   │────▶│   AIService     │
└─────────────────────┘     └─────────────────┘
           │                         │
           └─────────┬───────────────┘
                     │
           ┌─────────▼──────────┐
           │   Database Layer   │
           └────────────────────┘
```

**依赖说明**:
- `NotificationService` 依赖 `MemoryService`
- `MemoryService` 依赖 `ScreenshotService` 和 `AIService`
- `ScreenshotService` 触发 `AIService` 异步分析
- 所有服务依赖数据访问层

---

## 服务初始化

### main.rs 中的服务注��

```rust
#[tokio::main]
async fn main() {
    // 初始化数据库连接池
    let pool = create_db_pool().await.expect("Failed to create DB pool");

    // 初始化 Repositories
    let screenshot_repo = Arc::new(ScreenshotRepository::new(pool.clone()));
    let memory_repo = Arc::new(MemoryRepository::new(pool.clone()));
    let notification_repo = Arc::new(NotificationRepository::new(pool.clone()));

    // 初始化工具类
    let image_processor = Arc::new(ImageProcessor::new());
    let claude_client = Arc::new(ClaudeClient::new(&config.claude_api_key));
    let ocr_service = Arc::new(OCRService::new());

    // 初始化服务
    let screenshot_service = Arc::new(ScreenshotService::new(
        screenshot_repo.clone(),
        image_processor,
    ));

    let ai_service = Arc::new(AIService::new(
        claude_client,
        ocr_service,
    ));

    let memory_service = Arc::new(MemoryService::new(
        memory_repo,
        screenshot_repo.clone(),
        ai_service.clone(),
    ));

    let notification_service = Arc::new(NotificationService::new(
        notification_repo,
        memory_service.clone(),
    ));

    // 注入到 Tauri 状态管理
    tauri::Builder::default()
        .manage(screenshot_service)
        .manage(ai_service)
        .manage(memory_service)
        .manage(notification_service)
        .invoke_handler(tauri::generate_handler![
            // Commands...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 服务通信模式

### 1. 同步调用

```rust
// MemoryService 调用 AIService
let analysis = self.ai_service.analyze_screenshot(screenshot_id).await?;
```

### 2. 异步队列

```rust
// ScreenshotService 触发 AI 分析（异步队列）
tokio::spawn(async move {
    ai_service.analyze_screenshot(screenshot_id).await
});
```

### 3. 事件发布

```rust
// 发布事件到前端
app_handle.emit_all("memory_updated", payload)?;
```

---

## 测试策略

### 单元测试

每个服务提供独立的单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_memory() {
        let memory_service = create_test_memory_service();
        let time_window = TimeWindow::new(start, end);

        let result = memory_service.generate_memory(time_window).await;

        assert!(result.is_ok());
    }
}
```

### 集成测试

跨服务的集成测试：

```rust
#[tokio::test]
async fn test_screenshot_to_memory_flow() {
    // 1. 截图服务捕获
    let screenshot = screenshot_service.capture().await.unwrap();

    // 2. AI 服务分析
    let analysis = ai_service.analyze_screenshot(screenshot.id).await.unwrap();

    // 3. 记忆服务生成记忆
    let memory = memory_service.generate_memory(time_window).await.unwrap();

    assert!(memory.screenshots.contains(&screenshot.id));
}
```

---

## 性能监控

### 服务调用追踪

```rust
use tracing::{info, instrument};

#[instrument]
pub async fn analyze_screenshot(&self, screenshot_id: i64) -> Result<AnalysisResult> {
    info!("Starting analysis for screenshot {}", screenshot_id);
    // ...
}
```

### 性能指标

| 服务 | 关键指标 | 目标 |
|------|---------|------|
| ScreenshotService | 截图延迟 | < 100ms |
| AIService | 分析延迟 | < 3s |
| MemoryService | 生成延迟 | < 500ms |
| NotificationService | 推送延迟 | < 50ms |

---

## 相关文档

- [后端架构概述](../architecture/overview.md)
- [API 接口文档](../../api/README.md)
- [数据库设计](../../database/README.md)

---

**维护者**: 后端服务组
**最后更新**: 2026-02-04
