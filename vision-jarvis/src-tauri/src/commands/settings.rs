/// 设置相关 Commands

use tauri::State;
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
#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<ApiResponse<bool>, String> {
    let result = (*state.settings).update(settings);

    match result {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("更新设置失败: {}", e))),
    }
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
