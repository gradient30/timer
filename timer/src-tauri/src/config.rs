//! 配置管理模块 - E5: 配置持久化
//! 配置存储于 %APPDATA%/TimerApp/config.json

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// 应用配置版本
const CONFIG_VERSION: &str = "1.1";

/// 定时器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub interval_minutes: u64,
    pub advance_notice_seconds: u64,
    pub max_delay_times: u32,
    pub delay_options: Vec<u64>,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            interval_minutes: 30,
            advance_notice_seconds: 30,
            max_delay_times: 3,
            delay_options: vec![5, 10, 30],
        }
    }
}

/// 生效规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub time_limit_enabled: bool,
    pub weekday_limit_enabled: bool,
    pub start_time: String,  // HH:MM 格式
    pub end_time: String,    // HH:MM 格式
    pub weekdays: Vec<u32>,  // 1=周一, 7=周日
    pub logic: String,       // "AND"
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            time_limit_enabled: false,
            weekday_limit_enabled: false,
            start_time: "09:00".to_string(),
            end_time: "18:00".to_string(),
            weekdays: vec![1, 2, 3, 4, 5],
            logic: "AND".to_string(),
        }
    }
}

/// 执行动作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfig {
    pub action_type: String, // "lock", "suspend", "shutdown"
    pub show_notice: bool,
}

impl Default for ActionConfig {
    fn default() -> Self {
        Self {
            action_type: "lock".to_string(),
            show_notice: true,
        }
    }
}

/// 启动配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    pub auto_start: bool,
    pub start_minimized: bool,
    pub start_timer_automatically: bool,
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            start_minimized: false,
            start_timer_automatically: false,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,           // "trace", "debug", "info", "warn", "error"
    pub max_days: u32,
    pub max_file_size_mb: u64,
    pub max_total_size_mb: u64,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            max_days: 7,
            max_file_size_mb: 10,
            max_total_size_mb: 100,
        }
    }
}

/// 运行时状态持久化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub timer_status: String,    // "Idle", "Running", "Paused"
    pub remaining_seconds: u64,
    pub total_seconds: u64,
    pub last_update: String,     // ISO 8601 格式
    pub delay_count: u32,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            timer_status: "Idle".to_string(),
            remaining_seconds: 1800,  // 默认30分钟
            total_seconds: 1800,
            last_update: chrono::Local::now().to_rfc3339(),
            delay_count: 0,
        }
    }
}

/// 应用完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub timer: TimerConfig,
    pub schedule: ScheduleConfig,
    pub action: ActionConfig,
    pub startup: StartupConfig,
    pub log: LogConfig,
    pub runtime_state: RuntimeState,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION.to_string(),
            timer: TimerConfig::default(),
            schedule: ScheduleConfig::default(),
            action: ActionConfig::default(),
            startup: StartupConfig::default(),
            log: LogConfig::default(),
            runtime_state: RuntimeState::default(),
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config: Mutex<AppConfig>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// 创建配置管理器
    pub fn new() -> Result<Self, String> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_or_create(&config_path)?;

        Ok(Self {
            config: Mutex::new(config),
            config_path,
        })
    }

    /// 获取配置目录
    pub fn get_config_dir() -> Result<PathBuf, String> {
        dirs::config_dir()
            .map(|d| d.join("TimerApp"))
            .ok_or_else(|| "无法获取配置目录".to_string())
    }

    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf, String> {
        Ok(Self::get_config_dir()?.join("config.json"))
    }

    /// 加载或创建配置
    fn load_or_create(config_path: &PathBuf) -> Result<AppConfig, String> {
        if config_path.exists() {
            let content = fs::read_to_string(config_path)
                .map_err(|e| format!("读取配置文件失败: {}", e))?;
            let config: AppConfig = serde_json::from_str(&content)
                .map_err(|e| format!("解析配置文件失败: {}", e))?;
            Ok(config)
        } else {
            // 创建默认配置
            let config = AppConfig::default();
            // 确保目录存在
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建配置目录失败: {}", e))?;
            }
            // 保存默认配置
            let content = serde_json::to_string_pretty(&config)
                .map_err(|e| format!("序列化配置失败: {}", e))?;
            fs::write(config_path, content)
                .map_err(|e| format!("写入配置文件失败: {}", e))?;
            Ok(config)
        }
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), String> {
        let config = self.config.lock()
            .map_err(|e| format!("锁定配置失败: {}", e))?;
        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(&self.config_path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        Ok(())
    }

    /// 获取配置副本
    pub fn get(&self) -> Result<AppConfig, String> {
        let config = self.config.lock()
            .map_err(|e| format!("锁定配置失败: {}", e))?;
        Ok(config.clone())
    }

    /// 更新配置
    pub fn update<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.config.lock()
            .map_err(|e| format!("锁定配置失败: {}", e))?;
        f(&mut config);
        drop(config);
        self.save()
    }

    /// 获取配置路径
    pub fn path(&self) -> &PathBuf {
        &self.config_path
    }
}

// ===== Tauri Commands =====

/// 获取配置
#[tauri::command]
pub fn get_config(state: tauri::State<std::sync::Arc<ConfigManager>>) -> Result<AppConfig, String> {
    state.get()
}

/// 更新定时器配置
#[tauri::command]
pub fn update_timer_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: TimerConfig,
) -> Result<(), String> {
    state.update(|c| c.timer = config)
}

/// 更新生效规则配置
#[tauri::command]
pub fn update_schedule_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: ScheduleConfig,
) -> Result<(), String> {
    state.update(|c| c.schedule = config)
}

/// 更新执行动作配置
#[tauri::command]
pub fn update_action_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: ActionConfig,
) -> Result<(), String> {
    state.update(|c| c.action = config)
}

/// 更新启动配置
#[tauri::command]
pub fn update_startup_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: StartupConfig,
) -> Result<(), String> {
    state.update(|c| c.startup = config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.version, "1.1");
        assert_eq!(config.timer.interval_minutes, 30);
        assert_eq!(config.timer.max_delay_times, 3);
        assert_eq!(config.action.action_type, "lock");
    }

    #[test]
    fn test_serialize_config() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("interval_minutes"));
        assert!(json.contains("schedule"));
    }
}
