use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, WindowEvent,
};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};

// 导入各模块
mod timer;
mod system;
pub mod single_instance;
mod config;
mod schedule;
mod notifier;
mod startup;
mod logger;
pub mod activation;
mod security;

use timer::{TimerEngine, TimerRuntime, TimerConfig as TimerSettings, TimerState, TimerPhase};
use config::ConfigManager;
use std::sync::Arc;

// 全局定时器引擎状态
pub struct AppState {
    pub timer: TimerEngine,
}

#[derive(Clone)]
pub struct RuntimeFlags {
    pub safe_mode: bool,
}

const REST_RELOCK_INTERVAL_SECS: u64 = 3;

#[derive(Default)]
pub struct RestLockGuardState {
    enforce_during_rest: AtomicBool,
    last_relock_unix_secs: AtomicU64,
}

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn set_rest_lock_guard(rest_lock_guard: &RestLockGuardState, enforce: bool) {
    rest_lock_guard
        .enforce_during_rest
        .store(enforce, Ordering::Relaxed);
    rest_lock_guard
        .last_relock_unix_secs
        .store(if enforce { unix_now_secs() } else { 0 }, Ordering::Relaxed);
}

fn format_remaining(seconds: u64) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", minutes, secs)
}

fn update_tray_tooltip(app_handle: &tauri::AppHandle, runtime: &TimerRuntime) {
    let tooltip = match runtime.state {
        TimerState::Running => format!("TimerApp - {}", format_remaining(runtime.remaining_seconds)),
        TimerState::Paused => format!("TimerApp - 暂停 {}", format_remaining(runtime.remaining_seconds)),
        TimerState::Idle => "TimerApp - 停止".to_string(),
    };

    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(tooltip.as_str()));
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            timer: TimerEngine::new(),
        }
    }

    /// 从配置加载状态创建
    pub fn from_config(config_manager: &ConfigManager) -> Self {
        let config = config_manager.get().unwrap_or_default();
        let timer = if config.runtime_state.timer_status != "Idle" {
            TimerEngine::from_runtime_state_with_interval(
                &config.runtime_state,
                config.timer.interval_minutes,
            )
        } else {
            TimerEngine::from_interval_minutes(config.timer.interval_minutes)
        };

        Self { timer }
    }
}

fn persist_timer_runtime(
    config_manager: &Arc<ConfigManager>,
    timer: &TimerEngine,
) -> Result<(), String> {
    let mut runtime_state = timer.to_runtime_state();
    if let Ok(cfg) = config_manager.get() {
        runtime_state.delay_count = cfg.runtime_state.delay_count;
        runtime_state.delay_quota_date = cfg.runtime_state.delay_quota_date;
    }
    config_manager.update(|c| c.runtime_state = runtime_state)
}

fn sync_timer_on_launch_impl(
    app: &tauri::AppHandle,
    config_manager: &Arc<ConfigManager>,
    app_state: &Mutex<AppState>,
    rest_lock_guard: &RestLockGuardState,
) -> Result<(), String> {
    let config = config_manager.get()?;
    if !config.activation.activated {
        return Ok(());
    }

    let runtime = {
        let state = app_state.lock().map_err(|e| e.to_string())?;
        state.timer.get_runtime()
    };

    let should_resume = matches!(runtime.state, TimerState::Running | TimerState::Paused);
    let should_auto_start = config.startup.start_timer_automatically
        && runtime.state == TimerState::Idle
        && runtime.phase == TimerPhase::Work;

    if !should_resume && !should_auto_start {
        return Ok(());
    }

    if should_resume && runtime.state == TimerState::Running && runtime.phase == TimerPhase::Rest {
        let enforce = config.timer.enforce_relock_during_rest;
        set_rest_lock_guard(rest_lock_guard, enforce);
    }

    let state = app_state.lock().map_err(|e| e.to_string())?;
    configure_timer_callback(
        &state.timer,
        app.clone(),
        Arc::clone(config_manager),
    );

    if should_resume {
        if !state.timer.is_thread_active() {
            state.timer.resume_persisted_countdown()?;
        }
    } else {
        state
            .timer
            .set_phase_interval(TimerPhase::Work, config.timer.interval_minutes)?;
        state.timer.start()?;
    }

    let _ = persist_timer_runtime(config_manager, &state.timer);
    drop(state);

    let runtime = app_state.lock().map_err(|e| e.to_string())?.timer.get_runtime();
    let _ = app.emit("timer-update", &runtime);
    update_tray_tooltip(app, &runtime);
    Ok(())
}

