/// 主动建议规则 V3
///
/// 四种建议类型：
/// 1. HabitReminderRule - 习惯提醒（当前时间匹配已检测到的习惯）
/// 2. ContextSwitchRule - 上下文切换警告（频繁切换应用）
/// 3. SmartBreakRule - 智能休息提醒（基于连续工作+切换频率）
/// 4. ProjectProgressRule - 项目进度提醒（长时间未活跃的项目）

use super::super::{Notification, NotificationType, NotificationPriority};
use super::super::rules::{NotificationRule, RuleContext};

// ============================================================================
// 1. 习惯提醒
// ============================================================================

pub struct HabitReminderRule;

impl NotificationRule for HabitReminderRule {
    fn name(&self) -> &str {
        "habit_reminder"
    }

    fn cooldown_minutes(&self) -> i64 {
        60 // 每小时最多提醒一次
    }

    fn evaluate(&self, context: &RuleContext) -> Option<Notification> {
        let best = context.matching_habits.first()?;
        let (name, confidence) = best;

        if *confidence < 0.5 {
            return None;
        }

        Some(Notification::new(
            NotificationType::HabitReminder,
            NotificationPriority::Low,
            "习惯提醒".to_string(),
            format!("现在通常是「{}」的时间 (置信度 {:.0}%)", name, confidence * 100.0),
        ))
    }
}

// ============================================================================
// 2. 上下文切换警告
// ============================================================================

pub struct ContextSwitchRule {
    /// 10分钟内切换超过此次数则警告
    pub threshold: usize,
}

impl Default for ContextSwitchRule {
    fn default() -> Self {
        Self { threshold: 6 }
    }
}

impl NotificationRule for ContextSwitchRule {
    fn name(&self) -> &str {
        "context_switch_warning"
    }

    fn cooldown_minutes(&self) -> i64 {
        30 // 30分钟冷却
    }

    fn evaluate(&self, context: &RuleContext) -> Option<Notification> {
        if context.recent_app_switches < self.threshold {
            return None;
        }

        Some(Notification::new(
            NotificationType::ContextSwitchWarning,
            NotificationPriority::Normal,
            "频繁切换提醒".to_string(),
            format!(
                "最近10分钟内切换了 {} 次应用，频繁切换会降低专注度，试试集中处理一件事？",
                context.recent_app_switches
            ),
        ))
    }
}

// ============================================================================
// 3. 智能休息提醒（比久坐提醒更智能）
// ============================================================================

pub struct SmartBreakRule {
    /// 连续工作超过此分钟数触发
    pub work_threshold_minutes: i64,
}

impl Default for SmartBreakRule {
    fn default() -> Self {
        Self { work_threshold_minutes: 90 }
    }
}

impl NotificationRule for SmartBreakRule {
    fn name(&self) -> &str {
        "smart_break_reminder"
    }

    fn cooldown_minutes(&self) -> i64 {
        self.work_threshold_minutes
    }

    fn evaluate(&self, context: &RuleContext) -> Option<Notification> {
        if context.continuous_work_minutes < self.work_threshold_minutes {
            return None;
        }

        let hours = context.continuous_work_minutes / 60;
        let mins = context.continuous_work_minutes % 60;
        let duration = if hours > 0 {
            format!("{}小时{}分钟", hours, mins)
        } else {
            format!("{}分钟", mins)
        };

        // 根据切换频率给出不同建议
        let tip = if context.recent_app_switches > 4 {
            "而且切换频繁，说明注意力可能已经分散了"
        } else {
            "保持专注很棒，但也别忘了休息"
        };

        Some(Notification::new(
            NotificationType::SmartBreakReminder,
            NotificationPriority::Normal,
            "休息一下".to_string(),
            format!("你已经连续工作了 {}，{}。起来活动活动吧！", duration, tip),
        ))
    }
}

// ============================================================================
// 4. 项目进度提醒
// ============================================================================

pub struct ProjectProgressRule {
    /// 项目不活跃超过此天数触发
    pub inactive_threshold_days: i64,
}

impl Default for ProjectProgressRule {
    fn default() -> Self {
        Self { inactive_threshold_days: 7 }
    }
}

impl NotificationRule for ProjectProgressRule {
    fn name(&self) -> &str {
        "project_progress_reminder"
    }

    fn cooldown_minutes(&self) -> i64 {
        24 * 60 // 每天最多提醒一次
    }

    fn evaluate(&self, context: &RuleContext) -> Option<Notification> {
        let days = context.project_inactive_days?;
        let name = context.inactive_project_name.as_ref()?;

        if days < self.inactive_threshold_days {
            return None;
        }

        Some(Notification::new(
            NotificationType::ProjectProgressReminder,
            NotificationPriority::Low,
            "项目进度提醒".to_string(),
            format!("「{}」已经 {} 天没有活动了，要不要看看？", name, days),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn test_context() -> RuleContext {
        RuleContext {
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
    fn test_habit_reminder_triggers() {
        let rule = HabitReminderRule;
        let mut ctx = test_context();
        ctx.matching_habits = vec![("每天 08:00 使用 微信".to_string(), 0.8)];

        let notification = rule.evaluate(&ctx);
        assert!(notification.is_some());
        assert_eq!(notification.unwrap().notification_type, NotificationType::HabitReminder);
    }

    #[test]
    fn test_habit_reminder_low_confidence_skips() {
        let rule = HabitReminderRule;
        let mut ctx = test_context();
        ctx.matching_habits = vec![("低置信度习惯".to_string(), 0.3)];

        assert!(rule.evaluate(&ctx).is_none());
    }

    #[test]
    fn test_context_switch_triggers() {
        let rule = ContextSwitchRule::default();
        let mut ctx = test_context();
        ctx.recent_app_switches = 8;

        let notification = rule.evaluate(&ctx);
        assert!(notification.is_some());
        assert_eq!(notification.unwrap().notification_type, NotificationType::ContextSwitchWarning);
    }

    #[test]
    fn test_context_switch_below_threshold() {
        let rule = ContextSwitchRule::default();
        let mut ctx = test_context();
        ctx.recent_app_switches = 3;

        assert!(rule.evaluate(&ctx).is_none());
    }

    #[test]
    fn test_smart_break_triggers() {
        let rule = SmartBreakRule::default();
        let mut ctx = test_context();
        ctx.continuous_work_minutes = 100;

        let notification = rule.evaluate(&ctx);
        assert!(notification.is_some());
        assert_eq!(notification.unwrap().notification_type, NotificationType::SmartBreakReminder);
    }

    #[test]
    fn test_project_progress_triggers() {
        let rule = ProjectProgressRule::default();
        let mut ctx = test_context();
        ctx.project_inactive_days = Some(10);
        ctx.inactive_project_name = Some("Vision-Jarvis".to_string());

        let notification = rule.evaluate(&ctx);
        assert!(notification.is_some());
        assert!(notification.unwrap().message.contains("Vision-Jarvis"));
    }

    #[test]
    fn test_project_progress_no_project() {
        let rule = ProjectProgressRule::default();
        let ctx = test_context();
        assert!(rule.evaluate(&ctx).is_none());
    }
}
