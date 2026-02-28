// 模块声明
mod error;
mod db;
mod settings;
mod capture;
pub mod ai;
mod memory;
mod notification;
mod commands;
mod storage;

// 导出错误类型供其他模块使用
pub use error::{AppError, AppResult};

use commands::{AppState, AIConfigState};
use tauri::{Manager, LogicalPosition};
use log::{info, error};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

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

    // 创建 AI 配置状态（从数据库加载已保存的配置）
    let ai_config_state = AIConfigState::new(app_state.db.clone());

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

                    println!("Window position: x={}, y={}", x, y);
                    let _ = window.set_position(LogicalPosition::new(x, y));
                }
            }

            // 启动录制调度器（如果记忆功能启用）
            let state = app.state::<AppState>();
            let memory_enabled = state.settings.is_memory_enabled();
            let storage_path = state.settings.get_storage_path();
            info!("memory_enabled={}, storage={}", memory_enabled, storage_path.display());

            // 确保所有标准文件夹存在
            if let Ok(manager) = crate::storage::StorageManager::new(storage_path.clone()) {
                if let Err(e) = manager.ensure_all_folders() {
                    error!("创建标准文件夹失败: {}", e);
                }
            }

            if memory_enabled {
                let scheduler = state.scheduler.clone();

                tauri::async_runtime::spawn(async move {
                    let mut scheduler = scheduler.lock().await;
                    info!("Starting recorder (segment: {}s)", scheduler.interval_seconds);
                    match scheduler.start().await {
                        Ok(_) => info!("Recorder started"),
                        Err(e) => error!("Failed to start recorder: {}", e),
                    }
                });
            } else {
                info!("Memory disabled, recorder skipped");
            }

            // 启动通知调度器
            let notif_scheduler = state.notification_scheduler.clone();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                notif_scheduler.start(app_handle);
                info!("Notification scheduler started");
            });

            // 启动记忆管道调度器，并尝试自动连接 AI
            if memory_enabled {
                let pipeline = state.pipeline.clone();
                let ai_state = app.state::<AIConfigState>();
                let ai_provider = ai_state.get_active_provider_config();

                tauri::async_runtime::spawn(async move {
                    // 如果已有 AI 配置，自动连接到管道
                    if let Some(provider) = ai_provider {
                        match crate::ai::AIClient::new(provider.clone()) {
                            Ok(client) => {
                                pipeline.connect_ai(client).await;
                                pipeline.start();
                                info!("Pipeline started (AI: {} / {})", provider.name, provider.model);
                            }
                            Err(e) => {
                                error!("Failed to create AI client: {}", e);
                                pipeline.start();
                                info!("Pipeline started (AI connection failed)");
                            }
                        }
                    } else {
                        pipeline.start();
                        info!("Pipeline started (no AI configured)");
                    }
                });
            }

            // 启动鼠标 Idle 检测 + 回归提醒
            {
                let idle_settings = state.settings.get();
                if idle_settings.idle_reminder_enabled {
                    let db = state.db.clone();
                    let ai_state = app.state::<AIConfigState>();
                    let provider_config = ai_state.get_active_provider_config();
                    let app_handle_idle = app.handle().clone();
                    let threshold = idle_settings.idle_threshold_secs;
                    let min_trigger = idle_settings.idle_min_trigger_secs;

                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new()
                            .expect("Failed to create tokio runtime for IdleWatcher");

                        let advisor = std::sync::Arc::new(
                            crate::notification::return_advisor::ReturnAdvisor::new(
                                db,
                                provider_config,
                            )
                        );

                        let watcher = crate::capture::idle_watcher::IdleWatcher::new(
                            threshold,
                            min_trigger,
                        );

                        watcher.start(move |idle_secs| {
                            let advisor = advisor.clone();
                            let app = app_handle_idle.clone();
                            rt.spawn(async move {
                                if let Some(msg) = advisor.generate_return_hint(idle_secs).await {
                                    let notif = crate::notification::Notification::new(
                                        crate::notification::NotificationType::ReturnReminder,
                                        crate::notification::NotificationPriority::Normal,
                                        "欢迎回来".to_string(),
                                        msg,
                                    );
                                    let _ = crate::notification::delivery::send_system_notification(
                                        &app, &notif,
                                    );
                                    let _ = crate::notification::delivery::emit_notification_event(
                                        &app, &notif,
                                    );
                                }
                            });
                        });
                    });

                    info!("IdleWatcher started (threshold={}s, min_trigger={}s)", threshold, min_trigger);
                } else {
                    info!("IdleWatcher disabled in settings");
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::health_check,
            // 录制相关
            commands::recording::get_scheduler_status,
            // 记忆相关
            commands::memory::get_activities,
            commands::memory::get_activity_detail,
            commands::memory::get_projects,
            commands::memory::get_habits,
            commands::memory::get_summary,
            commands::memory::get_recording_stats,
            commands::memory::search_memories,
            commands::memory::trigger_daily_summary,
            // 通知相关
            commands::notification::get_pending_notifications,
            commands::notification::dismiss_notification,
            commands::notification::get_notification_history,
            commands::notification::respond_to_suggestion,
            commands::notification::get_suggestion_history,
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
            commands::ai_config::delete_ai_provider,
            commands::ai_config::reset_ai_config,
            commands::ai_config::connect_ai_to_pipeline,
            commands::ai_config::get_pipeline_status,
            // 窗口管理相关
            commands::window::open_memory_window,
            commands::window::open_popup_setting_window,
            commands::window::expand_to_header,
            commands::window::expand_to_asker,
            commands::window::collapse_to_ball,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let state = app_handle.state::<AppState>();
                let scheduler = state.scheduler.clone();
                tauri::async_runtime::block_on(async {
                    let mut s = scheduler.lock().await;
                    if s.is_running().await {
                        info!("App exiting, stopping recorder...");
                        let _ = s.stop().await;
                        info!("Recorder stopped");
                    }
                });
            }
        });
}