fn apply_startup_behavior(app: &tauri::AppHandle) {
    let config_manager = match app.try_state::<Arc<ConfigManager>>() {
        Some(cm) => cm,
        None => return,
    };

    let config = match config_manager.get() {
        Ok(cfg) => cfg,
        Err(_) => return,
    };

    if config.startup.start_minimized {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
    }

    let app_state = match app.try_state::<Mutex<AppState>>() {
        Some(state) => state,
        None => return,
    };

    let rest_lock_guard = match app.try_state::<RestLockGuardState>() {
        Some(guard) => guard,
        None => return,
    };

    if let Err(err) = sync_timer_on_launch_impl(
        app,
        config_manager.inner(),
        app_state.inner(),
        rest_lock_guard.inner(),
    ) {
        eprintln!("启动时同步计时器失败: {}", err);
    }
}

/// 前端就绪后再次同步计时器（确保后台线程与 UI 监听已连接）
#[tauri::command]
fn sync_timer_on_launch(
    app_handle: tauri::AppHandle,
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    rest_lock_guard: tauri::State<RestLockGuardState>,
) -> Result<TimerRuntime, String> {
    activation::ensure_activated(config_manager.inner())?;
    sync_timer_on_launch_impl(
        &app_handle,
        config_manager.inner(),
        state.inner(),
        rest_lock_guard.inner(),
    )?;
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.timer.get_runtime())
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

fn save_runtime_state_with_delay(
    config_manager: &tauri::State<Arc<ConfigManager>>,
    runtime_state: config::RuntimeState,
) -> Result<(), String> {
    let _ = config::reset_daily_delay_quota_if_needed(config_manager.inner());
    let mut state_to_save = runtime_state;
    if let Ok(cfg) = config_manager.get() {
        state_to_save.delay_count = cfg.runtime_state.delay_count;
        state_to_save.delay_quota_date = cfg.runtime_state.delay_quota_date;
    }
    config::update_runtime_state(config_manager, state_to_save)
}

fn configure_timer_callback(
    timer: &TimerEngine,
    app_handle: tauri::AppHandle,
    config_manager: Arc<ConfigManager>,
) {
    let app_handle_clone = app_handle.clone();
    timer.set_callback(move |runtime: TimerRuntime| {
        let _ = config::reset_daily_delay_quota_if_needed(&config_manager);
        let is_finished = runtime.remaining_seconds == 0 && runtime.state == TimerState::Idle;

        // 发送更新到前端
        let _ = app_handle_clone.emit("timer-update", &runtime);
        update_tray_tooltip(&app_handle_clone, &runtime);

        // 休息阶段软强制重锁：用户手动解锁后会被再次锁回，直到休息阶段结束
        if runtime.state == TimerState::Running && runtime.phase == TimerPhase::Rest {
            let guard_state = app_handle_clone.state::<RestLockGuardState>();
            if guard_state.enforce_during_rest.load(Ordering::Relaxed) {
                let now = unix_now_secs();
                let last = guard_state.last_relock_unix_secs.load(Ordering::Relaxed);
                if now.saturating_sub(last) >= REST_RELOCK_INTERVAL_SECS {
                    system::lock_screen();
                    guard_state
                        .last_relock_unix_secs
                        .store(now, Ordering::Relaxed);
                }
            }
        }

        // 仅工作阶段触发提前通知
        if runtime.state == TimerState::Running && runtime.phase == TimerPhase::Work {
            if let Ok(config) = config_manager.get() {
                let notice_seconds = config.timer.advance_notice_seconds;
                if notice_seconds > 0 && runtime.remaining_seconds == notice_seconds {
                    let is_effective = schedule::ScheduleChecker::is_effective(
                        config.schedule.time_limit_enabled,
                        &config.schedule.start_time,
                        &config.schedule.end_time,
                        config.schedule.weekday_limit_enabled,
                        &config.schedule.weekdays,
                        &config.schedule.logic,
                    );

                    if is_effective {
                        notifier::Notifier::notify_before_execution(
                            &app_handle_clone,
                            &config.action.action_type,
                            notice_seconds,
                            config.runtime_state.delay_count,
                            config.timer.max_delay_times,
                            config.timer.delay_options.clone(),
                        );
                    }
                }
            }
        }

        if !is_finished {
            return;
        }

        // 休息阶段结束，通知前端进入下一轮
        if runtime.phase == TimerPhase::Rest {
            let _ = app_handle_clone.emit("loop-rest-finished", ());
            return;
        }

        // 工作阶段结束，按生效规则执行动作
        match config_manager.get() {
            Ok(config) => {
                let is_effective = schedule::ScheduleChecker::is_effective(
                    config.schedule.time_limit_enabled,
                    &config.schedule.start_time,
                    &config.schedule.end_time,
                    config.schedule.weekday_limit_enabled,
                    &config.schedule.weekdays,
                    &config.schedule.logic,
                );

                if is_effective {
                    println!("倒计时结束，执行动作: {}", config.action.action_type);
                    system::execute_action(&config.action.action_type);
                } else {
                    println!("倒计时结束，但不在生效规则内，跳过动作执行");
                }
            }
            Err(err) => {
                eprintln!("读取配置失败，跳过动作执行: {}", err);
            }
        }

        // 发送计时结束事件，前端可决定是否进入下一轮
        let _ = app_handle_clone.emit("timer-finished", ());
    });
}

