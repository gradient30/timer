//! 开机自启
//!
//! 采用当前用户 HKCU Run 注册表（路径加引号），无需管理员权限。
//! 应用启动时若配置已开启，会按当前 exe 路径自愈写入，并清理旧版计划任务。

use std::env;
use std::path::Path;
use std::sync::Arc;

use crate::activation;
use crate::config::ConfigManager;

const RUN_SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "TimerApp";
const LEGACY_TASK_NAME: &str = "TimerApp";

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
        Ok(is_run_value_present(RUN_SUBKEY, APP_NAME)?)
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
            let exe_path = env::current_exe().map_err(|e| format!("获取程序路径失败: {e}"))?;
            let command = autostart_command(&exe_path);
            apply_run_value(RUN_SUBKEY, APP_NAME, true, &command)?;
        } else {
            apply_run_value(RUN_SUBKEY, APP_NAME, false, "")?;
        }
        let _ = delete_legacy_logon_task();
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("开机自启仅支持 Windows".to_string())
    }
}

/// 生成可供 Windows Run 项使用的命令行（必须加引号，并去掉 `\\?\` 前缀）
fn autostart_command(exe: &Path) -> String {
    let mut path = exe.to_string_lossy().replace('"', "");
    const VERBATIM: &str = r"\\?\";
    if let Some(stripped) = path.strip_prefix(VERBATIM) {
        path = stripped.to_string();
    }
    format!("\"{path}\"")
}

fn win32_error_message(action: &str, code: u32) -> String {
    format!("{action}失败，错误码 {code}")
}

#[cfg(windows)]
fn apply_run_value(subkey: &str, value_name: &str, enabled: bool, command: &str) -> Result<(), String> {
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_READ, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
    };

    unsafe {
        let mut key = HKEY::default();
        let status = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            &HSTRING::from(subkey),
            None,
            windows::core::PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_READ | KEY_WRITE,
            None,
            &mut key,
            None,
        );
        if status != ERROR_SUCCESS {
            return Err(win32_error_message("打开开机启动注册表", status.0));
        }

        let result = if enabled {
            if command.trim_matches('"').is_empty() {
                let _ = RegCloseKey(key);
                return Err("启用开机自启时程序路径不能为空".to_string());
            }
            let bytes: Vec<u8> = command
                .encode_utf16()
                .chain(std::iter::once(0))
                .flat_map(u16::to_le_bytes)
                .collect();
            let set_status = RegSetValueExW(
                key,
                &HSTRING::from(value_name),
                None,
                REG_SZ,
                Some(&bytes),
            );
            if set_status == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(win32_error_message("写入开机启动注册表", set_status.0))
            }
        } else {
            let delete_status = RegDeleteValueW(key, &HSTRING::from(value_name));
            if delete_status == ERROR_SUCCESS || delete_status == ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                Err(win32_error_message("删除开机启动注册表", delete_status.0))
            }
        };

        let _ = RegCloseKey(key);
        result
    }
}

#[cfg(windows)]
fn is_run_value_present(subkey: &str, value_name: &str) -> Result<bool, String> {
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    };

    unsafe {
        let mut key = HKEY::default();
        let open_status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &HSTRING::from(subkey),
            None,
            KEY_READ,
            &mut key,
        );
        if open_status == ERROR_FILE_NOT_FOUND {
            return Ok(false);
        }
        if open_status != ERROR_SUCCESS {
            return Err(win32_error_message("读取开机启动注册表", open_status.0));
        }

        let mut size = 0u32;
        let query_status = RegQueryValueExW(
            key,
            &HSTRING::from(value_name),
            None,
            None,
            None,
            Some(&mut size),
        );
        let _ = RegCloseKey(key);

        if query_status == ERROR_SUCCESS {
            Ok(true)
        } else if query_status == ERROR_FILE_NOT_FOUND {
            Ok(false)
        } else {
            Err(win32_error_message("查询开机启动注册表", query_status.0))
        }
    }
}

