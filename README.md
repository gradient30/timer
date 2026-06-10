# TimerApp

Windows 定时提醒与系统动作工具。到点后自动锁屏或关机，支持托盘驻留、离线激活与灵活的生效规则。

基于 **Rust + Tauri 2** 构建，界面提供深色、明亮、鲜艳三种风格。

## 界面预览

<p align="center">
  <img src="docs/assets/screenshot-main.png" alt="TimerApp 主界面：倒计时、预设时间与进度条" width="720" />
</p>

<p align="center"><em>主界面 — 倒计时显示、快速预设、自定义时长与进度追踪</em></p>

<p align="center">
  <img src="docs/assets/screenshot-settings.png" alt="TimerApp 设置面板：执行动作与生效规则" width="720" />
</p>

<p align="center"><em>设置面板 — 执行动作、时间段/星期规则、提前通知与循环模式</em></p>

## 功能

- 自定义倒计时（15/30/45/60 分钟或 1–1440 分钟）
- 提前通知、延后执行、循环休息
- 锁屏 / 关机、时间段与星期生效规则
- 系统托盘、开机自启、退出密码保护
- 离线激活码
- 三种界面风格（深色 / 明亮 / 鲜艳），左下角一键切换

## 环境要求

- Windows 10/11 x64
- [Rust](https://rustup.rs/) stable
- [Node.js](https://nodejs.org/) 20+
- Visual Studio Build Tools（「使用 C++ 的桌面开发」）

## 快速开始

```bash
git clone https://github.com/gradient30/timer.git
cd timer
./scripts/dev.sh setup-config
# 编辑 config/local/activation.env（勿提交）
cd timer && npm install
./scripts/dev.sh dev
```

## 常用命令

```bash
./scripts/dev.sh dev          # 开发模式
./scripts/dev.sh check        # 代码检查
./scripts/dev.sh release      # 构建公开发布 MSI
./scripts/dev.sh activation 5 # 生成激活码
```

## 文档

| 文档 | 说明 |
|------|------|
| [docs/release/CONFIGURATION.md](docs/release/CONFIGURATION.md) | 配置与构建密钥 |
| [docs/release/RELEASE.md](docs/release/RELEASE.md) | 发布流程 |
| [docs/activation/USAGE.md](docs/activation/USAGE.md) | 激活码 |
| [config/README.md](config/README.md) | 配置目录 |

## 目录结构

```
├── config/public/     # 公开配置模板
├── docs/              # 使用与发布文档
│   └── assets/        # README 展示截图
├── scripts/dev.sh     # 开发/构建脚本
└── timer/             # Tauri 应用源码
```

## License

[MIT](LICENSE)
