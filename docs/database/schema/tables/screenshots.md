# screenshots 表 (D1) - 截图记录表

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **表名**: `screenshots`
> **别名**: D1

---

## 表概述

存储所有屏幕截图的元数据和 AI 分析结果。这是 Vision-Jarvis 的核心数据表之一。

---

## 表结构

### DDL

```sql
CREATE TABLE screenshots (
    -- 主键
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 基本信息
    file_path TEXT NOT NULL,              -- 文件路径
    app_name TEXT NOT NULL,               -- 应用名称
    timestamp DATETIME NOT NULL,          -- 截图时间
    file_size INTEGER NOT NULL,           -- 文件大小 (bytes)
    content_hash TEXT,                    -- 内容哈希 SHA256 (用于去重)

    -- AI 分析字段
    ocr_text TEXT,                        -- OCR 识别的文本
    ai_summary TEXT,                      -- AI 生成的摘要
    keywords TEXT,                        -- 关键词 (逗号分隔)
    confidence REAL,                      -- AI 分析置信度 (0.0 - 1.0)
    status TEXT NOT NULL DEFAULT 'pending', -- 分析状态

    -- 关联字段
    memory_id INTEGER,                    -- 关联的短期记忆 ID (FK)

    -- 元数据
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (memory_id) REFERENCES short_term_memory(id) ON DELETE SET NULL
);
```

### 索引

```sql
-- 时间查询索引 (高频)
CREATE INDEX idx_screenshots_timestamp ON screenshots(timestamp);

-- 应用名称过滤索引
CREATE INDEX idx_screenshots_app_name ON screenshots(app_name);

-- 状态查询索引 (用于 AI 分析队列)
CREATE INDEX idx_screenshots_status ON screenshots(status);

-- 记忆关联索引
CREATE INDEX idx_screenshots_memory_id ON screenshots(memory_id);

-- 复合索引: 时间范围 + 状态
CREATE INDEX idx_screenshots_time_status ON screenshots(timestamp, status);
```

---

## 字段详解

### id

- **类型**: INTEGER
- **约束**: PRIMARY KEY, AUTOINCREMENT
- **描述**: 截图唯一标识符

### file_path

- **类型**: TEXT
- **约束**: NOT NULL
- **描述**: 截图文件的绝对路径
- **示例**: `/Users/user/Library/Application Support/Vision-Jarvis/screenshots/1738675200_VSCode.webp`

### app_name

- **类型**: TEXT
- **约束**: NOT NULL
- **描述**: 截图时的活跃应用名称
- **示例**: `Visual Studio Code`, `Google Chrome`

### timestamp

- **类型**: DATETIME
- **约束**: NOT NULL
- **描述**: 截图捕获时间 (UTC)
- **格式**: ISO 8601
- **示例**: `2026-02-04T10:30:00Z`

### file_size

- **类型**: INTEGER
- **约束**: NOT NULL
- **描述**: 截图文件大小，单位字节
- **示例**: `524288` (512 KB)

### content_hash

- **类型**: TEXT
- **约束**: NULLABLE
- **描述**: 图片内容的 SHA256 哈希值，用于去重和内容变化检测
- **示例**: `a3b5c7d9e1f2...`

### ocr_text

- **类型**: TEXT
- **约束**: NULLABLE
- **描述**: OCR 识别出的文本内容
- **示例**: `function calculateSum(a: number, b: number) { return a + b; }`

### ai_summary

- **类型**: TEXT
- **约束**: NULLABLE
- **描述**: AI 生成的截图内容摘要
- **示例**: `用户正在 VSCode 中编写 TypeScript 函数，实现数字求和功能`

### keywords

- **类型**: TEXT
- **约束**: NULLABLE
- **描述**: AI 提取的关键词，逗号分隔
- **示例**: `TypeScript, 函数, VSCode, 编程`

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

**维护者**: 数据库设计组
**最后更新**: 2026-02-04
