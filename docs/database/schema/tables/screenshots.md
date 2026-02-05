# screenshots 表 (D1) - 截图记录表

> **最后更新**: 2026-02-05
> **版本**: v1.2
> **表名**: `screenshots`
> **别名**: D1
> **实现状态**: ✅ 已实现

---

## 表概述

存储所有屏幕截图的元数据和 AI 分析结果。这是 Vision-Jarvis 的核心数据表之一。

**实现文件**: `src-tauri/src/db/migrations.rs`

---

## 表结构

### DDL（实际实现）

```sql
CREATE TABLE IF NOT EXISTS screenshots (
    id TEXT PRIMARY KEY,                   -- UUID v4
    path TEXT NOT NULL,                    -- 截图文件路径
    captured_at INTEGER NOT NULL,          -- Unix 时间戳（秒）
    analyzed INTEGER DEFAULT 0,            -- 是否已分析 (0/1)
    analysis_result TEXT,                  -- AI 分析结果（JSON）
    embedding BLOB,                        -- 向量嵌入（用于语义搜索）
    analyzed_at INTEGER,                   -- 分析完成时间戳（v1.2 新增）
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
```

### 索引

```sql
-- 捕获时间查询索引（降序，最新优先）
CREATE INDEX IF NOT EXISTS idx_screenshots_captured_at
    ON screenshots(captured_at DESC);

-- 分析状态索引（用于 AI 分析队列）
CREATE INDEX IF NOT EXISTS idx_screenshots_analyzed
    ON screenshots(analyzed);
```

### 字段说明

#### id
- **类型**: TEXT
- **约束**: PRIMARY KEY
- **描述**: 截图唯一标识符（UUID v4）
- **示例**: `550e8400-e29b-41d4-a716-446655440000`

#### path

### confidence

- **类型**: REAL
- **约束**: NULLABLE
- **范围**: 0.0 - 1.0
- **描述**: AI 分析结果的置信度
- **示例**: `0.95` (95% 置信度)

### status

- **类型**: TEXT (ENUM)
- **约束**: NOT NULL, DEFAULT 'pending'
- **可选值**:
  - `pending`: 等待 AI 分析
  - `analyzing`: AI 分析中
  - `completed`: 分析完成
  - `failed`: 分析失败
- **描述**: 截图的 AI 分析状态

### memory_id

- **类型**: INTEGER
- **约束**: NULLABLE, FOREIGN KEY
- **描述**: 关联的短期记忆 ID，如果这张截图属于某个记忆事项
- **外键**: `short_term_memory(id)` ON DELETE SET NULL

### created_at

- **类型**: DATETIME
- **约束**: NOT NULL, DEFAULT CURRENT_TIMESTAMP
- **描述**: 数据库记录创建时间

### updated_at

- **类型**: DATETIME
- **约束**: NOT NULL, DEFAULT CURRENT_TIMESTAMP
- **描述**: 数据库记录最后更新时间

---

## 状态机

### status 字段状态流转

```
[pending]       ← 初始状态（截图刚保存）
    ↓
[analyzing]     ← AI 分析开始
    ↓
    ├─ 成功 → [completed]  ← 分析完成
    └─ 失败 → [failed]     ← 分析失败

重试流程:
[failed] → 重新触发 → [analyzing] → [completed]
```

---

## 数据示例

### 插入示例

```sql
INSERT INTO screenshots (
    file_path,
    app_name,
    timestamp,
    file_size,
    content_hash,
    status
) VALUES (
    '/Users/user/Library/Application Support/Vision-Jarvis/screenshots/1738675200_VSCode.webp',
    'Visual Studio Code',
    '2026-02-04T10:30:00Z',
    524288,
    'a3b5c7d9e1f2...',
    'pending'
);
```

### 更新 AI 分析结果

```sql
UPDATE screenshots
SET
    ocr_text = 'function calculateSum(a: number, b: number) { return a + b; }',
    ai_summary = '用户正在 VSCode 中编写 TypeScript 函数',
    keywords = 'TypeScript, 函数, VSCode, 编程',
    confidence = 0.95,
    status = 'completed',
    updated_at = CURRENT_TIMESTAMP
WHERE id = 1;
```

### 查询示例

