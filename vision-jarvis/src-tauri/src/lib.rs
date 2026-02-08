// 模块声明
mod db;
mod settings;
mod capture;
mod ai;
mod memory;
mod notification;
mod commands;
mod storage;

use commands::{AppState, AIConfigState};
use tauri::{Manager, LogicalPosition};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化数据库
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("vision-jarvis")
        .join("vision-jarvis.db");

    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let db = db::Database::new(db_path).expect("Failed to create database");
    db.initialize().expect("Failed to initialize database");

    // 初始化设置管理器
    let settings_manager = settings::SettingsManager::new();

    // 创建应用状态
    let app_state = AppState::new(db, settings_manager);

    // 创建 AI 配置状态
    let ai_config_state = AIConfigState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(app_state)
        .manage(ai_config_state)
        .setup(|app| {
            // 设置悬浮球窗口位置到右上角
            if let Some(window) = app.get_webview_window("floating-ball") {
                // 获取主显示器尺寸
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let physical_size = monitor.size();
                    let scale_factor = monitor.scale_factor();

                    // 转换为逻辑像素
                    let logical_width = physical_size.width as f64 / scale_factor;

                    // 计算右上角位置：距右边缘 20px，距顶部 50px
                    // 确保完全在屏幕内
                    let window_width = 64.0;
                    let margin_right = 20.0;
                    let margin_top = 50.0;

                    let x = (logical_width - window_width - margin_right).max(0.0);
                    let y = margin_top;

                    println!("Setting window position: x={}, y={} (screen width={})", x, y, logical_width);
                    let _ = window.set_position(LogicalPosition::new(x, y));
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::health_check,
            // 截图相关
            commands::screenshot::capture_screenshot,
            commands::screenshot::get_screenshots,
            commands::screenshot::delete_screenshot,
            // 记忆相关
            commands::memory::search_memories,
            commands::memory::get_memories_by_date,
            commands::memory::generate_memory,
            // 通知相关
            commands::notification::get_pending_notifications,
            commands::notification::dismiss_notification,
            commands::notification::get_notification_history,
            // 设置相关
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
            // 存储管理相关
            commands::storage::get_storage_info,
            commands::storage::list_files,
            commands::storage::cleanup_old_files,
            commands::storage::delete_file,
            commands::storage::open_folder,
            // AI 配置相关
            commands::ai_config::get_ai_config_summary,
            commands::ai_config::get_ai_config,
            commands::ai_config::update_ai_api_key,
            commands::ai_config::update_ai_provider_config,
            commands::ai_config::set_active_ai_provider,
            commands::ai_config::test_ai_connection,
            commands::ai_config::get_available_ai_providers,
            commands::ai_config::reset_ai_config,
            // 窗口管理相关
            commands::window::open_memory_window,
            commands::window::open_popup_setting_window,
            commands::window::expand_to_header,
            commands::window::expand_to_asker,
            commands::window::collapse_to_ball,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
