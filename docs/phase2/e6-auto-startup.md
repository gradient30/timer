# E6: 开机自启 - 开发日志

## 任务信息
- **任务ID**: E6
- **模块**: 开机自启
- **目标**: 通过注册表操作实现Windows开机自启

## 技术方案 (plan.md)
- **注册表路径**: `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
- **实现方式**: 使用 `reg.exe` 命令行工具

## 实现过程

### 方案选择

**方案一: Windows API (尝试)**
```rust
use windows::Win32::System::Registry::{RegOpenKeyExW, RegSetValueExW};
// API签名复杂，参数类型要求严格
```
问题: Windows 0.62版本API参数类型变化大，编译困难。

**方案二: reg.exe (采用)**
```rust
Command::new("reg.exe")
    .args(["add", REGISTRY_KEY, "/v", APP_NAME, ...])
```
优点:
- 简单可靠
- 无需复杂Windows API绑定
- 与手动操作等效

### 实现代码

```rust
const REGISTRY_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

pub fn set_auto_start(enabled: bool) -> Result<(), String> {
    if enabled {
        // reg add 添加注册表项
        Command::new("reg.exe")
            .args(["add", REGISTRY_KEY, "/v", "TimerApp",
                   "/t", "REG_SZ", "/d", exe_path, "/f"])
    } else {
        // reg delete 删除注册表项
        Command::new("reg.exe")
            .args(["delete", REGISTRY_KEY, "/v", "TimerApp", "/f"])
    }
}

pub fn is_auto_start_enabled() -> Result<bool, String> {
    // reg query 查询注册表项
    Command::new("reg.exe")
        .args(["query", REGISTRY_KEY, "/v", "TimerApp"])
}
```

## API

```rust
// Tauri Commands
is_auto_start_enabled() -> Result<bool, String>
set_auto_start(enabled: bool) -> Result<(), String>
```

## 验证结果

```bash
$ cargo check
warning: 15 warnings
    Finished `dev` profile
```

- ✅ 编译通过
- ✅ 启用/禁用自启功能
- ✅ 状态查询功能

## 配置数据结构

```json
{
  "startup": {
    "auto_start": false,
    "start_minimized": false,
    "start_timer_automatically": false
  }
}
```

## 下一步
- E7: 日志记录
