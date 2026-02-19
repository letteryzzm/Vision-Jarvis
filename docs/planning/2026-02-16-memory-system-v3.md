# Vision-Jarvis 主动式AI记忆系统 - 实现计划

**日期**: 2026-02-16
**状态**: Phase 1 ✅ Phase 2 🔧进行中，视频录制+AI分析已接入

---

## 一、存储设计评估与改进

### 用户原始方案

```
Data/
  shots/{时间段}/          # 视频+JSON
  long_term_memory/        # 日/段总结
  project/                 # 项目追踪
  habits/                  # 习惯追踪
```

### 最终方案：FFmpeg 连续视频录制 + 分段存储

~~原方案：截图按日期和时段组织，每个时段内的截图压缩为 mp4 视频。~~

**新方案（已实现）**：使用 FFmpeg `avfoundation` 直接录制屏幕视频流，按固定时长分段保存 mp4。解决了离散截图丢失操作上下文的问题。

```
Data/
  recordings/                    # 视频录制存储（新）
    20231027/                    # 按日期分目录 (YYYYMMDD)
      0:00-12:00/                # 时段1: 凌晨到中午
        10-00-01_{uuid}.mp4      # 录制分段 (HH-MM-SS_{uuid})
        10-05-01_{uuid}.mp4      # 下一个分段
      12:00-18:00/               # 时段2: 中午到傍晚
      18:00-24:00/               # 时段3: 傍晚到午夜
  long_term_memory/              # 长期记忆
    daily_summary/               # 每日总结
    range_summary/               # 时段/周/月总结
  project/                       # 项目追踪
  habits/                        # 习惯模式追踪
```

### 存储策略

1. FFmpeg 以 2fps 连续录制屏幕，按可配置时长（默认5分钟）自动分段
2. 录制分段按日期 `YYYYMMDD` + 时段目录组织
3. 每个分段自动写入 `recordings` 表（start_time, end_time, duration）
4. 分析结果存入 SQLite（查询效率高）
5. 长期记忆（总结、项目、习惯）独立存储，不随录制清理
6. 编码参数：`libx264 -preset ultrafast -crf 30`，平衡质量与性能

---

## 二、数据库 Schema V3+V4（已实现）

V3 新增5张表，V4 新增 `recordings` 表：

| 表名 | 用途 |
|------|------|
| `screenshot_analyses` | AI截图理解缓存（每帧JSON） |
| `projects` | 项目追踪（自动提取+匹配） |
| `habits` | 行为模式（时间/触发/序列） |
| `summaries` | 日/周/月总结 |
| `proactive_suggestions` | 主动建议记录 |
| `recordings` | **V4新增** - 视频录制分段记录 |

---

## 三、AI处理逻辑框架

### 数据流

```
FFmpeg录制(2fps, 默认60s分段) → AI视频分析(90s批量, Gemini inline_data)
    ↓                                    ↓
recordings表 +                    activities表 + Markdown
screenshot_analyses表                   ↓
                              项目提取 + 习惯检测(每日)
                                        ↓
                              日总结(23:00) → 周/月总结
                                        ↓
                              主动建议触发(实时)
```

### Level 1: 录制分段分析

- 每90秒批量分析未处理的录制分段
- 视频直接发送给AI（Gemini inline_data 格式，非 OpenAI image_url）
- 返回结构化JSON：application, activity_type, key_elements, productivity_score
- 存入 `screenshot_analyses` 表，标记 `recordings.analyzed=1`

### Level 2: 活动分组

- 利用 `screenshot_analyses` 数据改进分组
- 按应用+活动类型+时间间隔聚合
- 生成活动Markdown文件

### Level 3: 项目提取

- AI提取项目名称 → 与现有项目相似度匹配
- 相似度 > 0.6 归入现有项目，否则创建新项目
- 支持规则提取（fallback）：开发类应用、学习类关键词

### Level 4: 习惯检测

