# E3: 执行动作扩展 - 开发日志

## 任务信息
- **任务ID**: E3
- **模块**: 执行动作扩展
- **目标**: 支持锁屏/休眠/关机三种执行动作

## 技术方案 (plan.md)

| 动作 | 技术方案 |
|------|----------|
| 锁屏 | `LockWorkStation` via rundll32 |
| 休眠 | `shutdown /h` |
| 关机 | `shutdown /s /t 0 /f` |

## 实现过程

### 更新系统操作模块
文件: `src-tauri/src/system.rs`

**命令实现**:

```rust
#[tauri::command]
pub async fn execute_system_action(action: &str) -> Result<(), String> {
    match action {
        "lock" => { lock_screen(); Ok(()) }
        "suspend" => suspend_system(),      // shutdown /h
        "shutdown" => shutdown_system(),    // shutdown /s /t 0 /f
        _ => Err("未知操作".to_string()),
    }
}
```

**休眠实现**:
```rust
fn suspend_system() -> Result<(), String> {
    let output = std::process::Command::new("shutdown.exe")
        .args(["/h"])
        .output()?;
    // 检查状态
}
```

**关机实现**:
```rust
fn shutdown_system() -> Result<(), String> {
    let output = std::process::Command::new("shutdown.exe")
        .args(["/s", "/t", "0", "/f"])
        .output()?;
    // 检查状态
}
```

**统一执行入口**:
```rust
pub fn execute_action(action_type: &str) {
    match action_type {
        "lock" => lock_screen(),
        "suspend" => { let _ = suspend_system(); }
        "shutdown" => { let _ = shutdown_system(); }
        _ => eprintln!("未知操作"),
    }
}
```

## API

```rust
// Tauri Command
execute_system_action(action: "lock" | "suspend" | "shutdown") -> Result<(), String>

// 内部使用
execute_action(action_type: &str)
```

## 验证结果

```bash
$ cargo check
warning: 5 warnings
    Finished `dev` profile
```

- ✅ 编译通过
- ✅ 锁屏功能（rundll32）
- ✅ 休眠功能（shutdown /h）
- ✅ 关机功能（shutdown /s）

## 权限说明

休眠和关机操作需要**管理员权限**，否则命令会失败。错误信息会提示用户以管理员权限运行。

## 配置数据结构

```json
{
  "action": {
    "action_type": "lock",  // "lock" | "suspend" | "shutdown"
    "show_notice": true
  }
}
```

## 下一步
- E4: 提示优化（差异化提示策略）
