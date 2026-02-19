/// 活动分组器 - 将连续的相似录制分段聚合为活动会话
///
/// 核心算法：
/// 1. 按时间排序已分析的录制分段
/// 2. 根据应用名称、活动类型、时间间隔判断是否属于同一活动
/// 3. 生成ActivitySession列表
///
/// V5: 数据源从 screenshots 切换到 recordings，所有字段直接从 screenshot_analyses 读取

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{Database, schema::{ActivitySession, ScreenshotAnalysisSummary, ActivityCategory}};

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

/// 分析后的录制分段信息（V5: 直接从 screenshot_analyses 表构建）
#[derive(Debug, Clone)]
pub struct AnalyzedRecording {
    pub id: String,
    pub path: String,
    pub start_time: i64,
    pub application: String,
    pub activity_type: String,
    pub activity_description: String,
    pub activity_category: String,
    pub activity_summary: String,
    pub key_elements: Vec<String>,
    pub context_tags: Vec<String>,
    #[allow(dead_code)]
    pub productivity_score: i32,
    #[allow(dead_code)]
    pub project_name: Option<String>,
}

/// 临时活动组(用于构建过程)
#[derive(Debug)]
struct ActivityGroup {
    recordings: Vec<AnalyzedRecording>,
    application: String,
    activity_description: String,
    category: ActivityCategory,
    start_time: i64,
    last_time: i64,
    merged_tags: Vec<String>,
    merged_key_elements: Vec<String>,
    activity_type: String,
}

impl ActivityGroup {
    fn new(recording: AnalyzedRecording) -> Self {
        let application = recording.application.clone();
        let activity_description = recording.activity_description.clone();
        let category = parse_activity_category(&recording.activity_category);
        let timestamp = recording.start_time;
        let merged_tags = recording.context_tags.clone();
        let merged_key_elements = recording.key_elements.clone();
        let activity_type = recording.activity_type.clone();

        Self {
            recordings: vec![recording],
            application,
            activity_description,
            category,
            start_time: timestamp,
            last_time: timestamp,
            merged_tags,
            merged_key_elements,
            activity_type,
        }
    }

    fn add(&mut self, recording: AnalyzedRecording) {
        self.last_time = recording.start_time;
        for tag in &recording.context_tags {
            if !self.merged_tags.contains(tag) {
                self.merged_tags.push(tag.clone());
            }
        }
        for elem in &recording.key_elements {
            if !self.merged_key_elements.contains(elem) {
                self.merged_key_elements.push(elem.clone());
            }
        }
        self.recordings.push(recording);
    }

    fn duration_seconds(&self) -> i64 {
        self.last_time - self.start_time
    }

    fn meets_minimum_criteria(&self, config: &GroupingConfig) -> bool {
        self.recordings.len() >= config.min_screenshots
            && self.duration_seconds() >= config.min_duration_seconds
    }

    /// 转换为ActivitySession
    fn finalize(&self, markdown_path: String) -> ActivitySession {
        let duration_minutes = (self.duration_seconds() / 60).max(1);

        let title = if !self.merged_key_elements.is_empty() {
            let elements_str = self.merged_key_elements.iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("、");
            format!("{}中{} ({})", self.application, self.activity_description, elements_str)
        } else {
            format!("{}中{}", self.application, self.activity_description)
        };

        let screenshot_ids: Vec<String> = self.recordings.iter()
            .map(|r| r.id.clone())
            .collect();

        let screenshot_analyses: Vec<ScreenshotAnalysisSummary> = self.recordings.iter()
            .map(|r| ScreenshotAnalysisSummary {
                id: r.id.clone(),
                timestamp: r.start_time,
                path: r.path.clone(),
                analysis: r.activity_summary.clone(),
            })
            .collect();

        let mut tags = Vec::new();
        tags.push(self.application.clone());
        tags.push(self.activity_description.clone());
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

    /// 获取未分组的已分析录制分段（V5: INNER JOIN screenshot_analyses）
    pub fn get_ungrouped_recordings(&self) -> Result<Vec<AnalyzedRecording>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT r.id, r.path, r.start_time,
                        sa.application, sa.activity_type, sa.activity_description,
                        sa.activity_category, sa.activity_summary,
                        sa.key_elements, sa.context_tags, sa.productivity_score,
                        sa.project_name
                 FROM recordings r
                 INNER JOIN screenshot_analyses sa ON r.id = sa.screenshot_id
                 WHERE r.analyzed = 1
                   AND r.activity_id IS NULL
                 ORDER BY r.start_time ASC"
            )?;

