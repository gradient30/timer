# 项目管理脚本

统一使用 `scripts/dev.sh` 管理开发、构建与激活码生成。

## 命令一览

| 命令 | 说明 |
|------|------|
| `dev` | Tauri 开发模式（启用 `activation-admin`） |
| `build` | 编译 `src-tauri`（开发 feature） |
| `check` | `cargo check` + `cargo clippy -D warnings` |
| `test` | `cargo test` |
| `clean` | 清理 `timer/dist` 与 `timer/src-tauri/target` |
| `docs` | 打开 `docs/` 目录 |
| `icons` | 重新生成时钟图标（`icon.ico`、托盘、MSI；需 Python + `scripts/requirements.txt`） |
| `release` | 公开发布 MSI（先执行 `icons`，不含 `activation-admin`） |
| `prepare-release <ver> [--check] [--commit] [--sync]` | 升级版本号；可选检查、提交、tcloud bundle 同步，并提示 GitHub 手动发布步骤 |
| `activation [count]` | CLI 生成离线激活码，默认 1 个 |
| `setup-config` | 从模板创建 `config/local/activation.env` |
| `setup-hooks` | 启用 `.githooks`，提交时自动剔除 Cursor co-author |
| `help` | 显示帮助 |

## 示例

```bash
./scripts/dev.sh setup-config   # 首次：创建并编辑 activation.env
./scripts/dev.sh dev
./scripts/dev.sh icons          # 仅更新图标资源
./scripts/dev.sh release        # 输出 MSI 安装包
./scripts/dev.sh activation 10  # 生成 10 个激活码
./scripts/dev.sh prepare-release 0.1.1 --commit --sync  # 发版前：升版本 + 提交 + 同步 tcloud
```

### 发版一条龙（GitHub 手动 Release）

```bash
# 1. 升版本、提交、同步到 tcloud（GitHub 网页发布仍需手动）
./scripts/dev.sh prepare-release 0.1.1 --commit --sync

# 2. 等 tcloud → GitHub main 更新后，在 GitHub 创建 Release：
#    Tag v0.1.1 / Title TimerApp v0.1.1 / Target main

# 3. Actions 完成后发码
./scripts/dev.sh activation 10
```

## 开发 vs 发布

| 命令 | Cargo Features | 应用内激活码入口 |
|------|----------------|------------------|
| `dev` / `build` / `check` / `test` | `activation-admin` | 有（托盘提示文字连点 10 次） |
| `release` | 无 | **无** |

**激活码与发布包：** `activation` 与 `release` 均通过 `build.rs` 读取同一份 `config/local/activation.env`。只要 `TIMER_ACTIVATION_SECRET_HEX` 未变，CLI 生成的激活码可用于对应 MSI；修改密钥后须重新 `release` 并重新发码。

构建密钥见 [docs/release/CONFIGURATION.md](../docs/release/CONFIGURATION.md)。

## 与 GitHub Actions 的对应关系

| 本地命令 | CI job | 说明 |
|----------|--------|------|
| `npm run build`（在 `timer/`） | `check`、`release-parity` | 前端 TypeScript + Vite 构建 |
| `dev.sh check` | `check` | `activation-admin` + clippy `-D warnings` |
| `dev.sh test` | `check` | `cargo test`（20 项单元测试） |
| `cargo check` / `clippy`（无 features） | `release-parity` | 与 `release` 相同的 Cargo feature 集 |
| `dev.sh release` | Release `build` | 需 GitHub Secrets；含 `icons` + MSI |

CI 在未配置 Secrets 时使用公开模板占位值；Release 工作流强制要求正式发行密钥。  
操作步骤见 [docs/release/RELEASE.md](../docs/release/RELEASE.md#零github-actions-操作步骤)。
