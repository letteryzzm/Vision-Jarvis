/// 鼠标 Idle 检测
///
/// 使用 rdev 监听全局鼠标事件，检测用户离开和回归。
/// - 鼠标静止超过 threshold_secs 时标记为 idle
/// - 从 idle 状态恢复时触发回调，传入 idle 持续秒数

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{info, warn};

/// Idle 状态
#[derive(Debug, Clone)]
struct IdleState {
    /// idle 开始时间（None 表示当前非 idle）
    idle_start: Option<Instant>,
    /// 上次鼠标活动时间
    last_activity: Instant,
}

impl IdleState {
    fn new() -> Self {
        Self {
            idle_start: None,
            last_activity: Instant::now(),
        }
    }
}

/// 鼠标 Idle 检测器
pub struct IdleWatcher {
    /// idle 阈值（秒）：鼠标静止超过此时长才认定为 idle
    threshold_secs: u64,
    /// 最小触发时长（秒）：idle 时长小于此值不触发回调
    min_trigger_secs: u64,
    /// 内部状态
    state: Arc<Mutex<IdleState>>,
}

impl IdleWatcher {
    /// 创建新的 IdleWatcher
    ///
    /// - `threshold_secs`: 鼠标静止多少秒后进入 idle 状态（检测周期为 1s）
    /// - `min_trigger_secs`: idle 时长至少多少秒才触发回调
    pub fn new(threshold_secs: u64, min_trigger_secs: u64) -> Self {
        Self {
            threshold_secs,
            min_trigger_secs,
            state: Arc::new(Mutex::new(IdleState::new())),
        }
    }

    /// 启动监听（阻塞调用，需在独立 OS 线程中运行）
    ///
    /// `on_return`: 用户从 idle 回归时的回调，参数为 idle 持续秒数
    pub fn start<F>(self, on_return: F)
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        let threshold = Duration::from_secs(self.threshold_secs);
        let min_trigger = self.min_trigger_secs;
        let state = self.state.clone();
        let on_return = Arc::new(on_return);

        // 启动后台线程，每秒检查一次是否进入 idle
        let state_checker = state.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let mut s = state_checker.lock().unwrap();
                if s.idle_start.is_none() && s.last_activity.elapsed() >= threshold {
                    s.idle_start = Some(Instant::now());
                    info!(
                        "[IdleWatcher] User went idle (no mouse activity for {}s)",
                        threshold.as_secs()
                    );
                }
            }
        });

        // 使用 rdev 监听全局鼠标事件（阻塞）
        let on_return_ref = on_return.clone();
        if let Err(e) = rdev::listen(move |event| {
            use rdev::EventType;
            match event.event_type {
                EventType::MouseMove { .. }
                | EventType::ButtonPress(_)
                | EventType::ButtonRelease(_)
                | EventType::Wheel { .. } => {
                    let mut s = state.lock().unwrap();
                    let was_idle = s.idle_start.is_some();

                    if was_idle {
                        if let Some(idle_start) = s.idle_start.take() {
                            let idle_secs = idle_start.elapsed().as_secs();
                            info!(
                                "[IdleWatcher] User returned after {}s idle",
                                idle_secs
                            );
                            if idle_secs >= min_trigger {
                                let cb = on_return_ref.clone();
                                let _ = std::thread::spawn(move || cb(idle_secs));
                            }
                        }
                    }

                    s.last_activity = Instant::now();
                }
                _ => {}
            }
        }) {
            warn!("[IdleWatcher] rdev listen error: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_watcher_creation() {
        let watcher = IdleWatcher::new(300, 60);
        assert_eq!(watcher.threshold_secs, 300);
        assert_eq!(watcher.min_trigger_secs, 60);
    }

    #[test]
    fn test_idle_state_initial() {
        let state = IdleState::new();
        assert!(state.idle_start.is_none());
    }
}
