/// 通知调度器
///
/// 定时评估规则并发送通知

use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;
use log::{info, error};
use super::{Notification, NotificationPriority};
use super::rules::{RuleEngine, RuleContext};
use crate::db::Database;
use chrono::Utc;

/// 通知调度器
pub struct NotificationScheduler {
    db: Arc<Database>,
    rule_engine: Arc<RuleEngine>,
}

impl NotificationScheduler {
    /// 创建新的调度器
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
            rule_engine: Arc::new(RuleEngine::with_default_rules()),
        }
    }

    /// 使用自定义规则引擎
    pub fn with_rules(db: Database, rule_engine: RuleEngine) -> Self {
        Self {
            db: Arc::new(db),
            rule_engine: Arc::new(rule_engine),
        }
    }

    /// 启动调度器
    pub fn start(&self) -> JoinHandle<()> {
        let check_interval = Duration::from_secs(300); // 每5分钟检查一次

        let db = Arc::clone(&self.db);
        let rules = Arc::clone(&self.rule_engine);

        tokio::spawn(async move {
            let mut ticker = interval(check_interval);

            loop {
                ticker.tick().await;

                if let Err(e) = Self::check_and_notify(&db, &rules).await {
                    error!("通知检查失败: {}", e);
                }
            }
        })
    }

    /// 检查规则并发送通知
    async fn check_and_notify(db: &Database, rules: &RuleEngine) -> Result<()> {
        info!("开始检查通知规则");

        // 构建规则上下文
        let context = Self::build_context(db).await?;

        // 评估规则
        let notifications = rules.evaluate(&context);

        if notifications.is_empty() {
            info!("无待发送通知");
            return Ok(());
        }

        info!("生成 {} 个通知", notifications.len());

        // 保存并发送通知
        for notification in notifications {
            Self::save_notification(db, &notification)?;
            Self::send_notification(&notification).await?;
        }

        Ok(())
    }

    /// 构建规则上下文
    async fn build_context(db: &Database) -> Result<RuleContext> {
        // TODO: 从数据库查询实际数据
        // 这里使用模拟数据
        let context = RuleContext {
            now: Utc::now(),
            continuous_work_minutes: 0,
            last_break_time: None,
            today_work_minutes: 0,
            current_activity: None,
        };

        Ok(context)
    }

    /// 保存通知到数据库
    fn save_notification(db: &Database, notification: &Notification) -> Result<()> {
        db.with_connection(|conn| {
            conn.execute(
                "INSERT INTO notifications (
                    id, type, priority, title, message,
                    created_at, scheduled_at, sent_at, dismissed
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                (
                    &notification.id,
                    serde_json::to_string(&notification.notification_type)?,
                    notification.priority.clone() as i32,
                    &notification.title,
                    &notification.message,
                    notification.created_at,
                    notification.scheduled_at,
                    notification.sent_at,
                    notification.dismissed,
                ),
            )?;
            Ok(())
        })
    }

    /// 发送通知
    async fn send_notification(notification: &Notification) -> Result<()> {
        info!(
            "发送通知: {} - {}",
            notification.title, notification.message
        );

        // TODO: 集成 tauri-plugin-notification
        // 当前仅记录日志

        Ok(())
    }

    /// 获取待发送的通知
    pub fn get_pending_notifications(db: &Database) -> Result<Vec<Notification>> {
        db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, type, priority, title, message,
                        created_at, scheduled_at, sent_at, dismissed
                 FROM notifications
                 WHERE sent_at IS NULL AND dismissed = 0
                 ORDER BY priority DESC, created_at ASC
                 LIMIT 10"
            )?;

            let notifications = stmt
                .query_map([], |row| {
                    let type_str: String = row.get(1)?;
                    let priority_int: i32 = row.get(2)?;

                    Ok(Notification {
                        id: row.get(0)?,
                        notification_type: serde_json::from_str(&type_str)
                            .unwrap_or(super::NotificationType::Custom),
                        priority: match priority_int {
                            0 => NotificationPriority::Low,
                            1 => NotificationPriority::Normal,
                            2 => NotificationPriority::High,
                            _ => NotificationPriority::Urgent,
                        },
                        title: row.get(3)?,
                        message: row.get(4)?,
                        created_at: row.get(5)?,
                        scheduled_at: row.get(6)?,
                        sent_at: row.get(7)?,
                        dismissed: row.get(8)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(notifications)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_scheduler_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();

        let scheduler = NotificationScheduler::new(db);
        assert!(Arc::strong_count(&scheduler.rule_engine) >= 1);
    }

    #[test]
    fn test_get_pending_notifications_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.initialize().unwrap();

        // 创建 notifications 表
        db.with_connection(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS notifications (
                    id TEXT PRIMARY KEY,
                    type TEXT NOT NULL,
                    priority INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    message TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    scheduled_at INTEGER,
                    sent_at INTEGER,
                    dismissed INTEGER DEFAULT 0
                )",
                [],
            )?;
            Ok(())
        })
        .unwrap();

        let notifications = NotificationScheduler::get_pending_notifications(&db).unwrap();
        assert_eq!(notifications.len(), 0);
    }
}
