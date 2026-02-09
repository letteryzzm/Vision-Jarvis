/// 文件存储相关 Commands

use tauri::State;
use super::{ApiResponse, AppState};
use crate::storage::{FolderType, StorageInfo, FileInfo};

/// 获取存储信息
#[tauri::command]
pub async fn get_storage_info(
    state: State<'_, AppState>,
) -> Result<ApiResponse<StorageInfo>, String> {
    let storage_path = (*state.settings).get_storage_path();
    
    match crate::storage::StorageManager::new(storage_path) {
        Ok(manager) => {
            match manager.get_storage_info() {
                Ok(info) => Ok(ApiResponse::success(info)),
                Err(e) => Ok(ApiResponse::error(format!("获取存储信息失败: {}", e))),
            }
        }
        Err(e) => Ok(ApiResponse::error(format!("创建存储管理器失败: {}", e))),
    }
}

/// 列出文件
#[tauri::command]
pub async fn list_files(
    state: State<'_, AppState>,
    folder_type: FolderType,
    limit: Option<usize>,
) -> Result<ApiResponse<Vec<FileInfo>>, String> {
    let storage_path = (*state.settings).get_storage_path();
    
    match crate::storage::StorageManager::new(storage_path) {
        Ok(manager) => {
            match manager.list_files(&folder_type, limit) {
                Ok(files) => Ok(ApiResponse::success(files)),
                Err(e) => Ok(ApiResponse::error(format!("列出文件失败: {}", e))),
            }
        }
        Err(e) => Ok(ApiResponse::error(format!("创建存储管理器失败: {}", e))),
    }
}

/// 清理旧文件
#[tauri::command]
pub async fn cleanup_old_files(
    state: State<'_, AppState>,
    folder_type: FolderType,
    days: u64,
) -> Result<ApiResponse<usize>, String> {
    let storage_path = (*state.settings).get_storage_path();
    
    match crate::storage::StorageManager::new(storage_path) {
        Ok(manager) => {
            match manager.cleanup_old_files(&folder_type, days) {
                Ok(count) => {
                    log::info!("清理了 {} 个旧文件", count);
                    Ok(ApiResponse::success(count))
                }
                Err(e) => Ok(ApiResponse::error(format!("清理文件失败: {}", e))),
            }
        }
        Err(e) => Ok(ApiResponse::error(format!("创建存储管理器失败: {}", e))),
    }
}

/// 删除单个文件
#[tauri::command]
pub async fn delete_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ApiResponse<bool>, String> {
    let storage_path = (*state.settings).get_storage_path();
    
    match crate::storage::StorageManager::new(storage_path) {
        Ok(manager) => {
            match manager.delete_file(&file_path) {
                Ok(_) => Ok(ApiResponse::success(true)),
                Err(e) => Ok(ApiResponse::error(format!("删除文件失败: {}", e))),
            }
        }
        Err(e) => Ok(ApiResponse::error(format!("创建存储管理器失败: {}", e))),
    }
}

/// 打开文件夹
#[tauri::command]
pub async fn open_folder(
    state: State<'_, AppState>,
    folder_type: FolderType,
) -> Result<ApiResponse<String>, String> {
    let storage_path = (*state.settings).get_storage_path();
    
    match crate::storage::StorageManager::new(storage_path) {
        Ok(manager) => {
            match manager.ensure_folder(&folder_type) {
                Ok(path) => {
                    let path_str = path.to_string_lossy().to_string();
                    
                    // 使用系统命令打开文件夹
                    #[cfg(target_os = "macos")]
                    {
                        std::process::Command::new("open")
                            .arg(&path)
                            .spawn()
                            .ok();
                    }
                    
                    #[cfg(target_os = "windows")]
                    {
                        std::process::Command::new("explorer")
                            .arg(&path)
                            .spawn()
                            .ok();
                    }
                    
                    #[cfg(target_os = "linux")]
                    {
                        std::process::Command::new("xdg-open")
                            .arg(&path)
                            .spawn()
                            .ok();
                    }
                    
                    Ok(ApiResponse::success(path_str))
                }
                Err(e) => Ok(ApiResponse::error(format!("打开文件夹失败: {}", e))),
            }
        }
        Err(e) => Ok(ApiResponse::error(format!("创建存储管理器失败: {}", e))),
    }
}
