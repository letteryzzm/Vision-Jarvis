/// 活动分组器 - 将连续的相似截图聚合为活动会话
///
/// 核心算法：
/// 1. 按时间排序分析后的截图
/// 2. 根据应用名称、活动类型、时间间隔判断是否属于同一活动
/// 3. 生成ActivitySession列表

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{Database, schema::{ActivitySession, ScreenshotAnalysisSummary, ActivityCategory, AnalysisResult}};

/// 分组配置
#[derive(Debug, Clone)]
pub struct GroupingConfig {
    /// 最大时间间隔(秒) - 超过此间隔视为新活动
    pub max_gap_seconds: i64,
    /// 最少截图数 - 少于此数量的活动将被过滤
    pub min_screenshots: usize,
    /// 最大持续时间(秒) - 超过此时长自动拆分
    pub max_duration_seconds: i64,
    /// 最短持续时间(秒) - 短于此时长的活动将被过滤
    pub min_duration_seconds: i64,
}

impl Default for GroupingConfig {
    fn default() -> Self {
        Self {
            max_gap_seconds: 300,      // 5分钟
            min_screenshots: 2,         // 至少2张截图
            max_duration_seconds: 7200, // 2小时
            min_duration_seconds: 60,   // 1分钟
        }
    }
}

/// 活动分组器
pub struct ActivityGrouper {
    db: Arc<Database>,
    config: GroupingConfig,
}

/// 分析后的截图信息
#[derive(Debug, Clone)]
pub struct AnalyzedScreenshot {
    pub id: String,
    pub path: String,
    pub captured_at: i64,
    pub analysis: AnalysisResult,
}

/// 临时活动组(用于构建过程)
#[derive(Debug)]
struct ActivityGroup {
    screenshots: Vec<AnalyzedScreenshot>,
    application: String,
    activity: String,
    category: ActivityCategory,
    start_time: i64,
    last_time: i64,
}

impl ActivityGroup {
    fn new(screenshot: AnalyzedScreenshot) -> Self {
        let application = screenshot.analysis.application.clone();
        let activity = screenshot.analysis.activity.clone();
        let category = screenshot.analysis.category.clone();
        let timestamp = screenshot.captured_at;

        Self {
            screenshots: vec![screenshot],
            application,
            activity,
            category,
            start_time: timestamp,
            last_time: timestamp,
        }
    }

    fn add(&mut self, screenshot: AnalyzedScreenshot) {
        self.last_time = screenshot.captured_at;
        self.screenshots.push(screenshot);
    }

    fn duration_seconds(&self) -> i64 {
        self.last_time - self.start_time
    }

    fn meets_minimum_criteria(&self, config: &GroupingConfig) -> bool {
        self.screenshots.len() >= config.min_screenshots
            && self.duration_seconds() >= config.min_duration_seconds
    }

    /// 转换为ActivitySession
    fn finalize(&self, markdown_path: String) -> ActivitySession {
        let duration_minutes = (self.duration_seconds() / 60).max(1);

        // 生成标题(后续可由AI生成更好的标题)
        let title = format!("{}中{}", self.application, self.activity);

        // 提取截图ID和分析摘要
        let screenshot_ids: Vec<String> = self.screenshots.iter()
            .map(|s| s.id.clone())
            .collect();

        let screenshot_analyses: Vec<ScreenshotAnalysisSummary> = self.screenshots.iter()
            .map(|s| ScreenshotAnalysisSummary {
                id: s.id.clone(),
                timestamp: s.captured_at,
                path: s.path.clone(),
                analysis: s.analysis.description.clone(),
            })
            .collect();

        // 提取标签(基于活动和应用)
        let mut tags = Vec::new();
        tags.push(self.application.clone());
        tags.push(self.activity.clone());

        ActivitySession {
            id: generate_activity_id(self.start_time),
            title,
            start_time: self.start_time,
            end_time: self.last_time,
            duration_minutes,
            application: self.application.clone(),
            category: self.category.clone(),
            screenshot_ids,
            screenshot_analyses,
            tags,
            markdown_path,
            summary: None, // 将由MarkdownGenerator生成AI总结
            indexed: false,
            created_at: Utc::now().timestamp(),
        }
    }
}

impl ActivityGrouper {
    pub fn new(db: Arc<Database>, config: GroupingConfig) -> Self {
        Self { db, config }
    }

