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
    /// V3: 来自 screenshot_analyses 表的增强数据
    pub context_tags: Vec<String>,
    pub activity_type: Option<String>,
    pub key_elements: Vec<String>,
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
    /// V3: 聚合的上下文标签
    merged_tags: Vec<String>,
    /// V3: 聚合的关键元素
    merged_key_elements: Vec<String>,
    /// V3: 活动类型（work/entertainment/communication/other）
    activity_type: Option<String>,
}

impl ActivityGroup {
    fn new(screenshot: AnalyzedScreenshot) -> Self {
        let application = screenshot.analysis.application.clone();
        let activity = screenshot.analysis.activity.clone();
        let category = screenshot.analysis.category.clone();
        let timestamp = screenshot.captured_at;
        let merged_tags = screenshot.context_tags.clone();
        let merged_key_elements = screenshot.key_elements.clone();
        let activity_type = screenshot.activity_type.clone();

        Self {
            screenshots: vec![screenshot],
            application,
            activity,
            category,
            start_time: timestamp,
            last_time: timestamp,
            merged_tags,
            merged_key_elements,
            activity_type,
        }
    }

    fn add(&mut self, screenshot: AnalyzedScreenshot) {
        self.last_time = screenshot.captured_at;
        // 合并 context_tags（去重）
        for tag in &screenshot.context_tags {
            if !self.merged_tags.contains(tag) {
                self.merged_tags.push(tag.clone());
            }
        }
        // 合并 key_elements（去重）
        for elem in &screenshot.key_elements {
            if !self.merged_key_elements.contains(elem) {
                self.merged_key_elements.push(elem.clone());
            }
        }
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

        // V3: 利用 key_elements 生成更好的标题
        let title = if !self.merged_key_elements.is_empty() {
            let elements_str = self.merged_key_elements.iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("、");
            format!("{}中{} ({})", self.application, self.activity, elements_str)
        } else {
            format!("{}中{}", self.application, self.activity)
        };

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

        // V3: 合并 context_tags 到 tags
        let mut tags = Vec::new();
        tags.push(self.application.clone());
        tags.push(self.activity.clone());
        for tag in &self.merged_tags {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }

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
            summary: None,
            indexed: false,
            created_at: Utc::now().timestamp(),
        }
    }
}

impl ActivityGrouper {
    pub fn new(db: Arc<Database>, config: GroupingConfig) -> Self {
        Self { db, config }
    }

    /// 获取未分组的已分析截图（V3: LEFT JOIN screenshot_analyses 获取增强数据）
    pub fn get_ungrouped_screenshots(&self) -> Result<Vec<AnalyzedScreenshot>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT s.id, s.path, s.captured_at, s.analysis_result,
                        sa.context_tags, sa.activity_type, sa.key_elements
                 FROM screenshots s
                 LEFT JOIN screenshot_analyses sa ON s.id = sa.screenshot_id
                 WHERE s.analyzed = 1
                   AND s.analysis_result IS NOT NULL
                   AND s.activity_id IS NULL
                 ORDER BY s.captured_at ASC"
            )?;

            let screenshots = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(1)?;
                let captured_at: i64 = row.get(2)?;
                let analysis_json: String = row.get(3)?;
                let context_tags_json: Option<String> = row.get(4)?;
                let activity_type: Option<String> = row.get(5)?;
                let key_elements_json: Option<String> = row.get(6)?;

                let analysis: AnalysisResult = match serde_json::from_str(&analysis_json) {
                    Ok(a) => a,
                    Err(e) => {
                        log::warn!(
                            "Skipping screenshot {} - failed to parse analysis_result: {}",
                            id, e
                        );
                        AnalysisResult {
                            activity: String::new(),
                            application: String::new(),
                            description: String::new(),
                            category: ActivityCategory::Other,
                        }
                    }
                };

                // 解析 V3 JSON 数组字段
                let context_tags: Vec<String> = context_tags_json
                    .and_then(|j| serde_json::from_str(&j).ok())
                    .unwrap_or_default();
                let key_elements: Vec<String> = key_elements_json
                    .and_then(|j| serde_json::from_str(&j).ok())
                    .unwrap_or_default();

                Ok(AnalyzedScreenshot {
                    id,
                    path,
                    captured_at,
                    analysis,
                    context_tags,
                    activity_type,
                    key_elements,
                })
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

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

                // V3: activity_type 不同则不合并（如 work vs entertainment）
                let activity_type_compatible = match (&group.activity_type, &screenshot.activity_type) {
                    (Some(a), Some(b)) => a == b,
                    _ => true, // 缺少类型信息时不阻止合并
                };

                // V3: context_tags 重叠度检查（有共同标签更容易合并）
                let tags_overlap = context_tags_overlap(&group.merged_tags, &screenshot.context_tags);
                let has_tag_affinity = tags_overlap > 0;

                // 判断是否应该合并到当前组
                // V3: 同类型 + (同活动 或 标签亲和) 即可合并
                let activity_match = is_same_activity || (has_tag_affinity && is_same_app);
                let should_merge = time_gap <= self.config.max_gap_seconds
                    && is_same_app
                    && activity_match
                    && activity_type_compatible
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

