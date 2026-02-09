use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub mod schema;
pub mod migrations;

/// 数据库管理器
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 创建新的数据库连接
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 初始化数据库表结构
    pub fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        migrations::run_migrations(&conn)?;
        Ok(())
    }

    /// 获取数据库连接的引用（需要锁定）
    pub fn with_connection<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Connection) -> Result<R>,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();

        let db = Database::new(db_path).unwrap();
        assert!(db.initialize().is_ok());
    }
}
