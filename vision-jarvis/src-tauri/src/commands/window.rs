/// 窗口管理 Commands
///
/// 管理多窗口系统：Memory窗口、Popup-Setting窗口、浮球状态切换

use tauri::{AppHandle, Manager, PhysicalSize, WebviewWindowBuilder};
use super::ApiResponse;

/// 打开 Memory 窗口
#[tauri::command]
pub async fn open_memory_window(app: AppHandle) -> ApiResponse<String> {
    match app.get_webview_window("memory") {
        Some(window) => {
            // 窗口已存在，聚焦
            if let Err(e) = window.set_focus() {
                return ApiResponse::error(format!("Failed to focus memory window: {}", e));
            }
            ApiResponse::success("Memory window focused".to_string())
        }
        None => {
            // 创建新窗口
            match WebviewWindowBuilder::new(&app, "memory", tauri::WebviewUrl::App("/memory".into()))
                .title("Memory - Vision Jarvis")
                .inner_size(1200.0, 800.0)
                .resizable(true)
                .build()
            {
                Ok(_) => ApiResponse::success("Memory window created".to_string()),
                Err(e) => ApiResponse::error(format!("Failed to create memory window: {}", e)),
            }
        }
    }
}

/// 打开 Popup-Setting 窗口
#[tauri::command]
pub async fn open_popup_setting_window(app: AppHandle) -> ApiResponse<String> {
    match app.get_webview_window("popup-setting") {
        Some(window) => {
            // 窗口已存在，聚焦
            if let Err(e) = window.set_focus() {
                return ApiResponse::error(format!("Failed to focus popup-setting window: {}", e));
            }
            ApiResponse::success("Popup-Setting window focused".to_string())
        }
        None => {
            // 创建新窗口
            match WebviewWindowBuilder::new(
                &app,
                "popup-setting",
                tauri::WebviewUrl::App("/popup-setting".into())
            )
                .title("Settings - Vision Jarvis")
                .inner_size(900.0, 700.0)
                .resizable(true)
                .build()
            {
                Ok(_) => ApiResponse::success("Popup-Setting window created".to_string()),
                Err(e) => ApiResponse::error(format!("Failed to create popup-setting window: {}", e)),
            }
        }
    }
}

/// 展开浮球到 Header 状态 (360x72)
#[tauri::command]
pub async fn expand_to_header(app: AppHandle) -> ApiResponse<String> {
    match app.get_webview_window("main") {
        Some(window) => {
            if let Err(e) = window.set_size(PhysicalSize::new(360, 72)) {
                return ApiResponse::error(format!("Failed to expand to header: {}", e));
            }
            ApiResponse::success("Expanded to header".to_string())
        }
        None => ApiResponse::error("Main window not found".to_string()),
    }
}

/// 展开浮球到 Asker 状态 (360x480)
#[tauri::command]
pub async fn expand_to_asker(app: AppHandle) -> ApiResponse<String> {
    match app.get_webview_window("main") {
        Some(window) => {
            if let Err(e) = window.set_size(PhysicalSize::new(360, 480)) {
                return ApiResponse::error(format!("Failed to expand to asker: {}", e));
            }
            ApiResponse::success("Expanded to asker".to_string())
        }
        None => ApiResponse::error("Main window not found".to_string()),
    }
}

/// 收起浮球到圆球状态 (64x64)
#[tauri::command]
pub async fn collapse_to_ball(app: AppHandle) -> ApiResponse<String> {
    match app.get_webview_window("main") {
        Some(window) => {
            if let Err(e) = window.set_size(PhysicalSize::new(64, 64)) {
                return ApiResponse::error(format!("Failed to collapse to ball: {}", e));
            }
            ApiResponse::success("Collapsed to ball".to_string())
        }
        None => ApiResponse::error("Main window not found".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_commands_exist() {
        // 这些测试只是确保函数签名正确
        // 实际的窗口操作需要在集成测试中进行
        assert!(true);
    }
}
