# 记忆服务 (MemoryService)

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **功能**: 短期记忆生成、意图识别、事项提取

---

## 目录

- [功能概述](#功能概述)
- [核心流程](#核心流程)
- [意图识别算法](#意图识别算法)
- [API 接口](#api-接口)
- [数据模型](#数据模型)
- [时间窗口管理](#时间窗口管理)

---

## 功能概述

记忆服务负责根据用户的截图和应用使用记录，智能生成短期记忆事项。

### 核心功能

1. **意图识别**: 分析用户行为，识别正在进行的事项
2. **事项提取**: 从截图序列中提取关键信息
3. **记忆生成**: 生成结构化的短期记忆片段
4. **记忆查询**: 支持向量搜索和语义查询
5. **长期记忆**: 聚合短期记忆生成长期总结

---

## 核心流程

### 短期记忆生成流程

```
[触发条件检测]
    ├─ Case 1: 应用切换
    │   └─ current_app != last_app
    ├─ Case 2: 时间窗口结束
    │   └─ 每 30 分钟自动触发
    └─ Case 3: 用户主动触发
        └─ 前端点击"生成记忆"
    ↓
[收集时间窗口内的数据]
    ├─ 查询 D1: 时间窗口内的所有截图
    │   └─ SELECT * FROM screenshots
    │       WHERE timestamp BETWEEN start AND end
    │       ORDER BY timestamp ASC
    ├─ 查询 D4: 应用使用记录
    │   └─ SELECT * FROM app_usage
    │       WHERE start_time BETWEEN start AND end
    └─ 获取用户 Todo 列表（如果有）
    ↓
[数据预处理]
    ├─ 过滤无效截图
    │   ├─ 跳过黑屏/锁屏
    │   └─ 跳过分析失败的截图
    ├─ 分组截图
    │   └─ 按应用分组: {app_name: [screenshots]}
    └─ 提取 OCR 文本
        └─ 合并所有 screenshot.ocr_text
    ↓
[意图识别]
    ├─ 分析应用使用模式
    │   ├─ 主要使用的应用 (占比 > 60%)
    │   ├─ 应用切换频率
    │   └─ 应用使用时长
    ├─ 分析截图内容
    │   ├─ OCR 文本关键词提取
    │   ├─ AI 摘要��容分析
    │   └─ 识别事项类型
    │       ├─ 编码: VSCode, JetBrains, etc.
    │       ├─ 文档: Word, Google Docs, etc.
    │       ├─ 会议: Zoom, Teams, etc.
    │       ├─ 浏览: Chrome, Safari, etc.
    │       └─ 其他
    └─ 匹配 Todo 列表
        └─ 如果关键词匹配 Todo，关联事项
    ↓
[生成记忆片段]
    ├─ 提取关键点
    │   ├─ 主要事项: "编写 Vision-Jarvis 后端文档"
    │   ├─ 使用工具: "VSCode"
    │   ├─ 涉及文件: ["memory-service.md", "api.md"]
    │   └─ 关键操作: "编辑 Markdown 文件"
    ├─ 生成总结
    │   └─ 调用 AI: "根据以下截图序列，生成一句话总结用户正在做什么"
    ├─ 计算时长
    │   └─ duration = end_time - start_time
    └─ 提取建议与分析
        └─ AI 生成: "建议定期保存文档，避免数据丢失"
    ↓
[创建 D3 记录]
    ├─ INSERT INTO short_term_memory (
    │     start_time,
    │     end_time,
    │     event_title,
    │     event_summary,
    │     keywords,
    │     screenshot_ids,
    │     app_names,
    │     suggestions
    │   )
    └─ 返回 memory_id
    ↓
[关联截图和应用]
    ├─ 更新 D1: 将 screenshot.memory_id 设置为 memory_id
    └─ 更新 D4: 将 app_usage.memory_id 设置为 memory_id
    ↓
[返回生成的记忆]
```

---

## 意图识别算法

### 算法概述

```rust
pub async fn identify_intent(
    &self,
    screenshots: &[Screenshot],
    app_usage: &[AppUsage],
) -> Result<Intent> {
    // 1. 应用使用分析
    let app_pattern = self.analyze_app_pattern(app_usage).await?;

    // 2. 内容关键词提取
    let keywords = self.extract_keywords_from_screenshots(screenshots).await?;

    // 3. 事项类型识别
    let activity_type = self.classify_activity(&app_pattern, &keywords).await?;

    // 4. 生成意图描述
    let description = self.generate_description(&activity_type, &keywords).await?;

    Ok(Intent {
        activity_type,
        description,
        keywords,
        confidence: self.calculate_confidence(&app_pattern, &keywords),
    })
}
```

### 应用使用模式分析

```rust
async fn analyze_app_pattern(&self, app_usage: &[AppUsage]) -> Result<AppPattern> {
    // 1. 计算每个应用的使用时长
    let mut app_durations: HashMap<String, i64> = HashMap::new();
    for usage in app_usage {
        let duration = (usage.end_time - usage.start_time).num_seconds();
        *app_durations.entry(usage.app_name.clone()).or_insert(0) += duration;
    }

    // 2. 找出主要应用 (占比 > 60%)
    let total_duration: i64 = app_durations.values().sum();
    let primary_app = app_durations
        .iter()
        .max_by_key(|(_, &duration)| duration)
        .map(|(app, duration)| {
            let percentage = (*duration as f64 / total_duration as f64) * 100.0;
            (app.clone(), percentage)
        });

    // 3. 计算应用切换频率
    let switch_count = app_usage.len();
    let switch_frequency = switch_count as f64 / (total_duration as f64 / 60.0); // 每分钟切换次数

    Ok(AppPattern {
        primary_app,
        total_apps: app_durations.len(),
        switch_frequency,
        durations: app_durations,
    })
}
```

### 事项类型分类

```rust
async fn classify_activity(
    &self,
    app_pattern: &AppPattern,
    keywords: &[String],
) -> Result<ActivityType> {
    // 基于应用名称的规则匹配
    if let Some((app_name, _)) = &app_pattern.primary_app {
        if CODING_APPS.contains(&app_name.as_str()) {
            return Ok(ActivityType::Coding);
        }
        if MEETING_APPS.contains(&app_name.as_str()) {
            return Ok(ActivityType::Meeting);
        }
        if WRITING_APPS.contains(&app_name.as_str()) {
            return Ok(ActivityType::Writing);
        }
        if BROWSING_APPS.contains(&app_name.as_str()) {
            // 进一步分析关键词
            if keywords.iter().any(|k| RESEARCH_KEYWORDS.contains(&k.as_str())) {
                return Ok(ActivityType::Research);
            }
            return Ok(ActivityType::Browsing);
        }
    }

    // 基于关键词的机器学习分类（未来）
    // let activity = self.ml_classifier.predict(keywords).await?;

    Ok(ActivityType::Other)
}

// 预定义应用分类
const CODING_APPS: &[&str] = &[
    "Visual Studio Code",
    "IntelliJ IDEA",
    "PyCharm",
    "WebStorm",
    "Xcode",
    "Android Studio",
];

const MEETING_APPS: &[&str] = &[
    "Zoom",
    "Microsoft Teams",
    "Google Meet",
    "Slack",
    "Discord",
];

const WRITING_APPS: &[&str] = &[
    "Microsoft Word",
    "Google Docs",
    "Notion",
    "Obsidian",
    "Typora",
];

const BROWSING_APPS: &[&str] = &[
    "Google Chrome",
    "Safari",
    "Firefox",
    "Microsoft Edge",
];
```

---

## API 接口

### Service API

```rust
pub struct MemoryService {
    memory_repo: Arc<MemoryRepository>,
    screenshot_repo: Arc<ScreenshotRepository>,
    ai_service: Arc<AIService>,
}

impl MemoryService {
    /// 生成短期记忆
    ///
    /// # 参数
    /// - time_window: 时间窗口 (开始时间, 结束时间)
    ///
    /// # 返回
    /// - Ok(ShortTermMemory): 生成的记忆片段
    /// - Err(AppError): 生成失败
    pub async fn generate_memory(
        &self,
        time_window: TimeWindow,
    ) -> Result<ShortTermMemory> {
        // 1. 收集数据
        let screenshots = self.screenshot_repo
            .get_by_time_range(time_window.start, time_window.end)
            .await?;

        let app_usage = self.app_usage_repo
            .get_by_time_range(time_window.start, time_window.end)
            .await?;

        // 2. 意图识别
        let intent = self.identify_intent(&screenshots, &app_usage).await?;

        // 3. 生成总结
        let summary = self.ai_service
            .generate_summary(&screenshots, &intent)
            .await?;

        // 4. 提取建议
        let suggestions = self.ai_service
            .generate_suggestions(&intent, &app_usage)
            .await?;

        // 5. 创建记忆记录
        let memory = self.memory_repo.create(CreateShortTermMemory {
            start_time: time_window.start,
            end_time: time_window.end,
            event_title: intent.description,
            event_summary: summary,
            keywords: intent.keywords.join(", "),
            screenshot_ids: screenshots.iter().map(|s| s.id).collect(),
            app_names: app_usage.iter().map(|a| a.app_name.clone()).collect(),
            suggestions: Some(suggestions),
        }).await?;

        Ok(memory)
    }

    /// 查询记忆（向量搜索）
    ///
    /// # 参数
    /// - query: 用户查询文本
    /// - limit: 返回结果数量
    ///
    /// # 返回
    /// - Ok(Vec<ShortTermMemory>): 匹配的记忆列表
    pub async fn search_memory(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ShortTermMemory>> {
        // 1. 生成查询向量
        let query_embedding = self.ai_service
            .generate_embedding(query)
            .await?;

        // 2. 向量相似度搜索
        let results = self.memory_repo
            .search_by_embedding(query_embedding, limit)
            .await?;

        Ok(results)
    }

    /// 获取指定日期的记忆列表
    pub async fn get_memories_by_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<ShortTermMemory>> {
        let start = date.and_hms_opt(0, 0, 0).unwrap();
        let end = date.and_hms_opt(23, 59, 59).unwrap();

        self.memory_repo
            .get_by_time_range(start, end)
            .await
    }

    /// 生成长期记忆（聚合短期记忆）
    pub async fn generate_long_term_memory(
        &self,
        date_range: (NaiveDate, NaiveDate),
    ) -> Result<LongTermMemory> {
        // 1. 获取日期范围内的所有短期记忆
        let short_memories = self.memory_repo
            .get_by_date_range(date_range.0, date_range.1)
            .await?;

        // 2. 按事项类型分组
        let grouped = self.group_by_activity_type(&short_memories);

        // 3. AI 生成长期总结
        let summary = self.ai_service
            .generate_long_term_summary(&short_memories)
            .await?;

        // 4. 提取主要事项
        let main_events = self.extract_main_events(&grouped);

        // 5. 创建长期记忆记录
        let long_memory = self.long_memory_repo.create(CreateLongTermMemory {
            start_date: date_range.0,
            end_date: date_range.1,
            summary,
            main_events,
            total_memories: short_memories.len() as i32,
        }).await?;

        Ok(long_memory)
    }
}
```

---

## 数据模型

### ShortTermMemory

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTermMemory {
    pub id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub event_title: String,        // "编写 Vision-Jarvis 后端文档"
    pub event_summary: String,       // AI 生成的详细总结
    pub keywords: String,            // "后端, 文档, Markdown, Rust"
    pub screenshot_ids: Vec<i64>,    // 关联的截图 IDs
    pub app_names: Vec<String>,      // 涉及的应用
    pub suggestions: Option<String>, // AI 生成的建议
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShortTermMemory {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub event_title: String,
    pub event_summary: String,
    pub keywords: String,
    pub screenshot_ids: Vec<i64>,
    pub app_names: Vec<String>,
    pub suggestions: Option<String>,
}
```

### Intent (意图)

```rust
#[derive(Debug, Clone)]
pub struct Intent {
    pub activity_type: ActivityType,
    pub description: String,      // "正在编写项目文档"
    pub keywords: Vec<String>,    // ["文档", "Markdown", "后端"]
    pub confidence: f64,          // 0.95 (95% 置信度)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Coding,      // 编码
    Writing,     // 写作
    Meeting,     // 会议
    Research,    // 研究/浏览
    Browsing,    // 一般浏览
    Communication, // 沟通
    Other,       // 其他
}
```

---

## 时间窗口管理

### TimeWindow

```rust
#[derive(Debug, Clone)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeWindow {
    /// 创建新的时间窗口
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    /// 从当前时间向前推 N 分钟
    pub fn from_now_back(minutes: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::minutes(minutes);
        Self { start, end }
    }

    /// 获取今天的时间窗口
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
        Self {
            start: DateTime::from_naive_utc_and_offset(start, Utc),
            end: DateTime::from_naive_utc_and_offset(end, Utc),
        }
    }

    /// 获取时长（秒）
    pub fn duration_seconds(&self) -> i64 {
        (self.end - self.start).num_seconds()
    }
}
```

### 自动触发策略

```rust
impl MemoryService {
    /// 启动自动记忆生成定时器
    pub async fn start_auto_generation(&self) {
        let service = Arc::clone(self);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(30 * 60) // 每 30 分钟
            );

            loop {
                interval.tick().await;

                // 生成过去 30 分钟的记忆
                let time_window = TimeWindow::from_now_back(30);

                match service.generate_memory(time_window).await {
                    Ok(memory) => {
                        info!("Auto-generated memory: {}", memory.event_title);
                    }
                    Err(e) => {
                        error!("Failed to auto-generate memory: {}", e);
                    }
                }
            }
        });
    }
}
```

---

## 边界条件

### 1. 最小记忆条件

- 最少截图数: 3 张
- 最小时长: 5 分钟
- 必须有 AI 分析完成的截图

### 2. 异常处理

- 无截图: 返回错误 "No screenshots in time window"
- 所有截图分析失败: 使用应用名称生成基础记忆
- AI 服务失败: 使用规则生成简单总结

---

## 性能指标

| 指标 | 目标值 |
|------|--------|
| 生成延迟 | < 500ms |
| 查询延迟 | < 100ms |
| 向量搜索精度 | > 85% |

---

## 相关文档

- [AI 服务](ai-service.md)
- [截屏服务](screenshot-service.md)
- [记忆 API](../../api/endpoints/memory.md)
- [数据库设计 - D3 表](../../database/schema/tables/short_term_memory.md)

---

**维护者**: 后端服务组
**最后更新**: 2026-02-04
