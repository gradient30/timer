# TimerApp 配置说明

配置分层与构建密钥指南。

## 一、配置分层

| 层级 | 位置 | 提交 Git | 说明 |
|------|------|----------|------|
| 构建密钥 | `config/local/activation.env` | 否 | 编译期注入 HMAC 密钥 |
| 公开模板 | `config/public/*` | 是 | Fork 参考 |
| 应用运行时 | `%APPDATA%/TimerApp/config.json` | 否 | 计时、密码、激活、主题等 |
| 应用日志 | `%APPDATA%/TimerApp/logs/` | 否 | 运行日志 |

## 二、构建密钥（`config/local/activation.env`）

### 初始化

```bash
./scripts/dev.sh setup-config
# 或：cp config/public/activation.env.example config/local/activation.env
```

### 字段

| 变量 | 必填 | 说明 |
|------|------|------|
| `TIMER_ACTIVATION_SECRET_HEX` | 是 | 64 位十六进制（32 字节），激活码 HMAC 签名；**release 与 CLI 发码必须一致** |
| `TIMER_GENERATOR_PASSWORD` | 是 | 编译期必填；**仅** `activation-admin` 应用内生成入口校验口令 |

### 生成随机密钥

```powershell
# PowerShell
-join ((1..32) | ForEach-Object { '{0:x2}' -f (Get-Random -Max 256) })
```

```bash
# bash
openssl rand -hex 32
```

### CI 与 Release 密钥策略

| 场景 | 密钥来源 | 说明 |
|------|----------|------|
| 本地开发 | `config/local/activation.env` | `dev.sh setup-config` 初始化 |
| CI（check/test） | Secrets **或** 公开模板占位 | `.github/scripts/setup-ci-env.sh` 自动回退 |
| Release（MSI） | **必须** Repository Secrets | 与 CLI 发码同一发行密钥 |

> 公开发布请使用**独立**发行密钥；客户端离线密钥无法绝对防逆向。

### GitHub Actions Secrets

在仓库 **Settings → Secrets and variables → Actions** 配置：

| Secret | CI | Release | 说明 |
|--------|:--:|:-------:|------|
| `TIMER_ACTIVATION_SECRET_HEX` | 可选 | **必填** | 64 位十六进制 |
| `TIMER_GENERATOR_PASSWORD` | 可选 | **必填** | 编译期口令 |

- **未配置 CI Secrets**：工作流使用 `config/public/activation.env.example` 中的占位值，足以通过 `cargo check/clippy/test`。
- **Release 工作流**：缺少任一 Secret 将直接失败，避免误发占位密钥包。

工作流文件：`.github/workflows/ci.yml`、`.github/workflows/release.yml`。  
推送、打 tag、Secrets 配置等操作步骤见 [RELEASE.md](./RELEASE.md#零github-actions-操作步骤)。

## 三、开发 vs 发布构建

| 类型 | 命令 | Features | 应用内生成入口 |
|------|------|----------|----------------|
| 开发 | `./scripts/dev.sh dev` | `activation-admin` | 有 |
| 发布 | `./scripts/dev.sh release` | 无 | **无** |

发布包仍包含：用户激活弹窗、维护者本地 CLI 发码（与发布包共用密钥即可验码）。

`release` 会自动执行 `icons`，生成时钟图标并打入 MSI。

## 四、运行时配置（`config.json`）

默认：`%APPDATA%/TimerApp/config.json`  
模板：[`config/public/config.example.json`](../../config/public/config.example.json)

| 模块 | 字段 | 说明 |
|------|------|------|
| `timer` | `interval_minutes` | 倒计时间隔（分钟） |
| `timer` | `loop_enabled` | 循环执行 |
| `startup` | `auto_start` | 开机自启 |
| `startup` | `start_timer_automatically` | 启动后自动开始计时 |
| `startup` | `start_minimized` | 启动时最小化到托盘 |
| `ui` | `theme` | `dark` / `light` / `vivid` |
| `activation` | `activated` | 是否已激活 |
| `security` | `password_hash` | 退出密码 Argon2id 哈希 |

敏感字段勿提交：`password_hash`、`security_answer_hash`、`activation_code_hash`。

## 五、勿提交清单

根目录 `.gitignore` 已覆盖：`config/local/`、`.env*`、`logs/`、`*.log`、`*.bundle`、`last_sync_id`、`timer/dist/`、`timer/src-tauri/target/` 等。

提交前执行 `git status`，确认无本地密钥与日志。

## 六、相关文档

- [发布流程](./RELEASE.md)
- [激活码使用](../activation/USAGE.md)
- [配置目录](../../config/README.md)
