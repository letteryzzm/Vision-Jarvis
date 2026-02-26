use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tokio::sync::Mutex as AsyncMutex;
use chrono::{Local, Timelike};
use log::{info, warn};
use uuid::Uuid;

fn find_screen_device_index() -> u32 {
    let output = std::process::Command::new("ffmpeg")
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
    fps: u8,
    screen_device_index: u32,
    /// std::sync::Mutex 使得 Drop 可以同步获取锁，确保 FFmpeg 进程被正确清理
    process: Mutex<Option<Child>>,
    current_path: AsyncMutex<Option<PathBuf>>,
}

impl ScreenRecorder {
    pub fn new(storage_path: PathBuf, _segment_duration_secs: u64, fps: u8) -> AppResult<Self> {
        std::fs::create_dir_all(&storage_path)
            .map_err(|e| AppError::capture(1, format!("创建存储目录失败: {}", e)))?;

        let idx = find_screen_device_index();
        info!("Screen device index: {}", idx);

        Ok(Self {
            storage_path,
            fps,
            screen_device_index: idx,
            process: Mutex::new(None),
            current_path: AsyncMutex::new(None),
        })
    }

    pub async fn start_segment(&self) -> AppResult<PathBuf> {
        // 确保旧进程已清理
        self.stop().await;

        let now = Local::now();
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

        *self.process.lock().unwrap() = Some(child);
        *self.current_path.lock().await = Some(path.clone());

        Ok(path)
    }

    pub async fn stop(&self) {
        // 使用 spawn_blocking 在阻塞线程上获取同步锁并操作子进程
        let process = &self.process;
        tokio::task::block_in_place(|| {
            let mut guard = process.lock().unwrap();
            Self::kill_child(guard.as_mut());
            *guard = None;
        });
    }

    /// 同步终止子进程（先 SIGTERM，超时后 SIGKILL）
    fn kill_child(child: Option<&mut Child>) {
        let Some(child) = child else { return };

        #[cfg(unix)]
        unsafe { libc::kill(child.id() as i32, libc::SIGTERM); }
        #[cfg(not(unix))]
        let _ = child.kill();

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        warn!("FFmpeg 未在 5s 内退出，强制终止");
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    warn!("FFmpeg try_wait error: {}", e);
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
            }
        }
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
        // std::sync::Mutex::lock() 阻塞等待，确保 FFmpeg 进程被清理
        // 即使 stop() 并发持锁，drop 也能在其完成后获取锁
        if let Ok(mut guard) = self.process.lock() {
            Self::kill_child(guard.as_mut());
        }
    }
}
