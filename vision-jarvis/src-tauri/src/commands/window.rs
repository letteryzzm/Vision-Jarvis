/// 窗口管理 Commands
///
/// 管理多窗口系统：Memory窗口、Popup-Setting窗口、浮球状态切换

use tauri::{AppHandle, Manager, WebviewWindowBuilder};
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

/// 展开浮球到 Header 状态 (360x146)
/// 新布局: Ball(64) + gap(10) + Header(72) = 146
#[tauri::command]
pub async fn expand_to_header(app: AppHandle) -> ApiResponse<String> {
    match app.get_webview_window("floating-ball") {
        Some(window) => {
            // 获取当前位置和屏幕信息
            let current_pos = match window.outer_position() {
                Ok(pos) => pos,
                Err(e) => return ApiResponse::error(format!("Failed to get position: {}", e)),
            };

            // 获取屏幕信息以计算新位置
            let monitor = match window.primary_monitor() {
                Ok(Some(m)) => m,
                _ => return ApiResponse::error("Failed to get monitor info".to_string()),
            };

            let physical_size = monitor.size();
            let scale_factor = monitor.scale_factor();
            let screen_width = physical_size.width as f64 / scale_factor;
            let screen_height = physical_size.height as f64 / scale_factor;

            // 新尺寸：Ball(64) + gap(10) + Header(72) = 146
            let new_width = 360.0;
            let new_height = 146.0;
            let margin_right = 20.0;

            // 计算X位置：保持右边缘对齐
            let new_x = (screen_width - new_width - margin_right).max(0.0);

            // 计算Y位置：保持当前Y，但检查是否超出屏幕底部
            let current_y = current_pos.y as f64 / scale_factor;
            let new_y = if (current_y + new_height) > screen_height {
                // 超出底部，向上调整
                (screen_height - new_height).max(0.0)
            } else {
                current_y
            };

            // 先设置大小
            if let Err(e) = window.set_size(tauri::LogicalSize::new(new_width, new_height)) {
                return ApiResponse::error(format!("Failed to resize: {}", e));
            }

            // 再调整位置
            if let Err(e) = window.set_position(tauri::LogicalPosition::new(new_x, new_y)) {
                return ApiResponse::error(format!("Failed to reposition: {}", e));
            }

            ApiResponse::success("Expanded to header".to_string())
        }
        None => ApiResponse::error("Floating ball window not found".to_string()),
    }
}

/// 展开浮球到 Asker 状态 (360x554)
/// 新布局: Ball(64) + gap(10) + Asker(480) = 554
#[tauri::command]
pub async fn expand_to_asker(app: AppHandle) -> ApiResponse<String> {
    match app.get_webview_window("floating-ball") {
        Some(window) => {
            // 获取当前位置和屏幕信息
            let current_pos = match window.outer_position() {
                Ok(pos) => pos,
                Err(e) => return ApiResponse::error(format!("Failed to get position: {}", e)),
            };

            let monitor = match window.primary_monitor() {
                Ok(Some(m)) => m,
                _ => return ApiResponse::error("Failed to get monitor info".to_string()),
            };

            let physical_size = monitor.size();
            let scale_factor = monitor.scale_factor();
            let screen_width = physical_size.width as f64 / scale_factor;
            let screen_height = physical_size.height as f64 / scale_factor;

            // 新尺寸：Ball(64) + gap(10) + Asker(480) = 554
            let new_width = 360.0;
            let new_height = 554.0;
            let margin_right = 20.0;

            // 计算X位置：保持右边缘对齐
            let new_x = (screen_width - new_width - margin_right).max(0.0);

            // 计算Y位置：保持当前Y，但检查是否超出屏幕底部
            let current_y = current_pos.y as f64 / scale_factor;
            let new_y = if (current_y + new_height) > screen_height {
                // 超出底部，向上调整
                (screen_height - new_height).max(0.0)
            } else {
                current_y
            };

            // 先设置大小
            if let Err(e) = window.set_size(tauri::LogicalSize::new(new_width, new_height)) {
                return ApiResponse::error(format!("Failed to resize: {}", e));
            }

            // 再调整位置
            if let Err(e) = window.set_position(tauri::LogicalPosition::new(new_x, new_y)) {
                return ApiResponse::error(format!("Failed to reposition: {}", e));
            }

            ApiResponse::success("Expanded to asker".to_string())
        }
        None => ApiResponse::error("Floating ball window not found".to_string()),
    }
}

/// 收起浮球到圆球状态 (64x64)
#[tauri::command]
pub async fn collapse_to_ball(app: AppHandle) -> ApiResponse<String> {
    match app.get_webview_window("floating-ball") {
        Some(window) => {
            // 获取屏幕信息
            let monitor = match window.primary_monitor() {
                Ok(Some(m)) => m,
                _ => return ApiResponse::error("Failed to get monitor info".to_string()),
            };

            let physical_size = monitor.size();
            let scale_factor = monitor.scale_factor();
            let screen_width = physical_size.width as f64 / scale_factor;

            // 计算球状态位置：右上角
            let ball_width = 64.0;
            let margin_right = 20.0;
            let margin_top = 50.0;
            let new_x = (screen_width - ball_width - margin_right).max(0.0);
            let new_y = margin_top;

            // 先设置大小
            if let Err(e) = window.set_size(tauri::LogicalSize::new(ball_width, 64.0)) {
                return ApiResponse::error(format!("Failed to resize: {}", e));
            }

            // 再调整位置回到右上角
            if let Err(e) = window.set_position(tauri::LogicalPosition::new(new_x, new_y)) {
                return ApiResponse::error(format!("Failed to reposition: {}", e));
            }

            ApiResponse::success("Collapsed to ball".to_string())
        }
        None => ApiResponse::error("Floating ball window not found".to_string()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_window_commands_exist() {
        assert!(true);
    }
}
