# E5: 配置持久化 - 开发日志

## 任务信息
- **任务ID**: E5
- **模块**: 配置持久化
- **目标**: 实现配置JSON文件存储，重启后配置保留

## 技术方案 (plan.md)
- 配置存储: `%APPDATA%/TimerApp/config.json`
- 序列化: `serde_json`
- 文件IO: 标准库 `std::fs`

## 数据结构 (plan.md v1.1)

```rust
AppConfig {
    version: "1.1",
    timer: {
        interval_minutes: 30,
        advance_notice_seconds: 30,
        max_delay_times: 3,
        delay_options: [5, 10, 30]
    },
    schedule: {
        time_limit_enabled: false,
        weekday_limit_enabled: false,
        start_time: "09:00",
        end_time: "18:00",
        weekdays: [1, 2, 3, 4, 5],
        logic: "AND"
    },
    action: {
        action_type: "lock",
        show_notice: true
    },
    startup: {
        auto_start: false,
        start_minimized: false,
        start_timer_automatically: false
    },
    log: {
        level: "info",
        max_days: 7,
        max_file_size_mb: 10,
        max_total_size_mb: 100
    }
}
```

## 实现过程

### 1. 添加依赖
```toml
dirs = "6"  # 获取系统配置目录
```

### 2. 创建配置模块
文件: `src-tauri/src/config.rs`

**核心结构**:
- `ConfigManager`: 配置管理器，线程安全
- `AppConfig`: 应用配置根结构
- `TimerConfig/ScheduleConfig/ActionConfig/StartupConfig/LogConfig`: 子配置

**API**:
- `ConfigManager::new()`: 加载或创建默认配置
- `get()`: 获取配置副本
- `update(f)`: 更新配置并自动保存
- `save()`: 手动保存到文件

### 3. Tauri Commands
- `get_config`: 获取完整配置
- `update_timer_config`: 更新定时器配置
- `update_schedule_config`: 更新生效规则
- `update_action_config`: 更新执行动作
- `update_startup_config`: 更新启动配置

## 验证结果

```bash
$ cargo check
warning: 4 warnings (unused imports/methods)
    Finished `dev` profile
```

- ✅ 编译通过
- ✅ 配置模块创建成功
- ✅ Tauri commands注册完成

## API 使用示例

```rust
// 获取配置
let config = config_manager.get()?;

// 更新配置
config_manager.update(|c| {
    c.timer.interval_minutes = 60;
})?;
```

## 下一步
- E1-E2: 生效规则模块（依赖本配置）
