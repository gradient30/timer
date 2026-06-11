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
| `icons` | 重新生成时钟图标（`icon.ico`、托盘、MSI） |
| `release` | 公开发布 MSI（先执行 `icons`，不含 `activation-admin`） |
| `activation [count]` | CLI 生成离线激活码，默认 1 个 |
| `setup-config` | 从模板创建 `config/local/activation.env` |
| `help` | 显示帮助 |

## 示例

```bash
./scripts/dev.sh setup-config   # 首次：创建并编辑 activation.env
./scripts/dev.sh dev
./scripts/dev.sh icons          # 仅更新图标资源
./scripts/dev.sh release        # 输出 MSI 安装包
./scripts/dev.sh activation 10  # 生成 10 个激活码
```

## 开发 vs 发布

| 命令 | Cargo Features | 应用内激活码入口 |
|------|----------------|------------------|
| `dev` / `build` / `check` / `test` | `activation-admin` | 有（托盘提示文字连点 10 次） |
| `release` | 无 | **无** |

**激活码与发布包：** `activation` 与 `release` 均通过 `build.rs` 读取同一份 `config/local/activation.env`。只要 `TIMER_ACTIVATION_SECRET_HEX` 未变，CLI 生成的激活码可用于对应 MSI；修改密钥后须重新 `release` 并重新发码。

构建密钥见 [docs/release/CONFIGURATION.md](../docs/release/CONFIGURATION.md)。
