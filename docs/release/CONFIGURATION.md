# TimerApp 配置说明

> 面向开源发布与本地开发的配置分层指南

---

## 一、配置分层总览

| 层级 | 位置 | 是否提交 Git | 说明 |
|------|------|--------------|------|
| 构建密钥 | `config/local/activation.env` | **否** | 编译期注入 HMAC 密钥与生成器口令 |
| 公开模板 | `config/public/*` | **是** | 示例配置，供 Fork 者参考 |
| 应用运行时 | `%APPDATA%/TimerApp/config.json` | **否** | 用户计时、密码、激活状态 |
| 应用日志 | `%APPDATA%/TimerApp/logs/` | **否** | 运行日志 |

---

## 二、构建密钥（`config/local/activation.env`）

### 初始化

```bash
cp config/public/activation.env.example config/local/activation.env
```

或使用脚本：

```bash
./scripts/dev.sh setup-config
```

### 字段说明

| 变量 | 必填 | 说明 |
|------|------|------|
| `TIMER_ACTIVATION_SECRET_HEX` | 是 | 64 位十六进制（32 字节），用于激活码 HMAC 签名 |
| `TIMER_GENERATOR_PASSWORD` | 是 | 激活码生成器口令（仅开发构建使用） |

### 生成随机密钥

**PowerShell：**

```powershell
-join ((1..32) | ForEach-Object { '{0:x2}' -f (Get-Random -Max 256) })
```

**bash：**

```bash
openssl rand -hex 32
```

### CI / GitHub Actions 发布

在仓库 Secrets 中配置同名变量，构建前导出：

```yaml
env:
  TIMER_ACTIVATION_SECRET_HEX: ${{ secrets.TIMER_ACTIVATION_SECRET_HEX }}
  TIMER_GENERATOR_PASSWORD: ${{ secrets.TIMER_GENERATOR_PASSWORD }}
```

> **安全提示**：客户端离线激活密钥无法做到绝对防逆向。公开发布时请为每个发行渠道使用独立密钥，并视需要接入服务端激活。

---

## 三、开发构建 vs 公开发布构建

| 构建类型 | 命令 | Cargo Features | 激活码 UI 后门 |
|----------|------|----------------|----------------|
| 本地开发 | `./scripts/dev.sh dev` | `activation-admin` | 启用（托盘提示连点） |
| 公开发布 | `./scripts/dev.sh release` | 无 | **禁用** |

公开发布构建仍保留：
- 用户激活弹窗（`activate_with_code`）
- CLI 生成器 `activation_gen`（维护者本地使用）

---

## 四、应用运行时配置（`config.json`）

默认路径：`%APPDATA%/TimerApp/config.json`

公开模板见 [`config/public/config.example.json`](../../config/public/config.example.json)。

### 主要字段

| 模块 | 字段 | 说明 |
|------|------|------|
| `timer` | `interval_minutes` | 主倒计时间隔（分钟） |
| `timer` | `loop_enabled` | 是否循环执行 |
| `startup` | `auto_start` | 开机自启（注册表） |
| `startup` | `start_timer_automatically` | 启动后自动开始计时 |
| `startup` | `start_minimized` | 启动时最小化到托盘 |
| `activation` | `activated` | 是否已激活 |
| `security` | `password_hash` | 退出密码 Argon2id 哈希 |

### 敏感字段（永不提交）

- `security.password_hash`
- `security.security_answer_hash`
- `activation.activation_code_hash`

---

## 五、禁止提交的文件清单

根目录 `.gitignore` 已覆盖：

```
config/local/
.env / .env.*
logs/ / *.log
firebase-debug.log
.claude/settings.local.json
timer/src-tauri/target/
timer/dist/
```

提交前请执行：

```bash
git status
```

确认无 `config/local/`、日志、本地 `.env` 文件。

---

## 六、相关文档

- [发布流程](./RELEASE.md)
- [配置目录说明](../../config/README.md)
- [激活码使用](../activation/USAGE.md)