// ===== Tauri Commands =====

/// 获取计时器运行时状态
#[tauri::command]
fn get_timer_runtime(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
) -> Result<TimerRuntime, String> {
    activation::ensure_activated(config_manager.inner())?;
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.timer.get_runtime())
}

/// 获取计时器配置(旧版)
#[tauri::command]
fn get_timer_engine_config(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
) -> Result<TimerSettings, String> {
    activation::ensure_activated(config_manager.inner())?;
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.timer.get_config())
}

/// 设置时间间隔（分钟）
#[tauri::command]
fn set_timer_interval(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    minutes: u64,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.timer.set_interval(minutes)?;

    // 持久化主间隔和运行时状态
    let runtime_state = app_state.timer.to_runtime_state();
    drop(app_state);
    config_manager.update(|c| c.timer.interval_minutes = minutes)?;
    let _ = save_runtime_state_with_delay(&config_manager, runtime_state);

    Ok(())
}

/// 开始计时
#[tauri::command]
fn start_timer(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    rest_lock_guard: tauri::State<RestLockGuardState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    set_rest_lock_guard(rest_lock_guard.inner(), false);
    let config = config_manager.get()?;
    let app_state = state.lock().map_err(|e| e.to_string())?;

    // 从空闲态开始时，强制回到工作阶段，使用当前主间隔
    if app_state.timer.get_runtime().state == TimerState::Idle {
        app_state
            .timer
            .set_phase_interval(TimerPhase::Work, config.timer.interval_minutes)?;
    }

    configure_timer_callback(&app_state.timer, app_handle.clone(), Arc::clone(config_manager.inner()));

    app_state.timer.start()?;

    // 保存运行状态
    let runtime_state = app_state.timer.to_runtime_state();
    drop(app_state);
    let _ = save_runtime_state_with_delay(&config_manager, runtime_state);

    // 立即发送一次状态更新
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime.clone());
    update_tray_tooltip(&app_handle, &runtime);

    Ok(())
}

