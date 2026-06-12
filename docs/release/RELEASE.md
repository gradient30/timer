# TimerApp 发布流程

GitHub 公开发布检查清单与 CI/CD 操作指南。

## 零、GitHub Actions 操作步骤

### 分阶段交付审核

| 阶段 | 目标 | 交付物 | 审核结论 |
|------|------|--------|----------|
| **1 — CI** | `main` push/PR 自动检查 | `.github/workflows/ci.yml`、`.github/scripts/setup-ci-env.sh` | **通过**：`npm build` → `dev.sh check` → `dev.sh test`；Secrets 可选（无则占位编译） |
| **2 — Release** | tag `v*` 自动构建 MSI 并发布 | `.github/workflows/release.yml` | **通过**：强制 Release Secrets → `dev.sh release` → 上传 Artifact + GitHub Release |
| **3 — 优化** | 缓存、发布 parity、分支保护 | `ci.yml` 中 `release-parity` job + 本文档 | **通过**：npm/Rust 缓存；无 `activation-admin` 的 `cargo check/clippy`；分支保护说明已文档化 |

本地等价验收（推送前建议执行）：

```bash
./scripts/dev.sh check
./scripts/dev.sh test
cd timer && npm run build
cd timer/src-tauri && cargo check && cargo clippy -- -D warnings
```

### 首次启用（一次性）

**1. 配置 GitHub 远程**

本仓库默认 `origin` 指向私有同步源（tcloud）；公开发布使用 `github` 远程：

```bash
# 若尚未添加
git remote add github https://github.com/gradient30/timer.git
git remote -v
```

**2. 推送工作流到 GitHub**

```bash
git push github main
```

推送后在 [Actions](https://github.com/gradient30/timer/actions) 确认 **CI / check** 与 **CI / release-parity** 均为绿色。

**3. 配置 Repository Secrets**

路径：**Settings → Secrets and variables → Actions → New repository secret**

| Secret | CI | Release | 值来源 |
|--------|:--:|:-------:|--------|
| `TIMER_ACTIVATION_SECRET_HEX` | 可选 | **必填** | 发行专用 64 位十六进制（`openssl rand -hex 32`） |
| `TIMER_GENERATOR_PASSWORD` | 可选 | **必填** | 任意强口令（编译期注入，release 包无应用内入口） |

- **仅跑 CI**：可不配置，工作流自动使用 `config/public/activation.env.example` 占位值。
- **正式发布**：必须使用**独立发行密钥**（勿与 `config/local/` 开发密钥混用）；与本地 `dev.sh activation` 发码保持一致。

**4. （推荐）启用 `main` 分支保护**

**Settings → Branches → Add rule**，勾选：

- Require a pull request before merging
- Require status checks：**CI / check**、**CI / release-parity**
- Require branches to be up to date before merging

### 日常开发

```bash
# 本地验证（与 CI job「check」一致）
./scripts/dev.sh check
./scripts/dev.sh test
cd timer && npm run build

git checkout -b feature/xxx
# ... 修改代码 ...
git push github feature/xxx   # 开 PR，自动触发 CI
```

PR 合并到 `main` 后再次触发 CI。双远程同步时，可按需 `git push origin main` 同步到 tcloud。

### 正式发布（GitHub Release）

**1. 同步版本号**（见下文「二、版本号同步」）

**2. 本地冒烟（可选但推荐）**

```bash
./scripts/dev.sh release
# 安装 MSI，验证激活、托盘、计时
./scripts/dev.sh activation 3   # 用发行密钥生成的码验活
```

**3. 打标签并推送**

```bash
git tag v0.1.0
git push github v0.1.0
```

`release.yml` 将自动：构建 MSI → 上传 Actions Artifact（`TimerApp-msi`）→ 创建 GitHub Release 并附上 MSI。

也可在 Actions 页手动 **Run workflow**（`workflow_dispatch`）做试构建；**无 tag 时不会创建 GitHub Release**，仅产出 Artifact。

**4. 发激活码**

```bash
# 本地 config/local/activation.env 须与 GitHub Release Secrets 使用同一发行密钥
./scripts/dev.sh activation 10
```

### 故障排查

| 现象 | 处理 |
|------|------|
| CI 编译报 `TIMER_ACTIVATION_SECRET_HEX` 格式错误 | 检查 Secret 是否为 64 位十六进制 |
| Release 第一步即失败 | 未配置 Release Secrets，在仓库 Settings 补全 |
| `dev.sh release` 本地成功、Actions 失败 | 查看日志：`icons` 需 Python 3.11 + `pip install -r scripts/requirements.txt`（Pillow）；或 `cargo`/`npm` 具体错误 |
| 激活码无法用于 MSI | MSI 与发码密钥不一致；更换密钥后须重新 `release` 并重发码 |

密钥与环境变量详解见 [CONFIGURATION.md](./CONFIGURATION.md#github-actions-secrets)。

---

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

> **CI/CD 自动化**已落地，完整操作见上文 [零、GitHub Actions 操作步骤](#零github-actions-操作步骤)。

### 工作流一览

| 工作流 | 文件 | 触发 | Job |
|--------|------|------|-----|
| CI | `.github/workflows/ci.yml` | `main` push/PR、`workflow_dispatch` | `check`、`release-parity` |
| Release | `.github/workflows/release.yml` | tag `v*`、`workflow_dispatch` | `build`（MSI + GitHub Release） |

## 四、创建 GitHub Release

**推荐（自动）：** 打 tag 并 `git push github vX.Y.Z`，见 [零、正式发布](#正式发布github-release)。

**手动上传（备用）：** 从 Actions 产物 `TimerApp-msi` 或本地 `target/release/bundle/msi/` 下载 MSI，在 GitHub Releases 页面上传，注明 Windows 10/11 x64、激活方式、变更说明。

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
