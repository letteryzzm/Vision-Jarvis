# 数据库设计文档

> **最后更新**: 2026-02-19
> **数据库**: SQLite（通过 rusqlite）

---

## 表结构总览

数据库随版本迭代累积了三代表结构，均在 `db/schema.rs` 中定义，迁移脚本在 `db/migrations.rs`。

### V1 表

| 表名 | 结构体 | 说明 |
|------|--------|------|
| `screenshots` | `Screenshot` | 截图记录（路径、时间、分析结果、向量嵌入） |
| `short_term_memories` | `ShortTermMemory` | 短期记忆（活动事项，按日期/时段） |
| `long_term_memories` | `LongTermMemory` | 长期记忆（日期范围总结） |
| `settings` | `AppSettings` | 应用配置（截图间隔、提醒设置等） |

### V2 表（活动驱动记忆）

| 表名 | 结构体 | 说明 |
|------|--------|------|
| `activity_sessions` | `ActivitySession` | 聚合后的活动会话（含 Markdown 路径） |
| `memory_chunks` | `MemoryChunk` | 记忆分块（用于向量索引） |
| `tracked_files` | `TrackedFile` | 文件追踪记录（增量索引用） |
| `embedding_cache` | `EmbeddingCacheEntry` | Embedding 缓存 |

### V3 表（主动式 AI 记忆）

| 表名 | 结构体 | 说明 |
|------|--------|------|
| `screenshot_analyses` | `ScreenshotAnalysis` | 截图 AI 分析结果（活动类型、标签、OCR） |
| `projects` | `Project` | 自动识别的项目 |
| `habits` | `Habit` | 检测到的习惯模式（时间/触发/序列） |
| `summaries` | `Summary` | 日/周/月总结 |
| `proactive_suggestions` | `ProactiveSuggestion` | 主动建议记录 |

---

## 关键结构体

### ScreenshotAnalysis（V3 核心）

```rust
pub struct ScreenshotAnalysis {
    pub screenshot_id: String,
    pub application: String,
    pub activity_type: String,       // work/entertainment/communication/other
    pub activity_description: String,
    pub key_elements: Vec<String>,
    pub ocr_text: Option<String>,
    pub context_tags: Vec<String>,
    pub productivity_score: i32,     // 1-10
    pub analysis_json: String,
    pub analyzed_at: i64,
}
```

### Habit（习惯模式）

```rust
pub enum HabitPatternType {
    TimeBased,      // 固定时间习惯
    TriggerBased,   // 触发模式
    SequenceBased,  // 序列模式
}
```

### AppSettings（配置）

主要字段：`capture_interval_seconds`、`storage_path`、`storage_limit_mb`、固定提醒（晨间/喝水/久坐）、智能提醒（屏幕无操作）。

---

## 注意

- `app_usage` 表**不存在**于当前 schema（旧文档有误）
- V1 的 `short_term_memories` 与 V3 的 `activity_sessions` 功能重叠，V3 为主
- 迁移脚本见 `vision-jarvis/src-tauri/src/db/migrations.rs`

---

## 相关文档

- [CODEMAP](../CODEMAP.md)
- [后端架构](../backend/architecture/overview.md)
