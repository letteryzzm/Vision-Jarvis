/// 设置相关 Commands

use tauri::State;
use log::{info, error};
use super::{ApiResponse, AppState};
use crate::settings::AppSettings;

/// 获取设置
#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<ApiResponse<AppSettings>, String> {
    let settings = (*state.settings).get();
    Ok(ApiResponse::success(settings))
}

/// 更新设置
///
/// 自动联动调度器：
/// - memory_enabled 变化时启动/停止调度器
/// - capture_interval_seconds 变化时重启调度器
#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    mut settings: AppSettings,
) -> Result<ApiResponse<bool>, String> {
    // 强制 clamp 录制分段时长到 30-300 秒
    settings.capture_interval_seconds = settings.capture_interval_seconds.clamp(30, 300);

    let old_settings = (*state.settings).get();

    let result = (*state.settings).update(settings.clone());

    if let Err(e) = result {
        return Ok(ApiResponse::error(format!("更新设置失败: {}", e)));
    }

    // 联动调度器
    let memory_changed = old_settings.memory_enabled != settings.memory_enabled;
    let interval_changed = old_settings.capture_interval_seconds != settings.capture_interval_seconds;

    if memory_changed || interval_changed {
        let mut scheduler = state.scheduler.lock().await;

        if memory_changed {
            if settings.memory_enabled {
                if !scheduler.is_running().await {
                    scheduler.interval_seconds = settings.capture_interval_seconds as u64;
                    if let Err(e) = scheduler.start().await {
                        error!("Failed to start scheduler: {}", e);
                    } else {
                        info!("Scheduler started (segment: {}s)", settings.capture_interval_seconds);
                    }
                }
            } else {
                if scheduler.is_running().await {
                    if let Err(e) = scheduler.stop().await {
                        error!("Failed to stop scheduler: {}", e);
                    } else {
                        info!("Scheduler stopped");
                    }
                }
            }
        } else if interval_changed && settings.memory_enabled {
            let was_running = scheduler.is_running().await;
            if was_running { let _ = scheduler.stop().await; }
            scheduler.interval_seconds = settings.capture_interval_seconds as u64;
            if was_running { let _ = scheduler.start().await; }
            info!("Segment duration updated: {}s", settings.capture_interval_seconds);
        }
    }

    Ok(ApiResponse::success(true))
}

/// 重置设置为默认值
#[tauri::command]
pub async fn reset_settings(
    state: State<'_, AppState>,
) -> Result<ApiResponse<AppSettings>, String> {
    let default_settings = AppSettings::default();
    let result = (*state.settings).update(default_settings.clone());

    match result {
        Ok(_) => Ok(ApiResponse::success(default_settings)),
        Err(e) => Ok(ApiResponse::error(format!("重置设置失败: {}", e))),
    }
}
