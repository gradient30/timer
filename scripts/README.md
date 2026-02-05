# 项目管理脚本

TimerApp 快速管理脚本，支持 PowerShell 和 CMD。

## 使用方式

```powershell
# PowerShell (推荐)
.\scripts\manage.ps1 [命令]

# CMD
scripts\manage.bat [命令]
```

## 可用命令

| 命令 | 说明 |
|------|------|
| `dev` | 启动开发服务器 |
| `build` | 构建项目 |
| `check` | 代码检查 (check + clippy) |
| `test` | 运行测试 |
| `clean` | 清理构建缓存 |
| `docs` | 打开文档目录 |
| `release` | 构建发布版本 |
| `help` | 显示帮助 |

## 示例

```powershell
# 启动开发服务器
.\scripts\manage.ps1 dev

# 检查代码
.\scripts\manage.ps1 check

# 清理缓存
.\scripts\manage.ps1 clean
```
