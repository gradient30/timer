# TimerApp 发布流程

GitHub 公开发布检查清单。

## 一、发布前检查

### 敏感信息

- [ ] `config/local/` 未出现在 `git status`
- [ ] 无硬编码密钥（搜索 `SECRET`、`PASSWORD`）
- [ ] 无 `logs/`、`*.log`、`*.bundle` 等待提交文件

### 构建与激活

- [ ] 发行使用**独立** `TIMER_ACTIVATION_SECRET_HEX`（勿与开发环境混用）
- [ ] 公开发布构建**未**启用 `activation-admin`
- [ ] 发码与 MSI 使用**同一**密钥（见 [USAGE.md](../activation/USAGE.md)）

### 功能验证

```bash
./scripts/dev.sh check
./scripts/dev.sh release
```

- [ ] MSI 可安装/卸载，快捷方式与托盘为时钟图标
- [ ] 未激活时弹出激活门禁
- [ ] 激活码可成功激活
- [ ] 托盘「关于」、开机自启与自动计时符合预期

## 二、版本号同步

| 文件 | 字段/位置 |
|------|-----------|
| `timer/src-tauri/Cargo.toml` | `version` |
| `timer/src-tauri/tauri.conf.json` | `version`、`productName` |
| `timer/package.json` | `version` |
| `timer/index.html` | `BUILD x.x.x`、关于弹窗版本 |

## 三、构建 MSI

```bash
./scripts/dev.sh setup-config
# 编辑 config/local/activation.env

./scripts/dev.sh release   # 含 icons 生成 + npm run tauri build
```

产物示例：

```
timer/src-tauri/target/release/bundle/msi/TimerApp_0.1.0_x64_zh-CN.msi
```

命名规则：`{productName}_{version}_x64_zh-CN.msi`（`productName` 当前为 `TimerApp`）。

仅更新图标时可先执行 `./scripts/dev.sh icons`，再 `release`。

### GitHub Actions（已落地）

| 工作流 | 文件 | 触发 | 说明 |
|--------|------|------|------|
| CI | `.github/workflows/ci.yml` | `main` push/PR | `npm build` + `dev.sh check/test` + release-parity（无 `activation-admin`） |
| Release | `.github/workflows/release.yml` | tag `v*` | `dev.sh release` 构建 MSI 并上传 GitHub Release |

CI 与 Release **共用同名 Secrets**，但用途不同：

- **CI**：未配置 Secrets 时自动使用 `config/public/activation.env.example` 占位值，仅用于编译/测试。
- **Release**：**必须**配置正式发行密钥，与本地 MSI 发码一致。

### 配置 Repository Secrets

在 GitHub 仓库 **Settings → Secrets and variables → Actions** 添加：

| Secret | 用途 |
|--------|------|
| `TIMER_ACTIVATION_SECRET_HEX` | 64 位十六进制发行密钥（Release 必填；CI 可选） |
| `TIMER_GENERATOR_PASSWORD` | 编译期口令（Release 必填；CI 可选） |

> 正式发布请使用**独立**发行密钥，勿与开发环境混用。详见 [CONFIGURATION.md](./CONFIGURATION.md#github-actions-secrets)。

### 分支保护（推荐）

在 **Settings → Branches → Branch protection rules** 为 `main` 启用：

- [ ] Require a pull request before merging
- [ ] Require status checks to pass：**CI / check**、**CI / release-parity**
- [ ] Require branches to be up to date before merging

## 四、创建 GitHub Release

**手动：**

```bash
git tag v0.1.0
git push github v0.1.0   # 或你的 GitHub remote 名称
```

推送 `v*` 标签后，`release.yml` 自动构建 MSI 并创建 Release（`softprops/action-gh-release`）。

**手动上传（备用）：** 从 Actions 产物或本地 `target/release/bundle/msi/` 下载 MSI，注明 Windows 10/11 x64、激活方式、变更说明。

## 五、发布后

| 事项 | 说明 |
|------|------|
| 激活码 | `./scripts/dev.sh activation [count]` |
| 密钥轮换 | 泄露时更换 `TIMER_ACTIVATION_SECRET_HEX`，重新 `release` 并发新码 |
| 配置兼容 | 升级时保持 `config.json` 的 `version` 字段向后兼容 |

## 六、Fork 快速上手

```bash
git clone https://github.com/gradient30/timer.git
cd timer
./scripts/dev.sh setup-config
# 编辑 config/local/activation.env
cd timer && npm install
./scripts/dev.sh dev
```

详见 [CONFIGURATION.md](./CONFIGURATION.md)。
