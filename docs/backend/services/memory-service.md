# 记忆服务 (Memory System V3)

> **最后更新**: 2026-02-19
> **版本**: v3.0
> **架构**: 主动式 AI 记忆系统（管道调度）

---

## 架构概述

V3 采用管道调度模式，由 `pipeline.rs` 统一调度所有记忆任务：

| 任务 | 调度间隔 | 模块 |
|------|---------|------|
| 截图 AI 分析 | 每 5 分钟 | `screenshot_analyzer.rs` |
| 活动分组 | 每 30 分钟 | `activity_grouper.rs` |
| 文件索引同步 | 每 10 分钟 | `index_manager.rs` |
| 习惯检测 | 每日 | `habit_detector.rs` |
| 日总结生成 | 每日 23:00 | `summary_generator.rs` |

---

## 核心模块

### pipeline.rs — 管道调度器

统一启动和管理所有记忆任务的后台调度。

```rust
pub struct PipelineScheduler {
    db: Arc<Database>,
    storage_path: PathBuf,
    ai_enabled: bool,
}
```

### screenshot_analyzer.rs — 截图 AI 分析器

将截图发给 AI 理解，提取结构化信息，存入 `screenshot_analyses` 表。

**AI 返回结构**:
```rust
struct AIScreenshotResult {
    application: String,
    activity_type: String,  // "coding" | "browsing" | "writing" 等
    title: String,
    tags: Vec<String>,
    confidence: f32,
}
```

### activity_grouper.rs — 活动分组器

将连续相似截图聚合为 `ActivitySession`。

**分组配置**:
```rust
pub struct GroupingConfig {
    pub max_gap_seconds: i64,       // 默认 300s
    pub min_screenshots: usize,     // 默认 2
    pub max_duration_seconds: i64,  // 默认 7200s
    pub min_duration_seconds: i64,  // 默认 60s
}
```

### summary_generator.rs — 总结生成器

聚合活动数据，调用 AI 生成日/周/月总结，存入 `summaries` 表和 Markdown 文件。

### project_extractor.rs — 项目提取器

从活动标题、应用、标签中识别项目，相似度匹配现有项目或创建新项目，维护项目 Markdown 文件。

### habit_detector.rs — 习惯检测器

从活动历史识别三种模式：
- **时间模式**: 固定时间的习惯（如"每天 8:00 打开微信"）
- **触发模式**: 特定事件触发的习惯
- **序列模式**: 固定顺序的活动序列

---

## 数据库表

| 表名 | 用途 |
|------|------|
| `screenshots` | 截图记录 |
| `screenshot_analyses` | AI 分析结果 |
| `activity_sessions` | 聚合后的活动会话 |
| `projects` | 自动识别的项目 |
| `habits` | 检测到的习惯模式 |
| `summaries` | 日/周/月总结 |

---

## V2 遗留模块

以下模块为 V2 遗留，仍在使用但不再是核心：

| 模块 | 说明 |
|------|------|
| `short_term.rs` | 短期记忆聚合 |
| `long_term.rs` | 长期记忆总结 |
| `scheduler.rs` | 旧版调度器，已被 `pipeline.rs` 替代 |

---

## 相关文档

- [架构概述](../architecture/overview.md)
- [CODEMAP](../../CODEMAP.md)
