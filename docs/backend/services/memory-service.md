# 记忆服务 (Memory System V3)

> **最后更新**: 2026-02-21
> **版本**: v3.2（AI 传播链路修复 + 前端命令接口重写 + V1 清理）
> **架构**: 主动式 AI 记忆系统（管道调度）

---

## 数据流总览

```
FFmpeg录制(2fps) → mp4分段 → recordings表
        ↓ 每90秒
AI视频分析(Gemini inline_data) → screenshot_analyses表
        ↓ 每30分钟
活动分组 → activities表 + Markdown
   ├── 项目提取 → projects表
   ├── 索引同步(每10分钟) → memory_chunks表
   ├── 日总结(23:00) → summaries表
   └── 习惯检测(每日) → habits表
```

## 管道调度

由 `pipeline.rs` 统一调度，使用 `tokio::select!` 并行运行：

| 任务 | 调度间隔 | 模块 | 数据源 |
|------|---------|------|--------|
| 录制分段 AI 分析 | 每 90 秒 | `screenshot_analyzer.rs` | `recordings` 表 |
| 活动分组 + 项目提取 | 每 30 分钟 | `activity_grouper.rs` + `project_extractor.rs` | `screenshot_analyses` 表 |
| 文件索引同步 | 每 10 分钟 | `index_manager.rs` | Markdown 文件 |
| 习惯检测 | 每 24 小时 | `habit_detector.rs` | `activities` 表 |
| 日总结生成 | 每日 23:00 | `summary_generator.rs` | `activities` 表 |

---

## 核心模块详解

### Layer 0: 屏幕录制 (`capture/screen_recorder.rs`)

FFmpeg `avfoundation` 连续录制 macOS 屏幕。

- 编码：`libx264 -preset ultrafast -crf 30`，2fps
- 分段时长：`capture_interval_seconds`（默认 60s，范围 30-300s）
- 存储路径：`recordings/YYYYMMDD/period/HH-MM-SS_{uuid}.mp4`（使用本地时间，非 UTC）
- 分段完成后写入 `recordings` 表（`analyzed=0`）

### Layer 1: AI 视频分析 (`screenshot_analyzer.rs`)

每 90 秒批量分析未处理的录制分段。

- 数据源：`SELECT FROM recordings WHERE analyzed=0 AND end_time IS NOT NULL`
- 发送方式：mp4 → base64 → Gemini `inline_data` 格式（非 OpenAI `image_url`）
- 批量大小：10 个/批，失败重试 2 次

**AI 返回结构**:
```json
{
  "application": "VSCode",
  "activity_type": "work|entertainment|communication|learning|other",
  "activity_description": "在VSCode中编写Rust代码",
  "key_elements": ["main.rs", "cargo.toml"],
  "ocr_text": "fn main() {...}",
  "context_tags": ["coding", "rust"],
  "productivity_score": 8
}
```

存入 `screenshot_analyses` 表，标记 `recordings.analyzed=1`。

### Layer 2: 活动分组 (`activity_grouper.rs`)

每 30 分钟将分析结果聚合为活动会话。

**分组规则**：
- 同一应用 + 相似活动 + 时间间隔 < 5 分钟 → 同一活动
- 合并 `context_tags` 和 `key_elements`（去重）

**拆分条件**：应用切换、间隔 > 5 分钟、持续 > 2 小时、活动类型不兼容

**过滤条件**：至少 2 条记录、持续 > 1 分钟

**输出**：`ActivitySession` → `activities` 表 + Markdown 文件

### Layer 2.5: 项目提取 (`project_extractor.rs`)

分组后立即执行，从活动中识别项目。

**提取策略**（优先级从高到低）：
1. AI 提取（可选）：发送活动信息，返回项目名或 "NONE"
2. 规则提取：开发类应用 → 从标题/标签提取；学习关键词 → 完整标题
3. 长活动（>30min）→ 提取关键词

