# 屏幕录屏分析输出字段设计 V2

**日期**: 2026-02-21
**状态**: 等待确认
**目标**: 设计 AI 首次分析录屏时的输出字段，用于记忆系统后续处理

---

## 一、现状分析

### 当前 AIAnalysisResult（V1，11个字段）

```rust
// vision-jarvis/src-tauri/src/memory/screenshot_analyzer.rs

struct AIAnalysisResult {
    application: String,           // 应用名称
    activity_type: String,         // 活动类型（与 activity_category 存在语义重叠）
    activity_description: String,  // 活动描述
    key_elements: Vec<String>,     // 关键元素（混合了URL/文件/代码等，未分类）
    ocr_text: Option<String>,      // OCR识别文本
    context_tags: Vec<String>,     // 上下文标签
    productivity_score: i32,       // 生产力评分 (默认5)
    activity_category: String,     // 活动分类 (默认"other")
    activity_summary: String,      // 活动摘要
    project_name: Option<String>,  // 项目名称
    accomplishments: Vec<String>,  // 完成事项
}
```

### 数据库结构（schema.rs）

```rust
pub struct ScreenshotAnalysis {
    pub screenshot_id: String,
    pub application: String,
    pub activity_type: String,
    pub activity_description: String,
    pub key_elements: Vec<String>,
    pub ocr_text: Option<String>,
    pub context_tags: Vec<String>,
    pub productivity_score: i32,
    pub analysis_json: String,      // 完整JSON存储（兜底）
    pub analyzed_at: i64,
    pub activity_category: String,
    pub activity_summary: String,
    pub project_name: Option<String>,
    pub accomplishments: Vec<String>,
}
```

### 已识别问题

| 问题 | 说明 |
|------|------|
| `activity_type` 与 `activity_category` 语义重叠 | 两者均表示活动分类，但字段名不同，下游代码存在混用 |
| `key_elements` 混杂多种数据类型 | URL、文件名、代码片段混在一个数组，下游无法精确处理 |
| 缺少多窗口感知 | 只记录主 application，无法感知切换/并行使用多个应用 |
| 缺少深度工作指标 | 无法区分「正在深度编码」vs「碎片化浏览」 |
| 缺少技术栈信息 | `project_extractor.rs` 只能从应用名推断，精度低 |

---

## 二、下游消费者字段映射

| 下游组件 | 当前消费字段 | 缺失但有用的字段 |
|----------|-------------|-----------------|
| `activity_grouper.rs` | `application`, `activity_type`, `activity_description`, `activity_category`, `context_tags`, `key_elements`, `activity_summary` | `is_continuation`（合并信号）、`secondary_applications`（多应用感知） |
| `project_extractor.rs` | `project_name` | `technologies`, `file_names`（提高项目识别精度） |
| `habit_detector.rs` | 操作活动会话数据 | `focus_level`（行为模式识别）、`interaction_mode` |
| `summary_generator.rs` | `accomplishments` | `people_mentioned`（会议/协作记录）、`emotional_tone`（心理状态趋势） |
| `markdown_generator.rs` | 通过 `ActivitySession` 的截图分析列表 | `window_title`（丰富 Markdown 输出）、`url` |
| `commands/memory.rs` | `activity_type`, `activity_description` | 暴露更多字段给前端 |

---

## 三、建议输出字段（V2，20个字段）

### Group 1: 应用识别（Application Context）

| 字段名 | 类型 | 说明 | 是否新增 |
|--------|------|------|----------|
| `application` | `String` | 主应用名称（例："VS Code", "Chrome"） | 保留 |
| `window_title` | `Option<String>` | 窗口标题（例："main.rs - vision-jarvis"） | **新增** |
| `url` | `Option<String>` | 当前URL（浏览器场景） | **新增** |

**用途**: `window_title` 用于丰富 Markdown 输出和项目识别；`url` 替代 `key_elements` 中混杂的URL，支持访问历史分析。

### Group 2: 活动分类（Activity Classification）

| 字段名 | 类型 | 说明 | 是否新增 |
|--------|------|------|----------|
| `activity_category` | `String` | 活动大类：`work`/`entertainment`/`communication`/`learning`/`other` | 保留（扩展值域） |
| `productivity_score` | `i32` | 生产力评分 1-10（默认5） | 保留 |
| `focus_level` | `String` | 专注程度：`deep`/`normal`/`fragmented` | **新增** |
| `interaction_mode` | `String` | 交互模式：`typing`/`reading`/`navigating`/`watching`/`idle`/`mixed` | **新增** |
| `is_continuation` | `bool` | 是否与上一个分段同一任务的延续（默认false） | **新增** |

