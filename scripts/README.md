# 项目管理脚本

统一使用跨平台脚本 `scripts/dev.sh` 管理项目。

## 使用方式

```bash
./scripts/dev.sh [命令]
```

## 可用命令

| 命令 | 说明 |
|------|------|
| `dev` | 启动 Tauri 开发模式 |
| `build` | 构建 Rust 项目（`src-tauri`） |
| `check` | 代码检查（`cargo check` + `cargo clippy -D warnings`） |
| `test` | 运行测试（`cargo test`） |
| `clean` | 清理构建缓存（`timer/dist` + `timer/src-tauri/target`） |
| `docs` | 打开文档目录 |
| `release` | 构建发布版本（`npm run tauri build`） |
| `activation [count]` | 生成离线激活码，默认 1 个 |
| `setup-config` | 从公开模板创建 `config/local/activation.env` |
| `help` | 显示帮助 |

## 示例

```bash
./scripts/dev.sh dev
./scripts/dev.sh check
./scripts/dev.sh activation 10
./scripts/dev.sh setup-config
```

## 开发 vs 发布构建

| 命令 | 说明 |
|------|------|
| `dev` / `build` / `check` | 启用 `activation-admin`（开发后门） |
| `release` | 公开发布构建，**不**包含激活码生成 UI 后门 |

构建密钥见 [docs/release/CONFIGURATION.md](../docs/release/CONFIGURATION.md)。
