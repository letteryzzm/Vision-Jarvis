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
        let tx = conn.unchecked_transaction()?;
        create_screenshots_table(&tx)?;
        create_short_term_memories_table(&tx)?;
        create_long_term_memories_table(&tx)?;
        create_settings_table(&tx)?;
        set_schema_version(&tx, 1)?;
        tx.commit()?;
    }

    // V2: 事项驱动记忆系统
    if version < 2 {
        let tx = conn.unchecked_transaction()?;
        create_activities_table(&tx)?;
        create_memory_files_table(&tx)?;
        create_memory_chunks_table(&tx)?;
        create_embedding_cache_table(&tx)?;
        add_activity_id_to_screenshots(&tx)?;
        set_schema_version(&tx, 2)?;
        tx.commit()?;
    }

    // V3: 主动式AI记忆系统
    if version < 3 {
        let tx = conn.unchecked_transaction()?;
        create_screenshot_analyses_table(&tx)?;
        create_projects_table(&tx)?;
        create_habits_table(&tx)?;
        create_summaries_table(&tx)?;
        create_proactive_suggestions_table(&tx)?;
        set_schema_version(&tx, 3)?;
        tx.commit()?;
    }

    // V4: 视频录制
    if version < 4 {
        let tx = conn.unchecked_transaction()?;
        create_recordings_table(&tx)?;
        set_schema_version(&tx, 4)?;
        tx.commit()?;
    }

    // V5: 一次性分析扩展 + recordings.activity_id
    if version < 5 {
        let tx = conn.unchecked_transaction()?;
        migrate_v5(&tx)?;
        set_schema_version(&tx, 5)?;
        tx.commit()?;
    }

    // V6: AI 配置持久化
    if version < 6 {
        let tx = conn.unchecked_transaction()?;
        create_ai_config_table(&tx)?;
        set_schema_version(&tx, 6)?;
        tx.commit()?;
    }

    // V7: projects.updated_at + activities.project_id
    if version < 7 {
        let tx = conn.unchecked_transaction()?;
        migrate_v7(&tx)?;
        set_schema_version(&tx, 7)?;
        tx.commit()?;
    }

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

// ============================================================================
// V3 Tables: Proactive AI Memory System
// ============================================================================

