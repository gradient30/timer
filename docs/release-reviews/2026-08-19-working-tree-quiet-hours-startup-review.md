# 发布审核记录 — 工作区：用餐/夜间休息时段 + HKCU 开机自启

> 本文件为唯一正式审核记录。代码、标识符、文件路径、命令、退出码、API 名称、错误码和原始输出保持原样；其含义在“中文解读”中说明。

## 审核元数据

| 字段 | 内容 |
| --- | --- |
| 审核时间 | 2026-08-19T14:46:29+08:00 |
| 仓库 / 分支 | `D:\workspace_test\github_repo\timer` / `main` |
| 修订基线 | `HEAD` = `9410fb6`（`chore(release): 升级版本至 0.1.2`）；审核对象为相对该提交的**未提交工作区**（含未跟踪 `timer/src-tauri/src/quiet_hours.rs`） |
| 变更范围 | 用餐/夜间休息时段（配置契约、后台监听、设置页）；HKCU Run 开机自启改写（`startup.rs`）；公开模板 `config/public/config.example.json`；文档 `docs/release/CONFIGURATION.md`、`scripts/README.md` |
| 审核人 | Cursor Agent（release-readiness-review） |

## 审查档位判定依据

- 选定档位：`完整审核`
- 已观察到的风险信号：
  - 运行时持久化 / 配置 schema（`AppConfig.quiet_hours`，serde 默认值）
  - 并发与可靠性（后台 `spawn_quiet_hours_watcher`、休息阶段重锁、锁屏动作）
  - 构建/发布配置（版本仍为 `0.1.2`；CI 要求 `clippy -D warnings`）
  - 公开契约（`config.example.json`、CONFIGURATION.md）
- 判定证据：
  - `docs/release/RELEASE.md` 将 `dev.sh check` / `test` / `npm run build` / 无 features 的 `cargo check|clippy` 列为本地等价验收
  - 工作区同时包含配置字段新增与定时器控制线程，符合技能中 “persistence / concurrency / release configuration” 的完整审核条件

## 证据清单

| 来源 | 状态 | 可证明的事项 | 链接或路径 |
| --- | --- | --- | --- |
| 任务 / Issue / PR | `可用` | 用户在本会话中给出的验收条件：默认 12:00–13:00、18:00–19:00、00:00–09:00；到点强制停表且不得因此无法解锁；结束时刻重置并按工作分钟自动开始；与循环番茄钟锁屏休息区分 | 本会话用户请求（仓库无对应 Issue/PR） |
| 规格 / ADR / 文档 | `可用` | 发布检查命令、配置分层；无独立 ADR。功能规格以用户请求 + `docs/release/CONFIGURATION.md` 新增 `quiet_hours` 行 + 设置为准 | `docs/release/RELEASE.md`、`docs/release/CONFIGURATION.md`、`config/public/config.example.json` |
| 契约 / 代码 / 测试 / 历史 | `可用` | 工作区 diff；`quiet_hours` 单元测试；`startup` 注册表往返测试 | `timer/src-tauri/src/quiet_hours.rs`、`lib.rs`、`startup.rs`、`config.rs`、`timer/src/main.ts`、`git log` / `git diff --stat HEAD` |
| 用户反馈来源 | `可用` | 公开 GitHub Issues 页面无条目；本机 `gh` 不可用。无独立缺陷跟踪记录 | https://github.com/gradient30/timer/issues （页面 “No results”）；https://github.com/gradient30/timer/pulls （欢迎页，无 PR 列表） |

## 五维审核矩阵

