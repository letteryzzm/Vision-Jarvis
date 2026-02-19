/// 通知规则引擎
///
/// 基于设置的可配置规则系统，支持冷却机制

use chrono::{DateTime, Utc, Timelike};
use std::collections::HashMap;
use std::sync::Mutex;
use super::{Notification, NotificationType, NotificationPriority};
use super::smart::proactive::{
    HabitReminderRule, ContextSwitchRule, SmartBreakRule, ProjectProgressRule,
};
use crate::settings::AppSettings;

/// 通知规则 trait
pub trait NotificationRule: Send + Sync {
    /// 规则名称（用于冷却追踪）
    fn name(&self) -> &str;

    /// 冷却时间（分钟）
    fn cooldown_minutes(&self) -> i64;

    /// 生成通知（返回 None 表示不触发）
    fn evaluate(&self, context: &RuleContext) -> Option<Notification>;
}

/// 规则上下文
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// 当前 UTC 时间
    pub now: DateTime<Utc>,
    /// 当前本地时间
    pub local_now: DateTime<chrono::Local>,
    /// 连续工作时长（分钟）
    pub continuous_work_minutes: i64,
    /// 屏幕无变化时长（分钟）
    pub inactive_minutes: i64,
    /// V3: 当前小时匹配的习惯列表 (pattern_name, confidence)
    pub matching_habits: Vec<(String, f32)>,
    /// V3: 最近10分钟内的应用切换次数
    pub recent_app_switches: usize,
    /// V3: 最近活跃项目距今天数（None=无项目）
    pub project_inactive_days: Option<i64>,
    /// V3: 最近活跃项目名称
    pub inactive_project_name: Option<String>,
}

/// 冷却追踪器
pub struct CooldownTracker {
    last_triggered: Mutex<HashMap<String, DateTime<Utc>>>,
}

impl CooldownTracker {
    pub fn new() -> Self {
        Self {
            last_triggered: Mutex::new(HashMap::new()),
        }
    }

    /// 检查规则是否可以触发
    pub fn can_trigger(&self, rule_name: &str, cooldown_minutes: i64) -> bool {
        let map = self.last_triggered.lock().unwrap();
        match map.get(rule_name) {
            Some(last) => {
                Utc::now().signed_duration_since(*last).num_minutes() >= cooldown_minutes
            }
            None => true,
        }
    }

    /// 记录触发时间
    pub fn record_trigger(&self, rule_name: &str) {
        let mut map = self.last_triggered.lock().unwrap();
        map.insert(rule_name.to_string(), Utc::now());
    }

    /// 重置每日规则（午夜调用）
    pub fn reset_daily(&self) {
        let mut map = self.last_triggered.lock().unwrap();
        map.retain(|k, _| !k.starts_with("morning_"));
    }
}

impl Default for CooldownTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 固定提醒规则
// ============================================================================

/// 早安提醒规则
pub struct MorningReminderRule {
    trigger_hour: u32,
    trigger_minute: u32,
    message: String,
}

impl MorningReminderRule {
    pub fn from_settings(settings: &AppSettings) -> Self {
        let (h, m) = parse_time(&settings.morning_reminder_time);
        Self {
            trigger_hour: h,
            trigger_minute: m,
            message: settings.morning_reminder_message.clone(),
        }
    }
}

impl NotificationRule for MorningReminderRule {
    fn name(&self) -> &str {
        "morning_reminder"
    }

    fn cooldown_minutes(&self) -> i64 {
        // 24 小时冷却，每天只触发一次
        24 * 60
    }

    fn evaluate(&self, context: &RuleContext) -> Option<Notification> {
        let hour = context.local_now.hour();
        let minute = context.local_now.minute();
        let current_minutes = hour * 60 + minute;
        let trigger_minutes = self.trigger_hour * 60 + self.trigger_minute;

        // 配置时间之后才触发
        if current_minutes >= trigger_minutes {
            Some(Notification::new(
                NotificationType::MorningReminder,
                NotificationPriority::Normal,
                "早安提醒".to_string(),
                self.message.clone(),
            ))
        } else {
            None
        }
    }
}

/// 喝水提醒规则
pub struct WaterReminderRule {
    start_hour: u32,
    start_minute: u32,
    end_hour: u32,
    end_minute: u32,
    interval_minutes: u16,
    message: String,
}

impl WaterReminderRule {
    pub fn from_settings(settings: &AppSettings) -> Self {
        let (sh, sm) = parse_time(&settings.water_reminder_start);
        let (eh, em) = parse_time(&settings.water_reminder_end);
        Self {
            start_hour: sh,
            start_minute: sm,
            end_hour: eh,
            end_minute: em,
            interval_minutes: settings.water_reminder_interval_minutes,
            message: settings.water_reminder_message.clone(),
        }
    }
}

impl NotificationRule for WaterReminderRule {
    fn name(&self) -> &str {
        "water_reminder"
    }