            let recordings = stmt.query_map([], |row| {
                let key_elements_json: String = row.get(8)?;
                let context_tags_json: String = row.get(9)?;

                let key_elements: Vec<String> = serde_json::from_str(&key_elements_json)
                    .unwrap_or_default();
                let context_tags: Vec<String> = serde_json::from_str(&context_tags_json)
                    .unwrap_or_default();

                Ok(AnalyzedRecording {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    start_time: row.get(2)?,
                    application: row.get(3)?,
                    activity_type: row.get(4)?,
                    activity_description: row.get(5)?,
                    activity_category: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "other".to_string()),
                    activity_summary: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                    key_elements,
                    context_tags,
                    productivity_score: row.get(10)?,
                    project_name: row.get(11)?,
                })
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(recordings)
        })
    }

    /// 将录制分段列表分组为活动会话
    pub fn group_recordings(&self, recordings: &[AnalyzedRecording]) -> Result<Vec<ActivitySession>> {
        if recordings.is_empty() {
            return Ok(Vec::new());
        }

        let mut groups = Vec::new();
        let mut current_group: Option<ActivityGroup> = None;
        let mut date_counters: HashMap<String, usize> = HashMap::new();

        for recording in recordings {
            if let Some(ref mut group) = current_group {
                let time_gap = recording.start_time - group.last_time;
                let is_same_app = recording.application == group.application;
                let is_same_activity = recording.activity_description == group.activity_description
                    || activities_are_related(&group.activity_description, &recording.activity_description);
                let current_duration = recording.start_time - group.start_time;

                let activity_type_compatible = recording.activity_type == group.activity_type;

                let tags_overlap = context_tags_overlap(&group.merged_tags, &recording.context_tags);
                let has_tag_affinity = tags_overlap > 0;

                let activity_match = is_same_activity || (has_tag_affinity && is_same_app);
                let should_merge = time_gap <= self.config.max_gap_seconds
                    && is_same_app
                    && activity_match
                    && activity_type_compatible
                    && current_duration <= self.config.max_duration_seconds;

                if should_merge {
                    group.add(recording.clone());
                } else {
                    if group.meets_minimum_criteria(&self.config) {
                        let date_str = format_timestamp_as_date(group.start_time);
                        let counter = date_counters.entry(date_str.clone()).or_insert(0);
                        *counter += 1;
                        let markdown_path = format!("activities/{}/activity-{:03}.md", date_str, counter);
                        groups.push(group.finalize(markdown_path));
                    }

                    current_group = Some(ActivityGroup::new(recording.clone()));
                }
            } else {
                current_group = Some(ActivityGroup::new(recording.clone()));
            }
        }

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

    /// 保存活动会话到数据库并关联录制分段
    pub fn save_activity(&self, activity: &ActivitySession) -> Result<()> {
        self.db.with_connection(|conn| {
            let tx = conn.unchecked_transaction()?;

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

            // 关联录制分段到此活动
            for recording_id in &activity.screenshot_ids {
                tx.execute(
                    "UPDATE recordings SET activity_id = ?1 WHERE id = ?2",
                    rusqlite::params![&activity.id, recording_id],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
    }
}

/// V5: 计算两组 context_tags 的重叠数量
fn context_tags_overlap(a: &[String], b: &[String]) -> usize {
    a.iter().filter(|tag| b.contains(tag)).count()
}

/// V5: 将字符串映射为 ActivityCategory 枚举
fn parse_activity_category(s: &str) -> ActivityCategory {
    match s {
        "work" => ActivityCategory::Work,
        "entertainment" => ActivityCategory::Entertainment,
        "communication" => ActivityCategory::Communication,
        _ => ActivityCategory::Other,
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

    fn create_test_recording(
        id: &str,
        timestamp: i64,
        app: &str,
        activity: &str,
    ) -> AnalyzedRecording {
        create_test_recording_full(id, timestamp, app, activity, vec![], "work", vec![])
    }

    fn create_test_recording_full(
        id: &str,
        timestamp: i64,
        app: &str,
        activity: &str,
        context_tags: Vec<&str>,
        activity_type: &str,
        key_elements: Vec<&str>,
    ) -> AnalyzedRecording {
        AnalyzedRecording {
            id: id.to_string(),
            path: format!("/path/to/{}.mp4", id),
            start_time: timestamp,
            application: app.to_string(),
            activity_type: activity_type.to_string(),
            activity_description: activity.to_string(),
            activity_category: "work".to_string(),
            activity_summary: format!("{}中{}", app, activity),
            key_elements: key_elements.into_iter().map(String::from).collect(),
            context_tags: context_tags.into_iter().map(String::from).collect(),
            productivity_score: 5,
            project_name: None,
        }
    }

    #[test]
    fn test_single_group() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let recordings = vec![
            create_test_recording("s1", 1000, "VSCode", "编写代码"),
            create_test_recording("s2", 1100, "VSCode", "编写代码"),
            create_test_recording("s3", 1200, "VSCode", "编写代码"),
        ];

        let groups = grouper.group_recordings(&recordings).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].screenshot_ids.len(), 3);
    }

    #[test]
    fn test_app_change_splits_group() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let recordings = vec![
            create_test_recording("s1", 1000, "VSCode", "编写代码"),
            create_test_recording("s2", 1100, "VSCode", "编写代码"),
            create_test_recording("s3", 1200, "Chrome", "浏览网页"),
            create_test_recording("s4", 1300, "Chrome", "浏览网页"),
        ];

        let groups = grouper.group_recordings(&recordings).unwrap();
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

        let recordings = vec![
            create_test_recording("s1", 1000, "VSCode", "编写代码"),
            create_test_recording("s2", 1100, "VSCode", "编写代码"),
            create_test_recording("s3", 1500, "VSCode", "编写代码"),
            create_test_recording("s4", 1600, "VSCode", "编写代码"),
        ];

        let groups = grouper.group_recordings(&recordings).unwrap();
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

        let recordings = vec![
            create_test_recording("s1", 1000, "VSCode", "编写代码"),
            create_test_recording("s2", 1100, "VSCode", "编写代码"),
        ];

        let groups = grouper.group_recordings(&recordings).unwrap();
        assert_eq!(groups.len(), 0);
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
            max_duration_seconds: 180,
            ..Default::default()
        };
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let recordings = vec![
            create_test_recording("s1", 1000, "VSCode", "编写代码"),
            create_test_recording("s2", 1060, "VSCode", "编写代码"),
            create_test_recording("s3", 1120, "VSCode", "编写代码"),
            create_test_recording("s4", 1200, "VSCode", "编写代码"),
            create_test_recording("s5", 1260, "VSCode", "编写代码"),
        ];

        let groups = grouper.group_recordings(&recordings).unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups[0].screenshot_ids.len() >= 2);
        assert!(groups[1].screenshot_ids.len() >= 2);
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

        let recordings = vec![
            create_test_recording_full("s1", 1000, "Chrome", "浏览", vec![], "work", vec![]),
            create_test_recording_full("s2", 1100, "Chrome", "浏览", vec![], "work", vec![]),
            create_test_recording_full("s3", 1200, "Chrome", "浏览", vec![], "entertainment", vec![]),
            create_test_recording_full("s4", 1300, "Chrome", "浏览", vec![], "entertainment", vec![]),
        ];

        let groups = grouper.group_recordings(&recordings).unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_tag_affinity_merges_different_activities() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let recordings = vec![
            create_test_recording_full("s1", 1000, "VSCode", "编写代码", vec!["rust", "vision-jarvis"], "work", vec![]),
            create_test_recording_full("s2", 1100, "VSCode", "调试程序", vec!["rust", "vision-jarvis"], "work", vec![]),
            create_test_recording_full("s3", 1200, "VSCode", "查看日志", vec!["rust", "vision-jarvis"], "work", vec![]),
        ];

        let groups = grouper.group_recordings(&recordings).unwrap();
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_key_elements_in_title() {
        let config = GroupingConfig::default();
        let db = Arc::new(Database::open_in_memory().unwrap());
        let grouper = ActivityGrouper::new(db, config);

        let recordings = vec![
            create_test_recording_full("s1", 1000, "VSCode", "编写代码", vec![], "work", vec!["pipeline.rs"]),
            create_test_recording_full("s2", 1100, "VSCode", "编写代码", vec![], "work", vec!["pipeline.rs", "mod.rs"]),
        ];

        let groups = grouper.group_recordings(&recordings).unwrap();
        assert_eq!(groups.len(), 1);
        assert!(groups[0].title.contains("pipeline.rs"));
    }
}
