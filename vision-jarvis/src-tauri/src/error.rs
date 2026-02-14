/// 统一错误处理系统
///
/// 提供类型安全的错误定义和用户友好的错误消息

use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 截图相关错误
    #[error("[ERR_SCREENSHOT_{0:03}] {1}")]
    Screenshot(u16, String),

    /// AI 相关错误
    #[error("[ERR_AI_{0:03}] {1}")]
    AI(u16, String),

    /// 设置相关错误
    #[error("[ERR_SETTINGS_{0:03}] {1}")]
    Settings(u16, String),

    /// 存储相关错误
    #[error("[ERR_STORAGE_{0:03}] {1}")]
    Storage(u16, String),

    /// 数据库相关错误
    #[error("[ERR_DATABASE_{0:03}] {1}")]
    Database(u16, String),

    /// 网络相关错误
    #[error("[ERR_NETWORK_{0:03}] {1}")]
    Network(u16, String),

    /// IO 相关错误
    #[error("[ERR_IO_{0:03}] {1}")]
    IO(u16, String),

    /// 序列化/反序列化错误
    #[error("[ERR_SERDE_{0:03}] {1}")]
    Serde(u16, String),

    /// 验证错误
    #[error("[ERR_VALIDATION_{0:03}] {1}")]
    Validation(u16, String),

    /// 权限错误
    #[error("[ERR_PERMISSION_{0:03}] {1}")]
    Permission(u16, String),

    /// 未知错误
    #[error("[ERR_UNKNOWN] {0}")]
    Unknown(String),
}

impl AppError {
    /// 创建截图错误
    pub fn screenshot(code: u16, msg: impl Into<String>) -> Self {
        Self::Screenshot(code, msg.into())
    }

    /// 创建 AI 错误
    pub fn ai(code: u16, msg: impl Into<String>) -> Self {
        Self::AI(code, msg.into())
    }

    /// 创建设置错误
    pub fn settings(code: u16, msg: impl Into<String>) -> Self {
        Self::Settings(code, msg.into())
    }

    /// 创建存储错误
    pub fn storage(code: u16, msg: impl Into<String>) -> Self {
        Self::Storage(code, msg.into())
    }

    /// 创建数据库错误
    pub fn database(code: u16, msg: impl Into<String>) -> Self {
        Self::Database(code, msg.into())
    }

    /// 创建网络错误
    pub fn network(code: u16, msg: impl Into<String>) -> Self {
        Self::Network(code, msg.into())
    }

    /// 创建 IO 错误
    pub fn io(code: u16, msg: impl Into<String>) -> Self {
        Self::IO(code, msg.into())
    }

    /// 创建序列化错误
    pub fn serde(code: u16, msg: impl Into<String>) -> Self {
        Self::Serde(code, msg.into())
    }

    /// 创建验证错误
    pub fn validation(code: u16, msg: impl Into<String>) -> Self {
        Self::Validation(code, msg.into())
    }

    /// 创建权限错误
    pub fn permission(code: u16, msg: impl Into<String>) -> Self {
        Self::Permission(code, msg.into())
    }
}

/// 从标准 IO 错误转换
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::io(1, "文件或目录不存在"),
            std::io::ErrorKind::PermissionDenied => Self::permission(1, "权限不足"),
            std::io::ErrorKind::AlreadyExists => Self::io(2, "文件或目录已存在"),
            _ => Self::io(999, format!("IO 错误: {}", err)),
        }
    }
}

/// 从 serde_json 错误转换
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::serde(1, format!("JSON 序列化错误: {}", err))
    }
}

/// 从 reqwest 错误转换
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::network(1, "请求超时")
        } else if err.is_connect() {
            Self::network(2, "网络连接失败")
        } else if let Some(status) = err.status() {
            match status.as_u16() {
                401 => Self::network(401, "API Key 无效或未授权"),
                403 => Self::network(403, "访问被拒绝"),
                404 => Self::network(404, "API 端点不存在"),
                429 => Self::network(429, "请求过于频繁，请稍后重试"),
                500..=599 => Self::network(500, "服务器错误"),
                _ => Self::network(999, format!("HTTP 错误: {}", status)),
            }
        } else {
            Self::network(999, format!("网络错误: {}", err))
        }
    }
}

/// 从 anyhow 错误转换
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::Unknown(err.to_string())
    }
}

/// 从 rusqlite 错误转换
impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::SqliteFailure(_, Some(msg)) => {
                Self::database(1, format!("数据库错误: {}", msg))
            }
            rusqlite::Error::QueryReturnedNoRows => {
                Self::database(2, "查询未返回结果")
            }
            _ => Self::database(999, format!("数据库错误: {}", err)),
        }
    }
}

/// 应用 Result 类型别名
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_error() {
        let err = AppError::screenshot(1, "截图失败");
        assert_eq!(err.to_string(), "[ERR_SCREENSHOT_001] 截图失败");
    }

    #[test]
    fn test_ai_error() {
        let err = AppError::ai(401, "API Key 无效");
        assert_eq!(err.to_string(), "[ERR_AI_401] API Key 无效");
    }

    #[test]
    fn test_settings_error() {
        let err = AppError::settings(1, "配置文件读取失败");
        assert_eq!(err.to_string(), "[ERR_SETTINGS_001] 配置文件读取失败");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(app_err.to_string().contains("ERR_IO"));
        assert!(app_err.to_string().contains("文件或目录不存在"));
    }

    #[test]
    fn test_permission_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let app_err: AppError = io_err.into();
        assert!(app_err.to_string().contains("ERR_PERMISSION"));
        assert!(app_err.to_string().contains("权限不足"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{invalid json}";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let app_err: AppError = json_err.into();
        assert!(app_err.to_string().contains("ERR_SERDE"));
        assert!(app_err.to_string().contains("JSON 序列化错误"));
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::validation(1, "截图频率必须在 1-15 秒之间");
        assert_eq!(
            err.to_string(),
            "[ERR_VALIDATION_001] 截图频率必须在 1-15 秒之间"
        );
    }

    #[test]
    fn test_network_error() {
        let err = AppError::network(429, "请求过于频繁");
        assert_eq!(err.to_string(), "[ERR_NETWORK_429] 请求过于频繁");
    }

    #[test]
    fn test_database_error_conversion() {
        let db_err = rusqlite::Error::QueryReturnedNoRows;
        let app_err: AppError = db_err.into();
        assert!(app_err.to_string().contains("ERR_DATABASE"));
        assert!(app_err.to_string().contains("查询未返回结果"));
    }
}