/// 创建 screenshot_analyses 表 - AI截图理解缓存
fn create_screenshot_analyses_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS screenshot_analyses (
            screenshot_id TEXT PRIMARY KEY,
            application TEXT NOT NULL,
            activity_type TEXT NOT NULL,
            activity_description TEXT NOT NULL,
            key_elements TEXT NOT NULL DEFAULT '[]',
            ocr_text TEXT,
            context_tags TEXT NOT NULL DEFAULT '[]',
            productivity_score INTEGER DEFAULT 5,
            analysis_json TEXT NOT NULL,
            analyzed_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (screenshot_id) REFERENCES screenshots(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sa_application
         ON screenshot_analyses(application)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sa_activity_type
         ON screenshot_analyses(activity_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sa_analyzed_at
         ON screenshot_analyses(analyzed_at DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 projects 表 - 项目追踪
fn create_projects_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            start_date INTEGER NOT NULL,
            last_activity_date INTEGER NOT NULL,
            activity_count INTEGER DEFAULT 0,
            tags TEXT NOT NULL DEFAULT '[]',
            status TEXT NOT NULL DEFAULT 'active',
            markdown_path TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_projects_status
         ON projects(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_projects_last_activity
         ON projects(last_activity_date DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 habits 表 - 行为模式追踪
fn create_habits_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habits (
            id TEXT PRIMARY KEY,
            pattern_name TEXT NOT NULL,
            pattern_type TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.0,
            frequency TEXT NOT NULL DEFAULT 'daily',
            trigger_conditions TEXT,
            typical_time TEXT,
            last_occurrence INTEGER,
            occurrence_count INTEGER DEFAULT 0,
            markdown_path TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_habits_pattern_type
         ON habits(pattern_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_habits_confidence
         ON habits(confidence DESC)",
        [],
    )?;

    Ok(())
}

/// 创建 summaries 表 - 聚合总结
fn create_summaries_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS summaries (
            id TEXT PRIMARY KEY,
            summary_type TEXT NOT NULL,
            date_start TEXT NOT NULL,
            date_end TEXT NOT NULL,
            content TEXT NOT NULL,
            activity_ids TEXT NOT NULL DEFAULT '[]',
            project_ids TEXT,
            markdown_path TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_summaries_type
         ON summaries(summary_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_summaries_date
         ON summaries(date_start, date_end)",
        [],
    )?;

    Ok(())
}

/// 创建 proactive_suggestions 表 - 主动建议
fn create_proactive_suggestions_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS proactive_suggestions (
            id TEXT PRIMARY KEY,
            suggestion_type TEXT NOT NULL,
            trigger_context TEXT NOT NULL,
            message TEXT NOT NULL,
            priority INTEGER DEFAULT 0,
            delivered INTEGER DEFAULT 0,
            delivered_at INTEGER,
            user_action TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ps_type
         ON proactive_suggestions(suggestion_type)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ps_delivered
         ON proactive_suggestions(delivered)",
        [],
    )?;

    Ok(())
}

// ============================================================================
// V4 Tables: Video Recording
// ============================================================================

/// 创建 recordings 表 - 视频录制分段
fn create_recordings_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS recordings (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL,
            start_time INTEGER NOT NULL,
            end_time INTEGER,
            duration_secs INTEGER,
            fps INTEGER DEFAULT 2,
            analyzed INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_recordings_start_time
         ON recordings(start_time DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_recordings_analyzed
         ON recordings(analyzed)",
        [],
    )?;

    Ok(())
}

// ============================================================================
// V5: One-shot Analysis Extension + recordings.activity_id
// ============================================================================

/// V5 迁移：扩展 screenshot_analyses 表 + recordings 添加 activity_id
fn migrate_v5(conn: &Connection) -> Result<()> {
    // screenshot_analyses 新增 4 列
    let sa_columns = [
        ("activity_category", "TEXT DEFAULT 'other'"),
        ("activity_summary", "TEXT DEFAULT ''"),
        ("project_name", "TEXT"),
        ("accomplishments", "TEXT DEFAULT '[]'"),
    ];
    for (col, typedef) in &sa_columns {
        let has: bool = conn
            .prepare(&format!(
                "SELECT COUNT(*) FROM pragma_table_info('screenshot_analyses') WHERE name='{}'",
                col
            ))?
            .query_row([], |row| row.get::<_, i64>(0))
            .map(|c| c > 0)?;
        if !has {
            conn.execute(
                &format!("ALTER TABLE screenshot_analyses ADD COLUMN {} {}", col, typedef),
                [],
            )?;
        }
    }

    // recordings 添加 activity_id
    let has_activity_id: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('recordings') WHERE name='activity_id'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|c| c > 0)?;
    if !has_activity_id {
        conn.execute("ALTER TABLE recordings ADD COLUMN activity_id TEXT", [])?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_activity_id ON recordings(activity_id)",
            [],
        )?;
    }

    Ok(())
}

// ============================================================================
// V6: AI Config Persistence
// ============================================================================

/// 创建 ai_config 表 - 存储 AI 配置 JSON
fn create_ai_config_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;
    Ok(())
}

// ============================================================================
// V7: projects.updated_at + activities.project_id
// ============================================================================

/// V7 迁移：projects 添加 updated_at，activities 添加 project_id
fn migrate_v7(conn: &Connection) -> Result<()> {
    // projects 添加 updated_at 列
    let has_updated_at: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name='updated_at'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|c| c > 0)?;
    if !has_updated_at {
        conn.execute(
            "ALTER TABLE projects ADD COLUMN updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))",
            [],
        )?;
    }

    // activities 添加 project_id 列（用于追踪活动-项目关联）
    let has_project_id: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('activities') WHERE name='project_id'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|c| c > 0)?;
    if !has_project_id {
        conn.execute("ALTER TABLE activities ADD COLUMN project_id TEXT", [])?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_activities_project_id ON activities(project_id)",
            [],
        )?;
    }

    Ok(())
}

// ============================================================================
// V2 Helpers
// ============================================================================

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

        // 验证V3表创建
        assert!(tables.contains(&"screenshot_analyses".to_string()));
        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"habits".to_string()));
        assert!(tables.contains(&"summaries".to_string()));
        assert!(tables.contains(&"proactive_suggestions".to_string()));

        // 验证V4表创建
        assert!(tables.contains(&"recordings".to_string()));

        // 验证版本号
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 7);
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
        assert_eq!(version, 7);
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