    fn cooldown_minutes(&self) -> i64 {
        self.interval_minutes as i64
    }

    fn evaluate(&self, context: &RuleContext) -> Option<Notification> {
        if is_in_time_range(
            &context.local_now,
            self.start_hour, self.start_minute,
            self.end_hour, self.end_minute,
        ) {
            Some(Notification::new(
                NotificationType::WaterReminder,
                NotificationPriority::Normal,
                "喝水提醒".to_string(),
                self.message.clone(),
            ))
        } else {
            None
        }
    }
}

/// 久坐提醒规则
pub struct SedentaryReminderRule {
    start_hour: u32,
    start_minute: u32,
    end_hour: u32,
    end_minute: u32,
    threshold_minutes: u16,
    message: String,
}

impl SedentaryReminderRule {
    pub fn from_settings(settings: &AppSettings) -> Self {
        let (sh, sm) = parse_time(&settings.sedentary_reminder_start);
        let (eh, em) = parse_time(&settings.sedentary_reminder_end);
        Self {
            start_hour: sh,
            start_minute: sm,
            end_hour: eh,
            end_minute: em,
            threshold_minutes: settings.sedentary_reminder_threshold_minutes,
            message: settings.sedentary_reminder_message.clone(),
        }
    }
}

impl NotificationRule for SedentaryReminderRule {
    fn name(&self) -> &str {
        "sedentary_reminder"
    }

    fn cooldown_minutes(&self) -> i64 {
        self.threshold_minutes as i64
    }

    fn evaluate(&self, context: &RuleContext) -> Option<Notification> {
        if !is_in_time_range(
            &context.local_now,
            self.start_hour, self.start_minute,
            self.end_hour, self.end_minute,
        ) {
            return None;
        }

        if context.continuous_work_minutes >= self.threshold_minutes as i64 {
            let hours = context.continuous_work_minutes / 60;
            let minutes = context.continuous_work_minutes % 60;

            let duration_text = if hours > 0 {
                format!("{}小时{}分钟", hours, minutes)
            } else {
                format!("{}分钟", minutes)
            };

            // 替换消息中的占位符
            let message = self.message.replace("xx", &duration_text);

            Some(Notification::new(
                NotificationType::SedentaryReminder,
                NotificationPriority::Normal,
                "久坐提醒".to_string(),
                message,
            ))
        } else {
            None
        }
    }
}

/// 屏幕无变化提醒规则（智能提醒占位）
pub struct ScreenInactivityRule {
    threshold_minutes: u16,
    message: String,
}

impl ScreenInactivityRule {
    pub fn from_settings(settings: &AppSettings) -> Self {
        Self {
            threshold_minutes: settings.screen_inactivity_minutes,
            message: if settings.screen_inactivity_message.is_empty() {
                // 默认消息（后续替换为 AI 生成）
                "刚才是不是被打断了？我看你一直没有操作电脑".to_string()
            } else {
                settings.screen_inactivity_message.clone()
            },
        }
    }
}

impl NotificationRule for ScreenInactivityRule {
    fn name(&self) -> &str {
        "screen_inactivity_reminder"
    }

    fn cooldown_minutes(&self) -> i64 {
        self.threshold_minutes as i64
    }

    fn evaluate(&self, context: &RuleContext) -> Option<Notification> {
        if context.inactive_minutes >= self.threshold_minutes as i64 {
            Some(Notification::new(
                NotificationType::ScreenInactivityReminder,
                NotificationPriority::Low,
                "屏幕无变化提醒".to_string(),
                self.message.clone(),
            ))
        } else {
            None
        }
    }
}

// ============================================================================
// 规则引擎
// ============================================================================

/// 规则引擎
pub struct RuleEngine {
    rules: Vec<Box<dyn NotificationRule>>,
}

impl RuleEngine {
    /// 从设置构建规则引擎
    pub fn from_settings(settings: &AppSettings) -> Self {
        let mut rules: Vec<Box<dyn NotificationRule>> = Vec::new();

        if settings.morning_reminder_enabled {
            rules.push(Box::new(MorningReminderRule::from_settings(settings)));
        }
        if settings.water_reminder_enabled {
            rules.push(Box::new(WaterReminderRule::from_settings(settings)));
        }
        if settings.sedentary_reminder_enabled {
            rules.push(Box::new(SedentaryReminderRule::from_settings(settings)));
        }
        if settings.screen_inactivity_reminder_enabled {
            rules.push(Box::new(ScreenInactivityRule::from_settings(settings)));
        }

        // V3: 主动建议规则（始终启用，由规则自身判断是否触发）
        rules.push(Box::new(HabitReminderRule));
        rules.push(Box::new(ContextSwitchRule::default()));
        rules.push(Box::new(SmartBreakRule::default()));
        rules.push(Box::new(ProjectProgressRule::default()));

        Self { rules }
    }

