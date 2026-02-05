use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, WindowEvent,
};
use std::sync::Mutex;

// 导入各模块
mod timer;
mod system;
pub mod single_instance;
mod config;
mod schedule;
mod notifier;
mod startup;
mod logger;

use timer::{TimerEngine, TimerRuntime, TimerConfig as TimerSettings, TimerState};
use config::{ConfigManager, AppConfig, TimerConfig, ScheduleConfig, ActionConfig, StartupConfig};
use std::sync::Arc;

// 全局定时器引擎状态
pub struct AppState {
    pub timer: TimerEngine,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            timer: TimerEngine::new(),
        }
    }
}

// ===== Tauri Commands =====

/// 获取计时器运行时状态
#[tauri::command]
fn get_timer_runtime(state: tauri::State<Mutex<AppState>>) -> Result<TimerRuntime, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.timer.get_runtime())
}

/// 获取计时器配置(旧版)
#[tauri::command]
fn get_timer_engine_config(state: tauri::State<Mutex<AppState>>) -> Result<TimerSettings, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.timer.get_config())
}

/// 设置时间间隔（分钟）
#[tauri::command]
fn set_timer_interval(
    state: tauri::State<Mutex<AppState>>,
    minutes: u64,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.timer.set_interval(minutes)
}

/// 开始计时
#[tauri::command]
fn start_timer(
    state: tauri::State<Mutex<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;

    // 设置回调，用于向前端发送更新和倒计时结束处理
    let app_handle_clone = app_handle.clone();
    app_state.timer.set_callback(move |runtime: TimerRuntime| {
        // 检查是否倒计时结束
        let is_finished = runtime.remaining_seconds == 0 && runtime.state == TimerState::Idle;

        // 发送更新到前端
        let _ = app_handle_clone.emit("timer-update", &runtime);

        // 倒计时结束，执行锁屏
        if is_finished {
            println!("倒计时结束，执行锁屏");
            system::lock_screen();
        }
    });

    app_state.timer.start()?;

    // 立即发送一次状态更新
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime);

    Ok(())
}

/// 暂停计时
#[tauri::command]
fn pause_timer(
    state: tauri::State<Mutex<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.timer.pause()?;

    // 发送状态更新
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime);

    Ok(())
}

/// 继续计时
#[tauri::command]
fn resume_timer(
    state: tauri::State<Mutex<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;

    // 设置回调
    let app_handle_clone = app_handle.clone();
    app_state.timer.set_callback(move |runtime: TimerRuntime| {
        let _ = app_handle_clone.emit("timer-update", runtime);
    });

    app_state.timer.resume()?;

    // 发送状态更新
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime);

    Ok(())
}

/// 停止并重置计时器
#[tauri::command]
fn stop_timer(
    state: tauri::State<Mutex<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.timer.stop();

    // 发送状态更新
    let runtime = app_state.timer.get_runtime();
    let _ = app_handle.emit("timer-update", runtime);

    Ok(())
}

/// 获取格式化的时间显示 (MM:SS)
#[tauri::command]
fn get_formatted_time(state: tauri::State<Mutex<AppState>>) -> Result<String, String> {
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

pub fn run() {
    // 初始化配置管理器
    let config_manager = Arc::new(ConfigManager::new().expect("Failed to initialize config manager"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::new()))
        .manage(config_manager)
        .invoke_handler(tauri::generate_handler![
            get_timer_runtime,
            get_timer_engine_config,
            set_timer_interval,
            start_timer,
            pause_timer,
            resume_timer,
            stop_timer,
            get_formatted_time,
            system::execute_system_action,
            config::get_config,
            config::update_timer_config,
            config::update_schedule_config,
            config::update_action_config,
            config::update_startup_config,
            check_schedule_effective,
            notifier::delay_execution,
            notifier::confirm_execution,
            notifier::cancel_execution,
            startup::is_auto_start_enabled,
            startup::set_auto_start,
            logger::get_log_directory,
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

            let _tray = TrayIconBuilder::new()
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
                            println!("TimerApp v0.1.0 - 一款简单的Windows定时器工具");
                        }
                        "quit" => {
                            app.exit(0);
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

            // 设置关闭时最小化到托盘
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
