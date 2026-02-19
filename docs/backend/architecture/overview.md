# 后端架构概述

> **最后更新**: 2026-02-18
> **版本**: v3.1（视频录制 + AI分析已接入）
> **架构模式**: 模块化 + 管道调度

---

## 实际目录结构

```
src-tauri/src/
├── main.rs                    # 应用入口
├── lib.rs                     # 模块注册、Tauri 插件、AppState 初始化
├── error.rs                   # 统一错误类型 AppError
│
├── commands/                  # Tauri IPC 命令层（前后端通信）
│   ├── mod.rs                 # AppState、ApiResponse<T> 定义
│   ├── screenshot.rs          # 截图命令
│   ├── memory.rs              # 记忆命令（V2 + V3）
│   ├── notification.rs        # 通知命令
│   ├── settings.rs            # 设置命令
│   ├── storage.rs             # 文件存储命令
│   ├── ai_config.rs           # AI 配置命令
│   └── window.rs              # 窗口管理命令
│
├── memory/                    # V3 主动式 AI 记忆系统
│   ├── pipeline.rs            # 管道调度器（统一调度所有记忆任务）
│   ├── screenshot_analyzer.rs # 截图 AI 分析
│   ├── activity_grouper.rs    # 活动分组（截图→ActivitySession）
│   ├── summary_generator.rs   # 日/周/月总结生成
│   ├── project_extractor.rs   # 项目自动识别
│   ├── habit_detector.rs      # 习惯模式检测
│   ├── markdown_generator.rs  # Markdown 文件生成
│   ├── index_manager.rs       # 文件索引管理
│   ├── vector_store.rs        # 向量存储
│   ├── hybrid_search.rs       # 混合搜索
│   ├── chunker.rs             # 文本分块
│   ├── short_term.rs          # 短期记忆（V2 遗留）
│   ├── long_term.rs           # 长期记忆（V2 遗留）
│   └── scheduler.rs           # 旧版调度器（V2 遗留）
│
├── notification/              # 通知系统
│   ├── mod.rs                 # Notification 结构体定义
│   ├── scheduler.rs           # 通知调度器
│   ├── rules.rs               # 通知规则接口
│   ├── context.rs             # 规则上下文构建
│   ├── delivery.rs            # 通知投递（Tauri 事件）
│   └── smart/
│       ├── mod.rs
│       └── proactive.rs       # V3 主动建议规则
│
├── capture/                   # 屏幕录制采集
│   ├── mod.rs                 # ScreenCapture（V1遗留）
│   ├── screen_recorder.rs     # FFmpeg 屏幕录制器（V4 核心）
│   ├── scheduler.rs           # CaptureScheduler（录制调度）
│   ├── storage.rs             # 截图文件存储（V1遗留）
│   └── video_compressor.rs    # 视频压缩工具
│
├── ai/                        # AI 客户端
│   ├── mod.rs
│   ├── client.rs              # HTTP 客户端
│   ├── provider.rs            # AI 提供商配置
│   └── prompt.rs              # Prompt 模板
│
├── db/                        # 数据库层
│   ├── mod.rs                 # Database 连接池
│   ├── schema.rs              # 所有表结构体定义
│   └── migrations.rs          # SQLite 迁移脚本
│
├── settings/                  # 设置管理
│   ├── mod.rs
│   └── config.rs              # 配置读写持久化
│
└── storage/                   # 文件存储
    └── mod.rs
```

---

## 架构数据流（V3.1）

```
FFmpeg屏幕录制 (capture/screen_recorder.rs)
    ↓ 每60秒分段(可配置30-300s)
录制AI分析 (memory/screenshot_analyzer.rs) [每90秒]
    ↓
活动分组 (memory/activity_grouper.rs) [每30分钟]
    ↓ 并行
├── 项目提取 (memory/project_extractor.rs) [分组后立即]
├── 习惯检测 (memory/habit_detector.rs) [每日]
├── 日总结生成 (memory/summary_generator.rs) [23:00]
└── 文件索引 (memory/index_manager.rs) [每10分钟]
    ↓
通知规则评估 (notification/smart/proactive.rs)
    ↓
前端推送 (notification/delivery.rs)
```

---

## AppState

```rust
pub struct AppState {
    pub db: Arc<Database>,
    pub settings: Arc<SettingsManager>,
    pub screen_capture: Arc<ScreenCapture>,
    pub scheduler: Arc<Mutex<CaptureScheduler>>,
    pub notification_scheduler: Arc<NotificationScheduler>,
    pub pipeline: Arc<PipelineScheduler>,
}
```

---

## 错误处理

统一错误类型定义在 `error.rs`，所有命令返回 `ApiResponse<T>`：

```rust
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```