/// 暂停计时
#[tauri::command]
fn pause_timer(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    rest_lock_guard: tauri::State<RestLockGuardState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    let app_state = state.lock().map_err(|e| e.to_string())?;
    if rest_lock_guard.enforce_during_rest.load(Ordering::Relaxed) {
        let runtime = app_state.timer.get_runtime();
        if runtime.phase == TimerPhase::Rest && runtime.state == TimerState::Running {
            return Err("休息阶段禁止暂停".to_string());
        }
    }
    app_state.timer.pause()?;

    // 保存暂停状态
    let _ = config::reset_daily_delay_quota_if_needed(config_manager.inner());
    let mut runtime_state = app_state.timer.to_runtime_state();
    drop(app_state);
    if let Ok(config) = config_manager.get() {
        runtime_state.delay_count = config.runtime_state.delay_count;
        runtime_state.delay_quota_date = config.runtime_state.delay_quota_date.clone();
    }
    let _ = config::update_runtime_state(&config_manager, runtime_state);

    // 发送状态更新
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime.clone());
    update_tray_tooltip(&app_handle, &runtime);

    Ok(())
}

/// 继续计时
#[tauri::command]
fn resume_timer(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    let app_state = state.lock().map_err(|e| e.to_string())?;

    configure_timer_callback(&app_state.timer, app_handle.clone(), Arc::clone(config_manager.inner()));

    app_state.timer.resume()?;

    // 保存运行状态
    let _ = config::reset_daily_delay_quota_if_needed(config_manager.inner());
    let mut runtime_state = app_state.timer.to_runtime_state();
    drop(app_state);
    if let Ok(config) = config_manager.get() {
        runtime_state.delay_count = config.runtime_state.delay_count;
        runtime_state.delay_quota_date = config.runtime_state.delay_quota_date.clone();
    }
    let _ = config::update_runtime_state(&config_manager, runtime_state);

    // 发送状态更新
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime.clone());
    update_tray_tooltip(&app_handle, &runtime);

    Ok(())
}

/// 开始循环休息阶段计时
#[tauri::command]
fn start_loop_rest_timer(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    rest_lock_guard: tauri::State<RestLockGuardState>,
    app_handle: tauri::AppHandle,
    minutes: u64,
    enforce_lock: bool,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    if minutes == 0 {
        return Err("循环间隔必须大于0分钟".to_string());
    }

    set_rest_lock_guard(rest_lock_guard.inner(), enforce_lock);
    if enforce_lock {
        system::lock_screen();
    }

    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state
        .timer
        .set_phase_interval(TimerPhase::Rest, minutes)?;
    configure_timer_callback(&app_state.timer, app_handle.clone(), Arc::clone(config_manager.inner()));
    app_state.timer.start()?;

    let _ = config::reset_daily_delay_quota_if_needed(config_manager.inner());
    let mut runtime_state = app_state.timer.to_runtime_state();
    drop(app_state);
    if let Ok(config) = config_manager.get() {
        runtime_state.delay_count = config.runtime_state.delay_count;
        runtime_state.delay_quota_date = config.runtime_state.delay_quota_date.clone();
    }
    let _ = config::update_runtime_state(&config_manager, runtime_state);

    let app_state = state.lock().map_err(|e| e.to_string())?;
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime.clone());
    update_tray_tooltip(&app_handle, &runtime);

    Ok(())
}

/// 开始下一轮工作阶段计时（使用配置中的主间隔）
#[tauri::command]
fn start_work_cycle_timer(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    rest_lock_guard: tauri::State<RestLockGuardState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    set_rest_lock_guard(rest_lock_guard.inner(), false);
    let _ = config::reset_daily_delay_quota_if_needed(config_manager.inner());
    let config = config_manager.get()?;

    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state
        .timer
        .set_phase_interval(TimerPhase::Work, config.timer.interval_minutes)?;
    configure_timer_callback(&app_state.timer, app_handle.clone(), Arc::clone(config_manager.inner()));
    app_state.timer.start()?;

    let mut runtime_state = app_state.timer.to_runtime_state();
    drop(app_state);
    runtime_state.delay_count = config.runtime_state.delay_count;
    runtime_state.delay_quota_date = config.runtime_state.delay_quota_date.clone();
    let _ = config::update_runtime_state(&config_manager, runtime_state);

    let app_state = state.lock().map_err(|e| e.to_string())?;
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime.clone());
    update_tray_tooltip(&app_handle, &runtime);

    Ok(())
}

