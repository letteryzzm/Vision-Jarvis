// 模块声明
mod db;
mod settings;
mod capture;
mod ai;
mod memory;
mod notification;
mod commands;

use commands::AppState;

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