```sql
-- 查询指定时间范围内的截图
SELECT * FROM screenshots
WHERE timestamp BETWEEN '2026-02-04T10:00:00Z' AND '2026-02-04T11:00:00Z'
ORDER BY timestamp DESC;

-- 查询待分析的截图
SELECT * FROM screenshots
WHERE status = 'pending'
ORDER BY timestamp ASC
LIMIT 10;

-- 查询指定应用的截图
SELECT id, file_path, timestamp, ai_summary
FROM screenshots
WHERE app_name = 'Visual Studio Code'
AND timestamp >= '2026-02-04T00:00:00Z'
ORDER BY timestamp DESC;

-- 查询属于某个记忆的截图
SELECT * FROM screenshots
WHERE memory_id = 123
ORDER BY timestamp ASC;
```

---

## 关联查询

### 联合查询短期记忆

```sql
SELECT
    s.id,
    s.file_path,
    s.timestamp,
    s.app_name,
    m.event_title,
    m.event_summary
FROM screenshots s
LEFT JOIN short_term_memory m ON s.memory_id = m.id
WHERE s.timestamp >= '2026-02-04T00:00:00Z'
ORDER BY s.timestamp DESC;
```

---

## 数据清理

### 删除旧截图 (30 天前)

```sql
-- 删除数据库记录
DELETE FROM screenshots
WHERE timestamp < datetime('now', '-30 days');
```

### 删除关联文件

```rust
impl ScreenshotRepository {
    pub async fn delete_with_file(&self, id: i64) -> Result<()> {
        // 1. 获取文件路径
        let screenshot = self.get_by_id(id).await?;

        // 2. 删除文件
        tokio::fs::remove_file(&screenshot.file_path).await?;

        // 3. 删除数据库记录
        sqlx::query("DELETE FROM screenshots WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
```

---

## 性能优化

### 1. 批量插入

```rust
pub async fn batch_insert(&self, screenshots: Vec<CreateScreenshot>) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    for screenshot in screenshots {
        sqlx::query(
            "INSERT INTO screenshots (file_path, app_name, timestamp, file_size, status)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&screenshot.file_path)
        .bind(&screenshot.app_name)
        .bind(&screenshot.timestamp)
        .bind(screenshot.file_size)
        .bind("pending")
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

### 2. 分页查询

```sql
-- 使用 LIMIT 和 OFFSET
SELECT * FROM screenshots
WHERE status = 'completed'
ORDER BY timestamp DESC
LIMIT 20 OFFSET 0;
```

### 3. 索引利用

```sql
-- ✅ 利用索引 (idx_screenshots_time_status)
SELECT * FROM screenshots
WHERE timestamp >= '2026-02-04T00:00:00Z'
AND status = 'completed';

-- ❌ 全表扫描 (LIKE 无法使用索引)
SELECT * FROM screenshots
WHERE file_path LIKE '%VSCode%';
```

---

## 约束和验证

### 应用层验证

```rust
#[derive(Debug, Validate)]
pub struct CreateScreenshot {
    #[validate(length(min = 1))]
    pub file_path: String,

    #[validate(length(min = 1))]
    pub app_name: String,

    #[validate(range(min = 0))]
    pub file_size: i64,

    #[validate(custom = "validate_status")]
    pub status: String,
}

fn validate_status(status: &str) -> Result<(), ValidationError> {
    match status {
        "pending" | "analyzing" | "completed" | "failed" => Ok(()),
        _ => Err(ValidationError::new("Invalid status")),
    }
}
```

---

## 相关文档

- [短期记忆表 (D3)](short_term_memory.md)
- [应用使用记录表 (D4)](app_usage.md)
- [截屏服务](../../../backend/services/screenshot-service.md)
- [AI 服务](../../../backend/services/ai-service.md)

---

## 版本历史

### v1.2 (2026-02-05) - Phase 3
- ✅ 新增 `analyzed_at` 字段 - 记录分析完成时间戳
- ✅ 用于追踪分析任务完成时间
- ✅ 后台调度器使用此字段进行任务统计

### v1.1 (2026-02-05) - Phase 1
- ✅ 初始表结构实现
- ✅ 基础索引创建

---

**维护者**: 数据库设计组
**最后更新**: 2026-02-05

**最后更新**: 2026-02-04
