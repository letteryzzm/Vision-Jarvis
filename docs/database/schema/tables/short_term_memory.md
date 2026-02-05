# short_term_memory 表 (D3) - 短期记忆表

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **表名**: `short_term_memory`
> **别名**: D3

---

## 表概述

存储用户的短期记忆事项，由 AI 根据截图序列和应用使用记录生成。每条记录代表用户在一段时间内完成的一个事项或活动。

---

## 表结构

### DDL

```sql
CREATE TABLE short_term_memory (
    -- 主键
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 时间范围
    start_time DATETIME NOT NULL,         -- 事项开始时间
    end_time DATETIME NOT NULL,           -- 事项结束时间

    -- 事项内容
    event_title TEXT NOT NULL,            -- 事项标题 (一句话描述)
    event_summary TEXT NOT NULL,          -- 事项详细总结
    keywords TEXT,                        -- 关键词 (逗号分隔)

    -- 关联数据 (JSON 数组)
    screenshot_ids TEXT NOT NULL,         -- 关联的截图 IDs: "[1,2,3]"
    app_names TEXT NOT NULL,              -- 涉及的应用列表: "["VSCode","Chrome"]"

    -- AI 生成内容
    suggestions TEXT,                     -- AI 生成的建议与分析

    -- 元数据
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 索引

```sql
-- 开始时间索引 (按日期查询)
CREATE INDEX idx_memory_start_time ON short_term_memory(start_time);

-- 结束时间索引
CREATE INDEX idx_memory_end_time ON short_term_memory(end_time);

-- 复合索引: 时间范围查询
CREATE INDEX idx_memory_time_range ON short_term_memory(start_time, end_time);
```

---

## 字段详解

### id

- **类型**: INTEGER
- **约束**: PRIMARY KEY, AUTOINCREMENT
- **描述**: 短期记忆唯一标识符

### start_time

- **类型**: DATETIME
- **约束**: NOT NULL
- **描述**: 事项开始时间 (UTC)
- **格式**: ISO 8601
- **示例**: `2026-02-04T10:00:00Z`

### end_time

- **类型**: DATETIME
- **约束**: NOT NULL
- **描述**: 事项结束时间 (UTC)
- **格式**: ISO 8601
- **示例**: `2026-02-04T11:30:00Z`
- **约束**: end_time > start_time

### event_title

- **类型**: TEXT
- **约束**: NOT NULL
- **描述**: 事项标题，一句话概括用户在做什么
- **示例**: `编写 Vision-Jarvis 后端文档`
- **最大长度**: 建议 ≤ 100 字符

### event_summary

- **类型**: TEXT
- **约束**: NOT NULL
- **描述**: AI 生成的事项详细总结
- **示例**:
  ```
  用户在 VSCode 中编写 Vision-Jarvis 项目的后端文档，
  主要涉及记忆服务 (memory-service.md) 和 API 接口设计 (api.md)。
  期间查阅了 Rust 文档和 Tauri 官方文档作为参考。
  ```

### keywords

- **类型**: TEXT
- **约束**: NULLABLE
- **描述**: AI 提取的关键词，逗号分隔
- **示例**: `后端文档, Rust, Tauri, memory-service, API设计`

### screenshot_ids

- **类型**: TEXT (JSON Array)
- **约束**: NOT NULL
- **描述**: 关联的截图 ID 列表，JSON 数组字符串
- **示例**: `"[1,2,3,5,7,9]"`
- **解析**: `serde_json::from_str::<Vec<i64>>()`

### app_names

- **类型**: TEXT (JSON Array)
- **约束**: NOT NULL
- **描述**: 涉及的应用名称列表，JSON 数组字符串
- **示例**: `"[\"Visual Studio Code\",\"Google Chrome\",\"Notion\"]"`
- **解析**: `serde_json::from_str::<Vec<String>>()`

### suggestions

- **类型**: TEXT
- **约束**: NULLABLE
- **描述**: AI 生成的建议与分析
- **示例**:
  ```
  建议定期保存文档，避免数据丢失。
  检测到频繁在文档和浏览器之间切换，建议使用双屏提升效率。
  ```

### created_at

- **类型**: DATETIME
- **约束**: NOT NULL, DEFAULT CURRENT_TIMESTAMP
- **描述**: 记忆生成时间

### updated_at

- **类型**: DATETIME
- **约束**: NOT NULL, DEFAULT CURRENT_TIMESTAMP
- **描述**: 记忆最后更新时间

---

## 数据示例

### 插入示例

```sql
INSERT INTO short_term_memory (
    start_time,
    end_time,
    event_title,
    event_summary,
    keywords,
    screenshot_ids,
    app_names,
    suggestions
) VALUES (
    '2026-02-04T10:00:00Z',
    '2026-02-04T11:30:00Z',
    '编写 Vision-Jarvis 后端文档',
    '用户在 VSCode 中编写 Vision-Jarvis 项目的后端文档，主要涉及记忆服务和 API 接口设计。',
    '后端文档, Rust, Tauri, memory-service, API设计',
    '[1,2,3,5,7,9,12,15]',
    '["Visual Studio Code","Google Chrome"]',
    '建议定期保存文档，避免数据丢失。'
);
```

### 更新示例

```sql
-- 用户手动编辑记忆标题
UPDATE short_term_memory
SET
    event_title = '完成后端文档编写',
    updated_at = CURRENT_TIMESTAMP