| 维度 | 状态 | 证据 | 发现 | 后续动作 |
| --- | --- | --- | --- | --- |
| 规格审核 | `已验证` | 用户请求中的默认窗口与半开区间行为，对照 `quiet_hours.rs` 测试 `default_windows_cover_lunch_dinner_and_night`、`tick_action`、设置页文案 | 无独立 ADR；规格来源为会话任务。默认 **enabled=true** 会使升级用户在午休/晚饭/夜间自动停表，与“默认”措辞一致，但是对已安装 `0.1.2` 用户的行为变化 | 发版说明中写明默认开启及可关闭位置（提案） |
| 事实审核 | `发现风险` | `git diff --stat HEAD`；`Cargo.toml`/`package.json`/`tauri.conf.json`/`index.html` 均为 `0.1.2`；`HEAD` 已是 `chore(release): 升级版本至 0.1.2` | ① 版本号未抬升，若直接打 MSI/tag 会与已发布 `0.1.2` 冲突。② 工作区还包含未跟踪 `docs/startup-on-boot.md`（文内写 Browser Switcher，非本产品）。③ 开机自启与休息时段两套未提交改动捆在同一工作区 | 发版前 `prepare-release` 升版本；**不要**把 `docs/startup-on-boot.md` 纳入提交 |
| 质量审核 | `证据缺失` | `cargo check --features activation-admin` 修正后无 warning；`./scripts/dev.sh check` 因 clippy 工具链失败；`query_run_value` 曾仅被测试调用，会导致 `-D warnings` 失败（审核中已改为 `#[cfg(all(windows, test))]`） | 本地 `cargo clippy -- -D warnings` 无法执行（工具链 manifest 缺失）。未跑 MSI `dev.sh release`。无 watcher 集成测试 | 在能跑 clippy 的环境（或 CI）补跑 `dev.sh check` 与无 features 的 clippy |
| 回归审核 | `发现风险` | `cargo test --features activation-admin`：`41 passed`；`npm run build` 成功；无 features 的 `cargo check` 成功 | 单元测试覆盖时段判定与进出动作，**不覆盖**真实锁屏/系统时钟跳变。`cargo test` 链接阶段出现 `linker_messages` warning（非测试失败） | 人工在 12:00/13:00 边界或改系统时间做一次停表/开表与解锁验证（提案） |
| 用户反馈审核 | `已验证` | GitHub Issues 公开页无 open/closed 结果；本会话请求即当前功能来源 | 无历史缺陷单可闭环。`gh` 未安装，Issue 结论来自公开页面抓取而非 `gh issue list` | 无 |

## 已执行命令与日志

| 原始命令 | 声明来源 | 退出码 | 原始输出 | 中文解读 |
| --- | --- | --- | --- | --- |
| `git status --short` / `git log -8 --oneline` / `git diff --stat HEAD` | 审核基线（工作区相对 `HEAD`） | `0` | 见附录 A | `main` 停在 `9410fb6`；工作区 10 个已修改文件 + 2 个未跟踪文件 |
| `& "C:\Program Files\Git\bin\bash.exe" -lc "cd /d/workspace_test/github_repo/timer && ./scripts/dev.sh check"` | `docs/release/RELEASE.md`、`.github/workflows/ci.yml` (`./scripts/dev.sh check`) | `1` | 见附录 B | `cargo check` 完成但报 `query_run_value` unused；随后 `cargo clippy` 因工具链 manifest 失败。**此为修正前证据** |
| `cargo check --features activation-admin`（修正后） | 同上 `dev.sh check` 的 check 半段 | `0` | `Finished dev profile ... in 1.10s`（无 warning） | 修正 `#[cfg(all(windows, test))]` 后库检查干净 |
| `cargo check`（无 features） | `RELEASE.md` / CI `release-parity` | `0` | `Finished dev profile ... in 5.79s` | 与发布包 feature 集一致的 check 通过 |
| `cargo clippy --features activation-admin -- -D warnings` | `dev.sh check` / CI | `1` | `error: Missing manifest in toolchain 'stable-x86_64-pc-windows-msvc'` | 本机 clippy 不可用，**不能**当作 clippy 通过 |
| `cargo test --features activation-admin` | `dev.sh test` / CI | `0` | 见附录 C | lib 单元测试 41 通过 |
| `npm run build`（`timer/`） | CI `check` / `release-parity`、`RELEASE.md` | `0` | 见附录 D | `tsc && vite build` 成功 |
| `gh issue list --repo gradient30/timer` | 用户反馈检索 | 非 0（命令未找到） | `The term 'gh' is not recognized...` | 本机无 `gh`，改用公开 Issues 页 |
| Web 抓取 `https://github.com/gradient30/timer/issues` | 可访问的产品反馈 | 页面可访问 | 正文含 `### No results` / `is:issue state:open` | 公开仓库当前无 Issue 条目 |

## 原始输出附录

### 附录 A — git 基线（修正后 `git status --short` 与 `git diff --stat HEAD`）

