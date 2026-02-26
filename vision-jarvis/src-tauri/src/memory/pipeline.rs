/// 记忆管道调度器
///
/// 整合记忆系统的所有组件:
/// 1. 录制分析 (90秒) - AI理解每个录制分段
/// 2. 活动分组 (30分钟) - 聚合录制分段为活动会话
/// 3. 索引同步 (10分钟) - 增量文件索引
/// 4. 习惯检测 (每日) - 识别行为模式
/// 5. 日总结 (每日23:00) - 生成日总结

use anyhow::Result;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;
use chrono::{Local, Timelike};
use log::{info, error, warn};

use crate::ai::AIClient;
use crate::db::Database;
use super::{
    activity_grouper::{ActivityGrouper, GroupingConfig},
    markdown_generator::{MarkdownGenerator, GeneratorConfig},
    index_manager::{IndexManager, IndexConfig},
    screenshot_analyzer::{ScreenshotAnalyzer, AnalyzerConfig},
    summary_generator::{SummaryGenerator, SummaryConfig},
    project_extractor::{ProjectExtractor, ProjectExtractorConfig},
    habit_detector::{HabitDetector, HabitDetectorConfig},
};

/// 管道调度器
pub struct PipelineScheduler {
    db: Arc<Database>,
    grouper: Arc<ActivityGrouper>,
    markdown_gen: Arc<MarkdownGenerator>,
    index_manager: Arc<IndexManager>,
    /// 动态可更新的录制分析器（支持运行时接入AI）
    screenshot_analyzer: Arc<RwLock<Option<Arc<ScreenshotAnalyzer>>>>,
    summary_generator: Arc<SummaryGenerator>,
    project_extractor: Arc<ProjectExtractor>,
    habit_detector: Arc<HabitDetector>,
    /// 即时分析 channel receiver（录制完成后立刻触发）
    analysis_rx: std::sync::Mutex<Option<tokio::sync::mpsc::Receiver<(String, std::path::PathBuf)>>>,
}

impl PipelineScheduler {
    /// 创建新的调度器（接受共享的数据库引用）
    pub fn new(
        db: Arc<Database>,
        storage_root: PathBuf,
        enable_ai_summary: bool,
    ) -> Result<Self> {
        let grouper = Arc::new(ActivityGrouper::new(
            Arc::clone(&db),
            GroupingConfig::default(),
        ));

        let markdown_gen = Arc::new(MarkdownGenerator::new(GeneratorConfig {
            storage_root: storage_root.clone(),
            enable_ai_summary,
        }));

        let index_manager = Arc::new(IndexManager::new(
            Arc::clone(&db),
            IndexConfig {
                memory_root: storage_root.clone(),
                ..Default::default()
            },
        ));

        let summary_generator = Arc::new(SummaryGenerator::new(
            None,
            Arc::clone(&db),
            SummaryConfig {
                storage_root: storage_root.clone(),
                enable_ai: enable_ai_summary,
            },
        ));

        let project_extractor = Arc::new(ProjectExtractor::new(
            Arc::clone(&db),
            ProjectExtractorConfig {
                storage_root: storage_root.clone(),
                ..Default::default()
            },
        ));

        let habit_detector = Arc::new(HabitDetector::new(
            Arc::clone(&db),
            HabitDetectorConfig {
                storage_root: storage_root.clone(),
                ..Default::default()
            },
        ));

        Ok(Self {
            db,
            grouper,
            markdown_gen,
            index_manager,
            screenshot_analyzer: Arc::new(RwLock::new(None)),
            summary_generator,
            project_extractor,
            habit_detector,
            analysis_rx: std::sync::Mutex::new(None),
        })
    }

    /// 设置即时分析 channel receiver
    pub fn with_analysis_receiver(
        self,
        rx: tokio::sync::mpsc::Receiver<(String, std::path::PathBuf)>,
    ) -> Self {
        *self.analysis_rx.lock().unwrap() = Some(rx);
        self
    }

    /// 动态连接AI客户端（可在管道运行中调用）
    pub async fn connect_ai(&self, ai_client: AIClient) {
        let ai_client = Arc::new(ai_client);

        let analyzer = ScreenshotAnalyzer::new(
            Arc::clone(&ai_client),
            Arc::clone(&self.db),
            AnalyzerConfig::default(),
        );

        let mut guard = self.screenshot_analyzer.write().await;
        *guard = Some(Arc::new(analyzer));

        // 传播 AI client 到 SummaryGenerator 和 MarkdownGenerator
        self.summary_generator.set_ai_client(Arc::clone(&ai_client)).await;
        self.markdown_gen.set_ai_client(Arc::clone(&ai_client)).await;

        info!("[Pipeline] AI客户端已连接，录制分析/总结/Markdown生成已启用");
    }

    /// 检查AI是否已连接
    pub async fn is_ai_connected(&self) -> bool {
        self.screenshot_analyzer.read().await.is_some()
    }

