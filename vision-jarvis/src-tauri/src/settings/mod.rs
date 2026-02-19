/// 应用设置管理模块
///
/// 使用 tauri-plugin-store 进行持久化存储

use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub mod config;
pub use config::AppSettings;

/// 设置管理器
pub struct SettingsManager {
    settings: Arc<Mutex<AppSettings>>,
}

impl SettingsManager {
    /// 创建新的设置管理器
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(AppSettings::default())),
        }
    }

    /// 获取当前设置的副本
    pub fn get(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    /// 更新设置
    pub fn update(&self, new_settings: AppSettings) -> AppResult<()> {
        self.validate_settings(&new_settings)?;
        let mut settings = self.settings.lock().unwrap();
        *settings = new_settings;
        Ok(())
    }

    /// 验证设置
    fn validate_settings(&self, settings: &AppSettings) -> AppResult<()> {
        // 验证录制分段时长
        if settings.capture_interval_seconds < 30 || settings.capture_interval_seconds > 300 {
            return Err(AppError::validation(1, "录制分段时长必须在 30-300 秒之间"));
        }

        // 验证存储限制
        if settings.storage_limit_mb == 0 {
            return Err(AppError::validation(2, "存储限制必须大于 0"));
        }

        // 早安提醒
        self.validate_time_format(&settings.morning_reminder_time)?;

        // 喝水提醒
        self.validate_time_format(&settings.water_reminder_start)?;
        self.validate_time_format(&settings.water_reminder_end)?;
        if settings.water_reminder_enabled && settings.water_reminder_interval_minutes == 0 {
            return Err(AppError::validation(3, "喝水提醒间隔必须大于 0"));
        }

        // 久坐提醒
        self.validate_time_format(&settings.sedentary_reminder_start)?;
        self.validate_time_format(&settings.sedentary_reminder_end)?;
        if settings.sedentary_reminder_enabled && settings.sedentary_reminder_threshold_minutes == 0 {
            return Err(AppError::validation(4, "久坐提醒阈值必须大于 0"));
        }

        // 屏幕无变化提醒
        if settings.screen_inactivity_reminder_enabled && settings.screen_inactivity_minutes == 0 {
            return Err(AppError::validation(5, "屏幕无变化检测阈值必须大于 0"));
        }

        Ok(())
    }

    /// 验证时间格式 (HH:MM)
    fn validate_time_format(&self, time: &str) -> AppResult<()> {
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 2 {
            return Err(AppError::validation(5, "时间格式必须是 HH:MM"));
        }

        let hour: u8 = parts[0].parse()
            .map_err(|_| AppError::validation(6, "小时必须是有效的数字"))?;
        let minute: u8 = parts[1].parse()
            .map_err(|_| AppError::validation(7, "分钟必须是有效的数字"))?;

        if hour > 23 {
            return Err(AppError::validation(8, "小时必须在 0-23 之间"));
        }

        if minute > 59 {
            return Err(AppError::validation(9, "分钟必须在 0-59 之间"));
        }

        Ok(())
    }

    /// 获取存储路径
    pub fn get_storage_path(&self) -> PathBuf {
        let settings = self.settings.lock().unwrap();
        PathBuf::from(&settings.storage_path)
    }

    /// 是否启用记忆功能
    pub fn is_memory_enabled(&self) -> bool {
        let settings = self.settings.lock().unwrap();
        settings.memory_enabled
    }

    /// 获取录制分段时长（秒）
    pub fn get_capture_interval(&self) -> u16 {
        let settings = self.settings.lock().unwrap();
        settings.capture_interval_seconds
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let manager = SettingsManager::new();
        let settings = manager.get();

        assert_eq!(settings.capture_interval_seconds, 60);
        assert!(settings.memory_enabled);
        assert_eq!(settings.storage_limit_mb, 1024);
    }

    #[test]
    fn test_validate_capture_interval() {
        let manager = SettingsManager::new();
        let mut settings = AppSettings::default();

        settings.capture_interval_seconds = 0;
        assert!(manager.validate_settings(&settings).is_err());

        settings.capture_interval_seconds = 301;
        assert!(manager.validate_settings(&settings).is_err());

        settings.capture_interval_seconds = 60;
        assert!(manager.validate_settings(&settings).is_ok());
    }

    #[test]
    fn test_validate_reminder_settings() {
        let manager = SettingsManager::new();
        let mut settings = AppSettings::default();

        // 喝水提醒启用但间隔为 0
        settings.water_reminder_enabled = true;
        settings.water_reminder_interval_minutes = 0;
        assert!(manager.validate_settings(&settings).is_err());

        settings.water_reminder_interval_minutes = 30;
        assert!(manager.validate_settings(&settings).is_ok());

        // 久坐提醒启用但阈值为 0
        settings.sedentary_reminder_enabled = true;
        settings.sedentary_reminder_threshold_minutes = 0;
        assert!(manager.validate_settings(&settings).is_err());

        settings.sedentary_reminder_threshold_minutes = 60;
        assert!(manager.validate_settings(&settings).is_ok());

        // 屏幕无变化提醒启用但阈值为 0
        settings.screen_inactivity_reminder_enabled = true;
        settings.screen_inactivity_minutes = 0;
        assert!(manager.validate_settings(&settings).is_err());

        settings.screen_inactivity_minutes = 10;
        assert!(manager.validate_settings(&settings).is_ok());
    }

    #[test]
    fn test_validate_time_format() {
        let manager = SettingsManager::new();

        // 有效格式
        assert!(manager.validate_time_format("09:00").is_ok());
        assert!(manager.validate_time_format("23:59").is_ok());
        assert!(manager.validate_time_format("00:00").is_ok());

        // 无效格式
        assert!(manager.validate_time_format("9:00").is_ok()); // 单数字也可以
        assert!(manager.validate_time_format("25:00").is_err()); // 小时超出范围
        assert!(manager.validate_time_format("12:60").is_err()); // 分钟超出范��
        assert!(manager.validate_time_format("12-30").is_err()); // 错误分隔符
    }

    #[test]
    fn test_update_settings() {
        let manager = SettingsManager::new();
        let mut new_settings = AppSettings::default();
        new_settings.capture_interval_seconds = 120;

        assert!(manager.update(new_settings.clone()).is_ok());
        assert_eq!(manager.get().capture_interval_seconds, 120);
    }

    #[test]
    fn test_update_invalid_settings() {
        let manager = SettingsManager::new();
        let mut invalid_settings = AppSettings::default();
        invalid_settings.capture_interval_seconds = 0;

        assert!(manager.update(invalid_settings).is_err());
        // 原设置应该保持不变
        assert_eq!(manager.get().capture_interval_seconds, 60);
    }
}
