/// 截图相关 Commands

use tauri::State;
use serde::{Deserialize, Serialize};
use super::{ApiResponse, AppState};

/// 截图信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotInfo {
    pub id: String,
    pub path: String,
    pub captured_at: i64,
    pub analyzed: bool,
}

/// 捕获截图
#[tauri::command]
pub async fn capture_screenshot(state: State<'_, AppState>) -> Result<ApiResponse<ScreenshotInfo>, String> {
    match (*state.screen_capture).capture_screenshot() {
        Ok(file_path) => {
            let timestamp = chrono::Utc::now().timestamp();
            let id = uuid::Uuid::new_v4().to_string();
            let path_str = file_path.to_string_lossy().to_string();

            // 保存到数据库
            let db_result = state.db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO screenshots (id, path, captured_at, analyzed)
                     VALUES (?1, ?2, ?3, ?4)",
                    (&id, &path_str, timestamp, 0),
                )?;
                Ok(())
            });

            if let Err(e) = db_result {
                return Ok(ApiResponse::error(format!("数据库保存失败: {}", e)));
            }

            Ok(ApiResponse::success(ScreenshotInfo {
                id,
                path: path_str,
                captured_at: timestamp,
                analyzed: false,
            }))
        }
        Err(e) => Ok(ApiResponse::error(format!("截图失败: {}", e))),
    }
}

/// 获取截图列表
#[tauri::command]
pub async fn get_screenshots(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<ApiResponse<Vec<ScreenshotInfo>>, String> {
    let limit = limit.unwrap_or(50);

    let result = state.db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, path, captured_at, analyzed
             FROM screenshots
             ORDER BY captured_at DESC
             LIMIT ?1"
        )?;

        let screenshots = stmt
            .query_map([limit], |row| {
                Ok(ScreenshotInfo {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    captured_at: row.get(2)?,
                    analyzed: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(screenshots)
    });

    Ok(result.into())
}

/// 删除截图
#[tauri::command]
pub async fn delete_screenshot(
    state: State<'_, AppState>,
    id: String,
) -> Result<ApiResponse<bool>, String> {
    // 先获取文件路径
    let path_result = state.db.with_connection(|conn| {
        let mut stmt = conn.prepare("SELECT path FROM screenshots WHERE id = ?1")?;
        let path: String = stmt.query_row([&id], |row| row.get(0))?;
        Ok(path)
    });

    let path = match path_result {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(format!("查询失败: {}", e))),
    };

    // 删除文件
    if let Err(e) = std::fs::remove_file(&path) {
        log::warn!("删除文件失败: {}", e);
    }

    // 删除数据库记录
    let db_result = state.db.with_connection(|conn| {
        conn.execute("DELETE FROM screenshots WHERE id = ?1", [&id])?;
        Ok(())
    });

    Ok(db_result.map(|_| true).into())
}

/// 调度器状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub is_running: bool,
    pub interval_seconds: u8,
    pub memory_enabled: bool,
    pub storage_path: String,
}

/// 获取调度器状态（调试用）
#[tauri::command]
pub async fn get_scheduler_status(state: State<'_, AppState>) -> Result<ApiResponse<SchedulerStatus>, String> {
    let scheduler = state.scheduler.lock().await;
    let is_running = scheduler.is_running().await;
    let interval = scheduler.interval_seconds;
    let memory_enabled = state.settings.is_memory_enabled();
    let storage_path = state.settings.get_storage_path().to_string_lossy().to_string();

    Ok(ApiResponse::success(SchedulerStatus {
        is_running,
        interval_seconds: interval,
        memory_enabled,
        storage_path,
    }))
}

#[cfg(test)]
mod tests {
    // Tests will be added in integration tests
}