```text
 M config/public/config.example.json
 M docs/release/CONFIGURATION.md
 M scripts/README.md
 M timer/index.html
 M timer/src-tauri/src/config.rs
 M timer/src-tauri/src/lib.rs
 M timer/src-tauri/src/notifier.rs
 M timer/src-tauri/src/startup.rs
 M timer/src/layout-v2.css
 M timer/src/main.ts
?? docs/startup-on-boot.md
?? timer/src-tauri/src/quiet_hours.rs
====
 config/public/config.example.json |   8 +
 docs/release/CONFIGURATION.md     |   4 +-
 scripts/README.md                 |   2 +-
 timer/index.html                  |  34 +++-
 timer/src-tauri/src/config.rs     |  35 ++++
 timer/src-tauri/src/lib.rs        | 225 +++++++++++++++++++++--
 timer/src-tauri/src/notifier.rs   |  12 ++
 timer/src-tauri/src/startup.rs    | 369 +++++++++++++++++++++----------
 timer/src/layout-v2.css           |  45 +++++
 timer/src/main.ts                 | 131 +++++++++++++-
 10 files changed, 706 insertions(+), 159 deletions(-)
```

`git log -8 --oneline`（审核开始时）：

```text
9410fb6 chore(release): 升级版本至 0.1.2
515c761 chore(repo): 忽略个人 bundle 同步脚本
5c6126b fix(startup): 修复开机自启不能稳定生效
79a6050 chore(deps): 同步 Cargo.lock 版本至 0.1.1
d0c74f2 docs(readme): 更新 v2 界面预览截图与说明
01fc0e3 chore(release): 升级版本至 0.1.1
2728b4f feat(ui): 重做主界面 v2 并新增 prepare-release 发版脚本
7b73d62 fix(ci): 修复 Release 构建缺少 Pillow 依赖
```

中文解读：发布审核对象不是已打 tag 的干净 `0.1.2`，而是其上的未提交功能增量。

### 附录 B — 修正前 `./scripts/dev.sh check`（截取 warning 与失败尾）

中间为依赖 `Compiling`/`Checking` 行，从对本仓库有意义的 warning 起逐字保留：

```text
warning: function `query_run_value` is never used
   --> src\startup.rs:196:4
    |
196 | fn query_run_value(subkey: &str, value_name: &str) -> Result<Option<String>, String> {
    |    ^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `timer` (lib) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 22s
[INFO] Running cargo clippy -D warnings
error: Missing manifest in toolchain 'stable-x86_64-pc-windows-msvc'
```

中文解读：`query_run_value` 只在测试里使用，非 test 编译会 `dead_code`；CI 的 `clippy -- -D warnings` 会把该 warning 当错误。本机 clippy 组件/工具链清单缺失，check 脚本因此以 1 退出。

### 附录 C — `cargo test --features activation-admin` 结果段

```text
running 41 tests
...
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.81s
```

测试编译阶段另有：

```text
warning: linker stdout: 正在创建库 D:\workspace_test\github_repo\timer\timer\src-tauri\target\debug\deps\timer_lib.dll.lib 和对象 D:\workspace_test\github_repo\timer\timer\src-tauri\target\debug\deps\timer_lib.dll.exp
  |
  = note: `#[warn(linker_messages)]` on by default

