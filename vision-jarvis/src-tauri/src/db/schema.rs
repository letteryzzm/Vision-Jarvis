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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub memory_enabled: bool,
    pub capture_interval_seconds: u8, // 1-15
    pub storage_path: String,
    pub storage_limit_mb: u64,
    pub auto_start: bool,
    pub app_launch_text: String,
    pub timed_reminder_enabled: bool,
    pub timed_reminder_start: String, // "09:00"
    pub timed_reminder_end: String,   // "21:00"
    pub timed_reminder_interval_minutes: u16,
    pub inactivity_reminder_enabled: bool,
    pub inactivity_threshold_minutes: u16,
    pub openai_api_key: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            memory_enabled: true,
            capture_interval_seconds: 5,
            storage_path: String::from("./screenshots"),
            storage_limit_mb: 1024, // 1GB
            auto_start: false,
            app_launch_text: String::from(
                "If today were the last day of my life, would I want to do what I am about to do today?"
            ),
            timed_reminder_enabled: false,
            timed_reminder_start: String::from("09:00"),
            timed_reminder_end: String::from("21:00"),
            timed_reminder_interval_minutes: 30,
            inactivity_reminder_enabled: false,
            inactivity_threshold_minutes: 10,
            openai_api_key: None,
        }
    }
}