#[cfg(all(windows, test))]
fn query_run_value(subkey: &str, value_name: &str) -> Result<Option<String>, String> {
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
        REG_VALUE_TYPE,
    };

    unsafe {
        let mut key = HKEY::default();
        let open_status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &HSTRING::from(subkey),
            None,
            KEY_READ,
            &mut key,
        );
        if open_status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if open_status != ERROR_SUCCESS {
            return Err(win32_error_message("读取开机启动注册表", open_status.0));
        }

        let mut ty = REG_VALUE_TYPE::default();
        let mut size = 0u32;
        let probe = RegQueryValueExW(
            key,
            &HSTRING::from(value_name),
            None,
            Some(&mut ty),
            None,
            Some(&mut size),
        );
        if probe == ERROR_FILE_NOT_FOUND {
            let _ = RegCloseKey(key);
            return Ok(None);
        }
        if probe != ERROR_SUCCESS {
            let _ = RegCloseKey(key);
            return Err(win32_error_message("查询开机启动注册表", probe.0));
        }

        let mut buffer = vec![0u8; size as usize];
        let read_status = RegQueryValueExW(
            key,
            &HSTRING::from(value_name),
            None,
            Some(&mut ty),
            Some(buffer.as_mut_ptr()),
            Some(&mut size),
        );
        let _ = RegCloseKey(key);
        if read_status != ERROR_SUCCESS {
            return Err(win32_error_message("读取开机启动注册表值", read_status.0));
        }

        let u16_data = buffer
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|unit| *unit != 0)
            .collect::<Vec<u16>>();
        Ok(Some(String::from_utf16_lossy(&u16_data)))
    }
}

#[cfg(windows)]
fn delete_legacy_logon_task() -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let output = Command::new("schtasks.exe")
        .args(["/Delete", "/TN", LEGACY_TASK_NAME, "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("清理旧版登录任务失败: {e}"))?;

    if output.status.success() {
        return Ok(());
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    if text.contains("无法找到")
        || text.contains("cannot find")
        || text.contains("Cannot find")
        || text.contains("ERROR: The system cannot find")
        || text.contains("Unable to find")
    {
        Ok(())
    } else {
        Err(format!("清理旧版登录任务失败: {}", text.trim()))
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

    #[test]
    fn autostart_command_strips_windows_verbatim_prefix() {
        let path = PathBuf::from(r"\\?\C:\Program Files\TimerApp\timer.exe");
        assert_eq!(
            autostart_command(&path),
            r#""C:\Program Files\TimerApp\timer.exe""#
        );
    }

    #[cfg(windows)]
    #[test]
    fn run_key_roundtrip_on_temp_subkey() {
        const TEST_SUBKEY: &str = r"Software\TimerApp\UnitTestRun";
        const TEST_NAME: &str = "TimerAppStartupTest";
        let command = r#""C:\Program Files\TimerApp\timer.exe""#;

        apply_run_value(TEST_SUBKEY, TEST_NAME, true, command).unwrap();
        assert!(is_run_value_present(TEST_SUBKEY, TEST_NAME).unwrap());
        assert_eq!(
            query_run_value(TEST_SUBKEY, TEST_NAME).unwrap().as_deref(),
            Some(command)
        );

        apply_run_value(TEST_SUBKEY, TEST_NAME, false, "").unwrap();
        assert!(!is_run_value_present(TEST_SUBKEY, TEST_NAME).unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn enabling_with_empty_path_is_rejected() {
        const TEST_SUBKEY: &str = r"Software\TimerApp\UnitTestRun";
        const TEST_NAME: &str = "TimerAppStartupEmptyPath";
        let err = apply_run_value(TEST_SUBKEY, TEST_NAME, true, r#""""#).unwrap_err();
        assert!(err.contains("路径不能为空"));
        let _ = apply_run_value(TEST_SUBKEY, TEST_NAME, false, "");
    }
}
