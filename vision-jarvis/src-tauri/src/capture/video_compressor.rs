/// 截图视频压缩器
///
/// 将一个时段目录内的 JPG 截图压缩为 mp4 视频
/// 依赖系统安装的 FFmpeg

use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use chrono::{Utc, Timelike, NaiveDateTime};
use serde::{Serialize, Deserialize};

/// 视频帧元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameInfo {
    /// 原始文件名
    pub filename: String,
    /// 截图时间戳 (Unix epoch)
    pub timestamp: i64,
    /// 在视频中的帧序号
    pub frame_index: u32,
}

/// 视频元数据（保存为 JSON）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    /// 视频文件名
    pub video_file: String,
    /// 日期 YYYYMMDD
    pub date: String,
    /// 时段
    pub period: String,
    /// 总帧数
    pub total_frames: u32,
    /// 帧率 (fps)
    pub fps: f32,
    /// 创建时间
    pub created_at: i64,
    /// 帧信息列表
    pub frames: Vec<FrameInfo>,
}

/// 检查 FFmpeg 是否可用
pub fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 压缩指定目录中的截图为 mp4 视频
///
/// 成功后删除原始截图文件，保留 mp4 和 JSON 元数据
pub fn compress_period_screenshots(period_dir: &Path) -> Result<Option<PathBuf>> {
    if !period_dir.exists() || !period_dir.is_dir() {
        return Ok(None);
    }

    // 收集所有 JPG 文件并按文件名排序
    let mut jpg_files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(period_dir).context("读取时段目录失败")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "jpg" || ext == "jpeg") {
            jpg_files.push(path);
        }
    }

    if jpg_files.len() < 2 {
        // 太少截图，不值得压缩
        return Ok(None);
    }

    jpg_files.sort();

    // 构建帧信息
    let mut frames: Vec<FrameInfo> = Vec::new();
    for (i, path) in jpg_files.iter().enumerate() {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        // 从文件名提取时间: HH-MM-SS_{uuid}.jpg
        let timestamp = parse_screenshot_timestamp(&filename, period_dir);
        frames.push(FrameInfo {
            filename: filename.clone(),
            timestamp,
            frame_index: i as u32,
        });
    }

    // 创建临时文件列表供 FFmpeg 使用
    let file_list_path = period_dir.join("_ffmpeg_input.txt");
    let file_list_content: String = jpg_files
        .iter()
        .map(|p| format!("file '{}'\nduration 1", p.to_string_lossy()))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&file_list_path, &file_list_content)?;

    // 生成视频文件名: 使用第一帧的时间
    let first_frame_name = jpg_files[0]
        .file_stem()
        .unwrap()
        .to_string_lossy();
    let video_name = first_frame_name.split('_').next().unwrap_or("video");
    let video_path = period_dir.join(format!("{}.mp4", video_name));
    let json_path = period_dir.join(format!("{}.json", video_name));

    // 使用 FFmpeg 编码
    let ffmpeg_result = Command::new("ffmpeg")
        .args([
            "-y",                           // 覆盖已存在的文件
            "-f", "concat",                 // 使用 concat 模式
            "-safe", "0",                   // 允许绝对路径
            "-i", &file_list_path.to_string_lossy(),
            "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2", // 确保尺寸为偶数
            "-c:v", "libx264",              // H.264 编码
            "-preset", "fast",              // 编码速度
            "-crf", "28",                   // 质量（较高值=更小文件）
            "-pix_fmt", "yuv420p",          // 兼容性
            &video_path.to_string_lossy(),
        ])
        .output()
        .context("执行 FFmpeg 失败")?;

    // 清理临时文件
    let _ = fs::remove_file(&file_list_path);

    if !ffmpeg_result.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_result.stderr);
        bail!("FFmpeg 编码失败: {}", stderr);
    }

    // 解析日期和时段信息
    let period_name = period_dir
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let date_name = period_dir
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // 写入 JSON 元数据
    let metadata = VideoMetadata {
        video_file: video_path.file_name().unwrap().to_string_lossy().to_string(),
        date: date_name,
        period: period_name,
        total_frames: frames.len() as u32,
        fps: 1.0,
        created_at: Utc::now().timestamp(),
        frames,
    };

    let json_content = serde_json::to_string_pretty(&metadata)?;
    fs::write(&json_path, json_content)?;

    // 删除原始截图
    for path in &jpg_files {
        let _ = fs::remove_file(path);
    }

    eprintln!(
        "[VideoCompressor] Compressed {} screenshots into {}",
        jpg_files.len(),
        video_path.display()
    );

    Ok(Some(video_path))
}

