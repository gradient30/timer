# 配置目录

将**可公开模板**与**本地私密配置**分离，便于开源发布。

## 结构

```
config/
├── public/                   # 可提交
│   ├── activation.env.example
│   └── config.example.json
└── local/                    # 禁止提交
    └── activation.env        # 构建密钥（HMAC + 生成器口令）
```

## 快速开始

```bash
./scripts/dev.sh setup-config
# 编辑 config/local/activation.env
./scripts/dev.sh dev
```

`activation.env` 中 `TIMER_ACTIVATION_SECRET_HEX` 同时用于：

- `./scripts/dev.sh release` 编译进 MSI
- `./scripts/dev.sh activation` 生成可验码

二者密钥须一致；修改后须重新打 MSI 并重新发码。

## 分层说明

| 层级 | 路径 | 提交 Git | 用途 |
|------|------|----------|------|
| 构建密钥 | `config/local/activation.env` | 否 | 编译期注入 |
| 运行时配置 | `%APPDATA%/TimerApp/config.json` | 否 | 用户数据 |
| 公开模板 | `config/public/*` | 是 | 文档与首次参考 |

详见 [docs/release/CONFIGURATION.md](../docs/release/CONFIGURATION.md)。
