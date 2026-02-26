/// 鼠标 Idle 检测
///
/// macOS: 使用 CoreGraphics CGEventSourceSecondsSinceLastEventType 轮询，
///        完全线程安全，无需 RunLoop，不调用 TSM API，消除 dispatch_assert_queue_fail 崩溃。
/// 其他平台: 使用 rdev 监听全局鼠标事件。

use std::sync::Arc;
use std::time::Duration;
use log::info;

/// 鼠标 Idle 检测器
pub struct IdleWatcher {
    /// idle 阈值（秒）：鼠标静止超过此时长才认定为 idle
    threshold_secs: u64,
    /// 最小触发时长（秒）：idle 时长小于此值不触发回调
    min_trigger_secs: u64,
}

#[cfg(target_os = "macos")]
mod platform {
    // CGEventSourceSecondsSinceLastEventType 直接 FFI 绑定
    // state_id: CGEventSourceStateID, HIDSystemState = 1
    // event_type: CGEventType; kCGAnyInputEventType = 0xFFFFFFFF
    // 该函数完全线程安全，可在任意线程轮询，不涉及 TSM/主线程约束。
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(
            state_id: i32,
            event_type: u32,
        ) -> f64;
    }

    /// 返回距最后一次鼠标（或任意输入）事件的秒数
    pub fn seconds_since_last_mouse_activity() -> f64 {
        // kCGAnyInputEventType = 0xFFFFFFFF，覆盖鼠标移动、点击、键盘等全部输入
        unsafe { CGEventSourceSecondsSinceLastEventType(1, 0xFFFF_FFFF) }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use std::sync::{Arc, Mutex};
    use std::time::Instant;
    use log::warn;

    pub struct NonMacosState {
        pub last_activity: Mutex<Instant>,
    }

    impl NonMacosState {
        pub fn new() -> Arc<Self> {
            Arc::new(Self {
                last_activity: Mutex::new(Instant::now()),
            })
        }
    }

    pub fn start_rdev_listener(state: Arc<NonMacosState>) {
        if let Err(e) = rdev::listen(move |event| {
            use rdev::EventType;
            match event.event_type {
                EventType::MouseMove { .. }
                | EventType::ButtonPress(_)
                | EventType::ButtonRelease(_)
                | EventType::Wheel { .. } => {
                    *state.last_activity.lock().unwrap() = Instant::now();
                }
                _ => {}
            }
        }) {
            warn!("[IdleWatcher] rdev listen error: {:?}", e);
        }
    }
}

impl IdleWatcher {
    /// 创建新的 IdleWatcher
    ///
    /// - `threshold_secs`: 鼠标静止多少秒后进入 idle 状态
    /// - `min_trigger_secs`: idle 时长至少多少秒才触发回调
    pub fn new(threshold_secs: u64, min_trigger_secs: u64) -> Self {
        Self {
            threshold_secs,
            min_trigger_secs,
        }
    }

    /// 启动监听（阻塞调用，需在独立 OS 线程中运行）
    ///
    /// `on_return`: 用户从 idle 回归时的回调，参数为 idle 持续秒数
    #[cfg(target_os = "macos")]
    pub fn start<F>(self, on_return: F)
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        let threshold = self.threshold_secs;
        let min_trigger = self.min_trigger_secs;
        let on_return = Arc::new(on_return);

        loop {
            std::thread::sleep(Duration::from_secs(1));

            let idle_secs = platform::seconds_since_last_mouse_activity() as u64;

            if idle_secs < threshold {
                // 用户活跃，继续等待
                continue;
            }

            // 进入 idle 状态，记录 idle 时长
            info!(
                "[IdleWatcher] User went idle ({}s since last input)",
                idle_secs
            );

            // 等待用户回归（idle_secs 减小说明有新输入）
            let was_long_idle = idle_secs >= min_trigger;
            let snapshot = idle_secs;

            loop {
                std::thread::sleep(Duration::from_millis(500));
                let new_secs = platform::seconds_since_last_mouse_activity() as u64;
                if new_secs < snapshot {
                    // 检测到新输入，用户从 idle 回归
                    // 以回归时系统记录的 idle 时长为准（snapshot 是进入轮询时的值）
                    info!("[IdleWatcher] User returned after ~{}s idle", snapshot);
                    if was_long_idle {
                        let cb = on_return.clone();
                        std::thread::spawn(move || cb(snapshot));
                    }
                    break;
                }
            }
        }
    }

    /// 启动监听（非 macOS 平台，使用 rdev）
    #[cfg(not(target_os = "macos"))]
    pub fn start<F>(self, on_return: F)
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        use std::time::Instant;

        let threshold = Duration::from_secs(self.threshold_secs);
        let min_trigger = self.min_trigger_secs;
        let on_return = Arc::new(on_return);

        let state = platform::NonMacosState::new();
        let state_for_listener = state.clone();

        // 启动 rdev 监听线程
        std::thread::spawn(move || {
            platform::start_rdev_listener(state_for_listener);
        });

        // 主轮询循环
        let mut idle_start: Option<Instant> = None;

        loop {
            std::thread::sleep(Duration::from_secs(1));
            let last = *state.last_activity.lock().unwrap();
            let elapsed = last.elapsed();

            match idle_start {
                None => {
                    if elapsed >= threshold {
                        idle_start = Some(Instant::now());
                        info!(
                            "[IdleWatcher] User went idle (no activity for {}s)",
                            threshold.as_secs()
                        );
                    }
                }
                Some(start) => {
                    if elapsed < threshold {
                        // 用户回归
                        let idle_secs = start.elapsed().as_secs();
                        info!("[IdleWatcher] User returned after {}s idle", idle_secs);
                        if idle_secs >= min_trigger {
                            let cb = on_return.clone();
                            std::thread::spawn(move || cb(idle_secs));
                        }
                        idle_start = None;
                    }
                }
            }
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

    #[cfg(target_os = "macos")]
    #[test]
    fn test_seconds_since_last_activity_returns_nonnegative() {
        let secs = platform::seconds_since_last_mouse_activity();
        assert!(secs >= 0.0, "seconds should be non-negative, got {}", secs);
    }
}
