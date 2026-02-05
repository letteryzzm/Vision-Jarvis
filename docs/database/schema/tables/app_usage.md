# app_usage 表 (D4) - 应用使用记录表

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **表名**: `app_usage`
> **别名**: D4

---

## 表概述

记录用户的应用使用情况，包括应用切换时间、使用时长等。用于辅助生成短期记忆和分析用户行为模式。

---

## 表结构

### DDL

```sql
CREATE TABLE app_usage (
    -- 主键
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 应用信息
    app_name TEXT NOT NULL,               -- 应用名称
    bundle_id TEXT,                       -- macOS Bundle ID / Windows Process Name
    window_title TEXT,                    -- 窗口标题 (可选)

    -- 时间信息
    start_time DATETIME NOT NULL,         -- 开始使用时间
    end_time DATETIME,                    -- 结束使用时间 (NULL 表示正在使用)
    duration_seconds INTEGER,             -- 使用时长 (秒)

    -- 关联字段
    memory_id INTEGER,                    -- 关联的短期记忆 ID

    -- 元数据
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (memory_id) REFERENCES short_term_memory(id) ON DELETE SET NULL
);
```

### 索引

```sql
-- 应用名称索引 (统计查询)
CREATE INDEX idx_app_usage_app_name ON app_usage(app_name);

-- 开始时间索引 (时间范围查询)
CREATE INDEX idx_app_usage_start_time ON app_usage(start_time);

-- 记忆关联索引
CREATE INDEX idx_app_usage_memory_id ON app_usage(memory_id);

-- 复合索引: 应用 + 时间
CREATE INDEX idx_app_usage_app_time ON app_usage(app_name, start_time);
```

---

## 字段详解

### id

- **类型**: INTEGER
- **约束**: PRIMARY KEY, AUTOINCREMENT
- **描述**: 应用使用记录唯一标识符

### app_name

- **类型**: TEXT
- **约束**: NOT NULL
- **描述**: 应用程序名称
- **示例**: `Visual Studio Code`, `Google Chrome`, `Slack`

### bundle_id

- **类型**: TEXT
- **约束**: NULLABLE
- **描述**:
  - macOS: Bundle Identifier (如 `com.microsoft.VSCode`)
  - Windows: Process Name (如 `Code.exe`)
  - Linux: Desktop file ID
- **用途**: 用于更准确地识别应用，避免同名应用混淆

### window_title

- **类型**: TEXT
- **约束**: NULLABLE
- **描述**: 应用窗口标题，可用于更细粒度的活动分析
- **示例**:
  - `memory-service.md - Visual Studio Code`
  - `Claude API 文档 - Google Chrome`

### start_time

- **类型**: DATETIME
- **约束**: NOT NULL
- **描述**: 应用开始使用时间 (UTC)
- **格式**: ISO 8601
- **示例**: `2026-02-04T10:00:00Z`

### end_time

- **类型**: DATETIME
- **约束**: NULLABLE
- **描述**: 应用结束使用时间 (UTC)
- **特殊值**: `NULL` 表示该应用当前正在使用
- **示例**: `2026-02-04T10:30:00Z`

### duration_seconds

- **类型**: INTEGER
- **约束**: NULLABLE
- **描述**: 应用使用时长，单位秒
- **计算**: `end_time - start_time` (秒)
- **示例**: `1800` (30 分钟)

### memory_id

- **类型**: INTEGER
- **约束**: NULLABLE, FOREIGN KEY
- **描述**: 关联的短期记忆 ID
- **外键**: `short_term_memory(id)` ON DELETE SET NULL

### created_at

- **类型**: DATETIME
- **约束**: NOT NULL, DEFAULT CURRENT_TIMESTAMP
- **描述**: 记录创建时间

---

## 数据示例

### 插入示例（应用切换）

