# TimerApp 项目文档

> 研发实施阶段文档库

---

## 文档结构

```
docs/
├── README.md                  # 本文档
├── phase1/                    # 阶段一 - MVP核心功能 ✅
│   ├── README.md              # 阶段一索引
│   ├── ACCEPTANCE.md          # 验收报告
│   ├── m1-project-setup.md    # M1 项目搭建
│   ├── m2-system-tray.md      # M2 系统托盘
│   ├── m3-timer-engine.md     # M3 定时器核心
│   ├── m4-basic-ui.md         # M4 基础UI
│   ├── m5-lock-screen.md      # M5 锁屏功能
│   └── m6-single-instance.md  # M6 单实例
├── phase2/                    # 阶段二 - 进阶功能 ✅
│   ├── README.md              # 阶段二索引
│   ├── ACCEPTANCE.md          # 验收报告
│   ├── e1-e2-schedule-rules.md   # E1/E2 生效规则
│   ├── e3-system-actions.md      # E3 执行动作扩展
│   ├── e4-notification-optimize.md # E4 提示优化
│   ├── e5-config-persistence.md  # E5 配置持久化
│   ├── e6-auto-startup.md        # E6 开机自启
│   └── e7-logging.md             # E7 日志记录
├── phase3/                    # 阶段三 - 安全与完善 ✅
│   ├── README.md              # 阶段三索引
│   ├── ACCEPTANCE.md          # 验收报告
│   ├── s1-password-setup.md   # S1 密码设置
│   ├── s2-exit-verification.md# S2 退出验证
│   ├── s3-password-reset.md   # S3 密保重置
│   ├── s4-safe-mode.md        # S4 安全模式
│   ├── s5-ui-polish.md        # S5 UI优化
│   └── s6-msi-bundling.md     # S6 MSI 打包
├── activation/                # 激活码体系（离线已完成，线上计划）
│   ├── REQUIREMENTS.md        # 需求说明
│   ├── PLAN.md                # 实施计划
│   └── USAGE.md               # 使用方法
└── phase1-review-report.md    # 阶段一联合审核报告
```

---

## 阶段概览

| 阶段 | 状态 | 说明 |
|------|------|------|
| [阶段一：MVP核心功能](./phase1/) | ✅ 已完成 | 托盘/定时器/UI/锁屏 |
| [阶段二：进阶功能](./phase2/) | ✅ 已完成 | 生效规则/执行动作/配置持久化/日志 |
| [阶段三：安全与完善](./phase3/) | ✅ 已完成 | 密码保护/安全模式/MSI安装包 |
| 阶段四：测试与交付 | ⬜ 待开始 | 集成测试/正式发布 |

## 激活码体系

- [需求说明](./activation/REQUIREMENTS.md)
- [实施计划](./activation/PLAN.md)
- [使用方法](./activation/USAGE.md)

---

## 联合审核

- [阶段一联合审核报告](./phase1-review-report.md) - 四角色联合审核结论

---

## 开发规范

1. **阶段文档隔离** - 各阶段文档独立目录管理
2. **每个任务必须有开发日志** - 记录实现细节、问题、解决方案
3. **阶段结束必须有验收报告** - 对照计划逐项验证
4. **联合审核通过方可进入下一阶段**

---

## 快速导航

| 目标 | 链接 |
|------|------|
| 查看阶段一交付物 | [./phase1/README.md](./phase1/README.md) |
| 查看阶段二交付物 | [./phase2/README.md](./phase2/README.md) |
| 查看阶段三交付物 | [./phase3/README.md](./phase3/README.md) |
| 查看阶段一审核结论 | [./phase1-review-report.md](./phase1-review-report.md) |
| 激活码体系文档 | [./activation/REQUIREMENTS.md](./activation/REQUIREMENTS.md) |
