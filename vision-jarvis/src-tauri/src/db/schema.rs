/// 数据库表结构定义
///
/// 表说明：
/// - screenshots: 截图元数据和 AI 分析结果
/// - short_term_memories: 短期记忆（活动事项）
/// - long_term_memories: 长期记忆（日期范围总结）
/// - settings: 应用配置

use serde::{Deserialize, Serialize};

/// 截图记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub id: String,
    pub path: String,
    pub captured_at: i64, // Unix timestamp
    pub analyzed: bool,
    pub analysis_result: Option<String>, // JSON
    pub embedding: Option<Vec<u8>>, // 向量嵌入（序列化为 BLOB）
}

/// AI 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub activity: String,          // 主要活动
    pub application: String,        // 使用的应用
    pub description: String,        // 简要描述
    pub category: ActivityCategory, // 分类
}

/// 活动分类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityCategory {
    Work,
    Entertainment,
    Communication,
    Other,
}

/// 短期记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTermMemory {
    pub id: String,
    pub date: String,             // YYYY-MM-DD
    pub time_start: String,       // HH:MM
    pub time_end: String,         // HH:MM
    pub period: Period,           // 时段
    pub activity: String,         // 活动名称
    pub summary: Option<String>,  // AI 生成的总结
    pub screenshot_ids: Vec<String>, // 关联的截图 ID
    pub created_at: i64,          // Unix timestamp
}

/// 时段分类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    Morning,   // 00:00-12:00
    Afternoon, // 12:00-18:00
    Evening,   // 18:00-24:00
}

/// 长期记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermMemory {
    pub id: String,
    pub date_start: String,       // YYYY-MM-DD
    pub date_end: String,         // YYYY-MM-DD
    pub summary: String,          // AI 生成的总结
    pub main_activities: Vec<MainActivity>, // 主要活动列表
    pub created_at: i64,          // Unix timestamp
}

/// 主要活动概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainActivity {
    pub date: String,
    pub activity: String,
    pub duration: String,
}

/// 应用设置
/// NOTE: 与 settings/config.rs 中的 AppSettings 保持同步
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub memory_enabled: bool,
    pub capture_interval_seconds: u8,
    pub storage_path: String,
    pub storage_limit_mb: u64,
    pub auto_start: bool,
    pub app_launch_text: String,
    // 固定提醒
    pub morning_reminder_enabled: bool,
    pub morning_reminder_time: String,
    pub morning_reminder_message: String,
    pub water_reminder_enabled: bool,
    pub water_reminder_start: String,
    pub water_reminder_end: String,
    pub water_reminder_interval_minutes: u16,
    pub water_reminder_message: String,
    pub sedentary_reminder_enabled: bool,
    pub sedentary_reminder_start: String,
    pub sedentary_reminder_end: String,
    pub sedentary_reminder_threshold_minutes: u16,
    pub sedentary_reminder_message: String,
    // 智能提醒
    pub screen_inactivity_reminder_enabled: bool,
    pub screen_inactivity_minutes: u16,
    pub screen_inactivity_message: String,
    pub openai_api_key: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            memory_enabled: true,
            capture_interval_seconds: 5,
            storage_path: String::from("./screenshots"),
            storage_limit_mb: 1024,
            auto_start: false,
            app_launch_text: String::from(
                "If today were the last day of my life, would I want to do what I am about to do today?"
            ),
            morning_reminder_enabled: false,
            morning_reminder_time: String::from("08:00"),
            morning_reminder_message: String::from(
                "If today is the last day of my life, would I want to do what I am about to do today?"
            ),
            water_reminder_enabled: false,
            water_reminder_start: String::from("09:00"),
            water_reminder_end: String::from("21:00"),
            water_reminder_interval_minutes: 60,
            water_reminder_message: String::from("该喝喝水了"),
            sedentary_reminder_enabled: false,
            sedentary_reminder_start: String::from("09:00"),
            sedentary_reminder_end: String::from("21:00"),
            sedentary_reminder_threshold_minutes: 60,
            sedentary_reminder_message: String::from(
                "你已经连续工作很久了，再厉害的人也需要休息放松，是时候站起来走动走了"
            ),
            screen_inactivity_reminder_enabled: false,
            screen_inactivity_minutes: 10,
            screen_inactivity_message: String::new(),
            openai_api_key: None,
        }
    }
}

// ============================================================================
// V2 Schema: Activity-Driven Memory System
// ============================================================================

