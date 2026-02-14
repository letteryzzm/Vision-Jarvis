/// 通知投递
///
/// 通过系统通知和 Tauri 事件双通道投递

use super::Notification;

/// 通过 tauri-plugin-notification 发送系统通知
pub fn send_system_notification(
    app: &tauri::AppHandle,
    notification: &Notification,
) -> anyhow::Result<()> {
    use tauri_plugin_notification::NotificationExt;

    app.notification()
        .builder()
        .title(&notification.title)
        .body(&notification.message)
        .show()?;

    Ok(())
}

/// 通过 Tauri 事件发送到前端
pub fn emit_notification_event(
    app: &tauri::AppHandle,
    notification: &Notification,
) -> anyhow::Result<()> {
    use tauri::Emitter;

    app.emit("notification:new", serde_json::json!({
        "id": notification.id,
        "title": notification.title,
        "message": notification.message,
        "type": format!("{:?}", notification.notification_type),
    }))?;

    Ok(())
}
