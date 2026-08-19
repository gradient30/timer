//! 单实例模块 - M6: 单实例机制完善
//! 使用 Windows Mutex 实现进程级别的单实例控制

/// 单实例守卫。必须在进程生命周期内持有 Windows Mutex 句柄，
/// 否则开机自启若触发两次，会当成两个独立实例。
pub struct SingleInstance {
    #[cfg(windows)]
    _handle: Option<windows::Win32::Foundation::HANDLE>,
    is_first: bool,
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            if let Some(handle) = self._handle.take() {
                unsafe {
                    let _ = windows::Win32::Foundation::CloseHandle(handle);
                }
            }
        }
    }
}

impl SingleInstance {
    /// 尝试创建单实例
    ///
    /// 如果是第一个实例，返回 Some(SingleInstance)
    /// 如果已有实例在运行，返回 None
    pub fn new() -> Option<Self> {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::{
                CloseHandle, GetLastError, SetLastError, ERROR_ALREADY_EXISTS, ERROR_SUCCESS,
            };
            use windows::Win32::System::Threading::CreateMutexW;

            unsafe {
                let mutex_name = windows::core::w!("TimerApp_SingleInstance_Mutex_v1");
                SetLastError(ERROR_SUCCESS);
                let handle = CreateMutexW(None, true, mutex_name).ok()?;
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    let _ = CloseHandle(handle);
                    return None;
                }
                Some(Self {
                    _handle: Some(handle),
                    is_first: true,
                })
            }
        }

        #[cfg(not(windows))]
        {
            Some(Self { is_first: true })
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
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_single_instance() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let instance1 = SingleInstance::new();
        if let Some(instance) = instance1 {
            assert!(instance.is_first());
        }
    }

    #[test]
    fn second_instance_in_same_process_is_rejected() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let first = SingleInstance::new();
        if first.is_none() {
            // 本机已有 TimerApp 在运行，跳过以免误伤
            return;
        }
        let second = SingleInstance::new();
        assert!(
            second.is_none(),
            "Windows mutex handle must stay open for the process lifetime"
        );
    }
}
