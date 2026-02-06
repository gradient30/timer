//! 开机自启模块 - E6: 开机自启
//! 通过注册表操作实现 Windows 开机自启
//! 使用 reg.exe 命令行工具（更可靠）

use std::env;
use std::process::Command;
use std::sync::Arc;

use crate::activation;
use crate::config::ConfigManager;

const REGISTRY_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "TimerApp";

/// 检查开机自启是否启用
#[tauri::command]
pub fn is_auto_start_enabled(
    config_manager: tauri::State<Arc<ConfigManager>>,
) -> Result<bool, String> {
    activation::ensure_activated(config_manager.inner())?;
    is_auto_start_enabled_internal()
}

fn is_auto_start_enabled_internal() -> Result<bool, String> {
    #[cfg(windows)]
    {
        let output = Command::new("reg.exe")
            .args(["query", REGISTRY_KEY, "/v", APP_NAME])
            .output()
            .map_err(|e| format!("执行reg query失败: {}", e))?;

        Ok(output.status.success())
    }

    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

/// 设置开机自启
#[tauri::command]
pub fn set_auto_start(
    enabled: bool,
    config_manager: tauri::State<Arc<ConfigManager>>,
) -> Result<(), String> {
    activation::ensure_activated(config_manager.inner())?;
    set_auto_start_internal(enabled)
}

fn set_auto_start_internal(enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        if enabled {
            // 获取当前可执行文件路径
            let exe_path = env::current_exe()
                .map_err(|e| format!("获取程序路径失败: {}", e))?;
            let exe_path_str = exe_path.to_string_lossy();

            // 添加注册表项
            let output = Command::new("reg.exe")
                .args([
                    "add",
                    REGISTRY_KEY,
                    "/v", APP_NAME,
                    "/t", "REG_SZ",
                    "/d", &exe_path_str,
                    "/f",
                ])
                .output()
                .map_err(|e| format!("执行reg add失败: {}", e))?;

            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("设置开机自启失败: {}", stderr))
            }
        } else {
            // 删除注册表项
            let output = Command::new("reg.exe")
                .args([
                    "delete",
                    REGISTRY_KEY,
                    "/v", APP_NAME,
                    "/f",
                ])
                .output()
                .map_err(|e| format!("执行reg delete失败: {}", e))?;

            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // 如果注册表项不存在（错误代码2），也算成功
                if stderr.contains("无法找到") || stderr.contains("Unable to find") || stderr.contains("ERROR_FILE_NOT_FOUND") {
                    Ok(())
                } else {
                    Err(format!("取消开机自启失败: {}", stderr))
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        Err("开机自启仅支持 Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_start() {
        // 仅测试函数不 panic
        let _ = is_auto_start_enabled_internal();
    }
}