/// 停止并重置计时器
#[tauri::command]
fn stop_timer(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    rest_lock_guard: tauri::State<RestLockGuardState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    let _ = config::reset_daily_delay_quota_if_needed(config_manager.inner());
    let config = config_manager.get()?;
    let app_state = state.lock().map_err(|e| e.to_string())?;
    if rest_lock_guard.enforce_during_rest.load(Ordering::Relaxed) {
        let runtime = app_state.timer.get_runtime();
        if runtime.phase == TimerPhase::Rest && runtime.state == TimerState::Running {
            return Err("休息阶段禁止停止".to_string());
        }
    }
    set_rest_lock_guard(rest_lock_guard.inner(), false);
    app_state
        .timer
        .set_phase_interval(TimerPhase::Work, config.timer.interval_minutes)?;
    app_state.timer.stop();

    // 保存重置后的状态
    let mut runtime_state = app_state.timer.to_runtime_state();
    drop(app_state);
    runtime_state.delay_count = config.runtime_state.delay_count;
    runtime_state.delay_quota_date = config.runtime_state.delay_quota_date.clone();
    let _ = config::update_runtime_state(&config_manager, runtime_state);

    // 发送状态更新
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime.clone());
    update_tray_tooltip(&app_handle, &runtime);

    Ok(())
}

/// 获取格式化的时间显示 (MM:SS)
#[tauri::command]
fn get_formatted_time(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
) -> Result<String, String> {
    activation::ensure_activated(config_manager.inner())?;
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let runtime = app_state.timer.get_runtime();
    let minutes = runtime.remaining_seconds / 60;
    let seconds = runtime.remaining_seconds % 60;
    Ok(format!("{:02}:{:02}", minutes, seconds))
}

/// 检查当前时间是否在生效规则内
#[tauri::command]
fn check_schedule_effective(
    config_manager: tauri::State<Arc<ConfigManager>>,
) -> Result<bool, String> {
    activation::ensure_activated(config_manager.inner())?;
    let config = config_manager.get()?;
    Ok(schedule::ScheduleChecker::is_effective(
        config.schedule.time_limit_enabled,
        &config.schedule.start_time,
        &config.schedule.end_time,
        config.schedule.weekday_limit_enabled,
        &config.schedule.weekdays,
        &config.schedule.logic,
    ))
}