/// 活动会话 - 一段时间内用户的连贯活动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySession {
    pub id: String,                          // "activity-2024-01-15-001"
    pub title: String,                       // AI生成的标题
    pub start_time: i64,                     // Unix时间戳
    pub end_time: i64,                       // Unix时间戳
    pub duration_minutes: i64,               // 计算得出
    pub application: String,                 // 主要应用程序
    pub category: ActivityCategory,          // 活动分类
    pub screenshot_ids: Vec<String>,         // 关联的截图ID列表
    pub screenshot_analyses: Vec<ScreenshotAnalysisSummary>, // 截图分析摘要
    pub tags: Vec<String>,                   // AI生成的标签
    pub markdown_path: String,               // 相对路径
    pub summary: Option<String>,             // AI生成的总结
    pub indexed: bool,                       // 是否已索引到向量数据库
    pub created_at: i64,                     // 创建时间
}

/// 截图分析摘要 - 用于Activity的frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotAnalysisSummary {
    pub id: String,
    pub timestamp: i64,
    pub path: String,                        // 相对路径
    pub analysis: String,                    // 一行描述
}

/// 记忆分块 - 用于向量索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChunk {
    pub id: String,                          // UUID
    pub file_path: String,                   // 源文件相对路径
    pub source: MemorySource,                // 来源类型
    pub start_line: i32,                     // 分块起始行号
    pub end_line: i32,                       // 分块结束行号
    pub hash: String,                        // SHA-256哈希(前16字符)
    pub model: String,                       // Embedding模型
    pub text: String,                        // 分块文本内容
    pub embedding: Vec<f32>,                 // 向量embedding
    pub activity_id: Option<String>,         // 关联的activity ID
    pub updated_at: i64,                     // 更新时间
}

/// 分块元数据 - JSON序列化后存储在chunks表的metadata列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub application: Option<String>,
    pub activity: Option<String>,
    pub category: Option<String>,
    pub timestamp: Option<i64>,
    pub date: Option<String>,
}

/// 记忆来源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MemorySource {
    Activity,  // 来自活动Markdown
    Summary,   // 来自周/月总结
}

impl MemorySource {
    pub fn as_str(&self) -> &str {
        match self {
            MemorySource::Activity => "activity",
            MemorySource::Summary => "summary",
        }
    }
}

/// 文件追踪记录 - 用于增量索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedFile {
    pub path: String,                        // 相对路径
    pub source: MemorySource,                // 文件类型
    pub hash: String,                        // 文件内容哈希
    pub mtime: i64,                          // 最后修改时间
    pub size: i64,                           // 文件大小(字节)
}

/// Embedding缓存条目
#[derive(Debug, Clone)]
pub struct EmbeddingCacheEntry {
    pub provider: String,                    // "openai"
    pub model: String,                       // "text-embedding-3-small"
    pub hash: String,                        // 文本哈希
    pub embedding: Vec<u8>,                  // 序列化的向量
    pub dims: i32,                           // 向量维度
    pub created_at: i64,                     // 缓存时间
}

// ============================================================================
// V3 Schema: Proactive AI Memory System
// ============================================================================

/// 截图AI分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotAnalysis {
    pub screenshot_id: String,
    pub application: String,
    pub activity_type: String,              // work, entertainment, communication, other
    pub activity_description: String,        // 一句话描述
    pub key_elements: Vec<String>,          // 关键元素列表
    pub ocr_text: Option<String>,           // OCR提取的文本
    pub context_tags: Vec<String>,          // 上下文标签
    pub productivity_score: i32,            // 1-10
    pub analysis_json: String,              // 完整JSON
    pub analyzed_at: i64,
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
    pub confidence: f32,                    // 0.0-1.0
    pub frequency: String,                  // daily, weekly, etc.
    pub trigger_conditions: Option<String>, // JSON
    pub typical_time: Option<String>,       // HH:MM
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
    TimeBased,      // 时间模式
    TriggerBased,   // 触发模式
    SequenceBased,  // 序列模式
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
    pub date_start: String,                 // YYYY-MM-DD
    pub date_end: String,                   // YYYY-MM-DD
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

/// 主动建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveSuggestion {
    pub id: String,
    pub suggestion_type: SuggestionType,
    pub trigger_context: String,            // JSON
    pub message: String,
    pub priority: i32,                      // 0-10
    pub delivered: bool,
    pub delivered_at: Option<i64>,
    pub user_action: Option<UserAction>,
    pub created_at: i64,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SuggestionType {
    HabitReminder,
    ContextSwitch,
    BreakReminder,
    ProjectProgress,
}

impl SuggestionType {
    pub fn as_str(&self) -> &str {
        match self {
            SuggestionType::HabitReminder => "habit-reminder",
            SuggestionType::ContextSwitch => "context-switch",
            SuggestionType::BreakReminder => "break-reminder",
            SuggestionType::ProjectProgress => "project-progress",
        }
    }
}

/// 用户对建议的操作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserAction {
    Dismissed,
    Accepted,
    Snoozed,
}

impl UserAction {
    pub fn as_str(&self) -> &str {
        match self {
            UserAction::Dismissed => "dismissed",
            UserAction::Accepted => "accepted",
            UserAction::Snoozed => "snoozed",
        }
    }
}
