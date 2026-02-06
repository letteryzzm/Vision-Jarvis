/// 通知相关 Commands

use tauri::State;
use serde::{Deserialize, Serialize};
use super::{ApiResponse, AppState};
use crate::notification::Notification as NotificationModel;

/// 通知信息（前端格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationInfo {
    pub id: String,
    pub notification_type: String,
    pub priority: i32,
    pub title: String,
    pub message: String,
    pub created_at: i64,
    pub dismissed: bool,
}

impl From<NotificationModel> for NotificationInfo {
    fn from(n: NotificationModel) -> Self {
        Self {
            id: n.id,
            notification_type: format!("{:?}", n.notification_type),
            priority: n.priority as i32,
            title: n.title,
            message: n.message,
            created_at: n.created_at,
            dismissed: n.dismissed,
        }
    }
}

/// 获取待发送的通知
#[tauri::command]
pub async fn get_pending_notifications(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<NotificationInfo>>, String> {
    use crate::notification::scheduler::NotificationScheduler;

    let result = NotificationScheduler::get_pending_notifications(&state.db);

    match result {
        Ok(notifications) => {
            let info_list: Vec<NotificationInfo> = notifications
                .into_iter()
                .map(NotificationInfo::from)
                .collect();
            Ok(ApiResponse::success(info_list))
        }
        Err(e) => Ok(ApiResponse::error(format!("查询通知失败: {}", e))),
    }
}

/// 关闭通知
#[tauri::command]
pub async fn dismiss_notification(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<bool>, String> {
    let result = state.db.with_connection(|conn| {
        conn.execute(
            "UPDATE notifications SET dismissed = 1 WHERE id = ?1",
            [&id],
        )?;
        Ok(())
    });

    Ok(result.map(|_| true).into())
}

/// 获取通知历史
#[tauri::command]
pub async fn get_notification_history(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<NotificationInfo>>, String> {
    let limit = limit.unwrap_or(50);

    let result = state.db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, type, priority, title, message, created_at, dismissed
             FROM notifications
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let notifications = stmt
            .query_map([limit], |row| {
                let type_str: String = row.get(1)?;
                let priority: i32 = row.get(2)?;

                Ok(NotificationInfo {
                    id: row.get(0)?,
                    notification_type: type_str,
                    priority,
                    title: row.get(3)?,
                    message: row.get(4)?,
                    created_at: row.get(5)?,
                    dismissed: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(notifications)
    });

    Ok(result.into())
}
