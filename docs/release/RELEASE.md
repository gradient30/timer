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

### GitHub Actions 模板

```yaml
name: Release
on:
  push:
    tags: ['v*']

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
        env:
          TIMER_ACTIVATION_SECRET_HEX: ${{ secrets.TIMER_ACTIVATION_SECRET_HEX }}
          TIMER_GENERATOR_PASSWORD: ${{ secrets.TIMER_GENERATOR_PASSWORD }}
        run: |
          cd ..
          bash ./scripts/dev.sh release
```

## 四、创建 GitHub Release

```bash
git tag v0.1.0
git push origin v0.1.0
```

上传 MSI，注明：Windows 10/11 x64、激活方式、变更说明。

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
