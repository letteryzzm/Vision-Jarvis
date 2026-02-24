# screenshot_analyses 字段数据流转文档

> **最后更新**: 2026-02-23
> **当前 DB 版本**: V7（V5 schema，V8 尚未迁移）
> **状态**: 反映当前实际代码，非设计目标

---

## Pipeline 总览

```
屏幕录制（recordings 表）
        │
        ▼
  ┌─────────────┐
  │  AI 分析器   │  one_shot_analyzer.rs
  │（一次性分析）│  写入 screenshot_analyses
  └──────┬──────┘
         │  INSERT 14 列
         ▼
  ┌─────────────────────────┐
  │  screenshot_analyses 表  │
  │  (V3建表10列 + V5新增4列)│
  └──────────────┬──────────┘
                 │
     ┌───────────┼───────────┬──────────────┐
     ▼           ▼           ▼              ▼
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│ activity │ │ project  │ │ summary  │ │ commands │
│ _grouper │ │_extractor│ │_generator│ │/memory.rs│
│  (分组)  │ │(项目提取)│ │(总结生成)│ │(前端API) │
└────┬─────┘ └────┬─────┘ └────┬─────┘ └──────────┘
     │             │            │
     ▼             ▼            ▼
ActivitySession  project_name  accomplishments
     │
     ▼
markdown_generator.rs
（通过 ActivitySession.screenshot_analyses: Vec<ScreenshotAnalysisSummary>）
```

---

## 字段详细说明

以下 14 个字段按���表顺序排列（V3 原有 10 列，V5 新增 4 列）。

---

### 1. `screenshot_id`

| 属性 | 值 |
|------|----|
| 类型 | `TEXT PRIMARY KEY` |
| 来源 | recordings 表 ID（写入时由分析器填入） |
| 状态 | 正常 |

**消费情况：**
- `activity_grouper.rs`: `SELECT r.id` → `AnalyzedRecording.id`（通过 JOIN，不直接 SELECT 此列）
- `project_extractor.rs`: WHERE 条件（`screenshot_id IN (...)`）
- `summary_generator.rs`: WHERE 条件（`screenshot_id IN (...)`）
- `commands/memory.rs`: `SELECT screenshot_id`（前端 API 返回）
- `markdown_generator.rs`: 不直接消费（通过 `ScreenshotAnalysisSummary.id`）

---

### 2. `application`

| 属性 | 值 |
|------|----|
| 类型 | `TEXT NOT NULL` |
| 来源 | AI 分析输出 |
| 状态 | 正常，核心字段 |

**消费情况：**
- `activity_grouper.rs` L205: `SELECT ... sa.application ...` → `AnalyzedRecording.application`
  - 用于：按应用分组、活动标题生成
- `commands/memory.rs` L190: `SELECT ... application ...` → `ScreenshotAnalysisInfo.application`
  - 用于：前端显示

---

### 3. `activity_type`

| 属性 | 值 |
|------|----|
| 类型 | `TEXT NOT NULL` |
| 来源 | AI 分析输出（旧字段） |
| 状态 | **废弃不彻底** — 语义与 `activity_category` 重叠 |

**消费情况：**
- `activity_grouper.rs` L206: `SELECT ... sa.activity_type ...` → `AnalyzedRecording.activity_type`
  - **强依赖**：`group_by_activity` 方法通过此字段做分组判断
- `commands/memory.rs` L191: `SELECT ... activity_type ...` → `ScreenshotAnalysisInfo.activity_type`
  - 前端 API 仍暴露此字段

**问题：** `README.md` 标注"已废弃"，但实际代码仍强依赖。V2 迁移需在替换 `activity_category` 后才能移除此字段。

---

### 4. `activity_description`

| 属性 | 值 |
|------|----|
| 类型 | `TEXT NOT NULL` |
| 来源 | AI 分析输出（一句话描述） |
| 状态 | 正常 |

**消费情况：**
- `activity_grouper.rs` L207: `SELECT ... sa.activity_description ...` → `AnalyzedRecording.activity_description`
  - 用于：活动会话标题候选
- `commands/memory.rs` L192: `SELECT ... activity_description ...` → `ScreenshotAnalysisInfo.activity_description`
  - 前端 API 暴露

---

### 5. `key_elements`

| 属性 | 值 |
|------|----|
| 类型 | `TEXT NOT NULL DEFAULT '[]'`（JSON 数组） |
| 来源 | AI 分析输出 |
| 状态 | **行为不一致** — 加载后未进入 `ActivitySession.tags` |

