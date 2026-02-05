//! 系统操作模块 - E3: 执行动作扩展
//! 锁屏、休眠、关机功能实现

/// 执行系统操作（锁屏/休眠/关机）
///
/// # 参数
/// - `action`: "lock" | "suspend" | "shutdown"
///
/// # 注意
/// 休眠和关机需要管理员权限才能成功执行
#[tauri::command]
pub async fn execute_system_action(action: &str) -> Result<(), String> {
    match action {
        "lock" => {
            lock_screen();
            Ok(())
        }
        "suspend" => {
            #[cfg(windows)]
            {
                suspend_system()
                    .map_err(|e| format!("休眠失败: {}。请确保以管理员权限运行", e))
            }
            #[cfg(not(windows))]
            {
                Err("休眠功能仅支持 Windows".to_string())
            }
        }
        "shutdown" => {
            #[cfg(windows)]
            {
                shutdown_system()
                    .map_err(|e| format!("关机失败: {}。请确保以管理员权限运行", e))
            }
            #[cfg(not(windows))]
            {
                Err("关机功能仅支持 Windows".to_string())
            }
        }
        _ => Err(format!("未知的操作类型: {}", action)),
    }
}

/// 锁定工作站（锁屏）
/// 使用 rundll32 调用 LockWorkStation
pub fn lock_screen() {
    #[cfg(windows)]
    {
        std::process::Command::new("rundll32.exe")
            .args(["user32.dll,LockWorkStation"])
            .spawn()
            .ok();
    }
}

/// 使系统进入休眠状态
/// 使用 rundll32 调用 SetSuspendState
#[cfg(windows)]
fn suspend_system() -> Result<(), String> {
    // 使用 powercfg 或 rundll32 进入休眠
    // 方法1: 使用 rundll32 powrprof.dll,SetSuspendState
    // 方法2: 使用 powercfg /hibernate on 然后执行
    // 这里使用 shutdown /h 命令（最可靠）

    let output = std::process::Command::new("shutdown.exe")
        .args(["/h"])  // /h = 休眠
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("命令执行失败: {}", stderr))
    }
}

/// 关闭系统
/// 使用 shutdown.exe 命令
#[cfg(windows)]
fn shutdown_system() -> Result<(), String> {
    // /s = 关机
    // /t 0 = 立即执行
    // /f = 强制关闭应用程序
    let output = std::process::Command::new("shutdown.exe")
        .args(["/s", "/t", "0", "/f"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("命令执行失败: {}", stderr))
    }
}

/// 执行动作包装函数（内部使用）
/// 根据配置执行相应的系统操作
pub fn execute_action(action_type: &str) {
    match action_type {
        "lock" => lock_screen(),
        "suspend" => {
            #[cfg(windows)]
            {
                let _ = suspend_system();
            }
        }
        "shutdown" => {
            #[cfg(windows)]
            {
                let _ = shutdown_system();
            }
        }
        _ => {
            eprintln!("未知的操作类型: {}", action_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_system_action() {
        // 测试无效操作
        let result = tokio_test::block_on(execute_system_action("invalid"));
        assert!(result.is_err());
    }

    #[test]
    fn test_lock_screen() {
        // 仅测试函数不 panic
        lock_screen();
    }
}