    /// 获取未分组的已分析截图
    pub fn get_ungrouped_screenshots(&self) -> Result<Vec<AnalyzedScreenshot>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, path, captured_at, analysis_result
                 FROM screenshots
                 WHERE analyzed = 1
                   AND analysis_result IS NOT NULL
                   AND activity_id IS NULL
                 ORDER BY captured_at ASC"
            )?;

            let screenshots = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(1)?;
                let captured_at: i64 = row.get(2)?;
                let analysis_json: String = row.get(3)?;

                // 解析analysis_result JSON
                let analysis: AnalysisResult = match serde_json::from_str(&analysis_json) {
                    Ok(a) => a,
                    Err(e) => {
                        log::warn!(
                            "Skipping screenshot {} - failed to parse analysis_result: {}",
                            id, e
                        );
                        // 返回一个标记对象，后续过滤掉
                        AnalysisResult {
                            activity: String::new(),
                            application: String::new(),
                            description: String::new(),
                            category: ActivityCategory::Other,
                        }
                    }
                };

                Ok(AnalyzedScreenshot {
                    id,
                    path,
                    captured_at,
                    analysis,
                })
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            // 过滤掉解析失败的截图（activity为空）
            let screenshots: Vec<_> = screenshots.into_iter()
                .filter(|s| !s.analysis.activity.is_empty())
                .collect();

            Ok(screenshots)
        })
    }

    /// 将截图列表分组为活动会话
    pub fn group_screenshots(&self, screenshots: &[AnalyzedScreenshot]) -> Result<Vec<ActivitySession>> {
        if screenshots.is_empty() {
            return Ok(Vec::new());
        }

        let mut groups = Vec::new();
        let mut current_group: Option<ActivityGroup> = None;
        let mut date_counters: HashMap<String, usize> = HashMap::new();

        for screenshot in screenshots {
            if let Some(ref mut group) = current_group {
                let time_gap = screenshot.captured_at - group.last_time;
                let is_same_app = screenshot.analysis.application == group.application;
                let is_same_activity = screenshot.analysis.activity == group.activity
                    || activities_are_related(&group.activity, &screenshot.analysis.activity);
                let current_duration = screenshot.captured_at - group.start_time;

                // 判断是否应该合并到当前组
                let should_merge = time_gap <= self.config.max_gap_seconds
                    && is_same_app
                    && is_same_activity
                    && current_duration <= self.config.max_duration_seconds;

                if should_merge {
                    group.add(screenshot.clone());
                } else {
                    // 保存当前组(如果符合最小标准)
                    if group.meets_minimum_criteria(&self.config) {
                        let date_str = format_timestamp_as_date(group.start_time);
                        let counter = date_counters.entry(date_str.clone()).or_insert(0);
                        *counter += 1;
                        let markdown_path = format!("activities/{}/activity-{:03}.md", date_str, counter);
                        groups.push(group.finalize(markdown_path));
                    }

                    // 开启新组
                    current_group = Some(ActivityGroup::new(screenshot.clone()));
                }
            } else {
                // 第一个截图,创建新组
                current_group = Some(ActivityGroup::new(screenshot.clone()));
            }
        }

        // 处理最后一个组
        if let Some(group) = current_group {
            if group.meets_minimum_criteria(&self.config) {
                let date_str = format_timestamp_as_date(group.start_time);
                let counter = date_counters.entry(date_str.clone()).or_insert(0);
                *counter += 1;
                let markdown_path = format!("activities/{}/activity-{:03}.md", date_str, counter);
                groups.push(group.finalize(markdown_path));
            }
        }

        Ok(groups)
    }

    /// 保存活动会话到数据库并关联截图
    pub fn save_activity(&self, activity: &ActivitySession) -> Result<()> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;

            // 插入活动记录
            tx.execute(
                "INSERT INTO activities (
                    id, title, start_time, end_time, duration_minutes,
                    application, category, screenshot_ids, tags,
                    markdown_path, summary, indexed, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    &activity.id,
                    &activity.title,
                    activity.start_time,
                    activity.end_time,
                    activity.duration_minutes,
                    &activity.application,
                    serde_json::to_string(&activity.category)?,
                    serde_json::to_string(&activity.screenshot_ids)?,
                    serde_json::to_string(&activity.tags)?,
                    &activity.markdown_path,
                    &activity.summary,
                    if activity.indexed { 1 } else { 0 },
                    activity.created_at,
                ],
            )?;

            // 关联截图到此活动
            for screenshot_id in &activity.screenshot_ids {
                tx.execute(
                    "UPDATE screenshots SET activity_id = ?1 WHERE id = ?2",
                    rusqlite::params![&activity.id, screenshot_id],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
    }
}

