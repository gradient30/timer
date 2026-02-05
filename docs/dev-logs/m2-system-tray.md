# M2: 系统托盘 - 开发日志

## 任务信息
- **任务ID**: M2
- **模块**: 系统托盘
- **目标**: 实现系统托盘图标、右键菜单、左键切换显示/隐藏

## 执行过程

### 2026-02-05

#### 初始实现
修改 `src-tauri/src/lib.rs` 添加托盘功能。

**遇到的问题**:
1. `tray` 模块未找到
   - 原因: Tauri v2 需要显式启用 `tray-icon` 特性
   - 解决: 在 `Cargo.toml` 中添加 `features = ["tray-icon"]`

2. `set_submenu` 方法不存在
   - 原因: API变更，MenuItem不支持子菜单
   - 解决: 简化设计，直接展开快速设置选项

3. `menu_on_left_click` 已弃用
   - 解决: 改用 `show_menu_on_left_click`

#### 端口冲突问题
开发服务器端口1420被占用，修改为1422:
- `vite.config.ts`: port 1422
- `tauri.conf.json`: devUrl "http://localhost:1422"

## 实现的功能

### 托盘菜单项
| 菜单项 | 功能 |
|--------|------|
| 显示主窗口 | 显示并聚焦主窗口 |
| 暂停/继续 | 切换计时器状态(预留) |
| 停止并重置 | 重置计时器(预留) |
| 快速: 15分钟 | 快速设置15分钟(预留) |
| 快速: 30分钟 | 快速设置30分钟(预留) |
| 快速: 60分钟 | 快速设置60分钟(预留) |
| 关于 | 显示版本信息 |
| 退出 | 退出程序 |

### 窗口行为
- 左键单击托盘图标: 显示/隐藏主窗口
- 关闭窗口: 最小化到托盘（不退出）

### 代码结构
```rust
// 应用程序状态
pub struct AppState {
    pub timer_state: TimerState,
    pub remaining_seconds: u64,
}

// 计时器状态枚举
pub enum TimerState {
    Idle,      // 停止
    Running,   // 运行中
    Paused,    // 已暂停
}
```

## 关键代码片段

### 托盘创建
```rust
let _tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .tooltip("TimerApp - 停止")
    .menu(&menu)
    .show_menu_on_left_click(false)
    .on_menu_event(|app, event| { /* ... */ })
    .on_tray_icon_event(|tray, event| { /* ... */ })
    .build(app)?;
```

### 关闭时最小化到托盘
```rust
window.on_window_event(move |event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window_clone.hide();
    }
});
```

## 依赖变更

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
```

## 验收结果

| 验收项 | 状态 | 备注 |
|--------|------|------|
| 托盘图标显示 | ✅ | 使用默认图标 |
| 右键菜单 | ✅ | 8个菜单项 |
| 左键切换显示/隐藏 | ✅ | 正常 |
| 关闭窗口最小化到托盘 | ✅ | 正常 |
| 托盘退出功能 | ✅ | 正常 |

## 下一步
- M3: 定时器核心引擎
