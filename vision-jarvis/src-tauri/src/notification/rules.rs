/// 通知规则引擎
///
/// 基于用户行为模式生成通知规则

use anyhow::Result;
use chrono::{DateTime, Utc, Duration, Timelike};
use super::{Notification, NotificationType, NotificationPriority};

/// 通知规则
pub trait NotificationRule: Send + Sync {
    /// 规则名称
    fn name(&self) -> &str;

    /// 检查规则是否触发
    fn should_trigger(&self, context: &RuleContext) -> bool;

    /// 生成通知
    fn generate_notification(&self, context: &RuleContext) -> Option<Notification>;
}

/// 规则上下文
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// 当前时间
    pub now: DateTime<Utc>,
    /// 连续工作时长（分钟）
    pub continuous_work_minutes: i64,
    /// 上次休息时间
    pub last_break_time: Option<DateTime<Utc>>,
    /// 今日总工作时长（分钟）
    pub today_work_minutes: i64,
    /// 当前活动
    pub current_activity: Option<String>,
}

/// 休息提醒规则
pub struct RestReminderRule {
    /// 连续工作阈值（分钟）
    work_threshold_minutes: i64,
}

impl RestReminderRule {
    pub fn new(work_threshold_minutes: i64) -> Self {
        Self {
            work_threshold_minutes,
        }
    }
}

impl NotificationRule for RestReminderRule {
    fn name(&self) -> &str {
        "休息提醒规则"
    }

    fn should_trigger(&self, context: &RuleContext) -> bool {
        context.continuous_work_minutes >= self.work_threshold_minutes
    }

    fn generate_notification(&self, context: &RuleContext) -> Option<Notification> {
        if !self.should_trigger(context) {
            return None;
        }

        let hours = context.continuous_work_minutes / 60;
        let minutes = context.continuous_work_minutes % 60;

        let duration_text = if hours > 0 {
            format!("{}小时{}分钟", hours, minutes)
        } else {
            format!("{}分钟", minutes)
        };

        Some(Notification::new(
            NotificationType::RestReminder,
            NotificationPriority::Normal,
            "休息提醒".to_string(),
            format!("您已连续工作{}，建议休息5-10分钟，保护眼睛和身体健康。", duration_text),
        ))
    }
}

/// 每日总结提醒规则
pub struct DailySummaryRule {
    /// 提醒时间（小时）
    reminder_hour: u32,
}

impl DailySummaryRule {
    pub fn new(reminder_hour: u32) -> Self {
        Self { reminder_hour }
    }
}

impl NotificationRule for DailySummaryRule {
    fn name(&self) -> &str {
        "每日总结提醒"
    }

    fn should_trigger(&self, context: &RuleContext) -> bool {
        let current_hour = context.now.hour();
        current_hour == self.reminder_hour && context.today_work_minutes > 60
    }

    fn generate_notification(&self, context: &RuleContext) -> Option<Notification> {
        if !self.should_trigger(context) {
            return None;
        }

        let hours = context.today_work_minutes / 60;
        let minutes = context.today_work_minutes % 60;

        Some(Notification::new(
            NotificationType::SummaryReminder,
            NotificationPriority::Low,
            "每日总结".to_string(),
            format!("今天您已工作{}小时{}分钟，查看一下今天的工作记录吧！", hours, minutes),
        ))
    }
}

/// 长时间未活动提醒规则
pub struct InactivityReminderRule {
    /// 未活动阈值（分钟）
    inactivity_threshold_minutes: i64,
}

impl InactivityReminderRule {
    pub fn new(inactivity_threshold_minutes: i64) -> Self {
        Self {
            inactivity_threshold_minutes,
        }
    }
}

impl NotificationRule for InactivityReminderRule {
    fn name(&self) -> &str {
        "长时间未活动提醒"
    }

    fn should_trigger(&self, context: &RuleContext) -> bool {
        if let Some(last_break) = context.last_break_time {
            let inactive_duration = context.now.signed_duration_since(last_break);
            inactive_duration.num_minutes() >= self.inactivity_threshold_minutes
        } else {
            false
        }
    }

    fn generate_notification(&self, context: &RuleContext) -> Option<Notification> {
        if !self.should_trigger(context) {
            return None;
        }

        Some(Notification::new(
            NotificationType::Custom,
            NotificationPriority::Low,
            "活动提醒".to_string(),
            "您已经很久没有活动了，站起来走动一下吧！".to_string(),
        ))
    }
}

/// 规则引擎
pub struct RuleEngine {
    rules: Vec<Box<dyn NotificationRule>>,
}

impl RuleEngine {
    /// 创建默认规则引擎
    pub fn with_default_rules() -> Self {
        let mut rules: Vec<Box<dyn NotificationRule>> = Vec::new();

        // 60分钟工作提醒
        rules.push(Box::new(RestReminderRule::new(60)));

        // 每天20点总结提醒
        rules.push(Box::new(DailySummaryRule::new(20)));

        // 2小时未活动提醒
        rules.push(Box::new(InactivityReminderRule::new(120)));

        Self { rules }
    }

    /// 创建空规则引擎
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: Box<dyn NotificationRule>) {
        self.rules.push(rule);
    }

    /// 评估所有规则
    pub fn evaluate(&self, context: &RuleContext) -> Vec<Notification> {
        self.rules
            .iter()
            .filter_map(|rule| rule.generate_notification(context))
            .collect()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::with_default_rules()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_reminder_rule() {
        let rule = RestReminderRule::new(60);

        let context = RuleContext {
            now: Utc::now(),
            continuous_work_minutes: 70,
            last_break_time: None,
            today_work_minutes: 120,
            current_activity: Some("编程".to_string()),
        };

        assert!(rule.should_trigger(&context));
        let notification = rule.generate_notification(&context);
        assert!(notification.is_some());

        let notif = notification.unwrap();
        assert_eq!(notif.notification_type, NotificationType::RestReminder);
        assert!(notif.message.contains("1小时10分钟"));
    }

    #[test]
    fn test_rest_reminder_not_trigger() {
        let rule = RestReminderRule::new(60);

        let context = RuleContext {
            now: Utc::now(),
            continuous_work_minutes: 30,
            last_break_time: None,
            today_work_minutes: 30,
            current_activity: Some("编程".to_string()),
        };

        assert!(!rule.should_trigger(&context));
        assert!(rule.generate_notification(&context).is_none());
    }

    #[test]
    fn test_daily_summary_rule() {
        let rule = DailySummaryRule::new(20);

        let mut now = Utc::now();
        // 设置为20点
        now = now
            .with_hour(20)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .unwrap();

        let context = RuleContext {
            now,
            continuous_work_minutes: 30,
            last_break_time: None,
            today_work_minutes: 480, // 8小时
            current_activity: None,
        };

        assert!(rule.should_trigger(&context));
        let notification = rule.generate_notification(&context);
        assert!(notification.is_some());
    }

    #[test]
    fn test_rule_engine() {
        let engine = RuleEngine::with_default_rules();
        assert_eq!(engine.rules.len(), 3);
    }

    #[test]
    fn test_rule_engine_evaluate() {
        let mut engine = RuleEngine::new();
        engine.add_rule(Box::new(RestReminderRule::new(60)));

        let context = RuleContext {
            now: Utc::now(),
            continuous_work_minutes: 70,
            last_break_time: None,
            today_work_minutes: 120,
            current_activity: Some("编程".to_string()),
        };

        let notifications = engine.evaluate(&context);
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].notification_type, NotificationType::RestReminder);
    }
}
