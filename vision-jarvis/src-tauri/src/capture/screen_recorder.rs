use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tokio::sync::Mutex;
use chrono::{Utc, Timelike};
use log::info;
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

fn time_period(hour: u32) -> &'static str {
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
    process: Mutex<Option<Child>>,
    current_path: Mutex<Option<PathBuf>>,
}

impl ScreenRecorder {
    pub fn new(storage_path: PathBuf, segment_duration_secs: u64, fps: u8) -> AppResult<Self> {
        std::fs::create_dir_all(&storage_path)
            .map_err(|e| AppError::capture(1, format!("创建存储目录失败: {}", e)))?;

        let idx = find_screen_device_index();
        info!("Screen device index: {}", idx);

        Ok(Self {
            storage_path,
            segment_duration_secs,
            fps,
            screen_device_index: idx,
            process: Mutex::new(None),
            current_path: Mutex::new(None),
        })
    }

    pub async fn start_segment(&self) -> AppResult<PathBuf> {
        let now = Utc::now();
        let dir = self.storage_path
            .join("recordings")
            .join(now.format("%Y%m%d").to_string())
            .join(time_period(now.hour()));

        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::capture(2, format!("创建录制目录失败: {}", e)))?;

        let path = dir.join(format!("{}_{}.mp4", now.format("%H-%M-%S"), Uuid::new_v4()));

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
                path.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| AppError::capture(3, format!("启动 FFmpeg 失败: {}", e)))?;

        *self.process.lock().await = Some(child);
        *self.current_path.lock().await = Some(path.clone());

        Ok(path)
    }

    pub async fn wait_segment(&self) -> AppResult<()> {
        let mut guard = self.process.lock().await;
        if let Some(ref mut child) = *guard {
            let _ = child.wait();
        }
        *guard = None;
        Ok(())
    }

    pub async fn stop(&self) {
        let mut guard = self.process.lock().await;
        if let Some(ref mut child) = *guard {
            #[cfg(unix)]
            unsafe { libc::kill(child.id() as i32, libc::SIGTERM); }
            #[cfg(not(unix))]
            let _ = child.kill();
            let _ = child.wait();
        }
        *guard = None;
    }

    pub async fn delete_current_file(&self) {
        let path = self.current_path.lock().await.take();
        if let Some(p) = path {
            let _ = std::fs::remove_file(&p);
        }
    }
}

impl Drop for ScreenRecorder {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.process.try_lock() {
            if let Some(ref mut child) = *guard {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
