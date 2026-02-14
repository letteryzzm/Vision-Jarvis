/// 截图捕获调度器
///
/// 负责按配置间隔定时捕获截图，自动保存文件和数据库记录

use crate::error::{AppError, AppResult};
use crate::db::Database;
use log::error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;
use super::ScreenCapture;

/// 捕获调度器
pub struct CaptureScheduler {
    capture: Arc<ScreenCapture>,
    db: Option<Arc<Database>>,
    pub interval_seconds: u8,
    is_running: Arc<Mutex<bool>>,
    task_handle: Option<JoinHandle<()>>,
}

impl CaptureScheduler {
    /// 创建新的调度器
    pub fn new(capture: ScreenCapture, interval_seconds: u8) -> Self {
        Self {
            capture: Arc::new(capture),
            db: None,
            interval_seconds,
            is_running: Arc::new(Mutex::new(false)),
            task_handle: None,
        }
    }

    /// 设置数据库（截图记录会自动写入）
    pub fn with_db(mut self, db: Arc<Database>) -> Self {
        self.db = Some(db);
        self
    }

    /// 注入数据库引用
    pub fn set_db(&mut self, db: Arc<Database>) {
        self.db = Some(db);
    }

    /// 启动调度器
    pub async fn start(&mut self) -> AppResult<()> {
        let mut running = self.is_running.lock().await;
        if *running {
            return Err(AppError::screenshot(10, "调度器已经在运行"));
        }

        *running = true;
        drop(running); // 释放锁

        let capture = Arc::clone(&self.capture);
        let db = self.db.clone();
        let is_running = Arc::clone(&self.is_running);
        let interval_secs = self.interval_seconds as u64;

        // 启动后台任务
        let handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                let running = is_running.lock().await;
                if !*running {
                    break;
                }
                drop(running);

                // 捕获截图
                match capture.capture_screenshot() {
                    Ok(file_path) => {
                        eprintln!("[Scheduler] Screenshot captured: {}", file_path.display());
                        // 保存到数据库
                        if let Some(ref db) = db {
                            let id = uuid::Uuid::new_v4().to_string();
                            let path_str = file_path.to_string_lossy().to_string();
                            let timestamp = chrono::Utc::now().timestamp();

                            if let Err(e) = db.with_connection(|conn| {
                                conn.execute(
                                    "INSERT INTO screenshots (id, path, captured_at, analyzed)
                                     VALUES (?1, ?2, ?3, 0)",
                                    (&id, &path_str, timestamp),
                                )?;
                                Ok(())
                            }) {
                                eprintln!("[Scheduler] ERROR: Failed to save screenshot record: {}", e);
                            } else {
                                eprintln!("[Scheduler] Screenshot saved to DB: {}", id);
                            }
                        } else {
                            eprintln!("[Scheduler] WARNING: No database configured, screenshot not saved to DB");
                        }
                    }
                    Err(e) => {
                        eprintln!("[Scheduler] ERROR: Screenshot capture failed: {}", e);
                    }
                }
            }
        });

        self.task_handle = Some(handle);

        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&mut self) -> AppResult<()> {
        let mut running = self.is_running.lock().await;
        if !*running {
            return Err(AppError::screenshot(11, "调度器未运行"));
        }

        *running = false;
        drop(running);

        // 等待任务完成
        if let Some(handle) = self.task_handle.take() {
            handle.await
                .map_err(|e| AppError::screenshot(12, format!("等待任务完成失败: {}", e)))?;
        }

        Ok(())
    }

    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    /// 更新捕获间隔
    pub async fn update_interval(&mut self, interval_seconds: u8) -> AppResult<()> {
        let was_running = self.is_running().await;

        if was_running {
            self.stop().await?;
        }

        self.interval_seconds = interval_seconds;

        if was_running {
            self.start().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf()).unwrap();
        let scheduler = CaptureScheduler::new(capture, 5);

        assert!(!scheduler.is_running().await);
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf()).unwrap();
        let mut scheduler = CaptureScheduler::new(capture, 5);

        assert!(scheduler.start().await.is_ok());
        assert!(scheduler.is_running().await);

        assert!(scheduler.stop().await.is_ok());
        assert!(!scheduler.is_running().await);
    }

    #[tokio::test]
    async fn test_scheduler_double_start() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf()).unwrap();
        let mut scheduler = CaptureScheduler::new(capture, 5);

        assert!(scheduler.start().await.is_ok());
        assert!(scheduler.start().await.is_err()); // 第二次启动应失败

        scheduler.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_update_interval() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenCapture::new(temp_dir.path().to_path_buf()).unwrap();
        let mut scheduler = CaptureScheduler::new(capture, 5);

        scheduler.start().await.unwrap();
        assert_eq!(scheduler.interval_seconds, 5);

        scheduler.update_interval(10).await.unwrap();
        assert_eq!(scheduler.interval_seconds, 10);
        assert!(scheduler.is_running().await);

        scheduler.stop().await.unwrap();
    }
}
