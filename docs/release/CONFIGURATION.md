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

### CI 发布

构建前导出同名环境变量（或确保 `config/local/activation.env` 由 CI 写入）：

```yaml
env:
  TIMER_ACTIVATION_SECRET_HEX: ${{ secrets.TIMER_ACTIVATION_SECRET_HEX }}
  TIMER_GENERATOR_PASSWORD: ${{ secrets.TIMER_GENERATOR_PASSWORD }}
```

> 公开发布请使用独立发行密钥；客户端离线密钥无法绝对防逆向。

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
