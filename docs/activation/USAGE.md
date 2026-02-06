# 离线激活码使用说明

## 生成激活码

在项目根目录执行：

```bash
cargo run --bin activation_gen --manifest-path "D:\workspace_test\github_repo\timer\timer\src-tauri\Cargo.toml" -- 10
```

或使用快捷脚本：

```powershell
.\scripts\generate-activation.ps1 10
```

说明：
- `10` 表示生成数量，可替换为任意正整数
- 输出格式为 `XXXX-XXXX-XXXX-XXXX`

## 发放激活码

将生成的激活码复制并发送给用户。

## 客户端激活

1. 启动应用后会弹出“激活应用”弹窗
2. 输入激活码并确认
3. 激活成功后即可正常使用

## 重要说明

- 离线激活码为**本机一次性**激活记录
- 若需“跨设备/重装仍不可复用”，需后续接入服务器激活
