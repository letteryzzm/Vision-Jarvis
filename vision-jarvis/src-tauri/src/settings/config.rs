/// 应用配置定义

use serde::{Deserialize, Serialize};

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppSettings {
    /// 是否启用记忆功能
    pub memory_enabled: bool,

    /// 录制分段时长（秒）: 30-300
    pub capture_interval_seconds: u16,

    /// 存储路径
    pub storage_path: String,

    /// 存储容量限制（MB）
    pub storage_limit_mb: u64,

    /// 是否开机自启动
    pub auto_start: bool,

    /// 应用启动时显示的文本
    pub app_launch_text: String,

    // ========== 固定提醒 ==========

    /// 早安提醒：是否启用
    pub morning_reminder_enabled: bool,
    /// 早安提醒：触发时间 (HH:MM)
    pub morning_reminder_time: String,
    /// 早安提醒：自定义消息
    pub morning_reminder_message: String,

    /// 喝水提醒：是否启用
    pub water_reminder_enabled: bool,
    /// 喝水提醒：开始时间 (HH:MM)
    pub water_reminder_start: String,
    /// 喝水提醒：结束时间 (HH:MM)
    pub water_reminder_end: String,
    /// 喝水提醒：间隔（分钟）
    pub water_reminder_interval_minutes: u16,
    /// 喝水提醒：自定义消息
    pub water_reminder_message: String,

    /// 久坐提醒：是否启用
    pub sedentary_reminder_enabled: bool,
    /// 久坐提醒：开始时间 (HH:MM)
    pub sedentary_reminder_start: String,
    /// 久坐提醒：结束时间 (HH:MM)
    pub sedentary_reminder_end: String,
    /// 久坐提醒：连续工作阈值（分钟）
    pub sedentary_reminder_threshold_minutes: u16,
    /// 久坐提醒：自定义消息
    pub sedentary_reminder_message: String,

    // ========== 智能提醒 ==========

    /// 屏幕无变化提醒：是否启用
    pub screen_inactivity_reminder_enabled: bool,
    /// 屏幕无变化提醒：无操作阈值（分钟）
    pub screen_inactivity_minutes: u16,
    /// 屏幕无变化提醒：自定义消息（空字符串表示使用 AI 智能建议）
    pub screen_inactivity_message: String,

    /// OpenAI API 密钥（已废弃，使用 AI 配置系统）
    pub openai_api_key: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let default_storage_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vision-jarvis")
            .join("screenshots")
            .to_string_lossy()
            .to_string();

        Self {
            memory_enabled: true,
            capture_interval_seconds: 60,
            storage_path: default_storage_path,
            storage_limit_mb: 1024,
            auto_start: false,
            app_launch_text: String::from(
                "If today were the last day of my life, would I want to do what I am about to do today?"
            ),

            // 早安提醒
            morning_reminder_enabled: false,
            morning_reminder_time: String::from("08:00"),
            morning_reminder_message: String::from(
                "If today is the last day of my life, would I want to do what I am about to do today?"
            ),

            // 喝水提醒
            water_reminder_enabled: false,
            water_reminder_start: String::from("09:00"),
            water_reminder_end: String::from("21:00"),
            water_reminder_interval_minutes: 60,
            water_reminder_message: String::from("该喝喝水了"),

            // 久坐提醒
            sedentary_reminder_enabled: false,
            sedentary_reminder_start: String::from("09:00"),
            sedentary_reminder_end: String::from("21:00"),
            sedentary_reminder_threshold_minutes: 60,
            sedentary_reminder_message: String::from(
                "你已经连续工作很久了，再厉害的人也需要休息放松，是时候站起来走动走了"
            ),

            // 屏幕无变化提醒
            screen_inactivity_reminder_enabled: false,
            screen_inactivity_minutes: 10,
            screen_inactivity_message: String::new(), // 空 = AI 智能建议

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
        assert_eq!(settings.capture_interval_seconds, 60);
        assert!(settings.storage_path.contains("vision-jarvis"));
        assert!(settings.storage_path.contains("screenshots"));
        assert_eq!(settings.storage_limit_mb, 1024);
        assert!(!settings.auto_start);

        // 提醒默认关闭
        assert!(!settings.morning_reminder_enabled);
        assert!(!settings.water_reminder_enabled);
        assert!(!settings.sedentary_reminder_enabled);
        assert!(!settings.screen_inactivity_reminder_enabled);

        assert_eq!(settings.morning_reminder_time, "08:00");
        assert_eq!(settings.water_reminder_interval_minutes, 60);
        assert_eq!(settings.sedentary_reminder_threshold_minutes, 60);
        assert_eq!(settings.screen_inactivity_minutes, 10);
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

    #[test]
    fn test_backward_compatible_deserialize() {
        // 旧版 JSON 缺少新字段时应使用默认值
        let old_json = r#"{
            "memory_enabled": true,
            "capture_interval_seconds": 5,
            "storage_path": "/tmp/test",
            "storage_limit_mb": 1024,
            "auto_start": false,
            "app_launch_text": "hello",
            "openai_api_key": null
        }"#;

        let settings: AppSettings = serde_json::from_str(old_json).unwrap();
        assert!(!settings.morning_reminder_enabled);
        assert_eq!(settings.morning_reminder_time, "08:00");
        assert!(!settings.water_reminder_enabled);
        assert_eq!(settings.water_reminder_interval_minutes, 60);
        assert!(!settings.screen_inactivity_reminder_enabled);
    }
}