**消费情况：**
- `activity_grouper.rs` L209: `SELECT ... sa.key_elements ...`
  - 反序列化为 `Vec<String>` → `AnalyzedRecording.key_elements`
  - 加载后未见写入 `ActivitySession.tags`（仅 `context_tags` 写入了 tags）
- 其余下游模块：**无消费**

**问题：** 与 `context_tags` 行为不一致，`key_elements` 加载后实际上是 dead load。

---

### 6. `ocr_text`

| 属性 | 值 |
|------|----|
| 类型 | `TEXT`（nullable） |
| 来源 | AI 分析输出（屏幕重要文字） |
| 状态 | **存储但无任何消费者** |

**消费情况：**
- `activity_grouper.rs`: **未 SELECT**
- `project_extractor.rs`: **未 SELECT**
- `summary_generator.rs`: **未 SELECT**
- `commands/memory.rs`: **未 SELECT**
- `markdown_generator.rs`: 不访问此字段

**问题：** OCR 文本写入后完全无下游消费，既不进搜索索引，也不进 Markdown，也不暴露给前端。

---

### 7. `context_tags`

| 属性 | 值 |
|------|----|
| 类型 | `TEXT NOT NULL DEFAULT '[]'`（JSON 数组） |
| 来源 | AI 分析输出（2–5 个上下文标签） |
| 状态 | 正常，活跃使用 |

**消费情况：**
- `activity_grouper.rs` L210: `SELECT ... sa.context_tags ...`
  - 反序列化为 `Vec<String>` → `AnalyzedRecording.context_tags`
  - 写入 `ActivitySession.tags`，用于活动分组重叠评分

---

### 8. `productivity_score`

| 属性 | 值 |
|------|----|
| 类型 | `INTEGER DEFAULT 5` |
| 来源 | AI 分析输出（1–10 分） |
| 状态 | **Dead code** — `activity_grouper` 加载但未使用 |

**消费情况：**
- `activity_grouper.rs` L211: `SELECT ... sa.productivity_score ...`
  - 加载到 `AnalyzedRecording.productivity_score`
  - **未见在后续逻辑中使用**（分组、标题生成均不依赖此值）
- 其余下游模块：**无消费**

**问题：** 冗余加载，增加 SELECT 开销但无实际价值。

---

### 9. `analysis_json`

| 属性 | 值 |
|------|----|
| 类型 | `TEXT NOT NULL` |
| 来源 | AI 原始响应 JSON（完整 blob） |
| 状态 | 兜底存储，无主动消费 |

**消费情况：**
- 所有下游模块：**未 SELECT**
- 仅作为历史审计/调试用途保留

---

### 10. `analyzed_at`

| 属性 | 值 |
|------|----|
| 类型 | `INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))` |
| 来源 | 写入时自动填充 |
| 状态 | 正常 |

**消费情况：**
- `commands/memory.rs` L193: `SELECT ... analyzed_at ...` → `ScreenshotAnalysisInfo.analyzed_at`
  - 前端 API 暴露（排序、显示用）

---

### 11. `activity_category` *(V5 新增)*

| 属性 | 值 |
|------|----|
| 类型 | `TEXT DEFAULT 'other'` |
| 来源 | AI 分析输出 |
| 状态 | **枚举不完整** — `learning` 值 parse 时 fallthrough |

**消费情况：**
- `activity_grouper.rs` L208: `SELECT ... sa.activity_category ...`
  - `unwrap_or_else(|| "other".to_string())` → `AnalyzedRecording.activity_category`
  - 用于：映射到 `ActivityCategory` 枚举（`Work`/`Entertainment`/`Communication`/`Other`）

**问题：** `README.md` 和 AI prompt 中定义了 `learning` 枚举值，但 `ActivityCategory` 枚举缺少对应变体，parse 时 `learning` 值会 fallthrough 到 `Other`。

---

### 12. `activity_summary` *(V5 新增)*

| 属性 | 值 |
|------|----|
| 类型 | `TEXT DEFAULT ''` |
| 来源 | AI 分析输出（2–3 句详细摘要） |
| 状态 | 正常，但路径间接 |

