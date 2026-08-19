//! 开机自启
//!
//! 优先使用「用户登录后延迟启动」的计划任务，避免 HKCU Run 在部分开机场景被跳过。
//! 计划任务失败时回退到加引号的 Run 注册表项。

use std::env;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::Arc;

use crate::activation;
use crate::config::ConfigManager;

const REGISTRY_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "TimerApp";
const TASK_NAME: &str = "TimerApp";
const LOGON_DELAY: &str = "0000:20";

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
        Ok(logon_task_exists() || run_key_exists())
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
    sync_os_auto_start(enabled)
}

/// 按配置把操作系统启动项与当前可执行文件对齐（启动时自愈）
pub fn sync_os_auto_start(enabled: bool) -> Result<(), String> {
    set_auto_start_internal(enabled)
}

fn set_auto_start_internal(enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        if enabled {
            let exe_path = env::current_exe().map_err(|e| format!("获取程序路径失败: {}", e))?;
            let command = autostart_command(&exe_path);
            match create_logon_task(&command) {
                Ok(()) => {
                    let _ = delete_run_key();
                    Ok(())
                }
                Err(task_err) => set_run_key(&command).map_err(|reg_err| {
                    format!("设置开机自启失败: 计划任务 ({task_err}); 注册表 ({reg_err})")
                }),
            }
        } else {
            let task_result = delete_logon_task();
            let run_result = delete_run_key();
            match (task_result, run_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(task_err), Err(run_err)) => {
                    Err(format!("取消开机自启失败: {task_err}; {run_err}"))
                }
                (Err(task_err), Ok(())) => Err(task_err),
                (Ok(()), Err(run_err)) => Err(run_err),
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("开机自启仅支持 Windows".to_string())
    }
}

/// 生成可供 Windows 启动项使用的命令行（路径必须加引号，否则含空格时会启动失败）
fn autostart_command(exe: &Path) -> String {
    let path = exe.to_string_lossy().replace('"', "");
    format!("\"{path}\"")
}

#[cfg(windows)]
fn hidden_command(program: &str) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(windows)]
fn command_output_error(action: &str, output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if !stderr.trim().is_empty() {
        stderr.trim().to_string()
    } else {
        stdout.trim().to_string()
    };
    format!("{action}失败: {detail}")
}

#[cfg(windows)]
fn not_found_output(output: &Output) -> bool {
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    text.contains("无法找到")
        || text.contains("cannot find")
        || text.contains("Cannot find")
        || text.contains("ERROR: The system cannot find")
        || text.contains("Unable to find")
        || text.contains("ERROR_FILE_NOT_FOUND")
}

#[cfg(windows)]
fn logon_task_exists() -> bool {
    hidden_command("schtasks.exe")
        .args(["/Query", "/TN", TASK_NAME])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn run_key_exists() -> bool {
    hidden_command("reg.exe")
        .args(["query", REGISTRY_KEY, "/v", APP_NAME])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn create_logon_task(command: &str) -> Result<(), String> {
    let output = hidden_command("schtasks.exe")
        .args([
            "/Create",
            "/TN",
            TASK_NAME,
            "/TR",
            command,
            "/SC",
            "ONLOGON",
            "/DELAY",
            LOGON_DELAY,
            "/RL",
            "LIMITED",
            "/F",
        ])
        .output()
        .map_err(|e| format!("执行 schtasks create 失败: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(command_output_error("创建登录启动任务", &output))
    }
}

#[cfg(windows)]
fn delete_logon_task() -> Result<(), String> {
    let output = hidden_command("schtasks.exe")
        .args(["/Delete", "/TN", TASK_NAME, "/F"])
        .output()
        .map_err(|e| format!("执行 schtasks delete 失败: {e}"))?;

    if output.status.success() || not_found_output(&output) {
        Ok(())
    } else {
        Err(command_output_error("删除登录启动任务", &output))
    }
}

#[cfg(windows)]
fn set_run_key(command: &str) -> Result<(), String> {
    let output = hidden_command("reg.exe")
        .args([
            "add",
            REGISTRY_KEY,
            "/v",
            APP_NAME,
            "/t",
            "REG_SZ",
            "/d",
            command,
            "/f",
        ])
        .output()
        .map_err(|e| format!("执行reg add失败: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(command_output_error("设置开机自启注册表", &output))
    }
}

#[cfg(windows)]
fn delete_run_key() -> Result<(), String> {
    let output = hidden_command("reg.exe")
        .args(["delete", REGISTRY_KEY, "/v", APP_NAME, "/f"])
        .output()
        .map_err(|e| format!("执行reg delete失败: {e}"))?;

    if output.status.success() || not_found_output(&output) {
        Ok(())
    } else {
        Err(command_output_error("取消开机自启", &output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_auto_start() {
        let _ = is_auto_start_enabled_internal();
    }

    #[test]
    fn autostart_command_quotes_paths_with_spaces() {
        let path = PathBuf::from(r"C:\Program Files\TimerApp\timer.exe");
        assert_eq!(
            autostart_command(&path),
            r#""C:\Program Files\TimerApp\timer.exe""#
        );
    }

    #[test]
    fn autostart_command_quotes_paths_without_spaces() {
        let path = PathBuf::from(r"D:\workspace_test\timer.exe");
        assert_eq!(autostart_command(&path), r#""D:\workspace_test\timer.exe""#);
    }
}
