//! 生效规则模块 - E1/E2: 时间段与周循环检测
//! 判定当前时间是否在生效规则范围内

use chrono::{Local, DateTime, Datelike, Timelike, Weekday};

/// 生效规则检查器
pub struct ScheduleChecker;

impl ScheduleChecker {
    /// 检查当前时间是否符合生效规则
    ///
    /// # 参数
    /// - `time_enabled`: 时间段限制是否启用
    /// - `start_time`: 开始时间 (HH:MM)
    /// - `end_time`: 结束时间 (HH:MM)
    /// - `weekday_enabled`: 星期限制是否启用
    /// - `weekdays`: 允许的星期列表 [1=周一, ..., 7=周日]
    /// - `logic`: 逻辑关系 ("AND" 表示同时满足)
    ///
    /// # 返回
    /// - `true`: 当前时间在生效规则内
    /// - `false`: 当前时间不在生效规则内
    pub fn is_effective(
        time_enabled: bool,
        start_time: &str,
        end_time: &str,
        weekday_enabled: bool,
        weekdays: &[u32],
        logic: &str,
    ) -> bool {
        let now = Local::now();

        // 检查时间段
        let time_match = if time_enabled {
            Self::check_time_range(&now, start_time, end_time)
        } else {
            true // 未启用则视为通过
        };

        // 检查星期
        let weekday_match = if weekday_enabled {
            Self::check_weekday(&now, weekdays)
        } else {
            true // 未启用则视为通过
        };

        // 应用逻辑关系
        match logic {
            "AND" => time_match && weekday_match,
            "OR" => time_match || weekday_match,
            _ => time_match && weekday_match, // 默认AND
        }
    }

    /// 检查当前时间是否在指定时间段内
    fn check_time_range(now: &DateTime<Local>, start: &str, end: &str) -> bool {
        let current_minutes = now.hour() * 60 + now.minute();

        match (Self::parse_time(start), Self::parse_time(end)) {
            (Some(start_min), Some(end_min)) => {
                if start_min <= end_min {
                    // 正常区间，如 09:00-18:00
                    current_minutes >= start_min && current_minutes <= end_min
                } else {
                    // 跨午夜区间，如 22:00-06:00
                    current_minutes >= start_min || current_minutes <= end_min
                }
            }
            _ => true, // 解析失败时默认通过
        }
    }

    /// 检查当前星期是否在允许列表中
    fn check_weekday(now: &DateTime<Local>, allowed: &[u32]) -> bool {
        // chrono::Weekday: Mon=0, Sun=6
        // 转换为: 周一=1, 周日=7
        let weekday_num = match now.weekday() {
            Weekday::Mon => 1,
            Weekday::Tue => 2,
            Weekday::Wed => 3,
            Weekday::Thu => 4,
            Weekday::Fri => 5,
            Weekday::Sat => 6,
            Weekday::Sun => 7,
        };

        allowed.contains(&weekday_num)
    }

    /// 解析时间字符串 (HH:MM) 为分钟数
    fn parse_time(time_str: &str) -> Option<u32> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let hour: u32 = parts[0].parse().ok()?;
        let minute: u32 = parts[1].parse().ok()?;

        if hour > 23 || minute > 59 {
            return None;
        }

        Some(hour * 60 + minute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        assert_eq!(ScheduleChecker::parse_time("09:00"), Some(540));
        assert_eq!(ScheduleChecker::parse_time("18:30"), Some(1110));
        assert_eq!(ScheduleChecker::parse_time("23:59"), Some(1439));
        assert_eq!(ScheduleChecker::parse_time("invalid"), None);
    }

    #[test]
    fn test_weekday_check() {
        // 无法直接测试当前时间，但验证逻辑
        let allowed = vec![1, 2, 3, 4, 5]; // 工作日
        // 假设今天是周一，则应在允许列表
        // 实际测试依赖于当前系统时间
    }
}
