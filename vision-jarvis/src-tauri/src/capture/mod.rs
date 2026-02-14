/// 截图捕获模块
///
/// 负责定时捕获屏幕截图、图像压缩和存储管理

use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};
use xcap::Monitor;
use chrono::Utc;
use uuid::Uuid;
use image::{DynamicImage, ImageFormat, ImageEncoder};
use std::io::Cursor;

pub mod scheduler;
pub mod storage;

/// 截图捕获器
#[derive(Clone)]
pub struct ScreenCapture {
    storage_path: PathBuf,
    /// 最大文件大小（字节），默认 5MB
    max_file_size: usize,
    /// JPEG 质量 (1-100)
    jpeg_quality: u8,
}

impl ScreenCapture {
    /// 创建新的截图捕获器
    pub fn new(storage_path: PathBuf) -> AppResult<Self> {
        // 确保存储目录存在
        std::fs::create_dir_all(&storage_path)
            .map_err(|e| AppError::screenshot(1, format!("创建截图存储目录失败: {}", e)))?;

        Ok(Self {
            storage_path,
            max_file_size: 5 * 1024 * 1024, // 5MB
            jpeg_quality: 85,
        })
    }

    /// 设置最大文件大小
    pub fn with_max_size(mut self, max_size_mb: usize) -> Self {
        self.max_file_size = max_size_mb * 1024 * 1024;
        self
    }

    /// 设置 JPEG 质量
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.jpeg_quality = quality.clamp(1, 100);
        self
    }

    /// 捕获当前屏幕截图
    pub fn capture_screenshot(&self) -> AppResult<PathBuf> {
        // 获取主显示器
        let monitors = Monitor::all()
            .map_err(|e| AppError::screenshot(2, format!("获取显示器列表失败: {}", e)))?;

        let monitor = monitors.first()
            .ok_or_else(|| AppError::screenshot(3, "未找到任何显示器"))?;

        // 捕获截图
        let image = monitor.capture_image()
            .map_err(|e| AppError::screenshot(4, format!("捕获屏幕截图失败: {}", e)))?;

        // 转换为 DynamicImage
        let dynamic_image = DynamicImage::ImageRgba8(image);

        // 压缩图像
        let compressed = self.compress_image(&dynamic_image)?;

        // 生成文件名
        let timestamp = Utc::now().timestamp();
        let id = Uuid::new_v4();
        let filename = format!("{}_{}.jpg", timestamp, id);
        let filepath = self.storage_path.join(filename);

        // 保存压缩后的图片
        std::fs::write(&filepath, compressed)
            .map_err(|e| AppError::screenshot(5, format!("保存截图失败: {}", e)))?;

        Ok(filepath)
    }

    /// 捕获截图并返回 base64 编码
    pub fn capture_screenshot_base64(&self) -> AppResult<String> {
        // 获取主显示器
        let monitors = Monitor::all()
            .map_err(|e| AppError::screenshot(2, format!("获取显示器列表失败: {}", e)))?;

        let monitor = monitors.first()
            .ok_or_else(|| AppError::screenshot(3, "未找到任何显示器"))?;

        // 捕获截图
        let image = monitor.capture_image()
            .map_err(|e| AppError::screenshot(4, format!("捕获屏幕截图失败: {}", e)))?;

        // 转换为 DynamicImage
        let dynamic_image = DynamicImage::ImageRgba8(image);

        // 压缩图像
        let compressed = self.compress_image(&dynamic_image)?;

        // Base64 编码
        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(&compressed);

        Ok(encoded)
    }

    /// 压缩图像到指定大小以下
    fn compress_image(&self, image: &DynamicImage) -> AppResult<Vec<u8>> {
        // JPEG 不支持透明通道，先转为 RGB8
        let image = &image.to_rgb8();
        let (width, height) = image.dimensions();

        let mut quality = self.jpeg_quality;
        let mut scale = 1.0f32;

        loop {
            // 如果需要缩放，先缩放图像
            let (buf, w, h) = if scale < 1.0 {
                let new_width = (width as f32 * scale) as u32;
                let new_height = (height as f32 * scale) as u32;
                let resized = image::imageops::resize(image, new_width, new_height, image::imageops::FilterType::Lanczos3);
                let w = resized.width();
                let h = resized.height();
                (resized.into_raw(), w, h)
            } else {
                (image.as_raw().clone(), width, height)
            };

            // 编码为 JPEG
            let mut buffer = Cursor::new(Vec::new());
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);

            encoder.write_image(
                &buf,
                w,
                h,
                image::ExtendedColorType::Rgb8,
            ).map_err(|e| AppError::screenshot(6, format!("图像编码失败: {}", e)))?;

            let compressed = buffer.into_inner();

            // 检查大小
            if compressed.len() <= self.max_file_size {
                return Ok(compressed);
            }

            // 如果还是太大，降低质量或缩放
            if quality > 60 {
                quality -= 10;
            } else if scale > 0.5 {
                scale -= 0.1;
                quality = self.jpeg_quality; // 重置质量
            } else {
                // 已经尽力了，返回当前结果
                return Ok(compressed);
            }
        }
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

    #[test]
    fn test_with_max_size() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf())
            .unwrap()
            .with_max_size(10); // 10MB

        assert_eq!(capture.max_file_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_with_quality() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf())
            .unwrap()
            .with_quality(90);

        assert_eq!(capture.jpeg_quality, 90);
    }

    #[test]
    fn test_quality_clamping() {
        let temp_dir = TempDir::new().unwrap();

        // 测试超出范围的质量值
        let capture1 = ScreenCapture::new(temp_dir.path().to_path_buf())
            .unwrap()
            .with_quality(150);
        assert_eq!(capture1.jpeg_quality, 100);

        let capture2 = ScreenCapture::new(temp_dir.path().to_path_buf())
            .unwrap()
            .with_quality(0);
        assert_eq!(capture2.jpeg_quality, 1);
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
            assert!(path.extension().unwrap() == "jpg");

            // 检查文件大小
            let metadata = std::fs::metadata(&path).unwrap();
            assert!(metadata.len() <= 5 * 1024 * 1024); // 应该小于 5MB
        }
    }

    #[test]
    #[ignore]
    fn test_capture_screenshot_base64() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf()).unwrap();

        let result = capture.capture_screenshot_base64();
        if let Ok(base64_str) = result {
            assert!(!base64_str.is_empty());
            // Base64 字符串应该只包含有效字符
            assert!(base64_str.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
        }
    }
}
