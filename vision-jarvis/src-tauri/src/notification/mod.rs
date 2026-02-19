/// 通知服务
///
/// 基于用户行为和时间规则生成主动通知

use serde::{Deserialize, Serialize};
use chrono::Utc;

pub mod scheduler;
pub mod rules;
pub mod context;
pub mod delivery;
pub mod smart;

/// 通知类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    /// 早安提醒
    MorningReminder,
    /// 喝水提醒
    WaterReminder,
    /// 久坐提醒
    SedentaryReminder,
    /// 屏幕无变化提醒（智能提醒）
    ScreenInactivityReminder,
    /// V3: 习惯提醒
    HabitReminder,
    /// V3: 上下文切换警告
    ContextSwitchWarning,
    /// V3: 休息提醒（基于行为分析）
    SmartBreakReminder,
    /// V3: 项目进度提醒
    ProjectProgressReminder,
    /// 自定义通知
    Custom,
}

/// 通知优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// 通知记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub title: String,
    pub message: String,
    pub created_at: i64,
    pub scheduled_at: Option<i64>,
    pub sent_at: Option<i64>,
    pub dismissed: bool,
}

impl Notification {
    /// 创建新通知
    pub fn new(
        notification_type: NotificationType,
        priority: NotificationPriority,
        title: String,
        message: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            notification_type,
            priority,
            title,
            message,
            created_at: Utc::now().timestamp(),
            scheduled_at: None,
            sent_at: None,
            dismissed: false,
        }
    }

    /// 标记为已发送
    pub fn mark_sent(&mut self) {
        self.sent_at = Some(Utc::now().timestamp());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new(
            NotificationType::MorningReminder,
            NotificationPriority::Normal,
            "早安提醒".to_string(),
            "新的一天开始了".to_string(),
        );

        assert_eq!(notification.notification_type, NotificationType::MorningReminder);
        assert_eq!(notification.priority, NotificationPriority::Normal);
        assert!(!notification.dismissed);
        assert!(notification.sent_at.is_none());
    }

    #[test]
    fn test_mark_sent() {
        let mut notification = Notification::new(
            NotificationType::SedentaryReminder,
            NotificationPriority::Normal,
            "测试".to_string(),
            "测试消息".to_string(),
        );

        assert!(notification.sent_at.is_none());
        notification.mark_sent();
        assert!(notification.sent_at.is_some());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(NotificationPriority::Urgent > NotificationPriority::High);
        assert!(NotificationPriority::High > NotificationPriority::Normal);
        assert!(NotificationPriority::Normal > NotificationPriority::Low);
    }
}