/// 压缩所有已完成时段的截图
///
/// 只处理已过去的时段（不处理当前时段）
pub fn compress_completed_periods(storage_path: &Path) -> Result<Vec<PathBuf>> {
    let shots_dir = storage_path.join("shots");
    if !shots_dir.exists() {
        return Ok(Vec::new());
    }

    if !is_ffmpeg_available() {
        eprintln!("[VideoCompressor] FFmpeg not found, skipping compression");
        return Ok(Vec::new());
    }

    let now = Utc::now();
    let current_date = format!("{}", now.format("%Y%m%d"));
    let current_period = super::get_time_period(now.hour());

    let mut compressed: Vec<PathBuf> = Vec::new();

    // 遍历所有日期目录
    let mut date_dirs: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&shots_dir)? {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            date_dirs.push(entry.path());
        }
    }
    date_dirs.sort();

    for date_dir in &date_dirs {
        let date_name = date_dir.file_name().unwrap().to_string_lossy().to_string();

        // 遍历时段目录
        for entry in fs::read_dir(date_dir)? {
            let entry = entry?;
            if !entry.metadata()?.is_dir() {
                continue;
            }

            let period_name = entry.file_name().to_string_lossy().to_string();

            // 跳过当前正在使用的时段
            if date_name == current_date && period_name == current_period {
                continue;
            }

            // 检查该目录是否已有 mp4 文件（已压缩）
            let has_mp4 = fs::read_dir(entry.path())?
                .filter_map(|e| e.ok())
                .any(|e| e.path().extension().map_or(false, |ext| ext == "mp4"));

            if has_mp4 {
                continue;
            }

            // 压缩该时段
            match compress_period_screenshots(&entry.path()) {
                Ok(Some(video_path)) => compressed.push(video_path),
                Ok(None) => {}
                Err(e) => {
                    eprintln!("[VideoCompressor] Failed to compress {}: {}", entry.path().display(), e);
                }
            }
        }
    }

    Ok(compressed)
}

/// 从文件名解析截图时间戳
fn parse_screenshot_timestamp(filename: &str, period_dir: &Path) -> i64 {
    // 文件名格式: HH-MM-SS_{uuid}.jpg
    // 从父目录推断日期: .../YYYYMMDD/period/
    let date_str = period_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("20000101");

    let time_part = filename.split('_').next().unwrap_or("00-00-00");
    let time_parts: Vec<&str> = time_part.split('-').collect();

    if date_str.len() == 8 && time_parts.len() == 3 {
        let datetime_str = format!(
            "{} {}:{}:{}",
            date_str,
            time_parts[0],
            time_parts[1],
            time_parts[2]
        );
        if let Ok(dt) = NaiveDateTime::parse_from_str(&datetime_str, "%Y%m%d %H:%M:%S") {
            return dt.and_utc().timestamp();
        }
    }

    Utc::now().timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_is_ffmpeg_available() {
        // 只验证函数不 panic
        let _ = is_ffmpeg_available();
    }

    #[test]
    fn test_parse_screenshot_timestamp() {
        let dir = PathBuf::from("/tmp/shots/20231027/0_00-12_00");
        let ts = parse_screenshot_timestamp("10-30-45_abc.jpg", &dir);
        // 2023-10-27 10:30:45 UTC
        assert_eq!(ts, 1698402645);
    }

    #[test]
    fn test_compress_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = compress_period_screenshots(temp_dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_compress_too_few_screenshots() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("10-00-00_test.jpg");
        let mut f = fs::File::create(&file).unwrap();
        f.write_all(&[0u8; 100]).unwrap();

        let result = compress_period_screenshots(temp_dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_video_metadata_serialization() {
        let metadata = VideoMetadata {
            video_file: "10-00-00.mp4".to_string(),
            date: "20231027".to_string(),
            period: "0_00-12_00".to_string(),
            total_frames: 2,
            fps: 1.0,
            created_at: 1698400000,
            frames: vec![
                FrameInfo {
                    filename: "10-00-00_abc.jpg".to_string(),
                    timestamp: 1698400000,
                    frame_index: 0,
                },
                FrameInfo {
                    filename: "10-00-05_def.jpg".to_string(),
                    timestamp: 1698400005,
                    frame_index: 1,
                },
            ],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("10-00-00.mp4"));
        assert!(json.contains("20231027"));

        let deserialized: VideoMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_frames, 2);
        assert_eq!(deserialized.frames.len(), 2);
    }
}
