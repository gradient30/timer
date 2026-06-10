//! 通知模块 - E4: 提示优化
//! 提示策略：锁屏/关机使用弹窗通知

#![allow(dead_code)]

use tauri::{AppHandle, Emitter, Manager};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use crate::{
    activation,
    AppState,
    config::{self, ConfigManager, RuntimeState},
    system,
    timer::{TimerRuntime, TimerState},
};

fn format_remaining(seconds: u64) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", minutes, secs)
}

fn update_tray_tooltip(app_handle: &AppHandle, runtime: &TimerRuntime) {
    let tooltip = match runtime.state {
        TimerState::Running => format!("TimerApp - {}", format_remaining(runtime.remaining_seconds)),
        TimerState::Paused => format!("TimerApp - 暂停 {}", format_remaining(runtime.remaining_seconds)),
        TimerState::Idle => "TimerApp - 停止".to_string(),
    };

    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(tooltip.as_str()));
    }
}

/// 通知类型
#[derive(Debug, Clone, Serialize)]
pub enum NotificationType {
    /// 模态弹窗（需要用户响应）
    Modal,
    /// 托盘通知（非阻塞）
    Tray,
}

/// 通知内容
#[derive(Debug, Clone, Serialize)]
pub struct Notification {
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub countdown_seconds: u64,
    pub action_type: String, // "lock" | "shutdown"
    pub delay_count: u32,
    pub max_delay_times: u32,
    pub delay_options: Vec<u64>,
}

/// 通知管理器
pub struct Notifier;

impl Notifier {
    /// 发送执行前通知
    ///
    /// # 参数
    /// - `action_type`: "lock" | "shutdown"
    /// - `countdown_seconds`: 提前通知秒数
    /// - `delay_count`: 已延后次数
    /// - `max_delay_times`: 最大延后次数
    pub fn notify_before_execution(
        app_handle: &AppHandle,
        action_type: &str,
        countdown_seconds: u64,
        delay_count: u32,
        max_delay_times: u32,
        delay_options: Vec<u64>,
    ) {
        let (notification_type, title, message, effective_action_type) = match action_type {
            "lock" => {
                // 锁屏使用模态弹窗
                (
                    NotificationType::Modal,
                    "即将锁屏".to_string(),
                    format!("系统将在 {} 秒后自动锁屏", countdown_seconds),
                    "lock",
                )
            }
            "shutdown" => {
                // 关机使用模态弹窗
                (
                    NotificationType::Modal,
                    "即将关机".to_string(),
                    format!("系统将在 {} 秒后自动关机", countdown_seconds),
                    "shutdown",
                )
            }
            "suspend" => {
                // 兼容旧配置：休眠动作已禁用，改为锁屏
                (
                    NotificationType::Modal,
                    "休眠已禁用".to_string(),
                    format!("系统将在 {} 秒后自动锁屏（原计划动作：休眠）", countdown_seconds),
                    "lock",
                )
            }
            _ => {
                eprintln!("未知的操作类型: {}", action_type);
                return;
            }
        };

        let notification = Notification {
            notification_type,
            title,
            message,
            countdown_seconds,
            action_type: effective_action_type.to_string(),
            delay_count,
            max_delay_times,
            delay_options,
        };

        if matches!(notification.notification_type, NotificationType::Modal) {
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.set_always_on_top(true);
            }
        }

        // 发送通知事件
        if let Err(e) = app_handle.emit("timer-notify", notification) {
            eprintln!("发送通知失败: {}", e);
        }
    }

    /// 发送倒计时更新
    pub fn notify_countdown_update(
        app_handle: &AppHandle,
        remaining_seconds: u64,
    ) {
        let _ = app_handle.emit("timer-countdown-update", remaining_seconds);
    }

    /// 通知执行已延后
    pub fn notify_delayed(
        app_handle: &AppHandle,
        delay_minutes: u64,
        delay_count: u32,
        max_delay_times: u32,
    ) {
        let message = if delay_count >= max_delay_times {
            "已用完所有延后机会，下次将立即执行".to_string()
        } else {
            format!("已延后 {} 分钟，还剩余 {} 次机会", delay_minutes, max_delay_times - delay_count)
        };

        let _ = app_handle.emit("timer-delayed", serde_json::json!({
            "message": message,
            "delay_count": delay_count,
            "max_delay_times": max_delay_times,
        }));
    }

    /// 通知已取消
    pub fn notify_cancelled(app_handle: &AppHandle) {
        let _ = app_handle.emit("timer-cancelled", ());
    }
}

