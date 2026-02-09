/// 记忆管道调度器 V2
///
/// 整合事项驱动记忆系统的所有组件:
/// 1. 截图分析 (5分钟)
/// 2. 活动分组 (30分钟)
/// 3. 索引同步 (10分钟)
/// 4. AI总结生成 (24小时)

use anyhow::Result;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;
use log::{info, error};

use crate::db::Database;
use crate::ai::{OpenAIClient, analyzer::ScreenshotAnalyzer, embeddings::EmbeddingGenerator};
use super::{
    activity_grouper::{ActivityGrouper, GroupingConfig},
    markdown_generator::{MarkdownGenerator, GeneratorConfig},
    index_manager::{IndexManager, IndexConfig},
};

/// 管道调度器
pub struct PipelineScheduler {
    db: Arc<Database>,
    grouper: Arc<ActivityGrouper>,
    markdown_gen: Arc<MarkdownGenerator>,
    index_manager: Arc<IndexManager>,
    analyzer: Arc<ScreenshotAnalyzer>,
}

impl PipelineScheduler {
    /// 创建新的调度器
    pub fn new(
        db: Database,
        api_key: String,
        storage_root: PathBuf,
        enable_ai_summary: bool,
    ) -> Result<Self> {
        let db = Arc::new(db);

        // 初始化组件
        let grouper = Arc::new(ActivityGrouper::new(
            Arc::clone(&db),
            GroupingConfig::default(),
        ));

        let markdown_gen = Arc::new(MarkdownGenerator::new(GeneratorConfig {
            storage_root: storage_root.clone(),
            enable_ai_summary,
            openai_api_key: Some(api_key.clone()),
        }));

        let embedder = EmbeddingGenerator::new(api_key.clone())?;
        let index_manager = Arc::new(IndexManager::new(
            Arc::clone(&db),
            embedder,
            IndexConfig {
                memory_root: storage_root,
                ..Default::default()
            },
        ));

        let analyzer = Arc::new(ScreenshotAnalyzer::new(api_key)?);

        Ok(Self {
            db,
            grouper,
            markdown_gen,
            index_manager,
            analyzer,
        })
    }

    /// 启动管道调度
    pub fn start(&self) -> JoinHandle<()> {
        let analyzer_interval = Duration::from_secs(300);    // 5分钟 - 分析截图
        let grouping_interval = Duration::from_secs(1800);   // 30分钟 - 分组活动
        let indexing_interval = Duration::from_secs(600);    // 10分钟 - 同步索引
        let summary_interval = Duration::from_secs(86400);   // 24小时 - 生成总结

        let db = Arc::clone(&self.db);
        let grouper = Arc::clone(&self.grouper);
        let markdown_gen = Arc::clone(&self.markdown_gen);
        let index_manager = Arc::clone(&self.index_manager);
        let analyzer = Arc::clone(&self.analyzer);

        tokio::spawn(async move {
            let mut analyzer_tick = interval(analyzer_interval);
            let mut grouping_tick = interval(grouping_interval);
            let mut indexing_tick = interval(indexing_interval);
            let mut summary_tick = interval(summary_interval);

            loop {
                tokio::select! {
                    _ = analyzer_tick.tick() => {
                        if let Err(e) = Self::analyze_screenshots(&analyzer, &db).await {
                            error!("Screenshot analysis failed: {}", e);
                        }
                    }
                    _ = grouping_tick.tick() => {
                        if let Err(e) = Self::group_and_generate(&grouper, &markdown_gen).await {
                            error!("Activity grouping failed: {}", e);
                        }
                    }
                    _ = indexing_tick.tick() => {
                        if let Err(e) = Self::sync_index(&index_manager).await {
                            error!("Index sync failed: {}", e);
                        }
                    }
                    _ = summary_tick.tick() => {
                        if let Err(e) = Self::generate_summaries(&db).await {
                            error!("Summary generation failed: {}", e);
                        }
                    }
                }
            }
        })
    }

