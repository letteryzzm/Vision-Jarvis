/// 录制调度器
///
/// 管理 ScreenRecorder 的分段录制循环，每个分段结束后写入 DB 记录

use crate::error::{AppError, AppResult};
use crate::db::Database;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use log::{debug, error, warn};
use super::screen_recorder::ScreenRecorder;

/// 录制调度器
pub struct CaptureScheduler {
    recorder: Arc<ScreenRecorder>,
    db: Option<Arc<Database>>,
    pub interval_seconds: u64,
    is_running: Arc<Mutex<bool>>,
    task_handle: Option<JoinHandle<()>>,
}

impl CaptureScheduler {
    pub fn new(recorder: ScreenRecorder, segment_duration_secs: u64) -> Self {
        Self {
            recorder: Arc::new(recorder),
            db: None,
            interval_seconds: segment_duration_secs,
            is_running: Arc::new(Mutex::new(false)),
            task_handle: None,
        }
    }

    pub fn with_db(mut self, db: Arc<Database>) -> Self {
        self.db = Some(db);
        self
    }

    pub async fn start(&mut self) -> AppResult<()> {
        let mut running = self.is_running.lock().await;
        if *running {
            return Err(AppError::screenshot(10, "调度器已经在运行"));
        }
        *running = true;
        drop(running);

        let recorder = Arc::clone(&self.recorder);
        let db = self.db.clone();
        let is_running = Arc::clone(&self.is_running);

        let handle = tokio::spawn(async move {
            loop {
                let running = is_running.lock().await;
                if !*running { break; }
                drop(running);

                // 启动一个分段
                let output_path = match recorder.start_segment().await {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to start recording segment: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let start_time = chrono::Utc::now().timestamp();
                let filename = output_path.file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                debug!("Recording: {}", filename);

                // 等待分段自然结束
                if let Err(e) = recorder.wait_segment().await {
                    error!("Segment wait failed: {}", e);
                }

                let end_time = chrono::Utc::now().timestamp();
                let duration = end_time - start_time;

                // 写入 DB
                if let Some(ref db) = db {
                    let id = uuid::Uuid::new_v4().to_string();
                    let path_str = output_path.to_string_lossy().to_string();

                    if let Err(e) = db.with_connection(|conn| {
                        conn.execute(
                            "INSERT INTO recordings (id, path, start_time, end_time, duration_secs, fps, analyzed, created_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, 2, 0, ?3)",
                            rusqlite::params![id, path_str, start_time, end_time, duration],
                        )?;
                        Ok(())
                    }) {
                        error!("Failed to save recording: {}", e);
                    } else {
                        debug!("Saved: {}..{} ({}s)", &id[..8], &id[id.len()-4..], duration);
                    }
                }
            }
        });

        self.task_handle = Some(handle);
        Ok(())
    }

    pub async fn stop(&mut self) -> AppResult<()> {
        let mut running = self.is_running.lock().await;
        if !*running {
            return Err(AppError::screenshot(11, "调度器未运行"));
        }
        *running = false;
        drop(running);

        // 停止当前录制
        if let Err(e) = self.recorder.stop().await {
            warn!("Stop recorder failed: {}", e);
        }

        if let Some(handle) = self.task_handle.take() {
            handle.await
                .map_err(|e| AppError::screenshot(12, format!("等待任务完成失败: {}", e)))?;
        }

        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    pub async fn update_interval(&mut self, segment_duration_secs: u64) -> AppResult<()> {
        let was_running = self.is_running().await;
        if was_running {
            self.stop().await?;
        }
        self.interval_seconds = segment_duration_secs;
        if was_running {
            self.start().await?;
        }
        Ok(())
    }
}
