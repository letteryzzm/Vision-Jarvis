# 截屏服务 (ScreenshotService)

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **功能**: 屏幕截图采集、压缩、存储

---

## 目录

- [功能概述](#功能概述)
- [核心流程](#核心流程)
- [状态机设计](#状态机设计)
- [API 接口](#api-接口)
- [配置选项](#配置选项)
- [错误处理](#错误处理)
- [性能优化](#性能优化)

---

## 功能概述

截屏服务是 Vision-Jarvis 的核心模块之一，负责：

1. **定时截图**: 按配置的时间间隔捕获屏幕
2. **智能触发**: 检测应用切换和内容变化
3. **图片处理**: 压缩、格式转换、优化存储
4. **应用监控**: 记录当前活跃应用
5. **权限管理**: 检查和请求屏幕录制权限

---

## 核心流程

### 完整处理流程

```
[定时器触发 (间隔 5s)]
    ↓
[检查权限]
    ├─ 无权限 → [请求权限] → [引导用户授权]
    └─ 有权限 → 继续
    ↓
[获取当前活跃应用]
    ├─ macOS: NSWorkspace.shared.frontmostApplication
    ├─ Windows: GetForegroundWindow()
    └─ Linux: X11/Wayland API
    ↓
[判断是否需要截图]
    ├─ 应用切换检测
    │   └─ last_app != current_app → 需要截图
    ├─ 内容变化检测 (可选)
    │   └─ 图片哈希对比 → 需要截图
    └─ 定时触发
        └─ 距上次截图 >= interval → 需要截图
    ↓ 需要截图
[执行截图]
    ├─ 调用系统截图 API
    ├─ 捕获全屏或主屏幕
    └─ 获取原始图片数据
    ↓
[图片处理]
    ├─ 去噪处理
    ├─ 压缩分辨率 (1920x1080 → 1280x720)
    ├─ 格式转换 (PNG → WebP, 70% 质量)
    └─ 生成文件名 (timestamp-based)
    ↓
[保存到本地]
    ├─ 存储路径: ~/Library/Application Support/Vision-Jarvis/screenshots/
    ├─ 文件命名: {timestamp}_{app_name}.webp
    └─ 写入文件系统
    ↓
[创建数据库记录 (D1)]
    ├─ screenshot_id (自增)
    ├─ file_path
    ├─ app_name
    ├─ timestamp
    ├─ file_size
    └─ status: 'pending'
    ↓
[创建应用使用记录 (D4)]
    ├─ app_name
    ├─ start_time
    └─ end_time (下次切换时更新)
    ↓
[触发 AI 分析队列]
    └─ 异步: tokio::spawn(ai_service.analyze(screenshot_id))
    ↓
[返回完成]
```

---

## 状态机设计

### 截图处理状态

```
┌─────────┐
│  Idle   │ ◀──────────────────┐
└────┬────┘                    │
     │ 定时器触发              │
     ▼                         │
┌─────────┐                    │
│  Ready  │                    │
└────┬────┘                    │
     │ 权限检查通过            │
     ▼                         │
┌──────────┐                   │
│Capturing │                   │
└────┬─────┘                   │
     │ 截图成功                │
     ▼                         │
┌───────────┐                  │
│Processing │                  │
└────┬──────┘                  │
     │ 保存成功                │
     ▼                         │
┌──────────┐                   │
│Completed │───────────────────┘
└──────────┘

异常处理:
┌─────────┐
│  Error  │ ◀── [任意状态遇到错误]
└────┬────┘
     │ 判断是否可重试
     ├─ 可重试 (重试 < 3 次)
     │   └─▶ [Retry] ──▶ [Idle]
     └─ 不可重试
         └─▶ [Failed] ──▶ 记录日志 ──▶ [Idle]
```

### 数据库记录状态

```sql
-- D1 截图表的 status 字段
status ENUM:
  'pending'    -- 等待 AI 分析
  'analyzing'  -- AI 分析中
  'completed'  -- 分析完成
  'failed'     -- 分析失败
```

---

## API 接口

### Service API

```rust
use crate::models::Screenshot;
use crate::error::Result;

pub struct ScreenshotService {
    screenshot_repo: Arc<ScreenshotRepository>,
    image_processor: Arc<ImageProcessor>,
    config: ScreenshotConfig,
}

impl ScreenshotService {
    /// 创建新的截屏服务实例
    pub fn new(
        screenshot_repo: Arc<ScreenshotRepository>,
        image_processor: Arc<ImageProcessor>,
        config: ScreenshotConfig,
    ) -> Self {
        Self {
            screenshot_repo,
            image_processor,
            config,
        }
    }

    /// 捕获屏幕截图（主方法）
    ///
    /// # 返回
    /// - Ok(Screenshot): 成功捕获的截图记录
    /// - Err(AppError): 捕获失败
    pub async fn capture(&self) -> Result<Screenshot> {
        // 1. 检查权限
        self.check_permission().await?;

        // 2. 获取当前活跃应用
        let app_info = self.get_active_app().await?;

        // 3. 判断是否需要截图
        if !self.should_capture(&app_info).await? {
            return Err(AppError::Screenshot("Skip: No change detected".into()));
        }

        // 4. 执行截图
        let raw_image = self.capture_screen().await?;

        // 5. 处理图片
        let processed = self.image_processor.process(raw_image).await?;

        // 6. 保存到本地
        let file_path = self.save_to_disk(&processed, &app_info).await?;

        // 7. 创建数据库记录
        let screenshot = self.screenshot_repo.create(CreateScreenshot {
            file_path: file_path.clone(),
            app_name: app_info.name.clone(),
            timestamp: chrono::Utc::now(),
            file_size: processed.len() as i64,
            status: ScreenshotStatus::Pending,
        }).await?;

        // 8. 记录应用使用
        self.log_app_usage(&app_info).await?;

        // 9. 触发 AI 分析（异步）
        self.trigger_ai_analysis(screenshot.id).await?;

        Ok(screenshot)
    }

    /// 检查屏幕录制权限
    pub async fn check_permission(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::NSRunningApplication;
            // macOS 权限检查逻辑
            if !has_screen_capture_permission() {
                return Err(AppError::Permission(
                    "Screen recording permission required".into()
                ));
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 权限检查逻辑
        }

        Ok(())
    }

    /// 获取当前活跃应用
    pub async fn get_active_app(&self) -> Result<AppInfo> {
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::NSWorkspace;
            let app = NSWorkspace::shared().frontmost_application();
            Ok(AppInfo {
                name: app.localized_name(),
                bundle_id: app.bundle_identifier(),
                pid: app.process_identifier(),
            })
        }

        #[cfg(target_os = "windows")]
        {
            use winapi::um::winuser::GetForegroundWindow;
            // Windows 实现
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 实现 (X11/Wayland)
        }
    }

    /// 判断是否需要截图
    pub async fn should_capture(&self, app_info: &AppInfo) -> Result<bool> {
        // 1. 检查应用切换
        if let Some(last_screenshot) = self.screenshot_repo.get_latest().await? {
            if last_screenshot.app_name != app_info.name {
                return Ok(true); // 应用切换，立即截图
            }

            // 2. 检查时间间隔
            let elapsed = chrono::Utc::now() - last_screenshot.timestamp;
            if elapsed.num_seconds() >= self.config.interval_seconds {
                return Ok(true); // 达到时间间隔
            }

            // 3. 内容变化检测 (可选)
            if self.config.enable_content_detection {
                let current_hash = self.compute_screen_hash().await?;
                if current_hash != last_screenshot.content_hash {
                    return Ok(true); // 内容发生变化
                }
            }
        } else {
            return Ok(true); // 第一次截图
        }

        Ok(false)
    }

    /// 执行屏幕截图
    async fn capture_screen(&self) -> Result<RawImage> {
        use screenshots::Screen;

        let screen = Screen::all()
            .map_err(|e| AppError::ScreenshotCapture(e.to_string()))?
            .into_iter()
            .next()
            .ok_or_else(|| AppError::ScreenshotCapture("No screen found".into()))?;

        let image = screen.capture()
            .map_err(|e| AppError::ScreenshotCapture(e.to_string()))?;

        Ok(RawImage::from(image))
    }

    /// 保存图片到磁盘
    async fn save_to_disk(&self, image: &ProcessedImage, app_info: &AppInfo) -> Result<String> {
        let timestamp = chrono::Utc::now().timestamp();
        let filename = format!("{}_{}.webp", timestamp, app_info.name);
        let file_path = self.config.storage_path.join(&filename);

        tokio::fs::write(&file_path, &image.data)
            .await
            .map_err(|e| AppError::Screenshot(format!("Failed to save file: {}", e)))?;

        Ok(file_path.to_string_lossy().to_string())
    }

    /// 记录应用使用情况 (D4)
    async fn log_app_usage(&self, app_info: &AppInfo) -> Result<()> {
        // 实现应用使用记录逻辑
        Ok(())
    }

    /// 触发 AI 分析（异步队列）
    async fn trigger_ai_analysis(&self, screenshot_id: i64) -> Result<()> {
        // 发送到异步队列
        tokio::spawn(async move {
            // AI 服务分析
        });
        Ok(())
    }
}
```

---

## 配置选项

### ScreenshotConfig

```rust
#[derive(Debug, Clone)]
pub struct ScreenshotConfig {
    /// 截图间隔（秒）
    pub interval_seconds: i64,

    /// 存储路径
    pub storage_path: PathBuf,

    /// 最大分辨率
    pub max_resolution: (u32, u32),

    /// 压缩质量 (0-100)
    pub compression_quality: u8,

    /// 启用内容变化检测
    pub enable_content_detection: bool,

    /// 最大存储大小 (MB)
    pub max_storage_mb: u64,

    /// 自动清理天数
    pub auto_cleanup_days: u32,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 5,
            storage_path: default_storage_path(),
            max_resolution: (1280, 720),
            compression_quality: 70,
            enable_content_detection: false,
            max_storage_mb: 5000, // 5GB
            auto_cleanup_days: 30,
        }
    }
}
```

### 配置文件 (config.toml)

```toml
[screenshot]
interval_seconds = 5
storage_path = "~/Library/Application Support/Vision-Jarvis/screenshots"
max_resolution = "1280x720"
compression_quality = 70
enable_content_detection = false
max_storage_mb = 5000
auto_cleanup_days = 30
```

---

## 错误处理

### 错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Capture failed: {0}")]
    CaptureFailed(String),

    #[error("Image processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("No screen available")]
    NoScreen,
}
```

### 错误处理策略

| 错误 | 重试 | 用户反馈 | 日志级别 |
|------|------|---------|---------|
| 权限错误 | ❌ | 弹窗引导授权 | ERROR |
| 截图失败 | ✅ 3次 | Toast 提示 | WARN |
| 处理失败 | ✅ 1次 | 静默处理 | WARN |
| 存储失败 | ✅ 3次 | Toast 提示 | ERROR |

---

## 性能优化

### 1. 图片压缩

```rust
impl ImageProcessor {
    pub async fn process(&self, raw: RawImage) -> Result<ProcessedImage> {
        // 1. 调整分辨率
        let resized = self.resize(raw, (1280, 720)).await?;

        // 2. 转换为 WebP 格式
        let webp = self.to_webp(resized, 70).await?;

        // 3. 去噪 (可选)
        let denoised = self.denoise(webp).await?;

        Ok(ProcessedImage { data: denoised })
    }
}
```

**压缩效果**:
- 原始 PNG (1920x1080): ~5MB
- 压缩 WebP (1280x720, 70%): ~500KB
- 压缩比: **~10x**

### 2. 异步并发

```rust
// 并发处理多个截图任务
let tasks: Vec<_> = screenshots
    .iter()
    .map(|s| screenshot_service.capture())
    .collect();

let results = futures::future::join_all(tasks).await;
```

### 3. 存储管理

```rust
impl ScreenshotService {
    /// 自动清理旧截图
    pub async fn cleanup_old_screenshots(&self) -> Result<()> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(30);
        self.screenshot_repo.delete_before(cutoff).await?;
        Ok(())
    }

    /// 检查存储空间
    pub async fn check_storage_limit(&self) -> Result<bool> {
        let total_size = self.screenshot_repo.get_total_size().await?;
        Ok(total_size < self.config.max_storage_mb * 1024 * 1024)
    }
}
```

---

## 边界条件

### 1. 输入验证

- 截图间隔: 1s ≤ interval ≤ 60s
- 压缩质量: 1 ≤ quality ≤ 100
- 存储路径: 必须可写

### 2. 资源限制

- 最大存储: 5GB (可配置)
- 最大单张大小: 10MB
- 并发截图任务: 3 个

### 3. 异常场景

- 无屏幕: 跳过截图
- 无权限: 引导用户授权
- 磁盘满: Toast 提示，停止截图

---

## 监控指标

| 指标 | 描述 | 阈值 |
|------|------|------|
| capture_latency | 截图延迟 | < 100ms |
| processing_latency | 处理延迟 | < 200ms |
| storage_usage | 存储占用 | < 5GB |
| error_rate | 错误率 | < 1% |

---

## 相关文档

- [AI 服务](ai-service.md)
- [记忆服务](memory-service.md)
- [截图 API](../../api/endpoints/screenshot.md)
- [数据库设计 - D1 表](../../database/schema/tables/screenshots.md)

---

**维护者**: 后端服务组
**最后更新**: 2026-02-04
