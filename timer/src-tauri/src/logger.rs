//! 日志模块 - E7: 日志记录
//! 操作日志、错误日志记录

use std::fs;
use std::path::PathBuf;
use tracing::{info, error, debug, warn};

/// 初始化日志系统
///
/// # 参数
/// - `level`: 日志级别 (trace/debug/info/warn/error)
pub fn init_logger(level: &str) -> Result<(), String> {
    let log_dir = get_log_dir()?;
    fs::create_dir_all(&log_dir).map_err(|e| format!("创建日志目录失败: {}", e))?;

    // 清理旧日志
    cleanup_old_logs(&log_dir)?;

    let log_level = match level {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    // 初始化 tracing-subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(false)
        .compact()
        .init();

    info!("日志系统初始化完成，级别: {}", level);
    Ok(())
}

/// 获取日志目录
fn get_log_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|d| d.join("TimerApp").join("logs"))
        .ok_or_else(|| "无法获取日志目录".to_string())
}

/// 清理旧日志文件
///
/// 策略：
/// 1. 删除超过 max_days 天的日志
/// 2. 当日志总量超过 max_total_size_mb 时，删除最旧的日志
fn cleanup_old_logs(log_dir: &PathBuf) -> Result<(), String> {
    if !log_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(log_dir)
        .map_err(|e| format!("读取日志目录失败: {}", e))?;

    let mut log_files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
    let mut total_size: u64 = 0;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().map(|e| e == "log").unwrap_or(false) {
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            let modified = metadata.modified().map_err(|e| e.to_string())?;
            let size = metadata.len();

            log_files.push((path, modified));
            total_size += size;
        }
    }

    // 按修改时间排序（最旧的在前面）
    log_files.sort_by(|a, b| a.1.cmp(&b.1));

    // 删除超过7天的日志
    let now = std::time::SystemTime::now();
    let seven_days = std::time::Duration::from_secs(7 * 24 * 60 * 60);

    for (path, modified) in &log_files {
        if now.duration_since(*modified).unwrap_or_default() > seven_days {
            let _ = fs::remove_file(path);
        }
    }

    // 如果总量超过100MB，删除最旧的直到低于阈值
    const MAX_TOTAL_SIZE: u64 = 100 * 1024 * 1024; // 100MB
    if total_size > MAX_TOTAL_SIZE {
        for (path, _) in &log_files {
            if total_size <= MAX_TOTAL_SIZE {
                break;
            }
            if let Ok(metadata) = fs::metadata(path) {
                total_size -= metadata.len();
                let _ = fs::remove_file(path);
            }
        }
    }

    Ok(())
}

/// 记录操作日志
pub fn log_operation(action: &str, details: &str) {
    info!("[操作] {}: {}", action, details);
}

/// 记录错误日志
pub fn log_error(source: &str, error: &str) {
    error!("[错误] {}: {}", source, error);
}

/// 记录调试日志
pub fn log_debug(category: &str, message: &str) {
    debug!("[调试] {}: {}", category, message);
}

/// 记录警告日志
pub fn log_warn(category: &str, message: &str) {
    warn!("[警告] {}: {}", category, message);
}

/// Tauri命令：获取日志目录
#[tauri::command]
pub fn get_log_directory() -> Result<String, String> {
    get_log_dir()
        .map(|p| p.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_dir() {
        let _ = get_log_dir();
    }
}