/// 判断两个活动是否相关(可以合并)
fn activities_are_related(a: &str, b: &str) -> bool {
    // V1: 简单的前缀匹配
    if a.starts_with(b) || b.starts_with(a) {
        return true;
    }

    // 提取常见动词判断相关性
    let common_verbs = ["编写", "查看", "浏览", "阅读", "编辑", "调试", "运行", "测试"];
    let a_has_verb = common_verbs.iter().any(|v| a.contains(v));
    let b_has_verb = common_verbs.iter().any(|v| b.contains(v));

    // 如果都包含相同类别的动词,认为相关
    if a_has_verb && b_has_verb {
        for verb in common_verbs {
            if a.contains(verb) && b.contains(verb) {
                return true;
            }
        }
    }

    false
}

/// 生成活动ID
fn generate_activity_id(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| Utc::now());
    let date_str = dt.format("%Y-%m-%d").to_string();
    let uuid_short = Uuid::new_v4().to_string().split('-').next().unwrap().to_string();
    format!("activity-{}-{}", date_str, uuid_short)
}

/// 格式化时间戳为日期字符串 YYYY-MM-DD
fn format_timestamp_as_date(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| Utc::now())
        .format("%Y-%m-%d")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn create_test_screenshot(
        id: &str,
        timestamp: i64,
        app: &str,
        activity: &str,
    ) -> AnalyzedScreenshot {
        AnalyzedScreenshot {
            id: id.to_string(),
            path: format!("/path/to/{}.png", id),
            captured_at: timestamp,
            analysis: AnalysisResult {
                activity: activity.to_string(),
                application: app.to_string(),
                description: "test".to_string(),
                category: ActivityCategory::Work,
            },
        }
    }

    #[test]
    fn test_single_group() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let screenshots = vec![
            create_test_screenshot("s1", 1000, "VSCode", "编写代码"),
            create_test_screenshot("s2", 1100, "VSCode", "编写代码"),
            create_test_screenshot("s3", 1200, "VSCode", "编写代码"),
        ];

        let groups = grouper.group_screenshots(&screenshots).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].screenshot_ids.len(), 3);
    }

    #[test]
    fn test_app_change_splits_group() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let screenshots = vec![
            create_test_screenshot("s1", 1000, "VSCode", "编写代码"),
            create_test_screenshot("s2", 1100, "VSCode", "编写代码"),
            create_test_screenshot("s3", 1200, "Chrome", "浏览网页"), // 应用切换
            create_test_screenshot("s4", 1300, "Chrome", "浏览网页"),
        ];

        let groups = grouper.group_screenshots(&screenshots).unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_time_gap_splits_group() {
        let config = GroupingConfig {
            max_gap_seconds: 300,
            ..Default::default()
        };
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let screenshots = vec![
            create_test_screenshot("s1", 1000, "VSCode", "编写代码"),
            create_test_screenshot("s2", 1100, "VSCode", "编写代码"),
            create_test_screenshot("s3", 1500, "VSCode", "编写代码"), // 400秒间隔 > 300
            create_test_screenshot("s4", 1600, "VSCode", "编写代码"),
        ];

        let groups = grouper.group_screenshots(&screenshots).unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_min_screenshots_filter() {
        let config = GroupingConfig {
            min_screenshots: 3,
            ..Default::default()
        };
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let screenshots = vec![
            create_test_screenshot("s1", 1000, "VSCode", "编写代码"),
            create_test_screenshot("s2", 1100, "VSCode", "编写代码"), // 只有2个,应被过滤
        ];

        let groups = grouper.group_screenshots(&screenshots).unwrap();
        assert_eq!(groups.len(), 0); // 不满足最小截图数
    }

    #[test]
    fn test_activities_are_related() {
        assert!(activities_are_related("编写代码", "编写测试"));
        assert!(activities_are_related("查看文档", "查看代码"));
        assert!(!activities_are_related("编写代码", "浏览网页"));
    }

    #[test]
    fn test_max_duration_splits() {
        let config = GroupingConfig {
            max_duration_seconds: 180, // 3分钟
            ..Default::default()
        };
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let screenshots = vec![
            create_test_screenshot("s1", 1000, "VSCode", "编写代码"),
            create_test_screenshot("s2", 1060, "VSCode", "编写代码"),
            create_test_screenshot("s3", 1120, "VSCode", "编写代码"),
            create_test_screenshot("s4", 1200, "VSCode", "编写代码"), // 总时长200秒 > 180
            create_test_screenshot("s5", 1260, "VSCode", "编写代码"), // 第二组也有2张
        ];

        let groups = grouper.group_screenshots(&screenshots).unwrap();
        assert_eq!(groups.len(), 2); // 应该被拆分为两组
        assert!(groups[0].screenshot_ids.len() >= 2); // 第一组至少2张
        assert!(groups[1].screenshot_ids.len() >= 2); // 第二组至少2张
    }
}
