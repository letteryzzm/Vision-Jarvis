use crate::error::{AppError, AppResult};
use crate::db::Database;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use log::{error, info};
use super::screen_recorder::ScreenRecorder;

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
            return Err(AppError::capture(10, "调度器已在运行"));
        }
        *running = true;
        drop(running);

        let recorder = Arc::clone(&self.recorder);
        let db = self.db.clone();
        let is_running = Arc::clone(&self.is_running);

        let interval = self.interval_seconds;

        let handle = tokio::spawn(async move {
            loop {
                if !*is_running.lock().await {
                    recorder.stop().await;
                    break;
                }

                let output_path = match recorder.start_segment().await {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Start segment failed: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let start_time = chrono::Local::now().timestamp();
                let filename = output_path.file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                info!("Recording: {}", filename);

                // 异步等待分段时长，不阻塞 tokio
                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;

                // 被停止时不保存（scheduler.stop 已处理清理）
                if !*is_running.lock().await {
                    recorder.stop().await;
                    break;
                }

                // 发送 SIGTERM 结束 FFmpeg，等待写入文件尾
                recorder.stop().await;

                let end_time = chrono::Local::now().timestamp();
                let duration = end_time - start_time;

                let file_ok = output_path.exists()
                    && std::fs::metadata(&output_path).map(|m| m.len() > 0).unwrap_or(false);

                if !file_ok {
                    error!("Recording file missing or empty: {}", output_path.display());
                    continue;
                }

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
                        info!("Saved: {}..{} ({}s)", &id[..8], &id[id.len()-4..], duration);
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
            return Err(AppError::capture(11, "调度器未运行"));
        }
        *running = false;
        drop(running);

        // 停止录制并删除未完成的文件
        self.recorder.stop().await;
        self.recorder.delete_current_file().await;

        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }

        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }
}