warning: `timer` (lib) generated 1 warning
```

中文解读：41 项通过。`linker_messages` 来自 MSVC 链接器中文提示，不是测试断言失败；`dev.sh test` 不以 `-D warnings` 失败该 warning。

### 附录 D — `npm run build`

```text
npm notice run timer@0.1.2 build
npm notice run tsc && vite build
vite v6.4.1 building for production...
transforming...
✓ 13 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                 27.08 kB │ gzip:  5.87 kB
dist/assets/index-D-QW1SMw.css  40.38 kB │ gzip:  7.84 kB
dist/assets/index-DJlzz153.js   43.83 kB │ gzip: 12.47 kB
✓ built in 324ms
```

中文解读：前端 TypeScript 编译与 Vite 生产构建成功。

## 证据缺口

| 缺失或不可访问的证据 | 对本次审核的影响 | 跟进负责人 / 动作 |
| --- | --- | --- |
| 本机 `cargo clippy`（工具链 `Missing manifest`） | 无法证明无 clippy-only lint；`dead_code` 已用 `cargo check` 回归 | 在 CI 或修复 rustup clippy 后执行 `./scripts/dev.sh check` 及无 features clippy |
| `./scripts/dev.sh release` / MSI | 未验证打包与安装后的自启/休息时段 | 版本抬升后再打 MSI（提案） |
| `gh` CLI | 不能用 GitHub API 列 Issue/PR；已用公开网页替代 | 可选安装 `gh` 再查 |
| watcher / 锁屏的集成或手工边界测试记录 | 停表不锁屏、13:00 自动开表仅有单元测试与代码审查 | 发版前按测试计划手工过三个边界时刻（提案） |
| 历史审核记录 | `docs/release-reviews/` 此前不存在 | 本文件为该目录首份记录 |

## 风险登记

| 风险 | 严重性 | 影响范围 | 证据 | 建议的后续动作 |
| --- | --- | --- | --- | --- |
| 以 `0.1.2` 再发会与已发布版本号冲突 | 高（发布流程） | tag / MSI / `BUILD 0.1.2` | `9410fb6` 已升至 0.1.2；工作区未改版本 | 发版前 `./scripts/dev.sh prepare-release <新版本>`（提案） |
| CI `clippy -D warnings` 曾会被 `query_run_value` unused 打断 | 高（已在工作区修正） | `startup.rs`、CI `check` 与 `release-parity` | 附录 B warning | 保持 `#[cfg(all(windows, test))]`；CI 上再确认 clippy |
| 本机 clippy 未跑通 | 中 | 质量门禁证据不完整 | `Missing manifest in toolchain 'stable-x86_64-pc-windows-msvc'` | 修复 rustup 或依赖 CI |
| 误提交 `docs/startup-on-boot.md` | 中 | 公开仓库会混入另一产品（Browser Switcher）文档 | 文件首行 `# 开机启动实现方案（Browser Switcher）`；`git status` 为 `??` | **不要** add/commit 该文件 |
| 升级用户默认进入用餐/夜间停表 | 中（产品行为） | 已有 `%APPDATA%/TimerApp/config.json` 无该字段时 serde 填默认 enabled+三窗口 | `QuietHoursConfig::default`、`missing_quiet_hours_field_uses_defaults` | 发版说明写明；设置里可关（提案） |
| 停止/开始动作失败时若推进 `was_inside`，会漏掉补试 | 中（已修正） | `spawn_quiet_hours_watcher` | 原代码 `was_inside = Some(now_inside)` 无条件执行；现 `next_inside_flag` + 测试 `failed_action_keeps_previous_inside_flag` | 保持当前逻辑 |
| 工作区同时含自启改写与休息时段，且均未提交 | 低 | 审查/发版范围易混 | `git diff --stat` 同时列出 `startup.rs` 与 `quiet_hours.rs` | 按功能分提交（提案） |

## 审核中已落地的修正

| 问题 | 修正 |
| --- | --- |
| `query_run_value` 在非 test 构建中 `dead_code` | `#[cfg(all(windows, test))]` |
| 监听线程先 `sleep` 再轮询；动作失败仍推进时段状态 | 先轮询再 sleep；`next_inside_flag` 仅在 `action_ok` 时更新 |
| 用餐/夜间时段内 `delay_execution` 仍可能给计时器加时 | 与 `confirm_execution` 一样在时段内返回 `BLOCK_MESSAGE` |
| `scripts/README.md` 仍写「20 项单元测试」 | 改为不写死过期数字（当前 `cargo test` 为 41） |

## 发布建议

- 建议：`可发布，但存在已列明风险`
- 基于证据的理由：相对用户给出的停表/开表/不锁屏规格，代码与单元测试一致；配置缺省可加载。审核中已去掉会让 CI `clippy -D warnings` 失败的 `dead_code`。阻断级功能缺陷未在现有证据中复现。仍存在：版本号仍为已发布的 `0.1.2`、本机 clippy 无法执行、未打 MSI、未跟踪的他产品文档。这些不构成“证据已证明必须停止发布”的严重功能故障，但**不建议在抬升版本并在 CI 看到 clippy 绿灯之前打正式 Release**。
- 人工决定：`待定`

## 历史与用户反馈跟进

- 关联的历史审核记录：无（`docs/release-reviews/` 此前为空）
- 关联的用户反馈 / 缺陷记录：GitHub Issues 公开页无条目；规格来源于本会话用户请求
- 闭环状态：功能缺陷项已在工作区修正，**未提交**；版本抬升与 CI clippy 仍开放

## 技能改进建议 — 需人工确认

- 无