WHERE id = 1;
```

### 查询示例

```sql
-- 查询指定日期的记忆
SELECT * FROM short_term_memory
WHERE DATE(start_time) = '2026-02-04'
ORDER BY start_time ASC;

-- 查询最近 10 条记忆
SELECT * FROM short_term_memory
ORDER BY start_time DESC
LIMIT 10;

-- 查询时间范围内的记忆
SELECT * FROM short_term_memory
WHERE start_time >= '2026-02-01T00:00:00Z'
AND end_time <= '2026-02-07T23:59:59Z'
ORDER BY start_time DESC;

-- 全文搜索 (关键词匹配)
SELECT * FROM short_term_memory
WHERE keywords LIKE '%后端%'
OR event_title LIKE '%后端%'
OR event_summary LIKE '%后端%'
ORDER BY start_time DESC;
```

---

## 关联查询

### 联合查询截图

```sql
-- 查询记忆及其关联的截图
SELECT
    m.id AS memory_id,
    m.event_title,
    m.start_time,
    m.end_time,
    s.id AS screenshot_id,
    s.file_path,
    s.timestamp,
    s.ai_summary
FROM short_term_memory m
JOIN screenshots s ON s.memory_id = m.id
WHERE m.id = 1
ORDER BY s.timestamp ASC;
```

### 联合查询应用使用

```sql
-- 查询记忆及其应用使用统计
SELECT
    m.id,
    m.event_title,
    a.app_name,
    SUM(a.duration_seconds) AS total_duration
