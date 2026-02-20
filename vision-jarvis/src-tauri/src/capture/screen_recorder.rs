/// 屏幕录制器
///
/// 使用 xcap 截图 + FFmpeg 编码，替代 avfoundation 直接录制

use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{Utc, Timelike};
use log::{error, info};
use uuid::Uuid;
use xcap::Monitor;

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
    stop_flag: Arc<AtomicBool>,
    capture_handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl ScreenRecorder {
    pub fn new(storage_path: PathBuf, segment_duration_secs: u64, fps: u8) -> AppResult<Self> {
        std::fs::create_dir_all(&storage_path)
            .map_err(|e| AppError::capture(1, format!("创建录制存储目录失败: {}", e)))?;

        Monitor::all()
            .map_err(|e| AppError::capture(1, format!("无法获取显示器: {}", e)))?;

        info!("Screen recorder initialized (xcap)");

        Ok(Self {
            storage_path,
            segment_duration_secs,
            fps,
            stop_flag: Arc::new(AtomicBool::new(false)),
            capture_handle: Arc::new(Mutex::new(None)),
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

        let frames_dir = rec_dir.join(format!(".frames_{}", id));
        std::fs::create_dir_all(&frames_dir)
            .map_err(|e| AppError::capture(1, format!("创建帧目录失败: {}", e)))?;

        self.stop_flag.store(false, Ordering::SeqCst);

        let stop_flag = Arc::clone(&self.stop_flag);
        let fps = self.fps;
        let duration = self.segment_duration_secs;
        let frames_dir_c = frames_dir.clone();
        let output_path_c = output_path.clone();

        let handle = std::thread::spawn(move || {
            let monitors = match Monitor::all() {
                Ok(m) if !m.is_empty() => m,
                Ok(_) => { error!("未找到显示器"); return; }
                Err(e) => { error!("获取显示器失败: {}", e); return; }
            };
            let monitor = &monitors[0];
            let frame_interval = std::time::Duration::from_millis(1000 / fps as u64);
            let end_time = std::time::Instant::now() + std::time::Duration::from_secs(duration);
            let mut frame_idx: u32 = 0;

            while std::time::Instant::now() < end_time && !stop_flag.load(Ordering::SeqCst) {
                let t = std::time::Instant::now();

                match monitor.capture_image() {
                    Ok(image) => {
                        let frame_path = frames_dir_c.join(format!("f{:06}.png", frame_idx));
                        if image.save(&frame_path).is_ok() {
                            frame_idx += 1;
                        }
                    }
                    Err(e) => error!("截图失败: {}", e),
                }

                let elapsed = t.elapsed();
                if elapsed < frame_interval {
                    std::thread::sleep(frame_interval - elapsed);
                }
            }

            if frame_idx == 0 {
                error!("未捕获任何帧，跳过编码");
                let _ = std::fs::remove_dir_all(&frames_dir_c);
                return;
            }

            info!("捕获 {} 帧，开始编码...", frame_idx);

            let out = Command::new("ffmpeg")
                .args([
                    "-framerate", &fps.to_string(),
                    "-i", &frames_dir_c.join("f%06d.png").to_string_lossy().to_string(),
                    "-c:v", "libx264",
                    "-preset", "ultrafast",
                    "-crf", "30",
                    "-pix_fmt", "yuv420p",
                    "-y",
                    &output_path_c.to_string_lossy().to_string(),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output();

            match out {
                Ok(o) if o.status.success() => info!("编码完成: {}", output_path_c.display()),
                Ok(o) => error!("FFmpeg 编码失败: {}", String::from_utf8_lossy(&o.stderr)),
                Err(e) => error!("FFmpeg 启动失败: {}", e),
            }

            let _ = std::fs::remove_dir_all(&frames_dir_c);
        });

        *self.capture_handle.lock().await = Some(handle);

        Ok(output_path)
    }

    /// 停止当前录制
    pub async fn stop(&self) -> AppResult<()> {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.wait_segment().await
    }

    /// 等待当前分段自然结束
    pub async fn wait_segment(&self) -> AppResult<()> {
        let handle = {
            let mut guard = self.capture_handle.lock().await;
            guard.take()
        };

        if let Some(h) = handle {
            tokio::task::spawn_blocking(move || { let _ = h.join(); })
                .await
                .map_err(|e| AppError::capture(22, format!("等待捕获线程失败: {}", e)))?;
        }

        Ok(())
    }
}

impl Drop for ScreenRecorder {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
}