三种模式：
- **时间模式**: 按(application, hour)统计，计算频率+稳定性
- **触发模式**: 构建应用转移矩阵，计算条件概率P(B|A)
- **序列模式**: 检测长度3的应用序列

### Level 5: 主动建议（Phase 4实现）

触发器类型：
- 习惯提醒（基于时间模式）
- 上下文切换警告（频繁切换检测）
- 休息提醒（长时间工作）
- 项目进度提醒（长时间未活动）

---

## 四、分阶段实现计划

### Phase 1: 基础框架 ✅ 已完成

- [x] 数据库 Schema V3 迁移（5张新表）
- [x] 数据结构定义（schema.rs）
- [x] 截图分析器（screenshot_analyzer.rs）
- [x] 总结生成器（summary_generator.rs）
- [x] 项目提取器（project_extractor.rs）
- [x] 习惯检测器（habit_detector.rs）
- [x] 管道调度器V3（pipeline.rs 集成所有组件）
- [x] 编译验证通过

### Phase 2: AI接入与调优 🔧 进行中

- [x] 将录制分析器接入实际AI API（Gemini inline_data 格式）
- [x] 视频分析Prompt（录制分段直接发送，不提取帧）
- [x] AIClient 支持 InlineData 格式（video/mp4）
- [ ] 活动分组器适配 recordings 表（当前仍查 screenshots）
- [ ] summary_generator / project_extractor 动态接入 AI 客户端
- [ ] 实现Embedding生成（用于语义搜索和项目匹配）
- [ ] 实现AI驱动的日总结

### Phase 3: 行为模式学习

- [ ] 调优习惯检测算法参数
- [ ] 实现习惯Markdown自动更新
- [ ] 添加习惯衰减机制（长时间未出现降低置信度）
- [ ] 实现习惯可视化数据导出

### Phase 4: 主动建议系统

- [ ] 实现 ProactiveSuggestionEngine
- [ ] 集成到通知系统（notification/）
- [ ] 实现用户反馈循环（dismissed/accepted/snoozed）
- [ ] 基于反馈调整建议策略

### Phase 5: 用户回顾界面

- [ ] 前端记忆浏览器增强
- [ ] 项目时间线视图
- [ ] 习惯仪表盘
- [ ] 日/周总结展示
- [ ] 视频回放功能（直接播放录制分段）

---

## 五、技术决策

| 决策 | 方案 | 理由 |
|------|------|------|
| 屏幕捕获方式 | FFmpeg avfoundation 连续录制 | 保留完整操作上下文，避免截图间信息丢失 |
| 录制参数 | 2fps, libx264 ultrafast crf30 | 低CPU占用，文件体积小，足够AI分析 |
| 分段策略 | 可配置时长（默认60秒，范围30-300） | 平衡文件大小和处理粒度 |
| 视频发送格式 | Gemini inline_data（非 OpenAI image_url） | OpenAI兼容层不支持video MIME type |
| 分析模型 | gemini-2.5-flash-lite（通过 aihubmix） | 支持视频输入，速度快，成本低 |
| 项目匹配 | 字符Jaccard + 子串匹配 | 简单有效，后续可升级为Embedding |
| 习惯检测 | 统计学方法 | 不依赖AI，本地计算快 |
| AI调用频率 | 90秒批量 | 匹配60秒分段周期，留30秒缓冲 |

---

## 六、风险评估

| 风险 | 级别 | 缓解措施 |
|------|------|----------|
| AI API调用成本 | 中 | 批量处理、缓存、选择性分析 |
| 存储增长 | 低 | 已有清理机制，分析结果压缩存储 |
| 隐私 | 低 | 全部本地处理，无云端上传 |
| 习惯检测准确性 | 中 | 设置最小置信度阈值，用户可修正 |
| 项目匹配误判 | 中 | 相似度阈值可调，支持用户手动归类 |

---

**最后更新**: 2026-02-18 (视频录制+AI分析已接入，活动分组待适配)
