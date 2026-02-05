//! 通知模块 - E4: 提示优化
//! 差异化提示策略：锁屏=弹窗，休眠/关机=托盘通知

use tauri::{AppHandle, Emitter};
use serde::Serialize;

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
    pub action_type: String, // "lock" | "suspend" | "shutdown"
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
    /// - `action_type`: "lock" | "suspend" | "shutdown"
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
        let (notification_type, title, message) = match action_type {
            "lock" => {
                // 锁屏使用模态弹窗
                (
                    NotificationType::Modal,
                    "即将锁屏".to_string(),
                    format!("系统将在 {} 秒后自动锁屏", countdown_seconds),
                )
            }
            "suspend" => {
                // 休眠使用托盘通知
                (
                    NotificationType::Tray,
                    "即将休眠".to_string(),
                    format!("系统将在 {} 秒后自动进入休眠状态", countdown_seconds),
                )
            }
            "shutdown" => {
                // 关机使用托盘通知
                (
                    NotificationType::Tray,
                    "即将关机".to_string(),
                    format!("系统将在 {} 秒后自动关机", countdown_seconds),
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
            action_type: action_type.to_string(),
            delay_count,
            max_delay_times,
            delay_options,
        };

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
    app_handle: AppHandle,
    minutes: u64,
    delay_count: u32,
    max_delay_times: u32,
) -> Result<bool, String> {
    if delay_count >= max_delay_times {
        return Ok(false); // 不能再延后
    }

    Notifier::notify_delayed(&app_handle, minutes, delay_count + 1, max_delay_times);
    Ok(true)
}

/// Tauri命令：用户选择立即执行
#[tauri::command]
pub fn confirm_execution(app_handle: AppHandle) {
    let _ = app_handle.emit("timer-confirm-execute", ());
}

/// Tauri命令：用户选择取消
#[tauri::command]
pub fn cancel_execution(app_handle: AppHandle) {
    Notifier::notify_cancelled(&app_handle);
}
