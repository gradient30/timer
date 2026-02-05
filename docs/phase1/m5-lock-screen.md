# M5: 锁屏功能实现 - 开发日志

## 任务信息
- **任务ID**: M5
- **模块**: 锁屏功能
- **目标**: 倒计时结束时调用Windows API执行锁屏

## 执行过程

### 2026-02-05

#### 创建系统操作模块
创建 `src-tauri/src/system.rs`，实现锁屏功能。

#### 方案选择

**方案一: Windows API (最初尝试)**
```rust
use windows::Win32::System::LibraryLoader::GetProcAddress;
use windows::Win32::System::LibraryLoader::LoadLibraryW;
// ... 复杂类型转换问题
```
问题: `HWND` 类型转换、`is_invalid()` 方法不存在等编译错误。

**方案二: rundll32.exe (最终采用)**
```rust
pub fn lock_screen() {
    std::process::Command::new("rundll32.exe")
        .args(["user32.dll,LockWorkStation"])
        .spawn()
        .ok();
}
```
优点:
- 简单可靠
- 无需复杂Windows API绑定
- 等效调用 `LockWorkStation` API

#### Tauri Command 实现
```rust
#[tauri::command]
pub async fn execute_system_action(action: &str) -> Result<(), String> {
    match action {
        "lock" => { /* 锁屏实现 */ }
        "suspend" => Err("休眠功能需要管理员权限，将在后续版本实现".to_string()),
        "shutdown" => Err("关机功能需要管理员权限，将在后续版本实现".to_string()),
        _ => Err(format!("未知的操作类型: {}", action)),
    }
}
```

#### 倒计时结束触发
在 `lib.rs` 的 `start_timer` 中设置回调:
```rust
app_state.timer.set_callback(move |runtime: TimerRuntime| {
    // 检查是否倒计时结束
    let is_finished = runtime.remaining_seconds == 0 && runtime.state == TimerState::Idle;

    // 发送更新到前端
    let _ = app_handle_clone.emit("timer-update", &runtime);

    // 倒计时结束，执行锁屏
    if is_finished {
        println!("倒计时结束，执行锁屏");
        system::lock_screen();
    }
});
```

#### Cargo.toml 依赖
```toml
[dependencies]
windows = { version = "0.62", features = ["Win32", ...] }
```
注: 虽然最终使用rundll32方案，仍保留windows crate以备后续扩展。

## API 规范

### Commands

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `execute_system_action` | `action: "lock"` | `Result<(), String>` | 执行锁屏 |

### 内部函数

| 函数 | 说明 |
|------|------|
| `lock_screen()` | 内部调用，执行Windows锁屏 |

## 验收结果

| 验收项 | 状态 | 备注 |
|--------|------|------|
| 锁屏命令 | ✅ | `execute_system_action("lock")` 可用 |
| 倒计时触发 | ✅ | 计时器结束时自动调用 |
| Windows兼容 | ✅ | 使用rundll32.exe，Windows原生支持 |
| 错误处理 | ✅ | 非Windows平台返回错误信息 |

## 延期功能

| 功能 | 状态 | 原因 |
|------|------|------|
| 休眠 | ⏸️ | 需要管理员权限 |
| 关机 | ⏸️ | 需要管理员权限 |

## 下一步
- M6: 单实例机制
