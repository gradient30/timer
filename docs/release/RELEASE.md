# TimerApp 发布流程

> GitHub 公共资源发布检查清单

---

## 一、发布前检查（必做）

### 1. 敏感信息扫描

- [ ] `config/local/` 未出现在 `git status`
- [ ] 代码中无硬编码密钥/口令（搜索 `SECRET`、`PASSWORD`、`api_key`）
- [ ] 日志文件（`logs/`、`*.log`）未纳入提交
- [ ] 无个人脚本、本地密钥或日志文件纳入提交

### 2. 构建密钥

- [ ] 已使用**独立于开发环境**的 `TIMER_ACTIVATION_SECRET_HEX`
- [ ] GitHub Actions Secrets 已配置（若使用 CI 构建）
- [ ] 公开发布构建**未**启用 `activation-admin` feature

### 3. 功能验证

```bash
./scripts/dev.sh check
./scripts/dev.sh release
```

- [ ] MSI 安装包可正常安装/卸载
- [ ] 未激活时弹出激活门禁
- [ ] 托盘「关于」弹窗正常
- [ ] 开机自启 + 自动计时（若启用）符合预期

---

## 二、版本号更新

同步修改以下位置：

| 文件 | 字段 |
|------|------|
| `timer/src-tauri/Cargo.toml` | `version` |
| `timer/src-tauri/tauri.conf.json` | `version` |
| `timer/package.json` | `version` |
| `timer/index.html` | 标题栏版本展示 |

---

## 三、构建公开发布包

### 本地构建

```bash
# 1. 准备构建密钥（仅本机，不提交）
./scripts/dev.sh setup-config
# 编辑 config/local/activation.env

# 2. 构建 MSI（不含开发后门）
./scripts/dev.sh release
```

产物路径：

```
timer/src-tauri/target/release/bundle/msi/
```

### GitHub Actions（推荐模板）

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable

      - name: Install dependencies
        working-directory: timer
        run: npm ci

      - name: Build MSI
        working-directory: timer
        env:
          TIMER_ACTIVATION_SECRET_HEX: ${{ secrets.TIMER_ACTIVATION_SECRET_HEX }}
          TIMER_GENERATOR_PASSWORD: ${{ secrets.TIMER_GENERATOR_PASSWORD }}
        run: npm run tauri build

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: timer-msi
          path: timer/src-tauri/target/release/bundle/msi/*.msi
```

---

## 四、创建 GitHub Release

```bash
git tag v0.1.0
git push origin v0.1.0
```

在 GitHub Releases 页面：

1. 选择对应 tag
2. 上传 MSI 安装包
3. 填写 Release Notes（功能变更、已知问题）
4. 标注最低系统要求：Windows 10/11 x64

---

## 五、发布后维护

| 事项 | 说明 |
|------|------|
| 激活码 | 使用 `cargo run --bin activation_gen` 或 `./scripts/dev.sh activation` 生成 |
| 配置迁移 | 升级时保持 `config.json` 的 `version` 字段向后兼容 |
| 安全公告 | 密钥泄露时轮换 `TIMER_ACTIVATION_SECRET_HEX` 并重新发布 |

---

## 六、Fork 者快速上手

```bash
git clone <your-repo-url>
cd timer
cp config/public/activation.env.example config/local/activation.env
# 编辑 config/local/activation.env
cd timer && npm install
./scripts/dev.sh dev
```

详细配置见 [CONFIGURATION.md](./CONFIGURATION.md)。
