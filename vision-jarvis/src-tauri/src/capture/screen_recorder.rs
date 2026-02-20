/// 屏幕录制器
///
/// 使用 FFmpeg avfoundation 进行 macOS 屏幕分段录制

use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{Utc, Timelike};
use log::{debug, info, warn};
use uuid::Uuid;

fn find_screen_device_index() -> u32 {
    let output = Command::new("ffmpeg")
        .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();

    if let Ok(out) = output {
        let stderr = String::from_utf8_lossy(&out.stderr);
        for line in stderr.lines() {
            if line.contains("Capture screen") {
                if let Some(end) = line.rfind(']') {
                    if let Some(start) = line[..end].rfind('[') {
                        if let Ok(idx) = line[start + 1..end].parse::<u32>() {
                            return idx;
                        }
                    }
                }
            }
        }
    }
    1
}

fn get_time_period(hour: u32) -> &'static str {
    match hour {
        0..=11 => "0_00-12_00",
        12..=17 => "12_00-18_00",
        _ => "18_00-24_00",
    }
}

pub struct ScreenRecorder {
    storage_path: PathBuf,
    segment_duration_secs: u64,
    fps: u8,
    screen_device_index: u32,
    is_recording: Arc<AtomicBool>,
    current_process: Arc<Mutex<Option<Child>>>,
}

impl ScreenRecorder {
    pub fn new(storage_path: PathBuf, segment_duration_secs: u64, fps: u8) -> AppResult<Self> {
        std::fs::create_dir_all(&storage_path)
            .map_err(|e| AppError::capture(1, format!("创建录制存储目录失败: {}", e)))?;

        let screen_device_index = find_screen_device_index();
        info!("Screen capture device index: {}", screen_device_index);

        Ok(Self {
            storage_path,
            segment_duration_secs,
            fps,
            screen_device_index,
            is_recording: Arc::new(AtomicBool::new(false)),
            current_process: Arc::new(Mutex::new(None)),
        })
    }

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

        self.is_recording.store(true, Ordering::SeqCst);
        *self.current_process.lock().await = Some(child);

        Ok(output_path)
    }

    pub async fn stop(&self) -> AppResult<()> {
        self.is_recording.store(false, Ordering::SeqCst);

        let mut guard = self.current_process.lock().await;
        if let Some(ref mut child) = *guard {
            #[cfg(unix)]
            unsafe { libc::kill(child.id() as i32, libc::SIGTERM); }
            #[cfg(not(unix))]
            let _ = child.kill();
            let _ = child.wait();
        }
        *guard = None;
        Ok(())
    }

    pub async fn wait_segment(&self) -> AppResult<()> {
        let mut guard = self.current_process.lock().await;
        if let Some(ref mut child) = *guard {
            let status = child.wait()
                .map_err(|e| AppError::capture(21, format!("等待 FFmpeg 结束失败: {}", e)))?;
            if !status.success() {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    if status.signal().is_some() {
                        debug!("FFmpeg terminated by signal (normal)");
                    } else {
                        warn!("FFmpeg exited: {}", status);
                    }
                }
                #[cfg(not(unix))]
                warn!("FFmpeg exited: {}", status);
            }
        }
        *guard = None;
        Ok(())
    }
}

impl Drop for ScreenRecorder {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.current_process.try_lock() {
            if let Some(ref mut child) = *guard {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
