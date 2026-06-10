# 离线激活码使用说明

## 构建密钥准备

生成激活码前，需配置本地构建密钥（不提交 Git）：

```bash
./scripts/dev.sh setup-config
# 编辑 config/local/activation.env
```

详见 [docs/release/CONFIGURATION.md](../release/CONFIGURATION.md)。

## 生成激活码

**推荐：CLI 工具（公开发布构建也可用）**

```bash
./scripts/dev.sh activation 10
```

或：

```bash
cargo run --bin activation_gen --manifest-path timer/src-tauri/Cargo.toml -- 10
```

**开发构建专属：应用内生成（需 `activation-admin` feature）**

仅在 `./scripts/dev.sh dev` 开发模式下，托盘提示文字连点 10 次可打开生成弹窗。

说明：
- `10` 表示生成数量，可替换为任意正整数
- 输出格式为 `XXXX-XXXX-XXXX-XXXX`
- 公开发布 MSI **不包含**应用内生成入口

## 发放激活码

将生成的激活码复制并发送给用户。

## 客户端激活

1. 启动应用后会弹出“激活应用”弹窗
2. 输入激活码并确认
3. 激活成功后即可正常使用

## 重要说明

- 离线激活码为**本机一次性**激活记录
- 若需“跨设备/重装仍不可复用”，需后续接入服务器激活
