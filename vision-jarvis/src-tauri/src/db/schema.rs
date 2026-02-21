/// 数据库表结构定义
///
/// 表说明：
/// - recordings: 屏幕录制分段元数据
/// - screenshot_analyses: 录制分段AI分析结果（V5一次性提取）
/// - activities: 活动会话（由录制分段聚合生成）
/// - projects: 自动识别的项目
/// - habits: 习惯模式
/// - summaries: 日/周/月总结
/// - memory_files / memory_chunks: 向量索引
/// - settings: 应用配置

use serde::{Deserialize, Serialize};

/// 活动分类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityCategory {
    Work,
    Entertainment,
    Communication,
    Other,
}

/// 活动会话 - 一段时间内用户的连贯活动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySession {
    pub id: String,
    pub title: String,
    pub start_time: i64,
    pub end_time: i64,
    pub duration_minutes: i64,
    pub application: String,
    pub category: ActivityCategory,
    pub screenshot_ids: Vec<String>,
    pub screenshot_analyses: Vec<ScreenshotAnalysisSummary>,
    pub tags: Vec<String>,
    pub markdown_path: String,
    pub summary: Option<String>,
    pub indexed: bool,
    pub created_at: i64,
}

/// 截图分析摘要 - 用于Activity的frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotAnalysisSummary {
    pub id: String,
    pub timestamp: i64,
    pub path: String,
    pub analysis: String,
}

/// 截图AI分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotAnalysis {
    pub screenshot_id: String,
    pub application: String,
    pub activity_type: String,
    pub activity_description: String,
    pub key_elements: Vec<String>,
    pub ocr_text: Option<String>,
    pub context_tags: Vec<String>,
    pub productivity_score: i32,
    pub analysis_json: String,
    pub analyzed_at: i64,
    pub activity_category: String,
    pub activity_summary: String,
    pub project_name: Option<String>,
    pub accomplishments: Vec<String>,
}

/// 项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_date: i64,
    pub last_activity_date: i64,
    pub activity_count: i32,
    pub tags: Vec<String>,
    pub status: ProjectStatus,
    pub markdown_path: String,
    pub created_at: i64,
}

/// 项目状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Paused,
    Completed,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Paused => "paused",
            ProjectStatus::Completed => "completed",
        }
    }
}

/// 习惯模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: String,
    pub pattern_name: String,
    pub pattern_type: HabitPatternType,
    pub confidence: f32,
    pub frequency: String,
    pub trigger_conditions: Option<String>,
    pub typical_time: Option<String>,
    pub last_occurrence: Option<i64>,
    pub occurrence_count: i32,
    pub markdown_path: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 习惯模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HabitPatternType {
    TimeBased,
    TriggerBased,
    SequenceBased,
}

impl HabitPatternType {
    pub fn as_str(&self) -> &str {
        match self {
            HabitPatternType::TimeBased => "time-based",
            HabitPatternType::TriggerBased => "trigger-based",
            HabitPatternType::SequenceBased => "sequence-based",
        }
    }
}

/// 总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub summary_type: SummaryType,
    pub date_start: String,
    pub date_end: String,
    pub content: String,
    pub activity_ids: Vec<String>,
    pub project_ids: Option<Vec<String>>,
    pub markdown_path: String,
    pub created_at: i64,
}

/// 总结类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SummaryType {
    Daily,
    Weekly,
    Monthly,
}

impl SummaryType {
    pub fn as_str(&self) -> &str {
        match self {
            SummaryType::Daily => "daily",
            SummaryType::Weekly => "weekly",
            SummaryType::Monthly => "monthly",
        }
    }
}
