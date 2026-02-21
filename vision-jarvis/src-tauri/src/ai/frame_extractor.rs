use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use log::{info, warn};
use std::process::Command;
use tempfile::TempDir;

use crate::error::{AppError, AppResult};

/// 帧提取配置
pub struct FrameExtractConfig {
    pub num_frames: usize,
    pub scale_width: u32,
    pub jpeg_quality: u32,
}

impl Default for FrameExtractConfig {
    fn default() -> Self {
        Self {
            num_frames: 5,
            scale_width: 1280,
            jpeg_quality: 3,
        }
    }
}

/// 从 base64 编码的视频中提取帧
///
/// 使用 ffmpeg 均匀提取帧，返回 base64 编码的 JPEG 图像列表。
/// 需要系统安装 ffmpeg 和 ffprobe。
pub fn extract_frames(video_base64: &str, config: &FrameExtractConfig) -> AppResult<Vec<String>> {
    let ffmpeg = find_ffmpeg()?;
    let ffprobe = find_ffprobe()?;

    let tmp_dir = TempDir::new()
        .map_err(|e| AppError::io(1, format!("创建临时目录失败: {}", e)))?;

    let video_path = tmp_dir.path().join("input.mp4");
    let video_data = BASE64.decode(video_base64)
        .map_err(|e| AppError::ai(10, format!("解码 base64 视频失败: {}", e)))?;
    std::fs::write(&video_path, &video_data)
        .map_err(|e| AppError::io(2, format!("写入临时视频文件失败: {}", e)))?;

    let duration = get_video_duration(&ffprobe, &video_path);

    let mut frames = Vec::with_capacity(config.num_frames);
    for i in 0..config.num_frames {
        let t = duration * (i as f64 + 0.5) / config.num_frames as f64;
        let frame_path = tmp_dir.path().join(format!("frame_{:03}.jpg", i));

        let result = Command::new(&ffmpeg)
            .args([
                "-ss", &format!("{:.2}", t),
                "-i", video_path.to_str().unwrap_or("input.mp4"),
                "-vframes", "1",
                "-vf", &format!("scale={}:-1", config.scale_width),
                "-q:v", &config.jpeg_quality.to_string(),
                "-y", frame_path.to_str().unwrap_or("frame.jpg"),
                "-v", "quiet",
            ])
            .output();

        match result {
            Ok(output) if output.status.success() && frame_path.exists() => {
                match std::fs::read(&frame_path) {
                    Ok(data) => frames.push(BASE64.encode(&data)),
                    Err(e) => warn!("[FrameExtractor] 读取帧文件失败: {}", e),
                }
            }
            Ok(output) => {
                warn!("[FrameExtractor] ffmpeg 提取帧 {} 失败: exit={}", i, output.status);
            }
            Err(e) => {
                warn!("[FrameExtractor] ffmpeg 执行失败: {}", e);
            }
        }
    }

    if frames.is_empty() {
        return Err(AppError::ai(11, "帧提取失败: 未能提取任何帧"));
    }

    info!("[FrameExtractor] 成功提取 {}/{} 帧", frames.len(), config.num_frames);
    Ok(frames)
}

/// 使用 ffprobe 获取视频时长（秒）
fn get_video_duration(ffprobe: &str, video_path: &std::path::Path) -> f64 {
    let result = Command::new(ffprobe)
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            video_path.to_str().unwrap_or(""),
        ])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            serde_json::from_str::<serde_json::Value>(&stdout)
                .ok()
                .and_then(|v| v["format"]["duration"].as_str()?.parse::<f64>().ok())
                .unwrap_or(60.0)
        }
        _ => {
            warn!("[FrameExtractor] ffprobe 获取时长失败，使用默认 60s");
            60.0
        }
    }
}

/// 查找 ffmpeg 可执行文件
fn find_ffmpeg() -> AppResult<String> {
    find_executable("ffmpeg")
}

/// 查找 ffprobe 可执行文件
fn find_ffprobe() -> AppResult<String> {
    find_executable("ffprobe")
}

fn find_executable(name: &str) -> AppResult<String> {
    // 检查 PATH
    if let Ok(output) = Command::new("which").arg(name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
    }

    // macOS 常见位置
    let candidates = [
        format!("/opt/homebrew/bin/{}", name),
        format!("/usr/local/bin/{}", name),
        format!("/usr/bin/{}", name),
    ];

    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return Ok(candidate.clone());
        }
    }

    Err(AppError::ai(12, format!(
        "{} 未找到。请安装 ffmpeg: brew install ffmpeg", name
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_ffmpeg() {
        // 在 CI 环境中 ffmpeg 可能不存在，所以不 assert 结果
        let result = find_ffmpeg();
        if let Ok(path) = &result {
            assert!(!path.is_empty());
        }
    }

    #[test]
    fn test_default_config() {
        let config = FrameExtractConfig::default();
        assert_eq!(config.num_frames, 5);
        assert_eq!(config.scale_width, 1280);
        assert_eq!(config.jpeg_quality, 3);
    }
}
