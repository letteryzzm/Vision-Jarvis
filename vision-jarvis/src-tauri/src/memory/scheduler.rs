/// 记忆生成调度器
///
/// 协调截图分析和记忆生成的后台任务

use anyhow::Result;
use crate::db::Database;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;
use log::{info, error};

/// 记忆生成调度器
pub struct MemoryScheduler {
    db: Arc<Database>,
    api_key: Arc<String>,
}

impl MemoryScheduler {
    /// 创建新的调度器
    pub fn new(db: Database, api_key: String) -> Result<Self> {
        Ok(Self {
            db: Arc::new(db),
            api_key: Arc::new(api_key),
        })
    }

    /// 启动后台调度任务
    pub fn start(&self) -> JoinHandle<()> {
        let analyzer_interval = Duration::from_secs(300); // 每5分钟分析一次未处理的截图
        let short_term_interval = Duration::from_secs(1800); // 每30分钟生成一次短期记忆
        let long_term_interval = Duration::from_secs(86400); // 每24小时生成一次长期记忆

        let db = Arc::clone(&self.db);
        let api_key = Arc::clone(&self.api_key);

        tokio::spawn(async move {
            let mut analyzer_tick = interval(analyzer_interval);
            let mut short_term_tick = interval(short_term_interval);
            let mut long_term_tick = interval(long_term_interval);

            loop {
                tokio::select! {
                    _ = analyzer_tick.tick() => {
                        if let Err(e) = Self::analyze_pending_screenshots(&db, &api_key).await {
                            error!("截图分析任务失败: {}", e);
                        }
                    }
                    _ = short_term_tick.tick() => {
                        if let Err(e) = Self::generate_short_term_memories(&db).await {
                            error!("短期记忆生成失败: {}", e);
                        }
                    }
                    _ = long_term_tick.tick() => {
                        if let Err(e) = Self::generate_long_term_memories(&db).await {
                            error!("长期记忆生成失败: {}", e);
                        }
                    }
                }
            }
        })
    }

    /// 分析待处理的截图
    async fn analyze_pending_screenshots(
        db: &Database,
        _api_key: &str,
    ) -> Result<()> {
        info!("开始分析待处理的截图");

        // 获取未分析的截图
        let screenshots = Self::get_pending_screenshots(db)?;

        if screenshots.is_empty() {
            info!("没有待处理的截图");
            return Ok(());
        }

        info!("找到 {} 个待处理的截图", screenshots.len());

        // TODO: 实际的分析逻辑将在后续实现
        // 这里需要在每次调用时创建 ScreenshotAnalyzer 和 EmbeddingGenerator
        // 避免需要 Clone trait

        Ok(())
    }

    /// 生成短期记忆
    async fn generate_short_term_memories(_db: &Database) -> Result<()> {
        info!("开始生成短期记忆");
        // TODO: 实现短期记忆生成逻辑
        Ok(())
    }

    /// 生成长期记忆
    async fn generate_long_term_memories(_db: &Database) -> Result<()> {
        info!("开始生成长期记忆");
        // TODO: 实现长期记忆生成逻辑
        Ok(())
    }

    /// 获取待处理的截图
    fn get_pending_screenshots(db: &Database) -> Result<Vec<PendingScreenshot>> {
        db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, path FROM screenshots WHERE analyzed = 0 LIMIT 10"
            )?;

            let screenshots = stmt
                .query_map([], |row| {
                    Ok(PendingScreenshot {
                        id: row.get(0)?,
                        path: row.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(screenshots)
        })
    }
}

/// 待处理的截图信息
#[derive(Debug)]
struct PendingScreenshot {
    id: String,
    path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_scheduler_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();

        let scheduler = MemoryScheduler::new(db, "test-key".to_string());
        assert!(scheduler.is_ok());
    }

    #[test]
    fn test_get_pending_screenshots_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();

        let screenshots = MemoryScheduler::get_pending_screenshots(&db).unwrap();
        assert_eq!(screenshots.len(), 0);
    }
}
