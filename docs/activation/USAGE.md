# 离线激活码使用说明

## 1. 构建密钥（维护者）

生成激活码或打发布包前，配置本地密钥（**勿提交 Git**）：

```bash
./scripts/dev.sh setup-config
# 编辑 config/local/activation.env
```

字段说明见 [CONFIGURATION.md](../release/CONFIGURATION.md)。

## 2. 生成激活码

### CLI（推荐，适用于公开发布 MSI）

```bash
./scripts/dev.sh activation 10
```

或：

```bash
cargo run --bin activation_gen --manifest-path timer/src-tauri/Cargo.toml -- 10
```

- `10` 为数量，可改为任意正整数
- 格式：`XXXX-XXXX-XXXX-XXXX`（16 位 Base32，分段显示）
- **与 release 包关系：** CLI 与 `./scripts/dev.sh release` 使用同一 `TIMER_ACTIVATION_SECRET_HEX`。密钥不变时，此处生成的码可给对应 MSI 的外部用户使用；**修改密钥后必须重新打 MSI 并重新发码。**

### 应用内（仅开发构建）

仅在 `./scripts/dev.sh dev`（`activation-admin` feature）下可用：

1. 点击底部状态栏 **「关闭窗口最小化到托盘」** 文字连点 10 次
2. 输入 `TIMER_GENERATOR_PASSWORD` 口令
3. 一次生成 **5 个** 激活码

公开发布 MSI **不包含**上述入口。

## 3. 用户激活

1. 首次启动弹出「激活应用」
2. 输入激活码并确认
3. 成功后写入 `%APPDATA%/TimerApp/config.json`，可正常使用

## 4. 离线模式说明

| 项目 | 当前行为 |
|------|----------|
| 校验方式 | HMAC-SHA256 签名校验，无联网 |
| 单机记录 | 每台设备本地只激活一次（已激活不可重复输入） |
| 跨设备 | **同一激活码可在多台设备使用**（离线无法全局作废） |
| 全局一次性 | 需后续接入服务端激活（见 [REQUIREMENTS.md](./REQUIREMENTS.md)） |