#[tauri::command]
fn set_window_topmost(app_handle: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.set_always_on_top(enabled).map_err(|e| e.to_string())?;
        if enabled {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    Ok(())
}

/// 保存计时器完成后的空闲状态
#[tauri::command]
fn save_timer_finished_state(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    let runtime = app_state.timer.get_runtime();

    let _ = config::reset_daily_delay_quota_if_needed(config_manager.inner());
    let cfg = config_manager.get()?;
    // 保存空闲状态
    let runtime_state = crate::config::RuntimeState {
        timer_status: "Idle".to_string(),
        remaining_seconds: runtime.total_seconds,
        total_seconds: runtime.total_seconds,
        last_update: chrono::Local::now().to_rfc3339(),
        delay_count: cfg.runtime_state.delay_count,
        delay_quota_date: cfg.runtime_state.delay_quota_date,
        cycle_phase: "Work".to_string(),
    };
    drop(app_state);
    config::update_runtime_state(&config_manager, runtime_state)
}

pub fn run(safe_mode: bool) {
    // 初始化配置管理器
    let config_manager = Arc::new(ConfigManager::new().expect("Failed to initialize config manager"));
    let runtime_flags = RuntimeFlags { safe_mode };

    // 从配置加载应用状态
    let app_state = AppState::from_config(&config_manager);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(app_state))
        .manage(Arc::clone(&config_manager))
        .manage(runtime_flags)
        .manage(RestLockGuardState::default())
        .invoke_handler(tauri::generate_handler![
            get_timer_runtime,
            get_timer_engine_config,
            set_timer_interval,
            start_timer,
            pause_timer,
            resume_timer,
            start_loop_rest_timer,
            start_work_cycle_timer,
            stop_timer,
            get_formatted_time,
            save_timer_finished_state,
            system::execute_system_action,
            config::get_config,
            config::update_timer_config,
            config::update_schedule_config,
            config::update_action_config,
            config::update_startup_config,
            config::update_ui_config,
            check_schedule_effective,
            sync_timer_on_launch,
            notifier::delay_execution,
            notifier::confirm_execution,
            notifier::cancel_execution,
            set_window_topmost,
            startup::is_auto_start_enabled,
            startup::set_auto_start,
            logger::get_log_directory,
            logger::open_log_file,
            activation::get_activation_status,
            activation::activate_with_code,
            #[cfg(feature = "activation-admin")]
            activation::generate_activation_codes,
            security::get_security_status,
            security::setup_password,
            security::verify_exit_password,
            security::reset_password,
        ])
        .setup(|app| {
            // 单实例检查（简化版 - 使用文件锁方式）
            // 注：Windows下单实例通常在main.rs中更早处理
            // 这里仅作为运行时检查

            // 创建托盘菜单项
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let pause_item = MenuItem::with_id(app, "pause", "暂停/继续", true, None::<&str>)?;
            let stop_item = MenuItem::with_id(app, "stop", "停止并重置", true, None::<&str>)?;
            let separator1 = PredefinedMenuItem::separator(app)?;
            let quick_15 = MenuItem::with_id(app, "quick_15", "快速: 15分钟", true, None::<&str>)?;
            let quick_30 = MenuItem::with_id(app, "quick_30", "快速: 30分钟", true, None::<&str>)?;
            let quick_60 = MenuItem::with_id(app, "quick_60", "快速: 60分钟", true, None::<&str>)?;
            let separator2 = PredefinedMenuItem::separator(app)?;
            let about_item = MenuItem::with_id(app, "about", "关于", true, None::<&str>)?;
            let separator3 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[
                &show_item,
                &separator1,
                &pause_item,
                &stop_item,
                &quick_15,
                &quick_30,
                &quick_60,
                &separator2,
                &about_item,
                &separator3,
                &quit_item,
            ])?;

            // 克隆app_handle用于托盘事件
            let app_handle_for_tray = app.app_handle().clone();

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("TimerApp - 停止")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "pause" => {
                            // 获取当前状态，决定是暂停还是继续
                            if let Ok(state) = app.state::<Mutex<AppState>>().lock() {
                                let runtime = state.timer.get_runtime();
                                match runtime.state {
                                    TimerState::Running => {
                                        drop(state);
                                        let _ = app.emit("tray-pause", ());
                                    }
                                    TimerState::Paused => {
                                        drop(state);
                                        let _ = app.emit("tray-resume", ());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        "stop" => {
                            let _ = app.emit("tray-stop", ());
                        }
                        "quick_15" => {
                            let _ = app.emit("tray-quick-set", 15u64);
                        }
                        "quick_30" => {
                            let _ = app.emit("tray-quick-set", 30u64);
                        }
                        "quick_60" => {
                            let _ = app.emit("tray-quick-set", 60u64);
                        }
                        "about" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                            let _ = app.emit("show-about", ());
                        }
                        "quit" => {
                            let flags = app.state::<RuntimeFlags>();
                            if flags.safe_mode {
                                app.exit(0);
                            } else {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                                let _ = app.emit("exit-requested", ());
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // 保存托盘引用
            app.manage(Mutex::new(app_handle_for_tray));

            // 设置关闭时最小化到托盘，并持久化计时器状态
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            let app_handle_for_close = app.app_handle().clone();
            window.on_window_event(move |event| {
                match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        if let (Some(cm), Some(st)) = (
                            app_handle_for_close.try_state::<Arc<ConfigManager>>(),
                            app_handle_for_close.try_state::<Mutex<AppState>>(),
                        ) {
                            if let Ok(state) = st.lock() {
                                let _ = persist_timer_runtime(cm.inner(), &state.timer);
                            }
                        }
                        let _ = window_clone.hide();
                    }
                    WindowEvent::Focused(true) => {
                        if let Some(st) = app_handle_for_close.try_state::<Mutex<AppState>>() {
                            if let Ok(state) = st.lock() {
                                let runtime = state.timer.get_runtime();
                                let _ = app_handle_for_close.emit("timer-update", &runtime);
                                update_tray_tooltip(&app_handle_for_close, &runtime);
                            }
                        }
                    }
                    _ => {}
                }
            });

            let app_handle = app.app_handle().clone();
            apply_startup_behavior(&app_handle);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
