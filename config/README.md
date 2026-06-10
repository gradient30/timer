# TimerApp 配置目录

本目录将**可公开的配置模板**与**本地私密配置**分离，便于发布到 GitHub 公共资源。

## 目录结构

```
config/
├── README.md                 # 本说明
├── public/                   # 可提交：示例与文档模板
│   ├── activation.env.example
│   └── config.example.json
└── local/                    # 禁止提交：仅本机开发/发布构建使用
    └── activation.env        # 构建时密钥（gitignore）
```

## 快速开始（本地开发）

1. 复制激活密钥模板：

```bash
cp config/public/activation.env.example config/local/activation.env
```

2. 编辑 `config/local/activation.env`，填入你自己的随机密钥与口令。

3. 使用开发脚本启动（默认启用 `activation-admin` 功能）：

```bash
./scripts/dev.sh dev
```

## 配置分层说明

| 层级 | 路径 | 是否提交 | 用途 |
|------|------|----------|------|
| 构建密钥 | `config/local/activation.env` | 否 | 编译期注入 HMAC 密钥与生成器口令 |
| 应用运行时配置 | `%APPDATA%/TimerApp/config.json` | 否 | 用户计时、密码、激活状态等 |
| 公开模板 | `config/public/*` | 是 | 文档与首次安装参考 |

详细说明见 [docs/release/CONFIGURATION.md](../docs/release/CONFIGURATION.md)。