> **注意**: 移除 `activity_type`（与 `activity_category` 语义重叠），`learning` 值合并到 `activity_category` 中。

**用途**:
- `focus_level` → `habit_detector.rs` 识别深度工作习惯
- `interaction_mode` → 行为模式分析（阅读 vs 编写 vs 浏览）
- `is_continuation` → `activity_grouper.rs` 合并连续分段的强信号

### Group 3: 活动描述（Activity Description）

| 字段名 | 类型 | 说明 | 是否新增 |
|--------|------|------|----------|
| `activity_description` | `String` | 单句活动描述（现在时态） | 保留 |
| `activity_summary` | `String` | 多句活动摘要 | 保留 |
| `accomplishments` | `Vec<String>` | 完成事项列表 | 保留 |

### Group 4: 上下文增强（Context Enrichment）

| 字段名 | 类型 | 说明 | 是否新增 |
|--------|------|------|----------|
| `key_elements` | `Vec<String>` | 其他关键元素（不含URL/文件，已分离） | 保留（精简范围） |
| `context_tags` | `Vec<String>` | 上下文标签（例：["debugging", "rust", "memory"]） | 保留 |
| `project_name` | `Option<String>` | 项目名称 | 保留 |
| `people_mentioned` | `Vec<String>` | 出现的人名（邮件/会议/PR场景） | **新增** |
| `technologies` | `Vec<String>` | 技术栈（编程语言/框架/工具）（例：["rust", "tauri", "sqlite"]） | **新增** |

**用途**:
- `people_mentioned` → `summary_generator.rs` 识别协作事件
- `technologies` → `project_extractor.rs` 精确识别技术栈，替代从应用名推断

### Group 5: 内容提取（Content Extraction）

| 字段名 | 类型 | 说明 | 是否新增 |
|--------|------|------|----------|
| `ocr_text` | `Option<String>` | OCR识别文本（原始） | 保留 |
| `file_names` | `Vec<String>` | 出现的文件名（替代混在 `key_elements` 的文件） | **新增** |
| `error_indicators` | `Vec<String>` | 错误/异常信息（例：["compile error", "test failed"]） | **新增** |

**用途**:
- `file_names` → 文件访问历史，帮助项目关联
- `error_indicators` → 识别调试行为，触发学习习惯检测

---

## 四、建议的 Rust 类型定义

```rust
/// AI返回的分析结果 V2
/// 一次性提取所有下游组件需要的信息
#[derive(Debug, serde::Deserialize)]
struct AIAnalysisResult {
    // === Group 1: 应用识别 ===
    application: String,
    #[serde(default)]
    window_title: Option<String>,
    #[serde(default)]
    url: Option<String>,

    // === Group 2: 活动分类 ===
    /// work | entertainment | communication | learning | other
    #[serde(default = "default_activity_category")]
    activity_category: String,
    #[serde(default = "default_productivity_score")]
    productivity_score: i32,
    /// deep | normal | fragmented
    #[serde(default = "default_focus_level")]
    focus_level: String,
    /// typing | reading | navigating | watching | idle | mixed
    #[serde(default = "default_interaction_mode")]
    interaction_mode: String,
    #[serde(default)]
    is_continuation: bool,

    // === Group 3: 活动描述 ===
    activity_description: String,
    #[serde(default)]
    activity_summary: String,
    #[serde(default)]
    accomplishments: Vec<String>,

    // === Group 4: 上下文增强 ===
    #[serde(default)]
    key_elements: Vec<String>,
    #[serde(default)]
    context_tags: Vec<String>,
    #[serde(default)]
    project_name: Option<String>,
    #[serde(default)]
    people_mentioned: Vec<String>,
    #[serde(default)]
    technologies: Vec<String>,

    // === Group 5: 内容提取 ===
    #[serde(default)]
    ocr_text: Option<String>,
    #[serde(default)]
    file_names: Vec<String>,
    #[serde(default)]
    error_indicators: Vec<String>,
}

fn default_productivity_score() -> i32 { 5 }
fn default_activity_category() -> String { "other".to_string() }
fn default_focus_level() -> String { "normal".to_string() }
fn default_interaction_mode() -> String { "mixed".to_string() }
```

