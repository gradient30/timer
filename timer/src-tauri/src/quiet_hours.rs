//! 用餐/夜间休息时段（非番茄钟循环锁屏休息）
//! 半开区间 [start, end)：开始时刻强制停止倒计时，结束时刻重置并自动开始工作倒计时。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuietHoursWindow {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuietHoursConfig {
    #[serde(default = "default_quiet_hours_enabled")]
    pub enabled: bool,
    #[serde(default = "default_windows")]
    pub windows: Vec<QuietHoursWindow>,
}

fn default_quiet_hours_enabled() -> bool {
    true
}

impl Default for QuietHoursConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            windows: default_windows(),
        }
    }
}

pub fn default_windows() -> Vec<QuietHoursWindow> {
    vec![
        QuietHoursWindow {
            start: "12:00".to_string(),
            end: "13:00".to_string(),
        },
        QuietHoursWindow {
            start: "18:00".to_string(),
            end: "19:00".to_string(),
        },
        QuietHoursWindow {
            start: "00:00".to_string(),
            end: "09:00".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuietHoursTickAction {
    None,
    ForceStop,
    ForceStartWork,
}

pub fn parse_hhmm(time_str: &str) -> Option<u32> {
    let mut parts = time_str.trim().split(':');
    let hour: u32 = parts.next()?.parse().ok()?;
    let minute: u32 = parts.next()?.parse().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(hour * 60 + minute)
}

pub fn is_in_half_open_range(now_minutes: u32, start: u32, end: u32) -> bool {
    if start == end {
        return false;
    }
    if start < end {
        now_minutes >= start && now_minutes < end
    } else {
        now_minutes >= start || now_minutes < end
    }
}

pub fn is_in_quiet_hours(now_minutes: u32, windows: &[QuietHoursWindow]) -> bool {
    windows.iter().any(|window| {
        match (parse_hhmm(&window.start), parse_hhmm(&window.end)) {
            (Some(start), Some(end)) => is_in_half_open_range(now_minutes, start, end),
            _ => false,
        }
    })
}

pub fn is_quiet_hours_active(config: &QuietHoursConfig, now_minutes: u32) -> bool {
    config.enabled && is_in_quiet_hours(now_minutes, &config.windows)
}

pub fn local_now_minutes() -> u32 {
    use chrono::Timelike;
    let now = chrono::Local::now();
    now.hour() * 60 + now.minute()
}

pub fn tick_action(was_inside: Option<bool>, now_inside: bool) -> QuietHoursTickAction {
    if now_inside {
        QuietHoursTickAction::ForceStop
    } else if was_inside == Some(true) {
        QuietHoursTickAction::ForceStartWork
    } else {
        QuietHoursTickAction::None
    }
}

pub fn next_inside_flag(
    was_inside: Option<bool>,
    now_inside: bool,
    action_ok: bool,
) -> Option<bool> {
    if action_ok {
        Some(now_inside)
    } else {
        was_inside
    }
}

pub fn should_force_reset(is_idle: bool, is_work_phase: bool, remaining_seconds: u64) -> bool {
    !is_idle || !is_work_phase || remaining_seconds == 0
}

pub const BLOCK_MESSAGE: &str = "当前处于用餐/夜间休息时段，倒计时已停止";

fn format_hhmm(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

pub fn normalize_config(config: &QuietHoursConfig) -> Result<QuietHoursConfig, String> {
    let mut windows = Vec::with_capacity(config.windows.len());
    for (index, window) in config.windows.iter().enumerate() {
        let start = parse_hhmm(&window.start)
            .ok_or_else(|| format!("第{}个休息时段的开始时间无效", index + 1))?;
        let end = parse_hhmm(&window.end)
            .ok_or_else(|| format!("第{}个休息时段的结束时间无效", index + 1))?;
        windows.push(QuietHoursWindow {
            start: format_hhmm(start),
            end: format_hhmm(end),
        });
    }
    Ok(QuietHoursConfig {
        enabled: config.enabled,
        windows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minutes(hour: u32, minute: u32) -> u32 {
        hour * 60 + minute
    }

    #[test]
    fn parse_hhmm_accepts_hour_minute_and_optional_seconds() {
        assert_eq!(parse_hhmm("00:00"), Some(0));
        assert_eq!(parse_hhmm("09:00"), Some(540));
        assert_eq!(parse_hhmm("12:00:00"), Some(720));
        assert_eq!(parse_hhmm("18:30"), Some(1110));
        assert_eq!(parse_hhmm("23:59"), Some(1439));
        assert_eq!(parse_hhmm("24:00"), None);
        assert_eq!(parse_hhmm("12"), None);
        assert_eq!(parse_hhmm("invalid"), None);
    }

    #[test]
    fn half_open_same_day_includes_start_excludes_end() {
        let start = minutes(12, 0);
        let end = minutes(13, 0);
        assert!(is_in_half_open_range(minutes(12, 0), start, end));
        assert!(is_in_half_open_range(minutes(12, 59), start, end));
        assert!(!is_in_half_open_range(minutes(13, 0), start, end));
        assert!(!is_in_half_open_range(minutes(11, 59), start, end));
    }

    #[test]
    fn half_open_overnight_includes_midnight_excludes_end() {
        let start = minutes(0, 0);
        let end = minutes(9, 0);
        assert!(is_in_half_open_range(minutes(0, 0), start, end));
        assert!(is_in_half_open_range(minutes(8, 59), start, end));
        assert!(!is_in_half_open_range(minutes(9, 0), start, end));
        assert!(!is_in_half_open_range(minutes(23, 59), start, end));
    }

    #[test]
    fn wrapping_window_uses_exclusive_end() {
        let start = minutes(22, 0);
        let end = minutes(6, 0);
        assert!(is_in_half_open_range(minutes(22, 0), start, end));
        assert!(is_in_half_open_range(minutes(23, 59), start, end));
        assert!(is_in_half_open_range(minutes(0, 0), start, end));
        assert!(is_in_half_open_range(minutes(5, 59), start, end));
        assert!(!is_in_half_open_range(minutes(6, 0), start, end));
        assert!(!is_in_half_open_range(minutes(21, 59), start, end));
    }

    #[test]
    fn empty_window_when_start_equals_end_is_never_inside() {
        assert!(!is_in_half_open_range(minutes(12, 0), minutes(12, 0), minutes(12, 0)));
    }

    #[test]
    fn default_windows_cover_lunch_dinner_and_night() {
        let windows = default_windows();
        assert_eq!(windows.len(), 3);
        assert!(is_in_quiet_hours(minutes(12, 0), &windows));
        assert!(is_in_quiet_hours(minutes(12, 30), &windows));
        assert!(!is_in_quiet_hours(minutes(13, 0), &windows));
        assert!(is_in_quiet_hours(minutes(18, 0), &windows));
        assert!(!is_in_quiet_hours(minutes(19, 0), &windows));
        assert!(is_in_quiet_hours(minutes(0, 0), &windows));
        assert!(is_in_quiet_hours(minutes(8, 59), &windows));
        assert!(!is_in_quiet_hours(minutes(9, 0), &windows));
        assert!(!is_in_quiet_hours(minutes(10, 0), &windows));
        assert!(!is_in_quiet_hours(minutes(23, 59), &windows));
    }

    #[test]
    fn disabled_config_is_never_active() {
        let config = QuietHoursConfig {
            enabled: false,
            windows: default_windows(),
        };
        assert!(!is_quiet_hours_active(&config, minutes(12, 0)));
    }

    #[test]
    fn first_poll_inside_forces_stop_outside_does_not_auto_start() {
        assert_eq!(
            tick_action(None, true),
            QuietHoursTickAction::ForceStop
        );
        assert_eq!(tick_action(None, false), QuietHoursTickAction::None);
    }

    #[test]
    fn entering_forces_stop_leaving_forces_start_work() {
        assert_eq!(
            tick_action(Some(false), true),
            QuietHoursTickAction::ForceStop
        );
        assert_eq!(
            tick_action(Some(true), false),
            QuietHoursTickAction::ForceStartWork
        );
        assert_eq!(
            tick_action(Some(true), true),
            QuietHoursTickAction::ForceStop
        );
        assert_eq!(tick_action(Some(false), false), QuietHoursTickAction::None);
    }

    #[test]
    fn failed_action_keeps_previous_inside_flag() {
        assert_eq!(next_inside_flag(Some(true), false, false), Some(true));
        assert_eq!(next_inside_flag(Some(true), false, true), Some(false));
        assert_eq!(next_inside_flag(None, true, false), None);
        assert_eq!(next_inside_flag(None, true, true), Some(true));
        assert_eq!(next_inside_flag(Some(false), false, true), Some(false));
    }

    #[test]
    fn force_reset_when_finished_or_not_idle_work() {
        assert!(!should_force_reset(true, true, 1800));
        assert!(should_force_reset(true, true, 0));
        assert!(should_force_reset(false, true, 100));
        assert!(should_force_reset(true, false, 300));
    }

    #[test]
    fn normalize_config_rejects_invalid_time() {
        let config = QuietHoursConfig {
            enabled: true,
            windows: vec![QuietHoursWindow {
                start: "nope".to_string(),
                end: "13:00".to_string(),
            }],
        };
        assert!(normalize_config(&config).is_err());
    }

    #[test]
    fn normalize_config_canonicalizes_hhmm() {
        let config = QuietHoursConfig {
            enabled: true,
            windows: vec![QuietHoursWindow {
                start: "12:00:00".to_string(),
                end: "13:00".to_string(),
            }],
        };
        let normalized = normalize_config(&config).unwrap();
        assert_eq!(normalized.windows[0].start, "12:00");
        assert_eq!(normalized.windows[0].end, "13:00");
    }

    #[test]
    fn empty_json_object_uses_enabled_default_windows() {
        let config: QuietHoursConfig = serde_json::from_str("{}").unwrap();
        assert!(config.enabled);
        assert_eq!(config.windows.len(), 3);
        assert_eq!(config.windows[0].start, "12:00");
    }
}