**消费情况：**
- `activity_grouper.rs` L209: `SELECT ... sa.activity_summary ...`
  - `unwrap_or_default()` → `AnalyzedRecording.activity_summary`
  - 聚合后写入 `ActivitySession`，`markdown_generator` 通过 `ScreenshotAnalysisSummary.analysis` 字段接收（注意字段名不同）

---

### 13. `project_name` *(V5 新增)*

| 属性 | 值 |
|------|----|
| 类型 | `TEXT`（nullable） |
| 来源 | AI 分析输出 |
| 状态 | **部分 Dead code** — `activity_grouper` 加载但未使用 |

**消费情况：**
- `activity_grouper.rs` L212: `SELECT ... sa.project_name ...`
  - 加载到 `AnalyzedRecording.project_name`
  - **未见在分组逻辑中使用**（仅 `context_tags` 参与分组评分）
- `project_extractor.rs` L113-121: 专用查询（`SELECT project_name, COUNT(*) ... GROUP BY project_name`）
  - **活跃使用**：投票选出最频繁项目名称

**问题：** `activity_grouper` 中的加载是冗余的，可从该模块的 SELECT 中移除。

---

### 14. `accomplishments` *(V5 新增)*

| 属性 | 值 |
|------|----|
| 类型 | `TEXT DEFAULT '[]'`（JSON 数组） |
| 来源 | AI 分析输出（完成事项列表） |
| 状态 | 正常，单一专用消费者 |

**消费情况：**
- `summary_generator.rs` L303-306: 专用查询（`SELECT accomplishments FROM screenshot_analyses WHERE ...`）
  - 反序列化后聚合用于生成日/周/月总结

---

## 字段状态汇总表

| 字段 | activity_grouper | project_extractor | summary_generator | commands/memory | markdown_gen | 状态 |
|------|:---:|:---:|:---:|:---:|:---:|------|
| `screenshot_id` | JOIN | WHERE | WHERE | SELECT | 间接 | 正常 |
| `application` | SELECT ✓ | — | — | SELECT ✓ | 间接 | 正常 |
| `activity_type` | SELECT ✓ | — | — | SELECT ✓ | — | ⚠️ 废弃不彻底 |
| `activity_description` | SELECT ✓ | — | — | SELECT ✓ | — | 正常 |
| `key_elements` | SELECT（未用）| — | — | — | — | ⚠️ Dead load |
| `ocr_text` | — | — | — | — | — | ❌ 无消费者 |
| `context_tags` | SELECT ✓ | — | — | — | — | 正常 |
| `productivity_score` | SELECT（未用）| — | — | — | — | ⚠️ Dead code |
| `analysis_json` | — | — | — | — | — | 兜底存储 |
| `analyzed_at` | — | — | — | SELECT ✓ | — | 正常 |
| `activity_category` | SELECT ✓ | — | — | — | — | ⚠️ 枚举不完整 |
| `activity_summary` | SELECT ✓ | — | — | — | 间接 ✓ | 正常 |
| `project_name` | SELECT（未用）| SELECT ✓ | — | — | — | ⚠️ 部分 Dead load |
| `accomplishments` | — | — | SELECT ✓ | — | — | 正常 |

图例：`✓` = 实际使用，`（未用）` = SELECT 但后续逻辑不消费，`—` = 未涉及

---

## 当前已知问���

### 问题 1：`activity_type` 废弃不彻底

**位置：**
- `activity_grouper.rs:206`（SELECT）
- `commands/memory.rs:191`（SELECT 并暴露前端）

**描述：** `README.md` 和设计文档标注此字段已废弃，功能由 `activity_category` 接替。但 `activity_grouper` 仍通过此字段做活动类型判断，`commands/memory.rs` 仍将其暴露给前端。

**修复方向：**
1. 在 `activity_grouper` 中将分组逻辑迁移至 `activity_category`
2. 在 `commands/memory.rs` 的 `ScreenshotAnalysisInfo` 中移除 `activity_type`
3. DB 迁移时删除此列（V8）

---

### 问题 2：`productivity_score` 和 `project_name` 在 `ActivityGrouper` 中冗余加载

**位置：** `activity_grouper.rs:211-212`

**描述：** `get_ungrouped_recordings` 的 SELECT 包含 `productivity_score` 和 `project_name`，但 `group_recordings` 和后续的 `ActivitySession` 构建逻辑均未使用这两个字段。

**修复方向：** 从该 SELECT 中移除这两列（`project_name` 由 `project_extractor` 专门查询）。

---

### 问题 3：`activity_category` 缺少 `learning` 枚举值