```sql
-- 用户从 Chrome 切换到 VSCode
-- 1. 结束 Chrome 使用
UPDATE app_usage
SET
    end_time = '2026-02-04T10:30:00Z',
    duration_seconds = 1800  -- 30 分钟
WHERE id = 1 AND end_time IS NULL;

-- 2. 开始 VSCode 使用
INSERT INTO app_usage (
    app_name,
    bundle_id,
    window_title,
    start_time
) VALUES (
    'Visual Studio Code',
    'com.microsoft.VSCode',
    'memory-service.md - Visual Studio Code',
    '2026-02-04T10:30:00Z'
);
```

### 查询示例

```sql
-- 查询指定时间范围内的应用使用
SELECT * FROM app_usage
WHERE start_time >= '2026-02-04T10:00:00Z'
AND (end_time IS NULL OR end_time <= '2026-02-04T11:00:00Z')
ORDER BY start_time ASC;

-- 查询当前正在使用的应用
SELECT * FROM app_usage
WHERE end_time IS NULL;

-- 查询指定应用的使用记录
SELECT * FROM app_usage
WHERE app_name = 'Visual Studio Code'
AND start_time >= '2026-02-04T00:00:00Z'
ORDER BY start_time DESC;

-- 统计今日各应用使用时长
SELECT
    app_name,
    SUM(duration_seconds) AS total_seconds,
    COUNT(*) AS switch_count
FROM app_usage
WHERE DATE(start_time) = '2026-02-04'
AND duration_seconds IS NOT NULL
GROUP BY app_name
ORDER BY total_seconds DESC;
```

---

## 关联查询

### 联合查询短期记忆

```sql
-- 查询记忆及其应用使用明细
SELECT
    m.id AS memory_id,
    m.event_title,
    a.app_name,
    a.start_time,
    a.end_time,
    a.duration_seconds
FROM app_usage a
JOIN short_term_memory m ON a.memory_id = m.id
WHERE m.id = 1
ORDER BY a.start_time ASC;
```

---

## 业务逻辑

### 应用切换检测

```rust
impl AppUsageService {
    /// 检测应用切换并记录
    pub async fn track_app_switch(&self, current_app: AppInfo) -> Result<()> {
        // 1. 获取当前正在使用的应用
        let active_usage = self.repo
            .get_active_usage()
            .await?;

        // 2. 如果有活跃应用且不是当前应用，结束旧应用使用
        if let Some(usage) = active_usage {
            if usage.app_name != current_app.name {
                let end_time = Utc::now();
                let duration = (end_time - usage.start_time).num_seconds();

                self.repo.end_usage(usage.id, end_time, duration).await?;
            } else {
                // 同一个应用，不需要记录
                return Ok(());
            }
        }

        // 3. 开始新应用使用
        self.repo.create(CreateAppUsage {
            app_name: current_app.name,
            bundle_id: Some(current_app.bundle_id),
            window_title: current_app.window_title,
            start_time: Utc::now(),
        }).await?;

        Ok(())
    }
}
```

### 应用使用统计

```rust
impl AppUsageService {
    /// 统计指定日期的应用使用情况
    pub async fn get_daily_stats(&self, date: NaiveDate) -> Result<Vec<AppUsageStats>> {
        let start = date.and_hms_opt(0, 0, 0).unwrap();
        let end = date.and_hms_opt(23, 59, 59).unwrap();

        let stats = sqlx::query_as::<_, AppUsageStats>(
            "SELECT
                app_name,
                SUM(duration_seconds) AS total_duration,
                COUNT(*) AS switch_count,
                MIN(start_time) AS first_use,
                MAX(end_time) AS last_use
             FROM app_usage
             WHERE start_time >= ? AND start_time <= ?
             AND duration_seconds IS NOT NULL
             GROUP BY app_name
             ORDER BY total_duration DESC"
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct AppUsageStats {
    pub app_name: String,
    pub total_duration: i64,      // 总使用时长 (秒)
    pub switch_count: i32,         // 切换次数
    pub first_use: DateTime<Utc>, // 首次使用时间
    pub last_use: DateTime<Utc>,  // 最后使用时间
}
```

