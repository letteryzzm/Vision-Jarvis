/// 通知调度器
///
/// 每分钟评估规则并发送通知

use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;
use super::{Notification, NotificationType, NotificationPriority};
use super::rules::{RuleEngine, CooldownTracker};
use super::context;
use super::delivery;
use crate::db::Database;
use crate::settings::SettingsManager;

/// 通知调度器
pub struct NotificationScheduler {
    db: Arc<Database>,
    settings: Arc<SettingsManager>,
    cooldown: Arc<CooldownTracker>,
}

impl NotificationScheduler {
    /// 创建新的调度器
    pub fn new(
        db: Arc<Database>,
        settings: Arc<SettingsManager>,
    ) -> Self {
        Self {
            db,
            settings,
            cooldown: Arc::new(CooldownTracker::new()),
        }
    }

    /// 启动调度器（需要 AppHandle 用于发送通知）
    pub fn start(&self, app_handle: tauri::AppHandle) -> JoinHandle<()> {
        let check_interval = Duration::from_secs(60); // 每分钟检查

        let db = Arc::clone(&self.db);
        let settings = Arc::clone(&self.settings);
        let cooldown = Arc::clone(&self.cooldown);

        tokio::spawn(async move {
            let mut ticker = interval(check_interval);
            let mut last_date = chrono::Local::now().date_naive();

            loop {
                ticker.tick().await;

                // 午夜重置每日规则冷却
                let today = chrono::Local::now().date_naive();
                if today != last_date {
                    cooldown.reset_daily();
                    last_date = today;
                    eprintln!("[NotificationScheduler] Daily cooldown reset");
                }

                // 从当前设置构建规则引擎（支持热更新）
                let current_settings = settings.get();
                let rule_engine = RuleEngine::from_settings(&current_settings);

                // 构建上下文
                let ctx = match context::build_context(&db) {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        eprintln!("[NotificationScheduler] Failed to build context: {}", e);
                        continue;
                    }
                };

                // 评估规则
                let notifications = rule_engine.evaluate_with_cooldown(&ctx, &cooldown);

                if notifications.is_empty() {
                    continue;
                }

                eprintln!("[NotificationScheduler] Generated {} notification(s)", notifications.len());

                // 保存并发送
                for mut notification in notifications {
                    // 保存到数据库
                    if let Err(e) = save_notification(&db, &notification) {
                        eprintln!("[NotificationScheduler] Failed to save: {}", e);
                    }

                    // 主动建议同时写入 proactive_suggestions 表
                    if is_proactive_type(&notification.notification_type) {
                        if let Err(e) = save_proactive_suggestion(&db, &notification) {
                            eprintln!("[NotificationScheduler] Failed to save proactive suggestion: {}", e);
                        }
                    }

                    // 标记已发送
                    notification.mark_sent();

                    // 系统通知
                    if let Err(e) = delivery::send_system_notification(&app_handle, &notification) {
                        eprintln!("[NotificationScheduler] System notification failed: {}", e);
                    }

                    // 前端事件
                    if let Err(e) = delivery::emit_notification_event(&app_handle, &notification) {
                        eprintln!("[NotificationScheduler] Event emission failed: {}", e);
                    }

                    eprintln!(
                        "[NotificationScheduler] Sent: {} - {}",
                        notification.title, notification.message
                    );
                }
            }
        })
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

/// 判断是否为主动建议类型
fn is_proactive_type(t: &NotificationType) -> bool {
    matches!(
        t,
        NotificationType::HabitReminder
            | NotificationType::ContextSwitchWarning
            | NotificationType::SmartBreakReminder
            | NotificationType::ProjectProgressReminder
    )
}

/// 保存主动建议到 proactive_suggestions 表
fn save_proactive_suggestion(db: &Database, notification: &Notification) -> Result<()> {
    let suggestion_type = format!("{:?}", notification.notification_type);
    let trigger_context = serde_json::json!({
        "notification_id": notification.id,
        "title": notification.title,
    })
    .to_string();

    db.with_connection(|conn| {
        conn.execute(
            "INSERT INTO proactive_suggestions (
                id, suggestion_type, trigger_context, message,
                priority, delivered, delivered_at, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
            (
                &notification.id,
                &suggestion_type,
                &trigger_context,
                &notification.message,
                notification.priority.clone() as i32,
                notification.created_at,
                notification.created_at,
            ),
        )?;
        Ok(())
    })
}
