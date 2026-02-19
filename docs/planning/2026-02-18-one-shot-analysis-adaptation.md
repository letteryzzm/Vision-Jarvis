# 一次性视频分析 + 全链路适配方案

**日期**: 2026-02-18
**状态**: 待审批

---

## 问题

当前系统存在三个断裂点：

1. **数据源断裂**：`activity_grouper` 查 `screenshots` 表，但新录制数据写入 `recordings` 表
2. **格式断裂**：AI 分析输出 `AIScreenshotResult`(7字段)，但分组器需要 `AnalysisResult`(4字段：activity/application/description/category)，字段名和结构不匹配
3. **重复调用风险**：如果下游组件各自再调AI获取额外信息（项目名、总结），会导致同一段视频被重复分析

## 设计原则

- **视频只分析一次**：AI 请求发生在 Layer 1（screenshot_analyzer），输出足够所有下游使用的完整 JSON
- **下游只读 DB**：activity_grouper、project_extractor、summary_generator、habit_detector 只读 `screenshot_analyses` 表的 JSON 数据，不碰视频文件
- **录制表取代截图表**：整条链路的数据源从 `screenshots` 切换到 `recordings`

---

## 方案概览

```
recordings 表 (视频文件引用)
    ↓ screenshot_analyzer 只在这里读视频
screenshot_analyses 表 (完整 JSON，一次分析所有信息)
    ↓ 以下所有层只读这张表的 JSON
├── activity_grouper → activities 表
├── project_extractor → projects 表
├── summary_generator → summaries 表
└── habit_detector → habits 表
```

---

## Phase 1: 扩展 AI 分析输出（一次性提取所有信息）

### 1.1 扩展 `AIScreenshotResult` 结构

当前：
```rust
struct AIScreenshotResult {
    application: String,
    activity_type: String,          // work|entertainment|communication|learning|other
    activity_description: String,
    key_elements: Vec<String>,
    ocr_text: Option<String>,
    context_tags: Vec<String>,
    productivity_score: i32,
}
```

新增字段（给下游直接用，避免二次AI调用）：
```rust
struct AIScreenshotResult {
    // --- 现有字段 ---
    application: String,
    activity_type: String,
    activity_description: String,
    key_elements: Vec<String>,
    ocr_text: Option<String>,
    context_tags: Vec<String>,
    productivity_score: i32,
    // --- 新增：给 activity_grouper 用 ---
    activity_category: String,      // "work"|"entertainment"|"communication"|"other"（映射到 ActivityCategory 枚举）
    activity_summary: String,       // 一句话总结（给 markdown 时间线用）
    // --- 新增：给 project_extractor 用 ---
    project_name: Option<String>,   // AI 直接提取项目名（如 "Vision-Jarvis"、"论文写作"），无项目返回 null
    // --- 新增：给 summary_generator 用 ---
    accomplishments: Vec<String>,   // 这段时间的成果要点（1-3条）
}
```

### 1.2 更新录制分析 Prompt

合并所有下游需求到一个 Prompt 中，在 `screenshot_analyzer.rs` 的 `recording_understanding_prompt()` 中修改：

```
分析这段屏幕录制视频，严格按JSON格式返回：
{
  "application": "主要应用名称",
  "activity_type": "work|entertainment|communication|learning|other",
  "activity_description": "用户在做什么（一句话，要具体）",
  "activity_category": "work|entertainment|communication|other",
  "activity_summary": "这段时间的活动概述（供时间线展示）",
  "key_elements": ["窗口标题", "文件名", "网页标题"],
  "ocr_text": "屏幕上的重要文本",
  "context_tags": ["标签1", "标签2"],
  "productivity_score": 5,
  "project_name": "项目名称或null",
  "accomplishments": ["完成了XX", "修改了YY"]
}
```

### 1.3 扩展 `screenshot_analyses` 表

新增列（V5迁移）：
```sql
ALTER TABLE screenshot_analyses ADD COLUMN activity_category TEXT DEFAULT 'other';
ALTER TABLE screenshot_analyses ADD COLUMN activity_summary TEXT DEFAULT '';
ALTER TABLE screenshot_analyses ADD COLUMN project_name TEXT;
ALTER TABLE screenshot_analyses ADD COLUMN accomplishments TEXT DEFAULT '[]';
```

### 1.4 更新 `save_analysis` 方法

扩展 INSERT 语句，写入新字段。

### 文件变更

| 文件 | 操作 |
|------|------|
| `memory/screenshot_analyzer.rs` | 修改 `AIScreenshotResult`、`recording_understanding_prompt()`、`save_analysis()` |
| `db/migrations.rs` | 新增 V5 迁移 |
| `db/schema.rs` | 更新 `ScreenshotAnalysis` struct |

---

## Phase 2: `recordings` 表添加 `activity_id`

当前 `recordings` 表缺少 `activity_id` 字段，活动分组后无法标记"已分组"。

### V5 迁移中一并添加

```sql
ALTER TABLE recordings ADD COLUMN activity_id TEXT;
CREATE INDEX idx_recordings_activity ON recordings(activity_id);
```

### 文件变更

| 文件 | 操作 |
|------|------|
| `db/migrations.rs` | V5 迁移添加 `activity_id` 列 |
| `db/schema.rs` | `Recording` struct 添加 `activity_id` 字段 |

