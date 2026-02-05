/// 截图捕获模块
///
/// 负责定时捕获屏幕截图和存储管理

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use xcap::Monitor;
use chrono::Utc;
use uuid::Uuid;

pub mod scheduler;
pub mod storage;

/// 截图捕获器
pub struct ScreenCapture {
    storage_path: PathBuf,
}

impl ScreenCapture {
    /// 创建新的截图捕获器
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        // 确保存储目录存在
        std::fs::create_dir_all(&storage_path)
            .context("创建截图存储目录失败")?;

        Ok(Self { storage_path })
    }

    /// 捕获当前屏幕截图
    pub fn capture_screenshot(&self) -> Result<PathBuf> {
        // 获取主显示器
        let monitors = Monitor::all()
            .context("获取显示器列表失败")?;

        let monitor = monitors.first()
            .context("未找到任何显示器")?;

        // 捕获截图
        let image = monitor.capture_image()
            .context("捕获屏幕截图失败")?;

        // 生成文件名
        let timestamp = Utc::now().timestamp();
        let id = Uuid::new_v4();
        let filename = format!("{}_{}.png", timestamp, id);
        let filepath = self.storage_path.join(filename);

        // 保存图片
        image.save(&filepath)
            .context("保存截图失败")?;

        Ok(filepath)
    }

    /// 获取存储路径
    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_screen_capture_creation() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf());

        assert!(capture.is_ok());
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_storage_path() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(capture.storage_path(), temp_dir.path());
    }

    // 注意：实际截图测试在 CI 环境中可能失败（无显示器）
    // 在本地开发环境中可以手动测试
    #[test]
    #[ignore]
    fn test_capture_screenshot() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf()).unwrap();

        let result = capture.capture_screenshot();
        if let Ok(path) = result {
            assert!(path.exists());
            assert!(path.extension().unwrap() == "png");
        }
    }
}
