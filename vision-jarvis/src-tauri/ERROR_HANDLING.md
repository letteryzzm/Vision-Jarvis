# 错误处理系统使用指南

## 概述

Vision-Jarvis 使用统一的错误处理系统，基于 `thiserror` 提供类型安全的错误定义和用户友好的错误消息。

## 错误类型

所有错误都通过 `AppError` 枚举定义，包含以下类别：

- `Screenshot` - 截图相关错误
- `AI` - AI 模型调用错误
- `Settings` - 设置管理错误
- `Storage` - 文件存储错误
- `Database` - 数据库操作错误
- `Network` - 网络请求错误
- `IO` - 文件系统 IO 错误
- `Serde` - 序列化/反序列化错误
- `Validation` - 数据验证错误
- `Permission` - 权限相关错误
- `Unknown` - 未知错误

## 错误码格式

所有错误消息遵循统一格式：

```
[ERR_{模块}_{编号}] 错误描述
```

示例：
- `[ERR_SCREENSHOT_001] 截图失败: 无法访问屏幕`
- `[ERR_AI_401] API Key 无效或未授权`
- `[ERR_SETTINGS_001] 配置文件读取失败`

## 使用方法

### 1. 创建错误

```rust
use crate::error::AppError;

// 方式 1: 使用构造函数
let err = AppError::screenshot(1, "截图失败");

// 方式 2: 使用枚举变体
let err = AppError::Screenshot(1, "截图失败".to_string());
```

### 2. 在函数中返回错误

```rust
use crate::error::{AppError, AppResult};

fn capture_screen() -> AppResult<Vec<u8>> {
    // 如果失败，返回错误
    if !has_permission() {
        return Err(AppError::permission(1, "未授予屏幕录制权限"));
    }

    // 成功返回数据
    Ok(image_data)
}
```

### 3. 错误转换

标准库错误会自动转换为 `AppError`：

```rust
use std::fs;

fn read_config() -> AppResult<String> {
    // std::io::Error 自动转换为 AppError::IO
    let content = fs::read_to_string("config.json")?;
    Ok(content)
}
```

支持的自动转换：
- `std::io::Error` → `AppError::IO` 或 `AppError::Permission`
- `serde_json::Error` → `AppError::Serde`
- `reqwest::Error` → `AppError::Network`
- `rusqlite::Error` → `AppError::Database`
- `anyhow::Error` → `AppError::Unknown`

### 4. 在 Tauri 命令中使用

```rust
use crate::commands::ApiResponse;
use crate::error::AppError;

#[tauri::command]
async fn my_command() -> Result<ApiResponse<String>, String> {
    match do_something() {
        Ok(data) => Ok(ApiResponse::success(data)),
        Err(e) => {
            // AppError 自动转换为用户友好的错误消息
            Ok(ApiResponse::error(e.to_string()))
        }
    }
}

// 或者使用 From trait
#[tauri::command]
async fn my_command_v2() -> Result<ApiResponse<String>, String> {
    let result = do_something();
    Ok(result.into()) // 自动转换为 ApiResponse
}
```

## 错误码规范

### Screenshot 模块 (001-099)
- 001: 截图失败
- 002: 图像压缩失败
- 003: Base64 编码失败
- 004: 屏幕录制权限未授予

### AI 模块 (001-099)
- 001: API 调用失败
- 002: Prompt 生成失败
- 401: API Key 无效
- 429: 请求过于频繁

### Settings 模块 (001-099)
- 001: 配置文件读取失败
- 002: 配置文件格式错误
- 003: 配置保存失败

### Storage 模块 (001-099)
- 001: 文件夹不存在
- 002: 文件删除失败
- 003: 文件清理失败

### Validation 模块 (001-099)
- 001: 参数验证失败
- 002: 数据格式错误

## 最佳实践

1. **使用具体的错误码**: 为每种错误场景分配唯一的错误码
2. **提供清晰的错误消息**: 错误消息应该告诉用户发生了什么以及如何解决
3. **使用 `?` 操作符**: 利用自动错误转换简化代码
4. **记录错误日志**: 在返回错误前记录详细的错误信息用于调试
5. **避免暴露敏感信息**: 错误消息不应包含 API Key、密码等敏感数据

## 示例

### 完整的模块错误处理

```rust
use crate::error::{AppError, AppResult};
use std::fs;

pub struct ScreenshotManager {
    storage_path: PathBuf,
}

impl ScreenshotManager {
    pub fn capture(&self) -> AppResult<Vec<u8>> {
        // 检查权限
        if !self.has_permission() {
            return Err(AppError::screenshot(4, "未授予屏幕录制权限"));
        }

        // 捕获屏幕
        let image = self.do_capture()
            .map_err(|e| AppError::screenshot(1, format!("截图失败: {}", e)))?;

        // 压缩图像
        let compressed = self.compress(image)
            .map_err(|e| AppError::screenshot(2, format!("图像压缩失败: {}", e)))?;

        Ok(compressed)
    }

    pub fn save(&self, data: &[u8]) -> AppResult<String> {
        let path = self.storage_path.join("screenshot.png");

        // IO 错误自动转换
        fs::write(&path, data)?;

        Ok(path.to_string_lossy().to_string())
    }
}
```

## 测试

错误系统包含完整的单元测试，运行：

```bash
cargo test error::tests
```

所有测试应该通过，确保错误转换和消息格式正确。