**匹配**：Jaccard 相似度 + 子串匹配，阈值 0.6

### Layer 3: 索引同步 (`index_manager.rs`)

每 10 分钟扫描 Markdown 文件，增量更新。

- 计算 SHA-256 哈希判断文件是否变化
- 变化的文件：删旧 chunks → 重新分块 → 存入 `memory_chunks`
- Embedding 生成当前未启用

### Layer 4: 日总结 (`summary_generator.rs`)

每日 23:00 自动触发（防重复：记录 `last_summary_date`）。

- AI 模式：发送当日活动描述 → 生成时间分配、成就、效率评估
- 模板模式（fallback）：统计应用使用时长 + 活动列表
- 输出：`summaries` 表 + `summaries/daily/YYYY-MM-DD.md`

### Layer 5: 习惯检测 (`habit_detector.rs`)

每 24 小时运行一次。

| 模式类型 | 算法 | 示例 |
|---------|------|------|
| 时间模式 | 按 (application, hour) 统计频率+稳定性 | "每天 08:00 使用微信" |
| 触发模式 | 应用转移矩阵，计算 P(B\|A)，5 分钟窗口 | "用 VSCode 后通常打开 Chrome" |
| 序列模式 | 3-app 序列检测，30 分钟窗口 | "微信→邮件→日历" |

**衰减机制**：2 倍回溯期未检测到 → 置信度降 30%，低于阈值 30% → 删除

---

## 动态 AI 连接

`PipelineScheduler` 支持运行时接入 AI 客户端：

```rust
// 启动时无需 AI（所有组件可独立运行）
let pipeline = PipelineScheduler::new(db, storage_root, enable_ai)?;

// 用户配置 AI 后动态连接
pipeline.connect_ai(ai_client).await;
```

`connect_ai` 创建 `ScreenshotAnalyzer` 并存入 `Arc<RwLock<Option<...>>>`，同时将 AI client 传播到 `SummaryGenerator` 和 `MarkdownGenerator`（均通过 `set_ai_client()` 动态注入）。分析 tick 检查是否有 analyzer 可用。

---

## 数据库表

| 表名 | 版本 | 用途 |
|------|------|------|
| `recordings` | V4 | 视频录制分段（path, start/end_time, fps, analyzed, activity_id） |
| `screenshot_analyses` | V3+V5 | AI 分析结果（application, activity_type, tags, score, accomplishments） |
| `activities` | V2+V7 | 聚合后的活动会话（V7: 新增 project_id） |
| `projects` | V3+V7 | 自动识别的项目（V7: 新增 updated_at） |
| `habits` | V3 | 检测到的习惯模式 |
| `summaries` | V3 | 日/周/月总结 |
| `memory_chunks` | V2 | Markdown 文本分块（用于搜索） |
| `screenshots` | V1 | 截图记录（遗留，录制模式下不再新增） |

---

## 已知缺口

| 缺口 | 说明 | 优先级 |
|------|------|--------|
| 周/月总结调度 | 只有日总结有自动触发 | 低 |
| Embedding 生成 | index_manager 中已禁用 | 低 |
| get_related_project_ids | summary_generator 中空实现 | 低 |

---

## V2 遗留模块

| 模块 | 说明 |
|------|------|
| `short_term.rs` | ~~短期记忆聚合~~ **已删除**（被 activity_grouper 替代） |
| `ai/analyzer.rs.bak` | ~~废弃备份~~ **已删除** |
| `ai/embeddings.rs.bak` | ~~废弃备份~~ **已删除** |
| `ai/providers.rs.bak` | ~~废弃备份~~ **已删除** |
| V1 前端命令 | ~~查 short_term_memories 的旧命令~~ **已重写为 V3 命令** |

---

## 相关文档

- [架构概述](../architecture/overview.md)
- [记忆系统 V3 规划](../../planning/2026-02-16-memory-system-v3.md)
