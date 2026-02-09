/// 数据库迁移
///
/// 负责创建和更新数据库表结构
/// V1: 原始schema (screenshots, short_term_memories, long_term_memories, settings)
/// V2: 事项驱动记忆系统 (activities, memory_files, memory_chunks, embedding_cache)

use anyhow::Result;
use rusqlite::Connection;

/// 运行所有迁移
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let version = get_schema_version(conn)?;

    // V1: 原始表结构 (已存在的用户是V1,新用户从V1开始)
    if version < 1 {
        create_screenshots_table(conn)?;
        create_short_term_memories_table(conn)?;
        create_long_term_memories_table(conn)?;
        create_settings_table(conn)?;
    }

    // V2: 事项驱动记忆系统
    if version < 2 {
        create_activities_table(conn)?;
        create_memory_files_table(conn)?;
        create_memory_chunks_table(conn)?;
        create_embedding_cache_table(conn)?;
        add_activity_id_to_screenshots(conn)?;
    }

    // 更新版本号
    set_schema_version(conn, 2)?;

    Ok(())
}

/// 获取当前schema版本
fn get_schema_version(conn: &Connection) -> Result<i32> {
    // 先创建meta表(如果不存在)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // 查询版本号
    let version: i32 = conn
        .prepare("SELECT value FROM schema_meta WHERE key = 'schema_version'")?
        .query_row([], |row| {
            let v: String = row.get(0)?;
            Ok(v.parse::<i32>().unwrap_or(0))
        })
        .unwrap_or(0); // 如果不存在记录,默认为0(需要从V1开始迁移)

    Ok(version)
}

/// 设置schema版本
fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('schema_version', ?1)",
        [version.to_string()],
    )?;
    Ok(())
}

// ============================================================================
// V1 Tables
// ============================================================================

/// 创建 screenshots 表
fn create_screenshots_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS screenshots (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL,
            captured_at INTEGER NOT NULL,
            analyzed INTEGER DEFAULT 0,
            analysis_result TEXT,
            embedding BLOB,
            analyzed_at INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_screenshots_captured_at
         ON screenshots(captured_at DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_screenshots_analyzed
         ON screenshots(analyzed)",
        [],
    )?;

    Ok(())
}

/// 创建 short_term_memories 表
fn create_short_term_memories_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS short_term_memories (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            time_start TEXT NOT NULL,
            time_end TEXT NOT NULL,
            period TEXT NOT NULL,
            activity TEXT NOT NULL,
            summary TEXT,
            screenshot_ids TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stm_date
         ON short_term_memories(date DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stm_period
         ON short_term_memories(period)",
        [],
    )?;

    Ok(())
}

/// 创建 long_term_memories 表
fn create_long_term_memories_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS long_term_memories (
            id TEXT PRIMARY KEY,
            date_start TEXT NOT NULL,
            date_end TEXT NOT NULL,
            summary TEXT NOT NULL,
            main_activities TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ltm_date_range
         ON long_term_memories(date_start, date_end)",
        [],
    )?;

    Ok(())
}

/// 创建 settings 表
fn create_settings_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    Ok(())
}

// ============================================================================
// V2 Tables: Activity-Driven Memory System
// ============================================================================

/// 创建 activities 表 - 活动会话元数据
fn create_activities_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS activities (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            start_time INTEGER NOT NULL,
            end_time INTEGER NOT NULL,
            duration_minutes INTEGER NOT NULL,
            application TEXT NOT NULL,
            category TEXT NOT NULL,
            screenshot_ids TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            markdown_path TEXT NOT NULL,
            summary TEXT,
            indexed INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_activities_time
         ON activities(start_time DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_activities_application
         ON activities(application)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_activities_category
         ON activities(category)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_activities_indexed
         ON activities(indexed)",
        [],
    )?;

    Ok(())
}

/// 创建 memory_files 表 - 文件追踪(用于增量索引)
fn create_memory_files_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_files (
            path TEXT PRIMARY KEY,
            source TEXT NOT NULL DEFAULT 'activity',
            hash TEXT NOT NULL,
            mtime INTEGER NOT NULL,
            size INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// 创建 memory_chunks 表 - 文本分块与向量
fn create_memory_chunks_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_chunks (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT 'activity',
            start_line INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            hash TEXT NOT NULL,
            model TEXT NOT NULL,
            text TEXT NOT NULL,
            embedding BLOB NOT NULL,
            activity_id TEXT,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (file_path) REFERENCES memory_files(path)
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_file_path
         ON memory_chunks(file_path)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_activity_id
         ON memory_chunks(activity_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_hash
         ON memory_chunks(hash)",
        [],
    )?;

    Ok(())
}

/// 创建 embedding_cache 表 - Embedding缓存
fn create_embedding_cache_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS embedding_cache (
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            hash TEXT NOT NULL,
            embedding BLOB NOT NULL,
            dims INTEGER,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (provider, model, hash)
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_embedding_cache_updated
         ON embedding_cache(updated_at)",
        [],
    )?;

    Ok(())
}