/// V3: 计算两组 context_tags 的重叠数量
fn context_tags_overlap(a: &[String], b: &[String]) -> usize {
    a.iter().filter(|tag| b.contains(tag)).count()
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
        create_test_screenshot_v3(id, timestamp, app, activity, vec![], None, vec![])
    }

    fn create_test_screenshot_v3(
        id: &str,
        timestamp: i64,
        app: &str,
        activity: &str,
        context_tags: Vec<&str>,
        activity_type: Option<&str>,
        key_elements: Vec<&str>,
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
            context_tags: context_tags.into_iter().map(String::from).collect(),
            activity_type: activity_type.map(String::from),
            key_elements: key_elements.into_iter().map(String::from).collect(),
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

    #[test]
    fn test_context_tags_overlap() {
        assert_eq!(context_tags_overlap(&[], &[]), 0);
        let a = vec!["rust".to_string(), "coding".to_string()];
        let b = vec!["coding".to_string(), "debug".to_string()];
        assert_eq!(context_tags_overlap(&a, &b), 1);
    }

    #[test]
    fn test_activity_type_incompatible_splits() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        // 同应用同活动，但 activity_type 不同 → 应拆分
        let screenshots = vec![
            create_test_screenshot_v3("s1", 1000, "Chrome", "浏览", vec![], Some("work"), vec![]),
            create_test_screenshot_v3("s2", 1100, "Chrome", "浏览", vec![], Some("work"), vec![]),
            create_test_screenshot_v3("s3", 1200, "Chrome", "浏览", vec![], Some("entertainment"), vec![]),
            create_test_screenshot_v3("s4", 1300, "Chrome", "浏览", vec![], Some("entertainment"), vec![]),
        ];

        let groups = grouper.group_screenshots(&screenshots).unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_tag_affinity_merges_different_activities() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        // 不同活动名但有共同 context_tags → 应合并
        let screenshots = vec![
            create_test_screenshot_v3("s1", 1000, "VSCode", "编写代码", vec!["rust", "vision-jarvis"], Some("work"), vec![]),
            create_test_screenshot_v3("s2", 1100, "VSCode", "调试程序", vec!["rust", "vision-jarvis"], Some("work"), vec![]),
            create_test_screenshot_v3("s3", 1200, "VSCode", "查看日志", vec!["rust", "vision-jarvis"], Some("work"), vec![]),
        ];

        let groups = grouper.group_screenshots(&screenshots).unwrap();
        assert_eq!(groups.len(), 1); // 共同标签使不同活动合并
    }

    #[test]
    fn test_key_elements_in_title() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let screenshots = vec![
            create_test_screenshot_v3("s1", 1000, "VSCode", "编写代码", vec![], None, vec!["pipeline.rs"]),
            create_test_screenshot_v3("s2", 1100, "VSCode", "编写代码", vec![], None, vec!["pipeline.rs", "mod.rs"]),
        ];

        let groups = grouper.group_screenshots(&screenshots).unwrap();
        assert_eq!(groups.len(), 1);
        assert!(groups[0].title.contains("pipeline.rs"));
    }
}
