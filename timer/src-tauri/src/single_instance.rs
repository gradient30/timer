//! 单实例模块 - M6: 单实例机制完善
//! 使用 Windows Mutex 实现进程级别的单实例控制

use std::sync::Mutex;

/// 单实例守卫
pub struct SingleInstance {
    _handle: Mutex<()>,
    is_first: bool,
}

impl SingleInstance {
    /// 尝试创建单实例
    ///
    /// 如果是第一个实例，返回 Some(SingleInstance)
    /// 如果已有实例在运行，返回 None
    pub fn new() -> Option<Self> {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
            use windows::Win32::System::Threading::{CreateMutexW, OpenMutexW, MUTEX_ALL_ACCESS};

            unsafe {
                // 使用 windows::core::w! 宏创建宽字符串
                let mutex_name = windows::core::w!("TimerApp_SingleInstance_Mutex_v1");

                // 先尝试打开已存在的 Mutex
                let existing = OpenMutexW(MUTEX_ALL_ACCESS, false, mutex_name);

                if existing.is_ok() {
                    // Mutex 已存在，说明有另一个实例在运行
                    return None;
                }

                // 创建新的 Mutex
                let handle = CreateMutexW(None, true, mutex_name);

                if handle.is_ok() {
                    // 检查是否是已存在的 Mutex (可能在我们检查和创建之间另一个实例创建了它)
                    let error = GetLastError();
                    if error == ERROR_ALREADY_EXISTS {
                        return None;
                    }
                    Some(Self {
                        _handle: Mutex::new(()),
                        is_first: true,
                    })
                } else {
                    // 创建失败，检查错误码
                    let error = GetLastError();
                    if error == ERROR_ALREADY_EXISTS {
                        None
                    } else {
                        // 其他错误，但仍然允许启动
                        Some(Self {
                            _handle: Mutex::new(()),
                            is_first: true,
                        })
                    }
                }
            }
        }

        #[cfg(not(windows))]
        {
            // 非 Windows 平台直接允许
            Some(Self {
                _handle: Mutex::new(()),
                is_first: true,
            })
        }
    }

    /// 是否是第一个实例
    pub fn is_first(&self) -> bool {
        self.is_first
    }
}

/// 激活已存在的窗口
#[cfg(windows)]
pub fn activate_existing_window() {
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    unsafe {
        // 查找窗口（使用窗口标题）
        let window_title = windows::core::w!("TimerApp");
        let hwnd = FindWindowW(None, window_title);

        if let Ok(hwnd) = hwnd {
            if !hwnd.0.is_null() {
                // 恢复窗口（如果最小化）
                let _ = ShowWindow(hwnd, SW_RESTORE);
                // 设置前台窗口
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }
}

#[cfg(not(windows))]
pub fn activate_existing_window() {
    // 非 Windows 平台空实现
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_instance() {
        // 若已有实例运行，则可能创建失败
        let instance1 = SingleInstance::new();
        if let Some(instance) = instance1 {
            assert!(instance.is_first());
        }

        // 第二个实例应该失败（在同一进程中可能仍然成功，取决于实现）
        // 实际测试需要在不同进程中进行
    }
}