---

## 数据可视化

### 生成时间线数据

```rust
impl AppUsageService {
    /// 生成应用使用时间线
    pub async fn get_timeline(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimelineEntry>> {
        let usages = self.repo
            .get_by_time_range(start, end)
            .await?;

        let timeline: Vec<TimelineEntry> = usages
            .into_iter()
            .map(|u| TimelineEntry {
                app_name: u.app_name,
                start: u.start_time,
                end: u.end_time.unwrap_or_else(|| Utc::now()),
                duration: u.duration_seconds.unwrap_or(0),
            })
            .collect();

        Ok(timeline)
    }
}

#[derive(Debug, Serialize)]
pub struct TimelineEntry {
    pub app_name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration: i64,
}
```

---

## 数据清理

### 清理旧记录

```sql
-- 删除 90 天前的应用使用记录
DELETE FROM app_usage
WHERE start_time < datetime('now', '-90 days');
```

### 合并短时间片段

```rust
impl AppUsageService {
    /// 合并短时间内的同应用使用记录
    /// 例如: 用户在 VSCode 中短暂切换到其他应用后又回到 VSCode
    pub async fn merge_short_intervals(&self, threshold_seconds: i64) -> Result<()> {
        // 实现合并逻辑
        // 1. 查找同一应用在短时间内的多次使用
        // 2. 合并为一条记录
        Ok(())
    }
}
```

---

## 性能优化

### 1. 批量插入

```rust
pub async fn batch_create(&self, usages: Vec<CreateAppUsage>) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    for usage in usages {
        sqlx::query(
            "INSERT INTO app_usage (app_name, bundle_id, window_title, start_time)
             VALUES (?, ?, ?, ?)"
        )
        .bind(&usage.app_name)
        .bind(&usage.bundle_id)
        .bind(&usage.window_title)
        .bind(&usage.start_time)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

### 2. 分区查询

```sql
-- 按月份分区查询（未来优化）
SELECT * FROM app_usage
WHERE strftime('%Y-%m', start_time) = '2026-02'
ORDER BY start_time DESC;
```

---

## 隐私考虑

### 敏感数据处理

```rust
impl AppUsageService {
    /// 过滤敏感窗口标题
    fn sanitize_window_title(&self, title: &str) -> String {
        // 移除可能包含敏感信息的窗口标题
        // 例如: "password.txt - Notepad" → "Notepad"

        let sensitive_keywords = ["password", "private", "confidential"];

        if sensitive_keywords.iter().any(|k| title.to_lowercase().contains(k)) {
            // 只保留应用名称
            return extract_app_name(title);
        }

        title.to_string()
    }
}
```

---

## 边界条件

### 1. 正在使用的应用

- `end_time` 为 `NULL`
- `duration_seconds` 为 `NULL`
- 应用切换时才更新这两个字段

### 2. 异常关闭

如果应用异常退出（用户关机、应用崩溃），可能存在 `end_time` 为 `NULL` 的记录。

**处理方案**:
```rust
impl AppUsageService {
    /// 清理未结束的使用记录
    pub async fn cleanup_stale_usages(&self) -> Result<()> {
        // 假设超过 4 小时未更新的记录为异常记录
        let cutoff = Utc::now() - chrono::Duration::hours(4);

        sqlx::query(
            "UPDATE app_usage
             SET end_time = start_time + 3600,  -- 假设使用了 1 小时
                 duration_seconds = 3600
             WHERE end_time IS NULL
             AND start_time < ?"
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

---

## 相关文档

- [截图表 (D1)](screenshots.md)
- [短期记忆表 (D3)](short_term_memory.md)
- [截屏服务](../../../backend/services/screenshot-service.md)
- [记忆服务](../../../backend/services/memory-service.md)

---

**维护者**: 数据库设计组
**最后更新**: 2026-02-04
