use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 定时器状态
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
}

/// 计时阶段
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TimerPhase {
    Work,
    Rest,
}

impl std::fmt::Display for TimerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerState::Idle => write!(f, "停止"),
            TimerState::Running => write!(f, "运行中"),
            TimerState::Paused => write!(f, "已暂停"),
        }
    }
}

/// 定时器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimerConfig {
    pub interval_minutes: u64,
    pub custom_interval: bool,
    pub min_interval: u64,
    pub max_interval: u64,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            interval_minutes: 30,
            custom_interval: false,
            min_interval: 1,
            max_interval: 1440,
        }
    }
}

/// 定时器运行时状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimerRuntime {
    pub state: TimerState,
    pub phase: TimerPhase,
    pub remaining_seconds: u64,
    pub total_seconds: u64,
    pub last_update: Option<String>,
}

type TimerCallback = Arc<Mutex<Option<Box<dyn Fn(TimerRuntime) + Send + 'static>>>>;

impl Default for TimerRuntime {
    fn default() -> Self {
        let total = 30 * 60; // 默认30分钟
        Self {
            state: TimerState::Idle,
            phase: TimerPhase::Work,
            remaining_seconds: total,
            total_seconds: total,
            last_update: None,
        }
    }
}

/// 定时器引擎
pub struct TimerEngine {
    runtime: Arc<Mutex<TimerRuntime>>,
    config: Arc<Mutex<TimerConfig>>,
    callback: TimerCallback,
}

impl TimerEngine {
    pub fn new() -> Self {
        let config = TimerConfig::default();
        let total_seconds = config.interval_minutes * 60;

        Self {
            runtime: Arc::new(Mutex::new(TimerRuntime {
                total_seconds,
                remaining_seconds: total_seconds,
                ..Default::default()
            })),
            config: Arc::new(Mutex::new(config)),
            callback: Arc::new(Mutex::new(None)),
        }
    }

    /// 从保存的运行时状态恢复定时器
    pub fn from_runtime_state(saved_state: &crate::config::RuntimeState) -> Self {
        Self::from_runtime_state_with_interval(saved_state, saved_state.total_seconds / 60)
    }

    /// 从保存的运行时状态恢复，并指定主间隔（分钟）
    pub fn from_runtime_state_with_interval(
        saved_state: &crate::config::RuntimeState,
        interval_minutes: u64,
    ) -> Self {
        let config = TimerConfig {
            interval_minutes: interval_minutes.max(1),
            custom_interval: true,
            min_interval: 1,
            max_interval: 1440,
        };

        let phase = match saved_state.cycle_phase.as_str() {
            "Rest" => TimerPhase::Rest,
            _ => TimerPhase::Work,
        };

        let (state, remaining_seconds) = match saved_state.timer_status.as_str() {
            "Running" => {
                let remaining = crate::config::offline_adjusted_remaining(saved_state);
                if remaining == 0 {
                    (TimerState::Idle, saved_state.total_seconds)
                } else {
                    (TimerState::Running, remaining)
                }
            }
            "Paused" => (TimerState::Paused, saved_state.remaining_seconds),
            _ => (TimerState::Idle, saved_state.total_seconds),
        };

        Self {
            runtime: Arc::new(Mutex::new(TimerRuntime {
                state,
                phase,
                remaining_seconds,
                total_seconds: saved_state.total_seconds,
                last_update: Some(saved_state.last_update.clone()),
            })),
            config: Arc::new(Mutex::new(config)),
            callback: Arc::new(Mutex::new(None)),
        }
    }

    /// 从配置中的主间隔创建空闲态计时器
    pub fn from_interval_minutes(interval_minutes: u64) -> Self {
        let engine = Self::new();
        let _ = engine.set_interval(interval_minutes.max(1));
        engine
    }

