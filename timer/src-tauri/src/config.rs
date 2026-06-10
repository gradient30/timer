//! 配置管理模块 - E5: 配置持久化
//! 配置存储于 %APPDATA%/TimerApp/config.json

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use crate::activation;

/// 应用配置版本
const CONFIG_VERSION: &str = "1.1";

/// 定时器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TimerConfig {
    pub interval_minutes: u64,
    pub advance_notice_seconds: u64,
    pub max_delay_times: u32,
    pub delay_options: Vec<u64>,
    pub loop_enabled: bool,
    pub loop_interval_minutes: u64,
    #[serde(default = "default_enforce_relock_during_rest")]
    pub enforce_relock_during_rest: bool,
}

pub fn default_enforce_relock_during_rest() -> bool {
    true
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            interval_minutes: 30,
            advance_notice_seconds: 30,
            max_delay_times: 3,
            delay_options: vec![5, 10, 30],
            loop_enabled: true,
            loop_interval_minutes: 5,
            enforce_relock_during_rest: true,
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
    pub action_type: String, // "lock", "shutdown"
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

fn normalize_action_type(action_type: &str) -> String {
    match action_type {
        "shutdown" => "shutdown".to_string(),
        _ => "lock".to_string(),
    }
}

/// 启动配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StartupConfig {
    pub auto_start: bool,
    pub start_minimized: bool,
    pub start_timer_automatically: bool,
}

fn default_ui_theme() -> String {
    "dark".to_string()
}

/// 界面配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    #[serde(default = "default_ui_theme")]
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_ui_theme(),
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

/// 激活配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivationConfig {
    pub activated: bool,
    pub activation_code_hash: Option<String>,
    pub activated_at: Option<String>,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    pub password_hash: Option<String>,
    pub security_question: Option<String>,
    pub security_answer_hash: Option<String>,
    pub failed_attempts: u32,
    pub lock_until: Option<String>,
}

/// 运行时状态持久化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub timer_status: String,    // "Idle", "Running", "Paused"
    pub remaining_seconds: u64,
    pub total_seconds: u64,
    pub last_update: String,     // ISO 8601 格式
    pub delay_count: u32,
    #[serde(default = "default_delay_quota_date")]
    pub delay_quota_date: String, // YYYY-MM-DD，本地日期
    #[serde(default = "default_cycle_phase")]
    pub cycle_phase: String,     // "Work", "Rest"
}

pub fn default_delay_quota_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// 关机/离线期间扣减运行中计时器的剩余秒数
pub fn offline_adjusted_remaining(saved_state: &RuntimeState) -> u64 {
    if saved_state.timer_status != "Running" {
        return saved_state.remaining_seconds;
    }

    let last_update = chrono::DateTime::parse_from_rfc3339(&saved_state.last_update)
        .map(|dt| dt.with_timezone(&chrono::Local))
        .unwrap_or_else(|_| chrono::Local::now());

    let offline_secs = (chrono::Local::now() - last_update)
        .num_seconds()
        .max(0) as u64;

    saved_state.remaining_seconds.saturating_sub(offline_secs)
}

fn default_cycle_phase() -> String {
    "Work".to_string()
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            timer_status: "Idle".to_string(),
            remaining_seconds: 1800,  // 默认30分钟
            total_seconds: 1800,
            last_update: chrono::Local::now().to_rfc3339(),
            delay_count: 0,
            delay_quota_date: default_delay_quota_date(),
            cycle_phase: "Work".to_string(),
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
    #[serde(default)]
    pub ui: UiConfig,
    pub log: LogConfig,
    #[serde(default)]
    pub activation: ActivationConfig,
    #[serde(default)]
    pub security: SecurityConfig,
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
            ui: UiConfig::default(),
            log: LogConfig::default(),
            activation: ActivationConfig::default(),
            security: SecurityConfig::default(),
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
            let mut config: AppConfig = serde_json::from_str(&content)
                .map_err(|e| format!("解析配置文件失败: {}", e))?;
            config.action.action_type = normalize_action_type(&config.action.action_type);
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
    let _ = reset_daily_delay_quota_if_needed(state.inner());
    state.get()
}

