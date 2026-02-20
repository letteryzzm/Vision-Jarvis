/// 录制相关 Commands

use tauri::State;
use serde::{Deserialize, Serialize};
use super::{ApiResponse, AppState};

/// 调度器状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub is_running: bool,
    pub interval_seconds: u64,
    pub memory_enabled: bool,
    pub storage_path: String,
}

/// 获取调度器状态
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