/// 为 screenshots 表添加 activity_id 列
fn add_activity_id_to_screenshots(conn: &Connection) -> Result<()> {
    // 检查列是否已存在
    let has_column: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('screenshots') WHERE name='activity_id'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|count| count > 0)?;

    if !has_column {
        conn.execute("ALTER TABLE screenshots ADD COLUMN activity_id TEXT", [])?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_screenshots_activity_id
             ON screenshots(activity_id)",
            [],
        )?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_fresh_install_v2() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(run_migrations(&conn).is_ok());

        // 验证V1表创建
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap();

        assert!(tables.contains(&"screenshots".to_string()));
        assert!(tables.contains(&"short_term_memories".to_string()));
        assert!(tables.contains(&"long_term_memories".to_string()));
        assert!(tables.contains(&"settings".to_string()));

        // 验证V2表创建
        assert!(tables.contains(&"schema_meta".to_string()));
        assert!(tables.contains(&"activities".to_string()));
        assert!(tables.contains(&"memory_files".to_string()));
        assert!(tables.contains(&"memory_chunks".to_string()));
        assert!(tables.contains(&"embedding_cache".to_string()));

        // 验证版本号
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_v1_to_v2_upgrade() {
        let conn = Connection::open_in_memory().unwrap();

        // 模拟V1安装
        create_screenshots_table(&conn).unwrap();
        create_short_term_memories_table(&conn).unwrap();
        create_long_term_memories_table(&conn).unwrap();
        create_settings_table(&conn).unwrap();

        // 插入一些V1数据
        conn.execute(
            "INSERT INTO screenshots (id, path, captured_at, analyzed)
             VALUES ('test-id', '/path/to/screenshot.png', 1706697600, 0)",
            [],
        )
        .unwrap();

        // 运行迁移
        assert!(run_migrations(&conn).is_ok());

        // 验证V2表已创建
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap();

        assert!(tables.contains(&"activities".to_string()));
        assert!(tables.contains(&"memory_chunks".to_string()));

        // 验证activity_id列已添加
        let has_activity_id: bool = conn
            .prepare("SELECT COUNT(*) FROM pragma_table_info('screenshots') WHERE name='activity_id'")
            .unwrap()
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|count| count > 0)
            .unwrap();

        assert!(has_activity_id);

        // 验证原有数据仍存在
        let count: i64 = conn
            .prepare("SELECT COUNT(*) FROM screenshots")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_idempotent_migrations() {
        let conn = Connection::open_in_memory().unwrap();

        // 运行两次迁移应该不报错
        assert!(run_migrations(&conn).is_ok());
        assert!(run_migrations(&conn).is_ok());

        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_activities_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_activities_table(&conn).unwrap();

        // 测试插入
        let result = conn.execute(
            "INSERT INTO activities
             (id, title, start_time, end_time, duration_minutes, application, category,
              screenshot_ids, markdown_path)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                "activity-2024-01-15-001",
                "编写Rust代码",
                "1706697600",
                "1706701200",
                "60",
                "VSCode",
                "work",
                "[\"id1\", \"id2\"]",
                "activities/2024-01-15/activity-001.md",
            ],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_chunks_foreign_key() {
        let conn = Connection::open_in_memory().unwrap();

        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        create_memory_files_table(&conn).unwrap();
        create_memory_chunks_table(&conn).unwrap();

        // 插入文件记录
        conn.execute(
            "INSERT INTO memory_files (path, hash, mtime, size)
             VALUES (?, ?, ?, ?)",
            ["activities/test.md", "abc123", "1706697600", "1024"],
        )
        .unwrap();

        // 插入chunk应该成功(有对应的file)
        let embedding_data = vec![0u8; 16];
        let result = conn.execute(
            "INSERT INTO memory_chunks
             (id, file_path, start_line, end_line, hash, model, text, embedding, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                "chunk-001",
                "activities/test.md",
                1,
                10,
                "def456",
                "text-embedding-3-small",
                "test content",
                embedding_data,
                1706697600_i64,
            ],
        );

        assert!(result.is_ok());
    }
}