FROM short_term_memory m
JOIN app_usage a ON a.memory_id = m.id
WHERE m.id = 1
GROUP BY a.app_name
ORDER BY total_duration DESC;
```

---

## 业务逻辑

### 生成记忆流程

```rust
impl MemoryService {
    pub async fn generate_memory(
        &self,
        time_window: TimeWindow,
    ) -> Result<ShortTermMemory> {
        // 1. 收集截图
        let screenshots = self.screenshot_repo
            .get_by_time_range(time_window.start, time_window.end)
            .await?;

        // 2. 收集应用使用记录
        let app_usage = self.app_usage_repo
            .get_by_time_range(time_window.start, time_window.end)
            .await?;

        // 3. AI 生成事项标题和总结
        let (title, summary, keywords, suggestions) = self.ai_service
            .analyze_user_intent(&screenshots, &app_usage)
            .await?;

        // 4. 提取截图 IDs 和应用列表
        let screenshot_ids: Vec<i64> = screenshots.iter().map(|s| s.id).collect();
        let app_names: Vec<String> = app_usage
            .iter()
            .map(|a| a.app_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // 5. 创建记忆记录
        let memory = self.memory_repo.create(CreateShortTermMemory {
            start_time: time_window.start,
            end_time: time_window.end,
            event_title: title,
            event_summary: summary,
            keywords: keywords.join(", "),
            screenshot_ids: serde_json::to_string(&screenshot_ids)?,
            app_names: serde_json::to_string(&app_names)?,
            suggestions: Some(suggestions),
        }).await?;

        // 6. 更新截图的 memory_id
        for screenshot_id in screenshot_ids {
            self.screenshot_repo
                .update_memory_id(screenshot_id, memory.id)
                .await?;
        }

        // 7. 更新应用使用的 memory_id
        for usage in app_usage {
            self.app_usage_repo
                .update_memory_id(usage.id, memory.id)
                .await?;
        }

        Ok(memory)
    }
}
```

---

## 数据导出

### 导出为 JSON

```rust
#[derive(Serialize)]
pub struct MemoryExport {
    pub id: i64,
    pub start_time: String,
    pub end_time: String,
    pub event_title: String,
    pub event_summary: String,
    pub keywords: Vec<String>,
    pub screenshots: Vec<ScreenshotExport>,
    pub apps: Vec<String>,
    pub suggestions: Option<String>,
}

impl MemoryService {
    pub async fn export_memory(&self, id: i64) -> Result<MemoryExport> {
        let memory = self.memory_repo.get_by_id(id).await?;

        let screenshot_ids: Vec<i64> = serde_json::from_str(&memory.screenshot_ids)?;
        let screenshots = self.screenshot_repo
            .get_by_ids(screenshot_ids)
            .await?;

        Ok(MemoryExport {
            id: memory.id,
            start_time: memory.start_time.to_rfc3339(),
            end_time: memory.end_time.to_rfc3339(),
            event_title: memory.event_title,
            event_summary: memory.event_summary,
            keywords: memory.keywords.split(", ").map(|s| s.to_string()).collect(),
            screenshots: screenshots.into_iter().map(|s| s.into()).collect(),
            apps: serde_json::from_str(&memory.app_names)?,
            suggestions: memory.suggestions,
        })
    }
}
```

---

## 性能优化

### 1. JSON 数组查询优化

```sql
-- ❌ 全表扫描
SELECT * FROM short_term_memory
WHERE screenshot_ids LIKE '%123%';

-- ✅ 通过 screenshots 表反向查询
SELECT DISTINCT m.*
FROM short_term_memory m
JOIN screenshots s ON s.memory_id = m.id
WHERE s.id = 123;
```

### 2. 分页查询

```rust
pub async fn list_memories_paginated(
    &self,
    page: i32,
    page_size: i32,
) -> Result<Vec<ShortTermMemory>> {
    let offset = (page - 1) * page_size;

    sqlx::query_as::<_, ShortTermMemory>(
        "SELECT * FROM short_term_memory
         ORDER BY start_time DESC
         LIMIT ? OFFSET ?"
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&self.pool)
    .await
    .map_err(Into::into)
}
```

---

## 数据完整性

### 约束检查

```rust
impl CreateShortTermMemory {
    pub fn validate(&self) -> Result<()> {
        // 1. 时间范围验证
        if self.end_time <= self.start_time {
            return Err(AppError::InvalidParams(
                "end_time must be greater than start_time".into()
            ));
        }

        // 2. 必填字段验证
        if self.event_title.is_empty() {
            return Err(AppError::InvalidParams("event_title is required".into()));
        }

        // 3. JSON 格式验证
        serde_json::from_str::<Vec<i64>>(&self.screenshot_ids)?;
        serde_json::from_str::<Vec<String>>(&self.app_names)?;

        Ok(())
    }
}
```

---

## 相关文档

- [截图表 (D1)](screenshots.md)
- [应用使用表 (D4)](app_usage.md)
- [长期记忆表](long_term_memory.md)
- [记忆服务](../../../backend/services/memory-service.md)

---

**维护者**: 数据库设计组
**最后更新**: 2026-02-04
