# 数据库设计文档

> **最后更新**: 2026-02-21
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

### ScreenshotAnalysis（录制分段分析结果，V3 核心）

AI 对每段屏幕录制进行一次性分析后写入此表，是所有下游组件（活动分组、项目提取、习惯检测、总结生成）的数据源。

字段分为 5 组：

**Group 1：应用识别**

| 字段 | 类型 | 说明 |
|------|------|------|
| `screenshot_id` | `String` | 对应 recordings 表的 ID（主键） |
| `application` | `String` | 主应用名称（如 VS Code、Chrome） |
| `window_title` | `Option<String>` | 窗口或标签页标题（V2 新增） |
| `url` | `Option<String>` | 浏览器当前 URL（V2 新增） |

**Group 2：活动分类**

| 字段 | 类型 | 说明 |
|------|------|------|
| `activity_category` | `String` | `work`/`entertainment`/`communication`/`learning`/`other` |
| `productivity_score` | `i32` | 生产力评分 1–10 |
| `focus_level` | `String` | 专注程度：`deep`/`normal`/`fragmented`（V2 新增） |
| `interaction_mode` | `String` | 交互方式：`typing`/`reading`/`navigating`/`watching`/`idle`/`mixed`（V2 新增） |
| `is_continuation` | `bool` | 是否是上一分段同一任务的延续（V2 新增，供 activity_grouper 合并用） |

> `activity_type` 已移除（与 `activity_category` 语义重叠）

**Group 3：活动描述**

| 字段 | 类型 | 说明 |
|------|------|------|
| `activity_description` | `String` | 一句话描述（现在时态） |
| `activity_summary` | `String` | 2–3 句详细摘要（供时间线展示） |
| `accomplishments` | `Vec<String>` | 完成事项列表（供 summary_generator 使用） |

**Group 4：上下文增强**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key_elements` | `Vec<String>` | 其他关键元素（已不含 URL 和文件名） |
| `context_tags` | `Vec<String>` | 2–5 个上下文标签（用于活动分组重叠评分） |
| `project_name` | `Option<String>` | 识别到的项目名称 |
| `people_mentioned` | `Vec<String>` | 出现的人名（V2 新增，供 summary_generator 协作记录） |
| `technologies` | `Vec<String>` | 技术栈（V2 新增，供 project_extractor 精确识别） |

**Group 5：内容提取**

| 字段 | 类型 | 说明 |
|------|------|------|
| `ocr_text` | `Option<String>` | 屏幕重要文字（OCR） |
| `file_names` | `Vec<String>` | 出现的文件名（V2 新增，从 key_elements 分离） |
| `error_indicators` | `Vec<String>` | 错误/异常信息（V2 新增，供 habit_detector 识别调试行为） |
| `analysis_json` | `String` | AI 原始响应 JSON（兜底存储） |
| `analyzed_at` | `i64` | 分析完成时间戳 |

```rust
// 当前实现（V1，schema.rs）- 待按 V2 迁移
pub struct ScreenshotAnalysis {
    pub screenshot_id: String,
    pub application: String,
    pub activity_type: String,        // 已废弃，迁移后移除
    pub activity_description: String,
    pub key_elements: Vec<String>,
    pub ocr_text: Option<String>,
    pub context_tags: Vec<String>,
    pub productivity_score: i32,
    pub analysis_json: String,
    pub analyzed_at: i64,
    pub activity_category: String,
    pub activity_summary: String,
    pub project_name: Option<String>,
    pub accomplishments: Vec<String>,
    // V2 新增字段（需 DB 迁移 V8）：
    // window_title, url, focus_level, interaction_mode, is_continuation,
    // people_mentioned, technologies, file_names, error_indicators
}
```

**数据库迁移**：V2 字段需新增 migrations.rs V8 迁移，见 `docs/planning/2026-02-21-screen-analysis-output-fields.md`。

---

### ActivitySession（活动会话）

由 `activity_grouper` 将多个连续 `ScreenshotAnalysis` 聚合生成。

```rust
pub struct ActivitySession {
    pub id: String,
    pub title: String,
    pub start_time: i64,
    pub end_time: i64,
    pub duration_minutes: i64,
    pub application: String,
    pub category: ActivityCategory,   // Work/Entertainment/Communication/Other
    pub screenshot_ids: Vec<String>,
    pub screenshot_analyses: Vec<ScreenshotAnalysisSummary>,
    pub tags: Vec<String>,
    pub markdown_path: String,
    pub summary: Option<String>,
    pub indexed: bool,
    pub created_at: i64,
}
```

### Habit（习惯模式）

```rust
pub struct Habit {
    pub id: String,
    pub pattern_name: String,
    pub pattern_type: HabitPatternType,  // TimeBased/TriggerBased/SequenceBased
    pub confidence: f32,
    pub frequency: String,
    pub trigger_conditions: Option<String>,
    pub typical_time: Option<String>,
    pub last_occurrence: Option<i64>,
    pub occurrence_count: i32,
    pub markdown_path: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub enum HabitPatternType {
    TimeBased,      // 固定时间习惯
    TriggerBased,   // 触发模式
    SequenceBased,  // 序列模式
}
```

### Summary（日/周/月总结）

```rust
pub struct Summary {
    pub id: String,
    pub summary_type: SummaryType,   // Daily/Weekly/Monthly
    pub date_start: String,
    pub date_end: String,
    pub content: String,
    pub activity_ids: Vec<String>,
    pub project_ids: Option<Vec<String>>,
    pub markdown_path: String,
    pub created_at: i64,
}
```

### AppSettings（配置）

主要字段：`capture_interval_seconds`、`storage_path`、`storage_limit_mb`、固定提醒（晨间/喝水/久坐）、智能提醒（屏幕无操作）。

---

## 注意

- `app_usage` 表**不存在**于当前 schema（旧文档有误）
- V1 的 `short_term_memories` 与 V3 的 `activity_sessions` 功能重叠，V3 为主
- `activity_type` 字段在 V2 中废弃，功能并入 `activity_category`（新增 `learning` 值）
- `screenshot_analyses` 的 V2 字段（`window_title`、`focus_level` 等 9 个）尚未迁移，需执行 DB 迁移 V8
- 迁移脚本见 `vision-jarvis/src-tauri/src/db/migrations.rs`

---

## 相关文档

- [CODEMAP](../CODEMAP.md)
- [后端架构](../backend/architecture/overview.md)
- [screenshot_analyses 字段数据流转](./screenshot-analyses-field-flow.md)
- [录屏分析字段设计 V2](../planning/2026-02-21-screen-analysis-output-fields.md)
- [录屏分析 All-in-One Prompt](../planning/allinone-prompt.md)