---

## 五、数据库迁移

需要在 `migrations.rs` 新增 V8 迁移，为 `screenshot_analyses` 表添加新列：

```sql
-- V8: Add V2 analysis fields
ALTER TABLE screenshot_analyses ADD COLUMN window_title TEXT;
ALTER TABLE screenshot_analyses ADD COLUMN url TEXT;
ALTER TABLE screenshot_analyses ADD COLUMN focus_level TEXT NOT NULL DEFAULT 'normal';
ALTER TABLE screenshot_analyses ADD COLUMN interaction_mode TEXT NOT NULL DEFAULT 'mixed';
ALTER TABLE screenshot_analyses ADD COLUMN is_continuation INTEGER NOT NULL DEFAULT 0;
ALTER TABLE screenshot_analyses ADD COLUMN people_mentioned TEXT NOT NULL DEFAULT '[]';
ALTER TABLE screenshot_analyses ADD COLUMN technologies TEXT NOT NULL DEFAULT '[]';
ALTER TABLE screenshot_analyses ADD COLUMN file_names TEXT NOT NULL DEFAULT '[]';
ALTER TABLE screenshot_analyses ADD COLUMN error_indicators TEXT NOT NULL DEFAULT '[]';
```

---

## 六、AI Prompt 更新

需更新 `recording_understanding_prompt()` 函数，要求模型输出新字段。关键指令：

```
请分析此屏幕录制视频，输出 JSON 格式的结构化信息：

{
  "application": "主应用名称",
  "window_title": "窗口标题（可选）",
  "url": "当前URL（浏览器场景，可选）",
  "activity_category": "work|entertainment|communication|learning|other",
  "productivity_score": 1到10的整数,
  "focus_level": "deep|normal|fragmented",
  "interaction_mode": "typing|reading|navigating|watching|idle|mixed",
  "is_continuation": false,
  "activity_description": "单句描述用户正在做什么",
  "activity_summary": "2-3句详细说明",
  "accomplishments": ["已完成事项1", "已完成事项2"],
  "key_elements": ["其他关键元素"],
  "context_tags": ["标签1", "标签2"],
  "project_name": "项目名称（可选）",
  "people_mentioned": ["涉及的人名"],
  "technologies": ["rust", "tauri"],
  "ocr_text": "屏幕上的重要文字（可选）",
  "file_names": ["main.rs", "schema.rs"],
  "error_indicators": ["编译错误信息"]
}
```

---

## 七、实施步骤

### Phase 1：更新 AIAnalysisResult 结构体
- 文件: `screenshot_analyzer.rs`
- 移除 `activity_type` 字段
- 添加 5 个新字段
- 添加对应的 default 函数

### Phase 2：更新 AI Prompt
- 文件: `screenshot_analyzer.rs` 中的 `recording_understanding_prompt()`
- 更新输出字段说明和示例 JSON

### Phase 3：更新数据库 Schema
- 文件: `db/schema.rs` 中的 `ScreenshotAnalysis` 结构体
- 文件: `db/migrations.rs` 添加 V8 迁移

### Phase 4：更新 save_analysis() 函数
- 文件: `screenshot_analyzer.rs`
- 更新 INSERT 语句包含新字段

### Phase 5：更新下游消费者
- `activity_grouper.rs`: 使用 `is_continuation` 优化合并逻辑
- `project_extractor.rs`: 利用 `technologies` 和 `file_names` 提升识别精度
- `commands/memory.rs`: 在 `ScreenshotAnalysisInfo` 中暴露新字段

---

## 八、风险评估

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| AI 输出新字段质量不稳定 | MEDIUM | 所有新字段都有 `#[serde(default)]`，解析失败不阻断流程 |
| 移除 `activity_type` 破坏现有代码 | HIGH | 需全量搜索引用并替换为 `activity_category` |
| DB 迁移在已有数据库上的兼容性 | LOW | 所有新列都有 DEFAULT 值 |
| Token 成本增加 | LOW | 新字段约增加 40-75% response tokens，成本影响可忽略 |

---

**等待用户确认**: 请确认此设计方案后，再开始实施。如需调整（例如减少字段、修改分组），请说明修改意见。
