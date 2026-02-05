/// 应用配置定义

use serde::{Deserialize, Serialize};

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// 是否启用记忆功能
    pub memory_enabled: bool,

    /// 截图间隔（秒）: 1-15
    pub capture_interval_seconds: u8,

    /// 存储路径
    pub storage_path: String,

    /// 存储容量限制（MB）
    pub storage_limit_mb: u64,

    /// 是否开机自启动
    pub auto_start: bool,

    /// 应用启动时显示的文本
    pub app_launch_text: String,

    /// 是否启用定时提醒
    pub timed_reminder_enabled: bool,

    /// 定时提醒开始时间 (HH:MM)
    pub timed_reminder_start: String,

    /// 定时提醒结束时间 (HH:MM)
    pub timed_reminder_end: String,

    /// 定时提醒间隔（分钟）
    pub timed_reminder_interval_minutes: u16,

    /// 是否启用不活动提醒
    pub inactivity_reminder_enabled: bool,

    /// 不活动阈值（分钟）
    pub inactivity_threshold_minutes: u16,

    /// OpenAI API 密钥
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();

        assert!(settings.memory_enabled);
        assert_eq!(settings.capture_interval_seconds, 5);
        assert_eq!(settings.storage_path, "./screenshots");
        assert_eq!(settings.storage_limit_mb, 1024);
        assert!(!settings.auto_start);
        assert!(!settings.timed_reminder_enabled);
        assert!(!settings.inactivity_reminder_enabled);
        assert!(settings.openai_api_key.is_none());
    }

    #[test]
    fn test_clone_settings() {
        let settings1 = AppSettings::default();
        let settings2 = settings1.clone();

        assert_eq!(settings1, settings2);
    }

    #[test]
    fn test_serialize_deserialize() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, deserialized);
    }
}
