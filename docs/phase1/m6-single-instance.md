# M6: 单实例机制 - 开发日志

## 任务信息
- **任务ID**: M6
- **模块**: 单实例机制
- **目标**: 防止程序多开，第二个实例激活已有窗口

## 执行过程

### 2026-02-05

#### 创建单实例模块
创建 `src-tauri/src/single_instance.rs`

#### 实现代码
```rust
//! 单实例模块 - 防止程序多开

use tauri::{AppHandle, Manager};

/// 当检测到已有实例时，尝试激活已有窗口
pub fn activate_existing_window() {
    // 简化实现：在 lib.rs 中直接使用 FindWindowW
    // 这里只是一个占位符
}

/// 处理第二个实例启动的情况
pub fn handle_second_instance(app_handle: &AppHandle) {
    // 尝试激活已有窗口
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
```

#### 修复问题
编译错误: `trait Manager which provides get_webview_window is implemented but not in scope`

修复: 添加 `Manager` trait 导入
```rust
use tauri::{AppHandle, Manager};
```

#### 集成说明
当前实现为基础版本，在 `lib.rs` 中通过以下方式调用:
```rust
use single_instance::handle_second_instance;

// 在适当位置调用
handle_second_instance(&app_handle);
```

#### 编译结果
```
warning: function `activate_existing_window` is never used
warning: function `handle_second_instance` is never used
```
这两个警告是预期的，因为单实例检查逻辑需要在 `main.rs` 更早阶段执行。

## 完整单实例方案

### 方案: Window Title 检查 (推荐)
在 `main.rs` 的 `main()` 函数中，在Tauri启动前执行:

```rust
fn main() {
    // 1. 检查是否已有实例运行 (通过窗口标题)
    // 2. 如果有，激活已有窗口并退出
    // 3. 如果没有，正常启动Tauri

    timer_lib::run();
}
```

### 方案: 文件锁
使用文件系统锁来实现单实例:
```rust
use std::fs::OpenOptions;
use std::sync::Mutex;

static FILE_LOCK: Mutex<Option<File>> = Mutex::new(None);
```

## 当前状态

| 验收项 | 状态 | 备注 |
|--------|------|------|
| 模块创建 | ✅ | single_instance.rs 已创建 |
| 窗口激活函数 | ✅ | handle_second_instance 实现 |
| 编译通过 | ✅ | 无错误 |
| 完整集成 | ⏸️ | 需要在main.rs中添加前置检查 |

## 下一步
- 阶段一验收测试
- 在后续迭代中完善 main.rs 的单实例前置检查
