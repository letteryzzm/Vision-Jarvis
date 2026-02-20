/// 屏幕录制器
///
/// 使用 FFmpeg avfoundation 进行 macOS 屏幕分段录制

use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{Utc, Timelike};
use log::{debug, error, info};
use uuid::Uuid;

/// 查找 macOS avfoundation 屏幕录制设备索引
fn find_screen_device_index() -> Option<u32> {
    let output = Command::new("ffmpeg")
        .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        // 匹配 "[AVFoundation ...] [4] Capture screen 0" 格式
        // 用 rfind 从右往左找，跳过 AVFoundation 前缀的括号
        if line.contains("Capture screen") {
            if let Some(end) = line.rfind(']') {
                if let Some(start) = line[..end].rfind('[') {
                    if let Ok(idx) = line[start + 1..end].parse::<u32>() {
                        return Some(idx);
                    }
                }
            }
        }
    }
    None
}

/// 根据小时返回时段目录名
fn get_time_period(hour: u32) -> &'static str {
    match hour {
        0..=11 => "0_00-12_00",
        12..=17 => "12_00-18_00",
        _ => "18_00-24_00",
    }
}

/// 屏幕录制器
pub struct ScreenRecorder {
    storage_path: PathBuf,
    segment_duration_secs: u64,
    fps: u8,
    screen_device_index: u32,
    current_process: Arc<Mutex<Option<Child>>>,
}

impl ScreenRecorder {
    pub fn new(storage_path: PathBuf, segment_duration_secs: u64, fps: u8) -> AppResult<Self> {
        std::fs::create_dir_all(&storage_path)
            .map_err(|e| AppError::capture(1, format!("创建录制存储目录失败: {}", e)))?;

        let screen_device_index = find_screen_device_index()
            .ok_or_else(|| AppError::capture(1, "未找到屏幕捕获设备"))?;
        info!("Screen capture device index: {}", screen_device_index);

        Ok(Self {
            storage_path,
            segment_duration_secs,
            fps,
            screen_device_index,
            current_process: Arc::new(Mutex::new(None)),
        })
    }

    /// 启动一个录制分段，返回输出文件路径
    pub async fn start_segment(&self) -> AppResult<PathBuf> {
        let now = Utc::now();
        let date_dir = format!("{}", now.format("%Y%m%d"));
        let period_dir = get_time_period(now.hour());
        let rec_dir = self.storage_path.join("recordings").join(&date_dir).join(period_dir);

        std::fs::create_dir_all(&rec_dir)
            .map_err(|e| AppError::capture(1, format!("创建录制目录失败: {}", e)))?;

        let id = Uuid::new_v4();
        let filename = format!("{}_{}.mp4", now.format("%H-%M-%S"), id);
        let output_path = rec_dir.join(&filename);

        let child = Command::new("ffmpeg")
            .args([
                "-f", "avfoundation",
                "-framerate", &self.fps.to_string(),
                "-i", &format!("{}:none", self.screen_device_index),
                "-t", &self.segment_duration_secs.to_string(),
                "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2",
                "-c:v", "libx264",
                "-preset", "ultrafast",
                "-crf", "30",
                "-pix_fmt", "yuv420p",
                "-y",
                output_path.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::capture(20, format!("启动 FFmpeg 失败: {}", e)))?;

        *self.current_process.lock().await = Some(child);

        Ok(output_path)
    }

    /// 停止当前录制
    pub async fn stop(&self) -> AppResult<()> {
        let mut guard = self.current_process.lock().await;
        if let Some(ref mut child) = *guard {
            // 发送 SIGTERM 让 FFmpeg 正常结束
            #[cfg(unix)]
            {
                unsafe {
                    libc::kill(child.id() as i32, libc::SIGTERM);
                }
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
        *guard = None;

        Ok(())
    }

    /// 等待当前分段自然结束（FFmpeg -t 超时退出）
    pub async fn wait_segment(&self) -> AppResult<()> {
        // 先取出 child 并释放 mutex，避免持锁期间阻塞 tokio 线程
        let child = {
            let mut guard = self.current_process.lock().await;
            guard.take()
        };

        if let Some(mut child) = child {
            // 先 wait（进程结束），再读 stderr（否则 read_to_string 会永久阻塞）
            let status = child.wait()
                .map_err(|e| AppError::capture(21, format!("等待 FFmpeg 结束失败: {}", e)))?;

            if !status.success() {
                let stderr_msg = child.stderr.take()
                    .and_then(|stderr| {
                        use std::io::Read;
                        let mut buf = String::new();
                        std::io::BufReader::new(stderr).read_to_string(&mut buf).ok();
                        Some(buf)
                    })
                    .unwrap_or_default();

                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    if status.signal().is_some() {
                        debug!("FFmpeg terminated by signal (normal)");
                    } else {
                        error!("FFmpeg failed ({}): {}", status, stderr_msg);
                    }
                }
                #[cfg(not(unix))]
                error!("FFmpeg failed ({}): {}", status, stderr_msg);
            }
        }
        Ok(())
    }

}

impl Drop for ScreenRecorder {
    fn drop(&mut self) {
        // 同步清理：尝试杀死残留进程
        if let Ok(mut guard) = self.current_process.try_lock() {
            if let Some(ref mut child) = *guard {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