**位置：** `activity_grouper.rs`（ActivityCategory parse 逻辑）

**描述：** AI prompt 定义了 5 种类别（`work`/`entertainment`/`communication`/`learning`/`other`），`README.md` 也记录了 `learning`，但 `ActivityCategory` 枚举没有 `Learning` 变体，导致 AI 返回 `learning` 时 parse fallthrough 到 `Other`，活动分类失真。

**修复方向：**
1. 在 `ActivityCategory` 枚举中添加 `Learning` 变体
2. 在 `db/schema.rs` 的 `category` 映射中添加对应处理

---

### 问题 4：`key_elements` 未进入 `ActivitySession.tags`

**位置：** `activity_grouper.rs`（`ActivitySession` 构建逻辑）

**描述：** `context_tags` 写入了 `ActivitySession.tags`，但同样 SELECT 进来的 `key_elements` 没有写入 `tags`，行为不一致。两个字段的语义原本都是"活动上下文"，但下游处理路径不同。

**修复方向：** 明确两者用途：
- 方案 A：将 `key_elements` 也合并入 `ActivitySession.tags`
- 方案 B：将 `key_elements` 从该 SELECT 中移除（如果决定完全不用）

---

### 问题 5：`ocr_text` 存储后无任何消费者

**位置：** `db/migrations.rs:372`（建表时定义此列）

**描述：** AI 分析会输出 `ocr_text`（屏幕重要文字），写入数据库后，所有下游模块均未 SELECT 此字段。既不进向量索引，也不进 Markdown 时间线，也不暴露给前端。

**修复方向：**
- 方案 A：接入全文搜索索引（未来实现）
- 方案 B：评估 AI 输出质量后决定是否继续采集
- 方案 C：V8 迁移时暂时保留，标记为"待用"

---

### 问题 6：`activity_type` 与 `activity_category` 语义重叠

**描述：** 这是历史遗留问题的根源。V3 建表时使用 `activity_type`（`TEXT NOT NULL`），V5 新增了更规范的 `activity_category`，但 V3 字段未被迁移移除，导致两套分类并存。

**修复方向：**
1. V8 迁移：移除 `activity_type` 列
2. 更新 `schema.rs` 的 `ScreenshotAnalysis` struct（移除 `activity_type` 字段）
3. 更新所有 SELECT 语句

---

## V2 迁移影响范围（DB V8）

### 需修改的文件

| 文件 | 改动类型 | 说明 |
|------|----------|------|
| `src-tauri/src/db/migrations.rs` | 新增 V8 | 移除 `activity_type`；新增 `window_title`、`url`、`focus_level` 等 V2 字段 |
| `src-tauri/src/db/schema.rs` | 修改 struct | 移除 `activity_type` 字段；新增 V2 字段；添加 `Learning` 枚举变体 |
| `src-tauri/src/memory/activity_grouper.rs` | 修改 SELECT | 移除 `activity_type`、`productivity_score`、`project_name` 冗余列；添加 V2 新字段 |
| `src-tauri/src/commands/memory.rs` | 修改 SELECT | 移除 `activity_type`；更新 `ScreenshotAnalysisInfo` struct |
| `src-tauri/src/memory/one_shot_analyzer.rs` | 修改写入逻辑 | 适配新字段的 INSERT |

### V2 新增字段（待 V8 实现）

参见 `docs/planning/2026-02-21-screen-analysis-output-fields.md`：

| 字段 | 类型 | 用途 |
|------|------|------|
| `window_title` | `TEXT` | 窗口标题（精确分组） |
| `url` | `TEXT` | 浏览器 URL（网页活动区分） |
| `focus_level` | `TEXT` | `deep`/`normal`/`fragmented` |
| `interaction_mode` | `TEXT` | `typing`/`reading`/`navigating`/`watching`/`idle`/`mixed` |
| `is_continuation` | `INTEGER` | 布尔值，供分组器合并连续任务 |
| `people_mentioned` | `TEXT` | JSON 数组，人名 |
| `technologies` | `TEXT` | JSON 数组，技术栈 |
| `file_names` | `TEXT` | JSON 数组，从 key_elements 分离 |
| `error_indicators` | `TEXT` | JSON 数组，错误信息 |

---

## 相关文档

- [数据库设计总览](./README.md)
- [录屏分析字段设计 V2](../planning/2026-02-21-screen-analysis-output-fields.md)