    /// Task 1: 分析待处理的截图
    async fn analyze_screenshots(
        analyzer: &ScreenshotAnalyzer,
        db: &Database,
    ) -> Result<()> {
        info!("Starting screenshot analysis...");

        // 获取未分析的截图（限制批量大小）
        let screenshots = Self::get_pending_screenshots(db, 10)?;

        if screenshots.is_empty() {
            return Ok(());
        }

        info!("Found {} pending screenshots", screenshots.len());

        for screenshot in screenshots {
            match analyzer.analyze_screenshot(&PathBuf::from(&screenshot.path)).await {
                Ok(analysis_result) => {
                    // 保存分析结果
                    let analysis_json = serde_json::to_string(&analysis_result)?;
                    db.with_connection(|conn| {
                        conn.execute(
                            "UPDATE screenshots
                             SET analyzed = 1,
                                 analysis_result = ?1,
                                 analyzed_at = strftime('%s', 'now')
                             WHERE id = ?2",
                            rusqlite::params![analysis_json, &screenshot.id],
                        )?;
                        Ok(())
                    })?;

                    info!("Analyzed screenshot: {}", screenshot.id);
                }
                Err(e) => {
                    error!("Failed to analyze {}: {}", screenshot.id, e);
                }
            }
        }

        Ok(())
    }

    /// Task 2: 分组活动并生成Markdown
    async fn group_and_generate(
        grouper: &ActivityGrouper,
        markdown_gen: &MarkdownGenerator,
    ) -> Result<()> {
        info!("Starting activity grouping...");

        // 获取未分组的截图
        let screenshots = grouper.get_ungrouped_screenshots()?;

        if screenshots.is_empty() {
            info!("No ungrouped screenshots");
            return Ok(());
        }

        info!("Found {} ungrouped screenshots", screenshots.len());

        // 分组
        let activities = grouper.group_screenshots(&screenshots)?;

        if activities.is_empty() {
            info!("No activities generated (may not meet minimum criteria)");
            return Ok(());
        }

        info!("Generated {} activities", activities.len());

        // 为每个活动生成Markdown并保存
        for activity in activities {
            // 生成Markdown文件
            match markdown_gen.generate(&activity).await {
                Ok(file_path) => {
                    info!("Generated markdown: {}", file_path.display());

                    // 保存活动到数据库
                    grouper.save_activity(&activity)?;

                    info!("Saved activity: {}", activity.id);
                }
                Err(e) => {
                    error!("Failed to generate markdown for {}: {}", activity.id, e);
                }
            }
        }

        Ok(())
    }

    /// Task 3: 同步索引
    async fn sync_index(index_manager: &IndexManager) -> Result<()> {
        info!("Starting index sync...");

        let stats = index_manager.sync().await?;

        info!(
            "Index sync completed - total: {}, indexed: {}, skipped: {}, failed: {}, chunks: {}",
            stats.total_files,
            stats.indexed_files,
            stats.skipped_files,
            stats.failed_files,
            stats.new_chunks
        );

        Ok(())
    }

    /// Task 4: 生成周/月总结
    async fn generate_summaries(_db: &Database) -> Result<()> {
        info!("Starting summary generation...");
        // TODO: Phase 7 实现
        Ok(())
    }

    /// 获取待处理的截图
    fn get_pending_screenshots(db: &Database, limit: usize) -> Result<Vec<PendingScreenshot>> {
        db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, path FROM screenshots
                 WHERE analyzed = 0
                 ORDER BY captured_at ASC
                 LIMIT ?1"
            )?;

            let screenshots = stmt
                .query_map([limit], |row| {
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

/// 待处理的截图
#[derive(Debug)]
struct PendingScreenshot {
    id: String,
    path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_pending_screenshots_empty() {
        let db = Database::open_in_memory().unwrap();
        let screenshots = PipelineScheduler::get_pending_screenshots(&db, 10).unwrap();
        assert_eq!(screenshots.len(), 0);
    }

    #[test]
    fn test_get_pending_screenshots_with_limit() {
        let db = Database::open_in_memory().unwrap();

        // 插入测试数据
        db.with_connection(|conn| {
            for i in 0..5 {
                conn.execute(
                    "INSERT INTO screenshots (id, path, captured_at, analyzed)
                     VALUES (?1, ?2, ?3, 0)",
                    [format!("s{}", i), format!("/path/s{}.png", i), i.to_string()],
                )?;
            }
            Ok::<(), anyhow::Error>(())
        }).unwrap();

        // 限制返回3条
        let screenshots = PipelineScheduler::get_pending_screenshots(&db, 3).unwrap();
        assert_eq!(screenshots.len(), 3);
    }
}
