# M1: 项目搭建 - 开发日志

## 任务信息
- **任务ID**: M1
- **模块**: 项目搭建
- **目标**: Tauri+Rust环境配置，创建可运行的空项目

## 执行过程

### 2026-02-05

#### 环境检查
```bash
$ rustc --version && cargo --version && node --version && npm --version
rustc 1.90.0 (1159e78c4 2025-09-14)
cargo 1.90.0 (840b83a10 2025-07-30)
v20.19.4
10.8.2
```
- ✅ Rust 1.90.0 (满足Tauri v2要求)
- ✅ Node.js v20.19.4 (满足要求)
- ✅ npm 10.8.2

#### 安装 create-tauri-app
```bash
$ cargo install create-tauri-app --locked
```
- 安装成功，版本 v4.7.0

#### 创建项目
```bash
$ cargo create-tauri-app timer --manager npm --template vanilla-ts --yes
```
- 使用 vanilla-ts 模板（原生TypeScript，轻量级）
- 包管理器选择 npm
- 项目创建成功

#### 安装依赖
```bash
$ cd timer && npm install
```
- 安装 19 个包
- 无安全漏洞

#### 首次运行验证
```bash
$ npm run tauri dev
```
- Vite dev server 启动成功 (http://localhost:1420)
- Tauri Rust 编译成功
- 应用窗口正常显示

## 项目结构
```
timer/
├── src/                    # 前端代码 (TypeScript)
│   ├── main.ts
│   └── styles.css
├── src-tauri/              # Rust后端代码
│   ├── src/
│   │   └── main.rs         # 程序入口
│   ├── Cargo.toml          # Rust依赖
│   └── tauri.conf.json     # Tauri配置
├── index.html              # 前端HTML入口
├── package.json            # npm配置
└── vite.config.ts          # Vite配置
```

## 遇到的问题

| 问题 | 解决方案 |
|------|----------|
| cargo create-tauri-app 命令不存在 | 先执行 `cargo install create-tauri-app --locked` |

## 决策记录

### D1: 选择 vanilla-ts 模板
- **选项**: React/Vue/Vanilla/其他
- **决策**: 使用 vanilla-ts
- **原因**:
  1. 项目规模较小，不需要大型框架
  2. 减少依赖，降低复杂度
  3. TypeScript 提供类型安全
  4. 更易于理解和维护

## 验收结果

| 验收项 | 状态 | 备注 |
|--------|------|------|
| `cargo tauri dev` 正常启动 | ✅ 通过 | 开发服务器运行正常 |
| 应用窗口正常显示 | ✅ 通过 | 默认窗口显示正常 |
| 项目结构清晰 | ✅ 通过 | 符合Tauri标准结构 |

## 下一步
- M2: 系统托盘实现