---

## Phase 3: 活动分组器适配 recordings

核心改造：`get_ungrouped_screenshots()` → `get_ungrouped_recordings()`

### 3.1 新 SQL 查询

```sql
SELECT r.id, r.path, r.start_time,
       sa.application, sa.activity_type, sa.activity_description,
       sa.activity_category, sa.activity_summary,
       sa.key_elements, sa.context_tags, sa.productivity_score,
       sa.project_name
FROM recordings r
INNER JOIN screenshot_analyses sa ON r.id = sa.screenshot_id
WHERE r.analyzed = 1
  AND r.activity_id IS NULL
ORDER BY r.start_time ASC
```

关键变化：
- `screenshots` → `recordings`
- `LEFT JOIN` → `INNER JOIN`（只要已分析的）
- 不再读 `s.analysis_result`（旧V1格式），所有字段直接从 `screenshot_analyses` 读
- 排序字段：`captured_at` → `start_time`

### 3.2 重构 `AnalyzedScreenshot`

不再依赖 `AnalysisResult`（旧V1结构），直接从 `screenshot_analyses` 构建：

```rust
pub struct AnalyzedRecording {
    pub id: String,
    pub path: String,
    pub start_time: i64,
    // 直接从 screenshot_analyses 来
    pub application: String,
    pub activity_type: String,        // work|entertainment|...
    pub activity_description: String,
    pub activity_category: String,    // 映射到 ActivityCategory
    pub activity_summary: String,
    pub key_elements: Vec<String>,
    pub context_tags: Vec<String>,
    pub productivity_score: i32,
    pub project_name: Option<String>,
}
```

### 3.3 更新 `save_activity`

```sql
-- 旧：UPDATE screenshots SET activity_id = ?1 WHERE id = ?2
-- 新：
UPDATE recordings SET activity_id = ?1 WHERE id = ?2
```

### 3.4 更新分组算法中的字段引用

所有 `screenshot.analysis.application` → `recording.application`
所有 `screenshot.analysis.activity` → `recording.activity_description`
所有 `screenshot.analysis.category` → 由 `recording.activity_category` 映射为 `ActivityCategory` 枚举

### 文件变更

| 文件 | 操作 |
|------|------|
| `memory/activity_grouper.rs` | 重构数据源和分组逻辑 |

---

## Phase 4: 下游组件适配

### 4.1 `project_extractor.rs`

- `process_unlinked_activities()` 中的 AI 调用可以移除
- 改为：查询 `screenshot_analyses.project_name`（已在 Phase 1 由 AI 一次性提取）
- 根据 activity 关联的 recording IDs，聚合其 `project_name`
- 仍保留 Jaccard 相似度匹配逻辑

### 4.2 `summary_generator.rs`

- `generate_ai_daily_summary()` 可以利用 `screenshot_analyses.accomplishments`
- 聚合当日所有 `accomplishments` 作为 AI 总结的输入素材
- 减少给 AI 的 prompt 复杂度（已有结构化要点）

### 4.3 `pipeline.rs`

- `connect_ai()` 同时更新 `summary_generator` 和 `project_extractor` 的 AI 客户端引用
- 或者：这两个组件不再需要独立 AI 客户端（所有 AI 数据已在 Phase 1 提取）

### 文件变更

| 文件 | 操作 |
|------|------|
| `memory/project_extractor.rs` | 读 `screenshot_analyses.project_name`，减少 AI 调用 |
| `memory/summary_generator.rs` | 利用 `accomplishments` 字段 |
| `memory/pipeline.rs` | 可选：简化 AI 客户端传递 |

---

## Phase 5: Markdown 生成器适配

### `markdown_generator.rs`

- `ScreenshotAnalysisSummary.analysis` 字段改为读取 `activity_summary`
- `ScreenshotAnalysisSummary.path` 改为录制文件路径
- 时间线格式：从截图时间点改为录制分段时间段

### 文件变更

| 文件 | 操作 |
|------|------|
| `memory/markdown_generator.rs` | 适配录制分段格式 |

---

## 全部文件变更清单

| 文件 | Phase | 操作说明 |
|------|-------|---------|
| `db/migrations.rs` | 1,2 | V5 迁移：screenshot_analyses 新增4列 + recordings 新增 activity_id |
| `db/schema.rs` | 1,2 | 更新 `ScreenshotAnalysis`、`Recording` struct |
| `memory/screenshot_analyzer.rs` | 1 | 扩展 `AIScreenshotResult`、Prompt、`save_analysis()` |
| `memory/activity_grouper.rs` | 3 | 重构：recordings 数据源、`AnalyzedRecording`、`save_activity` |
| `memory/project_extractor.rs` | 4 | 读 `project_name`，减少 AI 调用 |
| `memory/summary_generator.rs` | 4 | 利用 `accomplishments` |
| `memory/markdown_generator.rs` | 5 | 适配录制分段格式 |
| `memory/pipeline.rs` | 4 | 可选简化 |

---

## 验证计划

1. `cargo check` 编译通过
2. 单元测试全部通过
3. 集成测试：用真实 mp4 跑一次完整 Pipeline：
   - 录制 → AI 分析 → 检查 `screenshot_analyses` 表的新字段
   - 手动触发分组 → 检查 `activities` 表
   - 检查 `recordings.activity_id` 已更新
