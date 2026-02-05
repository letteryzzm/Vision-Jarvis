# 数据库设计文档

> **最后更新**: 2026-02-04
> **版本**: v1.0
> **数据库**: SQLite / libSQL

---

## 目录

- [数据库概述](#数据库概述)
- [表结构](#表结构)
- [ER 图](#er-图)
- [索引策略](#索引策略)
- [迁移管理](#迁移管理)
- [数据备份](#数据备份)

---

## 数据库概述

### 技术选型

**SQLite / libSQL**

选择理由:
- ✅ 零配置，嵌入式数据库
- ✅ 轻量级，适合桌面应用
- ✅ ACID 事务支持
- ✅ 跨平台兼容
- ✅ 可选 libSQL 云同步支持

### 数据库文件位置

```
macOS:   ~/Library/Application Support/Vision-Jarvis/vision-jarvis.db
Windows: %APPDATA%/Vision-Jarvis/vision-jarvis.db
Linux:   ~/.local/share/Vision-Jarvis/vision-jarvis.db
```

---

## 表结构

### 核心表概览

| 表名 | 别名 | 描述 | 文档 |
|------|------|------|------|
| `screenshots` | D1 | 截图记录表 | [详细](schema/tables/screenshots.md) |
| `short_term_memory` | D3 | 短期记忆表 | [详细](schema/tables/short_term_memory.md) |
| `app_usage` | D4 | 应用使用记录表 | [详细](schema/tables/app_usage.md) |
| `long_term_memory` | - | 长期记忆表 | [详细](schema/tables/long_term_memory.md) |
| `notifications` | - | 通知记录表 | [详细](schema/tables/notifications.md) |
| `app_config` | - | 应用配置表 | [详细](schema/tables/app_config.md) |

---

## ER 图

### 表关系图

```
┌─────────────────────┐
│   screenshots (D1)  │
│  - id (PK)          │
│  - file_path        │
│  - app_name         │
│  - timestamp        │
│  - ocr_text         │
│  - ai_summary       │
│  - keywords         │
│  - status           │
│  - memory_id (FK)   │◀────┐
└─────────────────────┘     │
                            │
┌─────────────────────┐     │ 1:N
│   app_usage (D4)    │     │
│  - id (PK)          │     │
│  - app_name         │     │
│  - start_time       │     │
│  - end_time         │     │
│  - memory_id (FK)   │◀────┤
└─────────────────────┘     │
                            │
        N:1                 │
         │                  │
         └──────────────────┤
                            │
┌─────────────────────────┐ │
│ short_term_memory (D3)  │─┘
│  - id (PK)              │
│  - start_time           │
│  - end_time             │
│  - event_title          │
│  - event_summary        │
│  - keywords             │
│  - screenshot_ids       │
│  - app_names            │
│  - suggestions          │
└──────────┬──────────────┘
           │
           │ N:1
           ▼
┌─────────────────────────┐
│ long_term_memory        │
│  - id (PK)              │
│  - start_date           │
│  - end_date             │
│  - summary              │
│  - main_events (JSON)   │
│  - short_memory_ids     │
└─────────────────────────┘

┌─────────────────────────┐
│   notifications         │
│  - id (PK)              │
│  - type                 │
│  - title                │
│  - content              │
│  - pushed_at            │
│  - read                 │
└─────────────────────────┘

┌─────────────────────────┐
│   app_config            │
│  - key (PK)             │
│  - value                │
│  - updated_at           │
└─────────────────────────┘
```

---

## 核心表详细设计

### 1. screenshots (D1) - 截图记录表

```sql
CREATE TABLE screenshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,              -- 文件路径
    app_name TEXT NOT NULL,               -- 应用名称
    timestamp DATETIME NOT NULL,          -- 截图时间
    file_size INTEGER NOT NULL,           -- 文件大小 (bytes)
    content_hash TEXT,                    -- 内容哈希 (用于去重)

    -- AI 分析字段
    ocr_text TEXT,                        -- OCR 识别文本
    ai_summary TEXT,                      -- AI 摘要
    keywords TEXT,                        -- 关键词 (逗号分隔)
    confidence REAL,                      -- AI 置信度 (0-1)
    status TEXT NOT NULL DEFAULT 'pending', -- pending|analyzing|completed|failed

    -- 关联字段
    memory_id INTEGER,                    -- 关联的短期记忆 ID

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (memory_id) REFERENCES short_term_memory(id) ON DELETE SET NULL
);

-- 索引
CREATE INDEX idx_screenshots_timestamp ON screenshots(timestamp);
CREATE INDEX idx_screenshots_app_name ON screenshots(app_name);
CREATE INDEX idx_screenshots_status ON screenshots(status);
CREATE INDEX idx_screenshots_memory_id ON screenshots(memory_id);
```

**详细文档**: [screenshots.md](schema/tables/screenshots.md)

---

### 2. short_term_memory (D3) - 短期记忆表

```sql
CREATE TABLE short_term_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time DATETIME NOT NULL,         -- 开始时间
    end_time DATETIME NOT NULL,           -- 结束时间
    event_title TEXT NOT NULL,            -- 事项标题
    event_summary TEXT NOT NULL,          -- 事项摘要
    keywords TEXT,                        -- 关键词 (逗号分隔)

    -- 关联数据 (JSON 数组)
    screenshot_ids TEXT NOT NULL,         -- [1, 2, 3]
    app_names TEXT NOT NULL,              -- ["VSCode", "Chrome"]

    -- AI 生成内容
    suggestions TEXT,                     -- 建议与分析

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_memory_start_time ON short_term_memory(start_time);
CREATE INDEX idx_memory_end_time ON short_term_memory(end_time);
```

**详细文档**: [short_term_memory.md](schema/tables/short_term_memory.md)

---

### 3. app_usage (D4) - 应用使用记录表

```sql
CREATE TABLE app_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL,               -- 应用名称
    bundle_id TEXT,                       -- macOS Bundle ID
    start_time DATETIME NOT NULL,         -- 开始使用时间
    end_time DATETIME,                    -- 结束使用时间 (NULL 表示正在使用)
    duration_seconds INTEGER,             -- 使用时长 (秒)

    -- 关联字段
    memory_id INTEGER,                    -- 关联的短期记忆 ID

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (memory_id) REFERENCES short_term_memory(id) ON DELETE SET NULL
);

-- 索引
CREATE INDEX idx_app_usage_app_name ON app_usage(app_name);
CREATE INDEX idx_app_usage_start_time ON app_usage(start_time);
CREATE INDEX idx_app_usage_memory_id ON app_usage(memory_id);
```

**详细文档**: [app_usage.md](schema/tables/app_usage.md)

---

### 4. long_term_memory - 长期记忆表

```sql
CREATE TABLE long_term_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_date DATE NOT NULL,             -- 开始日期
    end_date DATE NOT NULL,               -- 结束日期
    summary TEXT NOT NULL,                -- AI 生成的总结
    main_events TEXT NOT NULL,            -- 主要事项 (JSON 数组)
    total_memories INTEGER NOT NULL,      -- 短期记忆数量

    -- 关联短期记忆 IDs (JSON 数组)
    short_memory_ids TEXT NOT NULL,       -- [1, 2, 3, 4]

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_long_memory_start_date ON long_term_memory(start_date);
CREATE INDEX idx_long_memory_end_date ON long_term_memory(end_date);
```

**详细文档**: [long_term_memory.md](schema/tables/long_term_memory.md)

---

### 5. notifications - 通知记录表

```sql
CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL,                   -- reminder|suggestion|resume
    title TEXT NOT NULL,                  -- 通知标题
    content TEXT NOT NULL,                -- 通知内容
    pushed_at DATETIME NOT NULL,          -- 推送时间
    read BOOLEAN NOT NULL DEFAULT 0,      -- 是否已读

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_notifications_type ON notifications(type);
CREATE INDEX idx_notifications_pushed_at ON notifications(pushed_at);
CREATE INDEX idx_notifications_read ON notifications(read);
```

**详细文档**: [notifications.md](schema/tables/notifications.md)

---

### 6. app_config - 应用配置表

```sql
CREATE TABLE app_config (
    key TEXT PRIMARY KEY,                 -- 配置键
    value TEXT NOT NULL,                  -- 配置值 (JSON)
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 预设配置项
INSERT INTO app_config (key, value) VALUES
    ('screenshot.interval_seconds', '5'),
    ('screenshot.max_storage_mb', '5000'),
    ('memory.time_window_minutes', '30'),
    ('notification.work_reminder_minutes', '90'),
    ('notification.enabled', 'true');
```

---

## 索引策略

### 高频查询字段

| 表 | 索引字段 | 查询场景 |
|------|----------|---------|
| screenshots | timestamp | 按时间范围查询 |
| screenshots | status | 查询待分析截图 |
| screenshots | app_name | 按应用过滤 |
| short_term_memory | start_time, end_time | 日期范围查询 |
| app_usage | app_name | 应用统计 |

### 复合索引

```sql
-- 截图时间范围 + 状态查询
CREATE INDEX idx_screenshots_time_status
ON screenshots(timestamp, status);

-- 记忆时间范围查询
CREATE INDEX idx_memory_time_range
ON short_term_memory(start_time, end_time);
```

---

## 迁移管理

### 迁移文件

```
migrations/
├── 001_initial_schema.sql          # 初始表结构
├── 002_add_content_hash.sql        # 添加 content_hash 字段
└── 003_add_long_term_memory.sql    # 添加长期记忆表
```

### SQLx 迁移

```bash
# 创建迁移
sqlx migrate add initial_schema

# 执行迁移
sqlx migrate run
```

**详细文档**: [migrations/README.md](migrations/README.md)

---

## 数据备份

### 自动备份策略

- **频率**: 每天一次
- **保留**: 最近 7 天
- **位置**: `~/Library/Application Support/Vision-Jarvis/backups/`

### 备份实现

```rust
impl DatabaseService {
    pub async fn backup(&self) -> Result<()> {
        let backup_path = format!(
            "{}/vision-jarvis-{}.db",
            self.config.backup_dir,
            chrono::Utc::now().format("%Y%m%d")
        );

        tokio::fs::copy(&self.db_path, &backup_path).await?;

        // 清理旧备份
        self.cleanup_old_backups(7).await?;

        Ok(())
    }
}
```

**详细文档**: [backup-restore.md](backup-restore.md)

---

## 性能优化

### 1. 批量插入

```rust
// ❌ 逐条插入
for screenshot in screenshots {
    repo.insert(screenshot).await?;
}

// ✅ 批量插入
repo.batch_insert(screenshots).await?;
```

### 2. 事务处理

```rust
let mut tx = pool.begin().await?;

// 多个操作在一个事务中
screenshot_repo.create(&mut tx, screenshot).await?;
app_usage_repo.create(&mut tx, usage).await?;

tx.commit().await?;
```

### 3. 查询优化

```sql
-- 使用 LIMIT 限制结果集
SELECT * FROM screenshots
WHERE timestamp >= ?
ORDER BY timestamp DESC
LIMIT 20;

-- 避免 SELECT *，只查询需要的字段
SELECT id, file_path, timestamp
FROM screenshots
WHERE status = 'pending';
```

---

## 数据清理策略

### 自动清理规则

| 数据类型 | 保留时长 | 清理频率 |
|---------|---------|---------|
| 截图文件 | 30 天 | 每天 |
| 截图记录 | 90 天 | 每周 |
| 短期记忆 | 永久（除非用户删除） | - |
| 通知记录 | 30 天 | 每周 |

### 清理实现

```sql
-- 删除 30 天前的截图
DELETE FROM screenshots
WHERE timestamp < datetime('now', '-30 days');

-- 删除 30 天前的已读通知
DELETE FROM notifications
WHERE pushed_at < datetime('now', '-30 days')
AND read = 1;
```

---

## 相关文档

- [表结构详细设计](schema/tables/)
- [迁移管理](migrations/README.md)
- [查询优化](queries/optimization.md)
- [备份恢复](backup-restore.md)

---

**维护者**: 数据库设计组
**最后更新**: 2026-02-04
