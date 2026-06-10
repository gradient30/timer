# TimerApp

Windows 定时提醒与系统动作工具（Rust + Tauri 2）。

## 功能

- 自定义倒计时（15/30/45/60 分钟或 1–1440 分钟）
- 提前通知、延后执行、循环休息
- 锁屏 / 关机、时间段与星期生效规则
- 系统托盘、开机自启、退出密码保护
- 离线激活码

## 环境要求

- Windows 10/11 x64
- [Rust](https://rustup.rs/) stable
- [Node.js](https://nodejs.org/) 20+
- Visual Studio Build Tools（「使用 C++ 的桌面开发」）

## 快速开始

```bash
git clone <repository-url>
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
├── scripts/dev.sh     # 开发/构建脚本
└── timer/             # Tauri 应用源码
```

## License

[MIT](LICENSE)