/// Tauri命令：用户选择延后执行
#[tauri::command]
pub fn delay_execution(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    app_handle: AppHandle,
    minutes: u64,
    _delay_count: u32,
    _max_delay_times: u32,
) -> Result<bool, String> {
    activation::ensure_activated(config_manager.inner())?;
    config::reset_daily_delay_quota_if_needed(config_manager.inner())?;
    let config = config_manager.get()?;
    let current_delay_count = config.runtime_state.delay_count;
    let max_allowed = config.timer.max_delay_times;

    if current_delay_count >= max_allowed {
        return Ok(false); // 不能再延后
    }

    if minutes == 0 {
        return Ok(true);
    }

    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.timer.add_delay(minutes)?;
    let runtime = app_state.timer.get_runtime();
    let runtime_state = RuntimeState {
        timer_status: match runtime.state {
            TimerState::Idle => "Idle".to_string(),
            TimerState::Running => "Running".to_string(),
            TimerState::Paused => "Paused".to_string(),
        },
        remaining_seconds: runtime.remaining_seconds,
        total_seconds: runtime.total_seconds,
        last_update: runtime.last_update.clone().unwrap_or_else(|| chrono::Local::now().to_rfc3339()),
        delay_count: current_delay_count + 1,
        delay_quota_date: config.runtime_state.delay_quota_date.clone(),
        cycle_phase: match runtime.phase {
            crate::timer::TimerPhase::Work => "Work".to_string(),
            crate::timer::TimerPhase::Rest => "Rest".to_string(),
        },
    };
    drop(app_state);
    config::update_runtime_state(&config_manager, runtime_state)?;

    let _ = app_handle.emit("timer-update", &runtime);
    update_tray_tooltip(&app_handle, &runtime);

    Notifier::notify_delayed(&app_handle, minutes, current_delay_count + 1, max_allowed);
    Ok(true)
}

/// Tauri命令：用户选择立即执行
#[tauri::command]
pub fn confirm_execution(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    config::reset_daily_delay_quota_if_needed(config_manager.inner())?;
    let config = config_manager.get()?;
    system::execute_action(&config.action.action_type);

    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.timer.stop();
    let runtime = app_state.timer.get_runtime();
    drop(app_state);

    let runtime_state = RuntimeState {
        timer_status: "Idle".to_string(),
        remaining_seconds: runtime.total_seconds,
        total_seconds: runtime.total_seconds,
        last_update: chrono::Local::now().to_rfc3339(),
        delay_count: config.runtime_state.delay_count,
        delay_quota_date: config.runtime_state.delay_quota_date.clone(),
        cycle_phase: "Work".to_string(),
    };
    config::update_runtime_state(&config_manager, runtime_state)?;

    let _ = app_handle.emit("timer-update", &runtime);
    update_tray_tooltip(&app_handle, &runtime);
    Ok(())
}

/// Tauri命令：用户选择取消
#[tauri::command]
pub fn cancel_execution(
    state: tauri::State<Mutex<AppState>>,
    config_manager: tauri::State<Arc<ConfigManager>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    config::reset_daily_delay_quota_if_needed(config_manager.inner())?;
    let config = config_manager.get()?;
    let current_delay_count = config.runtime_state.delay_count;
    let max_allowed = config.timer.max_delay_times;
    if current_delay_count >= max_allowed {
        return Err("已达到当日取消上限（次日自动重置），请立即执行或等待自动执行".to_string());
    }

    let app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.timer.stop();
    let runtime = app_state.timer.get_runtime();
    drop(app_state);

    let runtime_state = RuntimeState {
        timer_status: "Idle".to_string(),
        remaining_seconds: runtime.total_seconds,
        total_seconds: runtime.total_seconds,
        last_update: chrono::Local::now().to_rfc3339(),
        delay_count: current_delay_count + 1,
        delay_quota_date: config.runtime_state.delay_quota_date.clone(),
        cycle_phase: "Work".to_string(),
    };
    config::update_runtime_state(&config_manager, runtime_state)?;

    let _ = app_handle.emit("timer-update", &runtime);
    update_tray_tooltip(&app_handle, &runtime);
    Notifier::notify_cancelled(&app_handle);
    Ok(())
}