    /// 创建空规则引擎
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 评估所有规则（带冷却检查）
    pub fn evaluate_with_cooldown(
        &self,
        context: &RuleContext,
        cooldown: &CooldownTracker,
    ) -> Vec<Notification> {
        self.rules
            .iter()
            .filter_map(|rule| {
                // 冷却检查
                if !cooldown.can_trigger(rule.name(), rule.cooldown_minutes()) {
                    return None;
                }

                // 评估规则
                let notification = rule.evaluate(context)?;

                // 记录触发
                cooldown.record_trigger(rule.name());

                Some(notification)
            })
            .collect()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 工具函数
// ============================================================================

/// 解析 "HH:MM" 格式时间
fn parse_time(time: &str) -> (u32, u32) {
    let parts: Vec<&str> = time.split(':').collect();
    let hour = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minute = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    (hour, minute)
}

/// 检查当前时间是否在指定范围内
fn is_in_time_range(
    now: &DateTime<chrono::Local>,
    start_h: u32, start_m: u32,
    end_h: u32, end_m: u32,
) -> bool {
    let current = now.hour() * 60 + now.minute();
    let start = start_h * 60 + start_m;
    let end = end_h * 60 + end_m;
    current >= start && current <= end
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn default_context() -> RuleContext {
        RuleContext {
            now: Utc::now(),
            local_now: Local::now(),
            continuous_work_minutes: 0,
            inactive_minutes: 0,
            matching_habits: vec![],
            recent_app_switches: 0,
            project_inactive_days: None,
            inactive_project_name: None,
        }
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("08:30"), (8, 30));
        assert_eq!(parse_time("23:59"), (23, 59));
        assert_eq!(parse_time("00:00"), (0, 0));
    }

    #[test]
    fn test_cooldown_tracker() {
        let tracker = CooldownTracker::new();

        assert!(tracker.can_trigger("test_rule", 60));
        tracker.record_trigger("test_rule");
        assert!(!tracker.can_trigger("test_rule", 60));
        // 不同规则不受影响
        assert!(tracker.can_trigger("other_rule", 60));
    }

    #[test]
    fn test_sedentary_rule_triggers() {
        let settings = AppSettings {
            sedentary_reminder_enabled: true,
            sedentary_reminder_start: "00:00".to_string(),
            sedentary_reminder_end: "23:59".to_string(),
            sedentary_reminder_threshold_minutes: 60,
            ..AppSettings::default()
        };

        let rule = SedentaryReminderRule::from_settings(&settings);
        let mut ctx = default_context();
        ctx.continuous_work_minutes = 70;

        let notification = rule.evaluate(&ctx);
        assert!(notification.is_some());
        assert_eq!(notification.unwrap().notification_type, NotificationType::SedentaryReminder);
    }

    #[test]
    fn test_sedentary_rule_not_triggers() {
        let settings = AppSettings {
            sedentary_reminder_enabled: true,
            sedentary_reminder_threshold_minutes: 60,
            ..AppSettings::default()
        };

        let rule = SedentaryReminderRule::from_settings(&settings);
        let mut ctx = default_context();
        ctx.continuous_work_minutes = 30;

        assert!(rule.evaluate(&ctx).is_none());
    }

    #[test]
    fn test_screen_inactivity_rule() {
        let settings = AppSettings {
            screen_inactivity_reminder_enabled: true,
            screen_inactivity_minutes: 10,
            ..AppSettings::default()
        };

        let rule = ScreenInactivityRule::from_settings(&settings);
        let mut ctx = default_context();
        ctx.inactive_minutes = 15;

        let notification = rule.evaluate(&ctx);
        assert!(notification.is_some());
        assert_eq!(
            notification.unwrap().notification_type,
            NotificationType::ScreenInactivityReminder
        );
    }

    #[test]
    fn test_rule_engine_from_settings() {
        let settings = AppSettings {
            morning_reminder_enabled: true,
            water_reminder_enabled: true,
            sedentary_reminder_enabled: false,
            screen_inactivity_reminder_enabled: true,
            ..AppSettings::default()
        };

        let engine = RuleEngine::from_settings(&settings);
        assert_eq!(engine.rules.len(), 7); // 3 fixed + 4 proactive
    }

    #[test]
    fn test_rule_engine_evaluate_with_cooldown() {
        let settings = AppSettings {
            screen_inactivity_reminder_enabled: true,
            screen_inactivity_minutes: 5,
            ..AppSettings::default()
        };

        let engine = RuleEngine::from_settings(&settings);
        let cooldown = CooldownTracker::new();
        let mut ctx = default_context();
        ctx.inactive_minutes = 10;

        // 第一次触发
        let notifications = engine.evaluate_with_cooldown(&ctx, &cooldown);
        assert_eq!(notifications.len(), 1);

        // 冷却中，不再触发
        let notifications = engine.evaluate_with_cooldown(&ctx, &cooldown);
        assert_eq!(notifications.len(), 0);
    }
}