    /// 恢复持久化的倒计时并启动后台线程（用于重启/开机后）
    pub fn resume_persisted_countdown(&self) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();
        match runtime.state {
            TimerState::Running | TimerState::Paused => {
                runtime.state = TimerState::Running;
                runtime.last_update = Some(chrono::Local::now().to_rfc3339());
            }
            TimerState::Idle => {
                return Err("计时器未处于可恢复状态".to_string());
            }
        }
        drop(runtime);
        self.spawn_timer_thread();
        Ok(())
    }

    /// 获取当前的运行时状态（用于保存）
    pub fn to_runtime_state(&self) -> crate::config::RuntimeState {
        let rt = self.runtime.lock().unwrap();
        crate::config::RuntimeState {
            timer_status: match rt.state {
                TimerState::Idle => "Idle".to_string(),
                TimerState::Running => "Running".to_string(),
                TimerState::Paused => "Paused".to_string(),
            },
            remaining_seconds: rt.remaining_seconds,
            total_seconds: rt.total_seconds,
            last_update: rt.last_update.clone().unwrap_or_else(|| chrono::Local::now().to_rfc3339()),
            delay_count: 0,
            delay_quota_date: crate::config::default_delay_quota_date(),
            cycle_phase: match rt.phase {
                TimerPhase::Work => "Work".to_string(),
                TimerPhase::Rest => "Rest".to_string(),
            },
        }
    }

    /// 设置回调函数，用于通知前端更新
    pub fn set_callback<F>(&self, callback: F)
    where
        F: Fn(TimerRuntime) + Send + 'static,
    {
        let mut cb = self.callback.lock().unwrap();
        *cb = Some(Box::new(callback));
    }

    /// 获取当前运行时状态
    pub fn get_runtime(&self) -> TimerRuntime {
        self.runtime.lock().unwrap().clone()
    }

    /// 获取配置
    pub fn get_config(&self) -> TimerConfig {
        self.config.lock().unwrap().clone()
    }

    /// 设置时间间隔（分钟）
    pub fn set_interval(&self, minutes: u64) -> Result<(), String> {
        let config = self.config.lock().unwrap();
        if minutes < config.min_interval || minutes > config.max_interval {
            return Err(format!(
                "时间间隔必须在 {}-{} 分钟之间",
                config.min_interval, config.max_interval
            ));
        }
        drop(config);

        let mut config = self.config.lock().unwrap();
        config.interval_minutes = minutes;
        config.custom_interval = true;

        let mut runtime = self.runtime.lock().unwrap();
        runtime.total_seconds = minutes * 60;
        runtime.remaining_seconds = runtime.total_seconds;
        runtime.state = TimerState::Idle;
        runtime.phase = TimerPhase::Work;

        Ok(())
    }

    /// 设置当前阶段及时间间隔（分钟）
    pub fn set_phase_interval(&self, phase: TimerPhase, minutes: u64) -> Result<(), String> {
        if minutes == 0 {
            return Err("时间间隔必须大于0分钟".to_string());
        }

        let mut runtime = self.runtime.lock().unwrap();
        runtime.phase = phase;
        runtime.total_seconds = minutes * 60;
        runtime.remaining_seconds = runtime.total_seconds;
        runtime.state = TimerState::Idle;
        runtime.last_update = Some(chrono::Local::now().to_rfc3339());
        Ok(())
    }

    /// 开始计时
    pub fn start(&self) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();

        match runtime.state {
            TimerState::Running => return Err("计时器已在运行".to_string()),
            TimerState::Idle => {
                runtime.remaining_seconds = runtime.total_seconds;
            }
            TimerState::Paused => {
                // 继续，不重置
            }
        }

        runtime.state = TimerState::Running;
        runtime.last_update = Some(chrono::Local::now().to_rfc3339());
        drop(runtime);

        // 启动后台线程
        self.spawn_timer_thread();

        Ok(())
    }

    /// 暂停计时
    pub fn pause(&self) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();

        if runtime.state != TimerState::Running {
            return Err("计时器未在运行".to_string());
        }

        runtime.state = TimerState::Paused;
        runtime.last_update = Some(chrono::Local::now().to_rfc3339());

        Ok(())
    }

    /// 继续计时
    pub fn resume(&self) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();

        if runtime.state != TimerState::Paused {
            return Err("计时器未暂停".to_string());
        }

        runtime.state = TimerState::Running;
        runtime.last_update = Some(chrono::Local::now().to_rfc3339());
        drop(runtime);

        // 重新启动后台线程
        self.spawn_timer_thread();

        Ok(())
    }

    /// 停止并重置
    pub fn stop(&self) {
        let mut runtime = self.runtime.lock().unwrap();
        runtime.state = TimerState::Idle;
        runtime.remaining_seconds = runtime.total_seconds;
        runtime.phase = TimerPhase::Work;
        runtime.last_update = Some(chrono::Local::now().to_rfc3339());
    }

    /// 延后执行（追加分钟）
    pub fn add_delay(&self, minutes: u64) -> Result<(), String> {
        let mut runtime = self.runtime.lock().unwrap();
        if runtime.state != TimerState::Running && runtime.state != TimerState::Paused {
            return Err("计时器未在运行".to_string());
        }

        let add_seconds = minutes * 60;
        runtime.remaining_seconds = runtime.remaining_seconds.saturating_add(add_seconds);
        runtime.total_seconds = runtime.total_seconds.saturating_add(add_seconds);
        runtime.last_update = Some(chrono::Local::now().to_rfc3339());
        Ok(())
    }

    /// 启动后台计时线程
    fn spawn_timer_thread(&self) {
        let runtime = Arc::clone(&self.runtime);
        let callback = Arc::clone(&self.callback);

        thread::spawn(move || {
            let mut last_tick = Instant::now();

            loop {
                thread::sleep(Duration::from_millis(100));

                let mut rt = runtime.lock().unwrap();

                // 检查是否应该停止线程
                if rt.state != TimerState::Running {
                    break;
                }

                // 计算经过的时间
                let now = Instant::now();
                let elapsed = now.duration_since(last_tick).as_secs();

                if elapsed >= 1 {
                    last_tick = now;

                    if rt.remaining_seconds > 0 {
                        rt.remaining_seconds -= 1;
                        rt.last_update = Some(chrono::Local::now().to_rfc3339());

                        // 检查是否倒计时结束
                        let is_finished = rt.remaining_seconds == 0;

                        // 如果倒计时结束，先设置状态为 Idle，再触发回调
                        if is_finished {
                            rt.state = TimerState::Idle;
                            rt.last_update = Some(chrono::Local::now().to_rfc3339());
                        }

                        // 触发回调
                        let runtime_clone = rt.clone();
                        drop(rt);

                        if let Ok(cb) = callback.lock() {
                            if let Some(ref callback_fn) = *cb {
                                callback_fn(runtime_clone);
                            }
                        }

                        // 如果倒计时结束，退出线程
                        if is_finished {
                            break;
                        }
                    }
                } else {
                    drop(rt);
                }
            }
        });
    }
}

impl Default for TimerEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_state_display() {
        assert_eq!(TimerState::Idle.to_string(), "停止");
        assert_eq!(TimerState::Running.to_string(), "运行中");
        assert_eq!(TimerState::Paused.to_string(), "已暂停");
    }

    #[test]
    fn test_set_interval() {
        let engine = TimerEngine::new();
        assert!(engine.set_interval(45).is_ok());
        assert!(engine.set_interval(0).is_err());
        assert!(engine.set_interval(2000).is_err());
    }

    #[test]
    fn test_set_phase_interval() {
        let engine = TimerEngine::new();
        assert!(engine.set_phase_interval(TimerPhase::Rest, 5).is_ok());
        let rt = engine.get_runtime();
        assert_eq!(rt.phase, TimerPhase::Rest);
        assert_eq!(rt.total_seconds, 300);
        assert!(engine.set_phase_interval(TimerPhase::Work, 0).is_err());
    }
}
