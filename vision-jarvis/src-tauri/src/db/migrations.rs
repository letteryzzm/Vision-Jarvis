/// 数据库迁移
///
/// 负责创建和更新数据库表结构

use anyhow::Result;
use rusqlite::Connection;

/// 运行所有迁移
pub fn run_migrations(conn: &Connection) -> Result<()> {
    create_screenshots_table(conn)?;
    create_short_term_memories_table(conn)?;
    create_long_term_memories_table(conn)?;
    create_settings_table(conn)?;

    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_run_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(run_migrations(&conn).is_ok());

        // 验证表是否创建成功
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
    }

    #[test]
    fn test_screenshots_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_screenshots_table(&conn).unwrap();

        // 测试插入
        let result = conn.execute(
            "INSERT INTO screenshots (id, path, captured_at, analyzed)
             VALUES (?, ?, ?, ?)",
            ["test-id", "/path/to/screenshot.png", "1706697600", "0"],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_short_term_memories_table_structure() {
        let conn = Connection::open_in_memory().unwrap();
        create_short_term_memories_table(&conn).unwrap();

        // 测试插入
        let result = conn.execute(
            "INSERT INTO short_term_memories
             (id, date, time_start, time_end, period, activity, screenshot_ids)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            [
                "test-id",
                "2026-02-05",
                "10:00",
                "11:00",
                "morning",
                "编程",
                "[\"id1\", \"id2\"]",
            ],
        );

        assert!(result.is_ok());
    }
}
