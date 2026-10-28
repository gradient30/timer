# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

TimerApp — Windows 定时提醒与系统动作工具（Rust + Tauri 2）。倒计时到点后锁屏/关机，支持托盘驻留、循环休息（Work/Rest 两阶段）、生效规则、退出密码与离线激活。仓库根目录放脚本、配置与文档，实际应用在 `timer/` 子目录（前端 + `timer/src-tauri/`）。代码注释、UI 文案与提交信息均为中文。

## 常用命令

所有开发操作统一走 `scripts/dev.sh`（Windows 下在 Git Bash/MSYS 中运行）：

```bash
./scripts/dev.sh dev           # 开发模式（features=activation-admin），端口 1422
./scripts/dev.sh check         # cargo check + clippy -D warnings（CI 强制）
./scripts/dev.sh test          # cargo test（--features activation-admin）
./scripts/dev.sh build         # cargo build
./scripts/dev.sh release       # 公开发布 MSI（先跑 icons，不带 activation-admin）
./scripts/dev.sh icons         # 重新生成图标资源（需 Python + scripts/requirements.txt）
./scripts/dev.sh activation 5  # 生成 5 个离线激活码（cargo run --bin activation_gen）
./scripts/dev.sh setup-config  # 首次：从模板创建 config/local/activation.env
```

- 前端类型检查/构建：`cd timer && npm run build`（tsc + vite build，CI 会跑）
- 运行单个测试：`cd timer/src-tauri && cargo test <测试名> --features activation-admin`
- 测试为各模块内的 `#[cfg(test)] mod tests` 单元测试，无前端测试

## 构建密钥（重要）

`build.rs` 在**任何** cargo 编译（含 check/test）时都强制要求激活密钥，缺失直接 panic。`dev.sh` 各命令会自动从 `config/public/activation.env.example` 复制模板到 `config/local/activation.env`（gitignore），但模板值仅占位。直接跑 cargo 前须确保该文件存在或已导出 `TIMER_ACTIVATION_SECRET_HEX` / `TIMER_GENERATOR_PASSWORD`。

密钥经 `env!()` 编译期注入：同一密钥既编译进 MSI，也用于 `activation` 命令生成可验激活码，二者必须一致。

## Cargo Features

- `activation-admin`：仅开发构建启用（dev/build/check/test 默认带），暴露激活码生成 command 与前端隐藏入口（托盘提示文字连点 10 次）
- 发布构建（`release`）不带任何 feature，激活码入口不存在于产物中
- CI 两个 job 都要过：`check`（带 activation-admin + clippy -D warnings + test）与 `release-parity`（无 feature 的 cargo check/clippy）

## 版本发布

版本号存在于多处：`timer/src-tauri/Cargo.toml`、`timer/src-tauri/tauri.conf.json`、`timer/index.html`、`timer/package.json`（+ lock）。用 `./scripts/dev.sh prepare-release <ver> [--check] [--commit] [--sync]` 统一升级，**不要手动改**。推送 `v*` 标签触发 GitHub Actions 构建 MSI。

## 架构

### 后端（timer/src-tauri/src/）

- `lib.rs` — 装配中心：注册全部 tauri command、创建托盘菜单、管理 `AppState`、驱动周期 tick、向前端 emit 事件
- `timer.rs` — `TimerEngine` 计时核心：后台线程 + `Arc<Mutex<TimerRuntime>>`，Work/Rest 两阶段；`from_runtime_state_with_interval` + `offline_adjusted_remaining`（config.rs）实现重启/关机后按离线流逝时间补偿恢复
- `config.rs` — `ConfigManager`：配置持久化到 `%APPDATA%/TimerApp/config.json`，带 `CONFIG_VERSION` 迁移逻辑
- `activation.rs` — 离线激活码（HMAC-SHA256，Base32 编码）。**`ensure_activated()` 是几乎所有 tauri command 的第一道门禁，新增 command 必须同样调用**
- `quiet_hours.rs` — 用餐/夜间休息时段：半开区间 `[start, end)`，开始时刻强制停止倒计时，结束时刻重置并自动开始工作倒计时
- `schedule.rs` — 生效规则：时间段 + 星期限制，AND/OR 组合
- `security.rs` — 退出密码：argon2 哈希、3 次失败锁 5 分钟、安全问题找回、safe_mode 逃生
- `startup.rs` — 开机自启：HKCU Run 注册表（无需管理员），启动时按当前 exe 路径自愈写入
- `single_instance.rs` — Windows Mutex 单实例守卫（句柄须在进程生命周期内持有）
- `system.rs` — 锁屏（rundll32 LockWorkStation）/关机/打开文件夹
- `notifier.rs` — 通知策略（模态弹窗/托盘）
- `logger.rs` — tracing 日志；`bin/activation_gen.rs` — CLI 发码器

注意：`timer.rs` 与 `config.rs` 各有一个 `TimerConfig`（lib.rs 中 `use timer::TimerConfig as TimerSettings` 消歧）。

### 前端（timer/src/）

无框架，纯 TypeScript + 原生 DOM 操作：

- `main.ts`（约 1600 行单文件）— 全部 UI 逻辑：圆环倒计时、各类弹窗（提醒/激活/密码设置）、通过 `invoke` 调 command、`listen` 收事件
- `theme.ts` — 三主题切换（深色/明亮/鲜艳）
- `system-dialog.ts` — 系统级 confirm/alert 封装

前后端通信：前端 `invoke("<command>")` 调用；后端 `app.emit("<event>", payload)` 推送（如 `timer-update`、`quiet-hours-update`）。

## Git 约定

- 提交信息：conventional commits + 中文描述，如 `feat(timer): 新增用餐夜间休息时段并改用 HKCU Run 自启`
- `.githooks/commit-msg` 自动剔除 Cursor co-author trailer（`./scripts/dev.sh setup-hooks` 启用 `core.hooksPath`）
- 根目录的 `git-sync-to-tcloud.sh`、`apply-bundle.sh` 等为个人 tcloud bundle 同步脚本，与构建无关