    /// 启动管道调度
    pub fn start(&self) -> JoinHandle<()> {
        let grouping_interval = Duration::from_secs(1800);    // 30分钟 - 分组活动
        let indexing_interval = Duration::from_secs(600);     // 10分钟 - 同步索引
        let habit_interval = Duration::from_secs(86400);      // 24小时 - 习惯检测
        let summary_check_interval = Duration::from_secs(600); // 10分钟 - 检查是否到日总结时间

        let grouper = Arc::clone(&self.grouper);
        let markdown_gen = Arc::clone(&self.markdown_gen);
        let index_manager = Arc::clone(&self.index_manager);
        let screenshot_analyzer = Arc::clone(&self.screenshot_analyzer);
        let habit_detector = Arc::clone(&self.habit_detector);
        let project_extractor = Arc::clone(&self.project_extractor);
        let summary_generator = Arc::clone(&self.summary_generator);
        // 将 analysis_rx 移动到 spawn 闭包中（取出所有权）
        let analysis_rx = self.analysis_rx.lock().unwrap().take();

        tokio::spawn(async move {
            let mut grouping_tick = interval(grouping_interval);
            let mut indexing_tick = interval(indexing_interval);
            let mut habit_tick = interval(habit_interval);
            let mut summary_tick = interval(summary_check_interval);
            // 记录上次生成日总结的日期，避免重复生成
            let mut last_summary_date: Option<String> = None;
            // 即时分析 receiver（从 channel 中取出，放入本地 mut 变量）
            let mut instant_rx = analysis_rx;

            loop {
                tokio::select! {
                    msg = async {
                        if let Some(ref mut rx) = instant_rx {
                            rx.recv().await
                        } else {
                            std::future::pending::<Option<(String, std::path::PathBuf)>>().await
                        }
                    } => {
                        if let Some((id, path)) = msg {
                            let analyzer = screenshot_analyzer.read().await;
                            if let Some(ref analyzer) = *analyzer {
                                info!("[Pipeline] 即时分析录制: {} path={}", &id[..8.min(id.len())], path.display());
                                match analyzer.analyze_single_direct(&id, &path).await {
                                    Ok(_) => {},
                                    Err(e) => warn!("即时分析失败: {} - {}", &id[..8.min(id.len())], e),
                                }
                            } else {
                                warn!("[Pipeline] 收到即时分析请求但 AI 未连接，录制 {}", &id[..8.min(id.len())]);
                            }
                        }
                    }
                    _ = grouping_tick.tick() => {
                        if let Err(e) = Self::group_and_generate(&grouper, &markdown_gen).await {
                            error!("Activity grouping failed: {}", e);
                        }
                        // 分组后尝试提取项目
                        if let Err(e) = project_extractor.process_unlinked_activities().await {
                            error!("Project extraction failed: {}", e);
                        }
                    }
                    _ = indexing_tick.tick() => {
                        if let Err(e) = Self::sync_index(&index_manager).await {
                            error!("Index sync failed: {}", e);
                        }
                    }
                    _ = habit_tick.tick() => {
                        match habit_detector.detect_all() {
                            Ok(result) => {
                                info!("Habit detection: {} detected, {} new, {} updated, {} decayed, {} removed",
                                    result.total_detected, result.new_habits, result.updated_habits,
                                    result.decayed, result.removed);
                            }
                            Err(e) => error!("Habit detection failed: {}", e),
                        }
                    }
                    _ = summary_tick.tick() => {
                        // 每10分钟检查：本地时间23点且今天未生成过 → 触发日总结
                        let now = Local::now();
                        let today = now.format("%Y-%m-%d").to_string();
                        let already_done = last_summary_date.as_ref() == Some(&today);

                        if now.hour() == 23 && !already_done {
                            info!("[Pipeline] Triggering daily summary for {}", today);
                            match summary_generator.generate_daily(&today).await {
                                Ok(summary) => {
                                    info!("[Pipeline] Daily summary generated for {} ({} activities)",
                                        summary.date_start, summary.activity_ids.len());
                                    last_summary_date = Some(today);
                                }
                                Err(e) => {
                                    warn!("[Pipeline] Daily summary failed for {}: {}", today, e);
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    /// Task: 分组活动并生成Markdown
    async fn group_and_generate(
        grouper: &ActivityGrouper,
        markdown_gen: &MarkdownGenerator,
    ) -> Result<()> {
        info!("Starting activity grouping...");

        let recordings = grouper.get_ungrouped_recordings()?;

        if recordings.is_empty() {
            info!("No ungrouped recordings");
            return Ok(());
        }

        info!("Found {} ungrouped recordings", recordings.len());

        let activities = grouper.group_recordings(&recordings)?;

        if activities.is_empty() {
            info!("No activities generated (may not meet minimum criteria)");
            return Ok(());
        }

        info!("Generated {} activities", activities.len());

        for activity in activities {
            match markdown_gen.generate(&activity).await {
                Ok(file_path) => {
                    info!("Generated markdown: {}", file_path.display());
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

    /// Task: 同步索引
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

}
