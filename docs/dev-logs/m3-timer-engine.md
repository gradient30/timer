# M3: 定时器核心引擎 - 开发日志

## 任务信息
- **任务ID**: M3
- **模块**: 定时器核心引擎
- **目标**: 实现精准的倒计时功能，支持开始/暂停/重置

## 执行过程

### 2026-02-05

#### 创建定时器模块
创建 `src-tauri/src/timer.rs`，实现计时器核心逻辑。

**核心数据结构**:
```rust
pub struct TimerEngine {
    runtime: Arc<Mutex<TimerRuntime>>,
    config: Arc<Mutex<TimerConfig>>,
    callback: Arc<Mutex<Option<Box<dyn Fn(TimerRuntime) + Send + 'static>>>>,
}

pub struct TimerRuntime {
    pub state: TimerState,
    pub remaining_seconds: u64,
    pub total_seconds: u64,
    pub last_update: Option<String>,
}
```

**遇到的问题**:
1. `runtime_clone` 被移动后再次使用
   - 解决: 在移动前检查 `remaining_seconds == 0`，保存结果到 `is_finished` 变量

2. `Emitter` trait 未导入
   - 解决: `use tauri::Emitter;`

3. 需要 `chrono` 库处理时间
   - 解决: 在 Cargo.toml 添加 `chrono = { version = "0.4", features = ["serde"] }`

#### Tauri Commands 实现
在 `lib.rs` 中添加前端调用的命令:
- `get_timer_runtime` - 获取运行时状态
- `get_timer_config` - 获取配置
- `set_timer_interval` - 设置时间间隔
- `start_timer` - 开始计时
- `pause_timer` - 暂停计时
- `resume_timer` - 继续计时
- `stop_timer` - 停止并重置
- `get_formatted_time` - 获取格式化时间 (MM:SS)

#### 事件系统
- `timer-update` - 每秒发送一次状态更新
- `tray-pause/resume/stop/quick-set` - 托盘菜单触发的事件

## API 规范

### Commands (前端 -> 后端)

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get_timer_runtime` | - | `TimerRuntime` | 获取当前状态 |
| `start_timer` | - | `Result<(), String>` | 开始计时 |
| `pause_timer` | - | `Result<(), String>` | 暂停计时 |
| `resume_timer` | - | `Result<(), String>` | 继续计时 |
| `stop_timer` | - | `Result<(), String>` | 停止并重置 |
| `set_timer_interval` | `minutes: u64` | `Result<(), String>` | 设置间隔(1-1440) |
| `get_formatted_time` | - | `String` | 获取 "MM:SS" |

### Events (后端 -> 前端)

| 事件名 | 数据类型 | 说明 |
|--------|----------|------|
| `timer-update` | `TimerRuntime` | 每秒发送状态更新 |
| `tray-pause` | `()` | 托盘请求暂停 |
| `tray-resume` | `()` | 托盘请求继续 |
| `tray-stop` | `()` | 托盘请求停止 |
| `tray-quick-set` | `u64` | 托盘快速设置分钟数 |

## 计时器精度

- 使用 `std::thread::sleep(Duration::from_millis(100))` 每100ms检查一次
- 使用 `Instant` 计算实际经过的时间
- 每秒更新一次倒计时和发送事件
- 目标精度: <1秒误差

## 验收结果

| 验收项 | 状态 | 备注 |
|--------|------|------|
| 开始计时 | ✅ | 正常启动后台线程 |
| 暂停计时 | ✅ | 状态变为Paused |
| 继续计时 | ✅ | 恢复倒计时 |
| 停止并重置 | ✅ | 回到初始状态 |
| 设置时间间隔 | ✅ | 支持1-1440分钟 |
| 每秒状态更新 | ✅ | 通过事件发送到前端 |
| 倒计时结束 | ✅ | 自动停止 |

## 下一步
- M4: 基础UI开发