/// 更新定时器配置
#[tauri::command]
pub fn update_timer_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: TimerConfig,
) -> Result<(), String> {
    activation::ensure_activated(state.inner())?;
    let mut normalized = config;
    if normalized.loop_interval_minutes == 0 {
        normalized.loop_interval_minutes = 5;
    }
    state.update(|c| c.timer = normalized)
}

/// 更新生效规则配置
#[tauri::command]
pub fn update_schedule_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: ScheduleConfig,
) -> Result<(), String> {
    activation::ensure_activated(state.inner())?;
    state.update(|c| c.schedule = config)
}

/// 更新执行动作配置
#[tauri::command]
pub fn update_action_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: ActionConfig,
) -> Result<(), String> {
    activation::ensure_activated(state.inner())?;
    let normalized = ActionConfig {
        action_type: normalize_action_type(&config.action_type),
        show_notice: config.show_notice,
    };
    state.update(|c| c.action = normalized)
}

/// 更新启动配置
#[tauri::command]
pub fn update_startup_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: StartupConfig,
) -> Result<(), String> {
    activation::ensure_activated(state.inner())?;
    state.update(|c| c.startup = config)
}

/// 更新界面配置
#[tauri::command]
pub fn update_ui_config(
    state: tauri::State<std::sync::Arc<ConfigManager>>,
    config: UiConfig,
) -> Result<(), String> {
    activation::ensure_activated(state.inner())?;
    let theme = match config.theme.as_str() {
        "light" => "light".to_string(),
        "vivid" => "vivid".to_string(),
        _ => "dark".to_string(),
    };
    state.update(|c| c.ui.theme = theme)
}

/// 更新运行时状态（内部使用，用于状态持久化）
pub fn update_runtime_state(
    state: &tauri::State<std::sync::Arc<ConfigManager>>,
    runtime_state: RuntimeState,
) -> Result<(), String> {
    let mut normalized = runtime_state;
    if normalized.delay_quota_date.trim().is_empty() {
        normalized.delay_quota_date = default_delay_quota_date();
    }
    state.update(|c| c.runtime_state = normalized)
}

pub fn reset_daily_delay_quota_if_needed(
    config_manager: &std::sync::Arc<ConfigManager>,
) -> Result<(), String> {
    let config = config_manager.get()?;
    let today = default_delay_quota_date();
    if config.runtime_state.delay_quota_date == today {
        return Ok(());
    }
    config_manager.update(|c| {
        c.runtime_state.delay_count = 0;
        c.runtime_state.delay_quota_date = today;
    })
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
        assert!(config.timer.loop_enabled);
        assert_eq!(config.timer.loop_interval_minutes, 5);
        assert!(config.timer.enforce_relock_during_rest);
        assert_eq!(config.action.action_type, "lock");
        assert!(config.security.password_hash.is_none());
        assert!(!config.activation.activated);
    }

    #[test]
    fn test_serialize_config() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("interval_minutes"));
        assert!(json.contains("schedule"));
        assert!(json.contains("security"));
        assert!(json.contains("activation"));
    }

    #[test]
    fn test_offline_adjusted_remaining_keeps_paused() {
        let saved = RuntimeState {
            timer_status: "Paused".to_string(),
            remaining_seconds: 600,
            total_seconds: 1800,
            last_update: chrono::Local::now().to_rfc3339(),
            delay_count: 0,
            delay_quota_date: default_delay_quota_date(),
            cycle_phase: "Work".to_string(),
        };

        assert_eq!(offline_adjusted_remaining(&saved), 600);
    }

    #[test]
    fn test_normalize_action_type() {
        assert_eq!(normalize_action_type("lock"), "lock");
        assert_eq!(normalize_action_type("shutdown"), "shutdown");
        assert_eq!(normalize_action_type("suspend"), "lock");
        assert_eq!(normalize_action_type("unknown"), "lock");
    }
}
