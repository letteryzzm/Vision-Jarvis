/// Tauri Commands
///
/// 前后端通信接口

use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::sync::Arc;
use crate::db::Database;
use crate::settings::SettingsManager;
use crate::capture::ScreenCapture;

pub mod screenshot;
pub mod memory;
pub mod notification;
pub mod settings;

/// 应用状态
pub struct AppState {
    pub db: Arc<Database>,
    pub settings: Arc<SettingsManager>,
    pub screen_capture: Arc<ScreenCapture>,
}

impl AppState {
    pub fn new(db: Database, settings: SettingsManager) -> Self {
        let settings = Arc::new(settings);
        let storage_path = settings.get_storage_path();
        let screen_capture = Arc::new(
            ScreenCapture::new(storage_path)
                .expect("Failed to create ScreenCapture")
        );

        Self {
            db: Arc::new(db),
            settings,
            screen_capture,
        }
    }
}

/// 通用响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// 将 Result 转换为 ApiResponse
impl<T, E: std::fmt::Display> From<Result<T, E>> for ApiResponse<T> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(e) => Self::error(e.to_string()),
        }
    }
}

/// 健康检查
#[tauri::command]
pub async fn health_check() -> ApiResponse<String> {
    ApiResponse::success("OK".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test data".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert!(response.success);
        assert_eq!(response.data, Some("OK".to_string()));
    }
}
