# Windows 平台验证记录

本文是 CopyPolish 的 Windows 平台验证记录文档，承载**流程落地参数、验证清单、状态标记和每次 Windows 验证的实际结果**。

- 流程规范：[../../AGENTS.md](../../AGENTS.md) 第 5 节（代理操作规则）与 [../development/cross-platform-validation.md](../development/cross-platform-validation.md)（规则正文与落地参数）；
- 验证队列与集中验证计划：本文 §4（由 [../development/cross-platform-validation.md](../development/cross-platform-validation.md) §16 与 `AGENTS.md` §6 规定）；
- 命令级执行步骤、通过条件、artifact 与清理要求：[../windows-e2e-runbook.md](../windows-e2e-runbook.md)；
- 测试层次与平台边界：[../testing.md](../testing.md) §2.2 / §2.3 / §7；
- 发布前 Windows 验收：[../release/manual-release.md](../release/manual-release.md) §6.1；
- 历史验收快照：[../archive/validation/windows-2026-09.md](../archive/validation/windows-2026-09.md)（只读历史，不承载新任务）。

本文不复制 Runbook 的命令细节，也不把巨大日志复制进正文；只记录验证项目、状态、命令、关键结果和错误摘要，并保留日志/artifact 的本地路径。

## 1. Source of Truth 与同步约定

| 角色 | 位置 | 说明 |
| --- | --- | --- |
| 同步源 | Linux 项目目录（本仓库根） | 源代码、项目状态和项目文档的主要事实来源；也是验证记录的保存位置 |
| 同步目标 | `E:\Shiraishi\VSCode Workspace\chinese_copywriting_formatter`（WSL 侧 `/mnt/e/Shiraishi/VSCode Workspace/chinese_copywriting_formatter`） | 仅作为 Windows 验证工作副本 |

- 同步方向固定为 Linux → Windows，单向执行；除本验证文档外，不把 Windows 工作副本中的代码反向同步回 Linux 项目。
- 同步内容与排除项按 [../development/cross-platform-validation.md](../development/cross-platform-validation.md) §7、`AGENTS.md` §5.7 和仓库 [`.gitignore`](../../.gitignore) 判定。不应同步：`node_modules`、`.venv`、`src-tauri/target`、`src-tauri/gen`、`frontend/dist`、`e2e/artifacts`、`e2e/settings-*`、`secrets/`、明文凭据、`.vscode/`、`.codex/` 等机器本地或可再生内容。
- 同步前必须先按 `AGENTS.md` §5.7（同步前检查）核对 Linux 源状态与 Windows 目标目录，不无条件删除未知文件，不覆盖 Windows 本地配置。
- **2026-09-15 观察到的目标状态**：该目录是 `dev` 分支的 Git checkout（HEAD `ac92636`，2026-09-01），并已存在大量本地未提交改动（源文件与文档已被覆盖为较新的 Linux 内容）。它不构成一个干净的 Git commit，因此任何基于该副本的验证都必须在记录中说明“包含 working tree changes”。

## 2. 状态标记

记录中只使用以下状态，禁止用含糊措辞代替：

| 状态 | 含义 |
| --- | --- |
| `PASS` | 实际执行并通过，有命令输出、计数或 artifact 支持 |
| `FAIL` | 实际执行且失败，必须记录关键错误 |
| `BLOCKED` | 因前置失败、依赖不可用或环境缺失而无法执行，必须写明依赖关系 |
| `NOT RUN` | 本轮未计划或主动未执行，必须写明原因 |
| `NOT APPLICABLE` | 当前项目或该 Windows 环境不适用，必须写明判定依据 |

附加约束：runner 出现 `exitCode=0` 但 `finished=0` 时只能记为未完成，不得记为 `PASS`。

Computer Use / GUI automation 不可用导致的**无法执行**不是功能失败：应标记 `BLOCKED` 并记录 `Blocker: COMPUTER_USE_UNAVAILABLE`，不得记为 `FAIL` / `PASS` / `NOT_APPLICABLE`。处理规则（分类、一次性重试、继续独立验证、非 GUI 替代、人工队列）见 [../development/cross-platform-validation.md](../development/cross-platform-validation.md) §17；本项目的人工测试步骤记录在 §4.6。

本节标记描述**单次验证项目**的执行结果；任务级生命周期状态（`IMPLEMENTED` → `LINUX_VERIFIED` → `WINDOWS_VERIFICATION_PENDING` → `WINDOWS_PASS`，失败回流见下）见 [../development/cross-platform-validation.md](../development/cross-platform-validation.md) §4。Windows 验证失败后的处理流程（Linux 修复、regression、状态回到 `WINDOWS_VERIFICATION_PENDING`）见同文档 §10。

## 3. Windows 验证清单与适用性

清单来源为项目自身文档，不新增无依据的验证项。

### 3.1 Required（项目文档明确要求的 Windows 门禁）

| ID | 验证项目 | 执行命令（PowerShell 7，项目根目录） | 通过条件 | 依据 |
| --- | --- | --- | --- | --- |
| W1 | 依赖安装与 E2E 类型检查 | `npm ci --prefix frontend`；`npm ci --prefix e2e`；`npm run typecheck --prefix e2e` | 退出码均为 0 | testing.md §2.3、§7.0-2 |
| W2 | 默认 embedded GUI 完整回归 | `npm run build:app --prefix e2e`；`npm run test --prefix e2e -- --spec specs/selection-and-persistence.spec.ts` | 3/3 passing；默认构建 capability=false、简繁归一化为 `conversion: none` | testing.md §2.2 门禁 1 |
| W3 | 简繁 feature GUI | `npm run build:app:simplified-trad --prefix e2e`；`npm run test --prefix e2e -- --spec specs/simplified-trad-conversion.spec.ts` | 2/2 passing；日志含 `Additional Cargo features: simplified-trad-conversion`；s2t/t2s 真实输出 | testing.md §2.2 门禁 2 |
| W4 | 标准 W3C provider 兼容性 smoke | `npm run build:app:webdriver --prefix e2e`；`npm run test:webdriver --prefix e2e` | `specs/w3c/smoke.spec.ts` 完成 session、主窗口、一次格式化、一次设置保存、退出与清理 | testing.md §2.2 门禁 3 |
| W5 | Windows MSVC Rust/TUI 编译与测试 | `cargo test --manifest-path src-tauri/Cargo.toml --features tui` | 退出码 0、0 failed，无平台条件编译错误 | testing.md §2.2 门禁 4 |
| W6 | 设置故障、ACL、artifact 与 transcript | `npm run test:corrupt-settings --prefix e2e`；`npm run test:acl-settings --prefix e2e`；`npm run test:gui-visual-artifacts --prefix e2e`；`npm run test:tui-transcript --prefix e2e` | 3/3、1/1、1/1、4/4；ACL 测试的 deny ACE 已恢复 | testing.md §2.2 门禁 5 |
| W7 | 发布资产启动 smoke | `.\scripts\build_release_local.ps1 <vX.Y.Z[-suffix]>`，随后启动 `CopyPolish.exe` 与 `CopyPolish-tui.exe` | 一次真实格式化、设置保存/重启恢复、剪贴板、退出检查；TUI raw-mode、粘贴、OSC 52、保存/退出；资产无 fixture/日志/`node_modules` 混入 | manual-release §6.1 步骤 1–3 |
| W8 | 发布资产校验 | `python3 scripts/verify_release_assets.py <tag> --dist-dir dist/windows --platform windows`；最终汇总目录执行 `--platform all` 与 `sha256sum -c SHA256SUMS` | 无校验错误；`--include-metadata` 用于最终汇总校验 | testing.md §2.2 门禁 6、manual-release §6.1 步骤 4 |

### 3.2 Applicable（按本次变更范围决定是否执行）

| ID | 验证项目 | 执行命令 | 触发条件 |
| --- | --- | --- | --- |
| A1 | Linux 侧同等门禁在 Windows 复跑 | `python3 scripts/verify.py --profile checks` / `frontend` / `rust` | 变更涉及文档、前端或 Rust 引擎 |
| A2 | embedded provider 全量 spec | `npm run test --prefix e2e` | 变更涉及 GUI 交互、设置或 IPC |
| A3 | 设置重启恢复 | `npm run test:restart-settings --prefix e2e` | 变更涉及设置持久化或 capability 归一化 |
| A4 | 失败 artifact 完整性 probe | `npm run test:artifact-probe --prefix e2e` | 变更涉及 E2E artifact、诊断或 runner |
| A5 | 设置控制台与 React act warning | `npm run test:settings-shortcut-console --prefix e2e` | 变更涉及设置窗口、快捷键或 React 版本 |
| A6 | Windows Terminal 交互 artifact | `npm run test:tui-terminal-artifact --prefix e2e` | 变更涉及 TUI 渲染、raw-mode、OSC 52、字体或 emoji 宽度 |
| A7 | TUI release binary 构建 | `cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui` | 需要真实终端或发布 smoke |
| A8 | 本地 artifact 清理 | `python3 scripts/clean.py --generated` | 每轮验证结束后（结果已写入文档） |

### 3.3 NOT APPLICABLE / 项目决定跳过

| 验证项目 | 判定 | 依据 |
| --- | --- | --- |
| GUI DPI 自动矩阵（100%/125%/150%） | `NOT APPLICABLE`：项目决定跳过，不纳入自动化门禁；125%/150% 人工 GUI 检查按历史记录已完成 | windows-e2e-runbook.md §3、§1.2 |
| GitLab Windows 可选 E2E stage | `NOT APPLICABLE`：项目决定不配置、不运行；不得记为通过 | windows-e2e-runbook.md §5 |
| 安装器 / MSI 校验 | `NOT APPLICABLE`：产品为免安装便携版，不生成任何安装器 | README、manual-release §6 |
| 交叉编译产物 | `NOT APPLICABLE`：项目未配置交叉编译，Windows 资产必须在 Windows 原生构建 | manual-release §1 / §6 |
| 独立代码生成步骤 | `NOT APPLICABLE`：仓库无独立 codegen 步骤；`src-tauri/gen/` 是 Tauri 构建生成物 | development.md、`.gitignore` |
| 网络 / 外部服务集成 | `NOT APPLICABLE`：当前无必需外部服务；GitLab 仅作为可选 Build Service | manual-release §1、development.md |
| Tauri sidecar / externalBin | `NOT APPLICABLE`：`src-tauri/tauri.conf.json` 未配置 `externalBin` | `src-tauri/tauri.conf.json` |

## 4. Windows Validation Queue 与集中验证计划

本节把 [../development/cross-platform-validation.md](../development/cross-platform-validation.md) §16 的“批量开发、集中验证”落到项目内：开发期间滚动累计待验证项，进入 `Windows Validation Preparation` 时再把整个 final diff 汇总为集中验证计划。默认状态为 `WINDOWS_VERIFICATION_PENDING`；只有缺少该 Windows 验证结果会导致后续实现无法可靠继续时才标为 `WINDOWS_VERIFICATION_BLOCKING`。

### 4.1 队列字段与优先级

每个验证项至少记录：validation item、related feature / change、relevant files / modules、why Windows validation is required、exact behavior to verify、prerequisite、expected result、priority、whether it blocks further Linux development。

优先级：`P0`（必须验证，失败意味着任务不能完成）、`P1`（重要的平台兼容性验证）、`P2`（建议验证，但不阻塞主要功能）。

### 4.2 当前队列（截至 2026-09-30）

| ID | 验证项 | 状态 | 优先级 | 阻塞后续 Linux 开发 |
| --- | --- | --- | --- | --- |
| Q1 | `--preset` 非交互 CLI 行为（TUI/CLI） | `WINDOWS_VERIFICATION_PENDING` | P1 | 否 |
| Q2 | Windows Terminal 原生 TUI 交互复验 | `WINDOWS_VERIFICATION_PENDING` | P1 | 否 |
| Q3 | 发布前 Windows 资产与启动 smoke（GUI + TUI） | `WINDOWS_VERIFICATION_PENDING` | P1 | 否 |
| Q4 | GUI 中文布局与排版（WebView2/DPI/窗口控制/剪贴板） | `WINDOWS_VERIFICATION_PENDING` | P1 | 否 |
| Q5 | 简繁转换 feature 构建（opencc-fmmseg 0.12.x） | `WINDOWS_VERIFICATION_PENDING` | P1 | 否 |
| Q6 | Tauri 运行时与 CLI 补丁升级（#85、#79） | `WINDOWS_VERIFICATION_PENDING` | P1 | 否 |
| Q7 | 发布链路 Actions 跨主版本升级（#69、#67、#65、#64） | `WINDOWS_VERIFICATION_PENDING` | P2 | 否 |

当前无 `P0` 项，无 `WINDOWS_VERIFICATION_BLOCKING` 项。Q5–Q7 于 2026-09-30 开放 PR 评估后新增，评估过程见 `docs/roadmap.md`「P0：开放 PR 评估修复」。

#### Q1 — `--preset` 非交互 CLI 行为

- related change：`6e8c034 feat(cli): add --preset parameter for non-interactive mode`；
- relevant files / modules：`src-tauri/src/tui/cli.rs`、`README.md`、`docs/testing.md`；
- why Windows validation is required：TUI/CLI 以 Windows 资产 `CopyPolish-tui.exe` 发布，参数解析、stdout/stderr 与退出码属于 Windows 运行时可验证行为（`docs/testing.md` §2.2 门禁 5、§7.12）；
- exact behavior to verify：`copypolish-tui.exe --input <file> --output <file> --preset pdf-cleaning`（以及 `copywriting`、`technical-docs`）按预设启用规则；未知预设输出 `文案净排：未知预设：<key>（可选：copywriting、pdf-cleaning、technical-docs）` 并以退出码 1 结束；`--preset` 与显式 `--rules` 并存时 `--rules` 优先；`--input` / `--output` 覆盖 Windows 盘符路径与 Unicode 文件名；
- prerequisite：Windows 原生 checkout，并执行 `cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui`；
- expected result：与 Linux 行为一致，无 panic、无编码问题；可按 `docs/testing.md` §7.12 的 transcript 结构留证（若需把 preset 场景加入 transcript runner，属独立开发任务）；
- 未覆盖原因：`6e8c034` 的提交时间为 2026-09-07 23:09，晚于 Runbook 2026-09-07 用户确认条目（文件写入时间 21:51），也晚于 2026-09-04 的完整 Windows 复验，因此无法确认已被覆盖。

#### Q2 — Windows Terminal 原生 TUI 交互复验

- related change：`docs/roadmap.md` P0「依赖与安全维护」条目要求（ratatui / `lru` 升级后需复验），并叠加 `6e8c034` 的 TUI/CLI 改动；
- relevant files / modules：`src-tauri/src/tui/**`、`docs/windows-e2e-runbook.md` §4；
- why Windows validation is required：raw-mode、规则面板、快捷键、Unicode 粘贴、OSC 52 与终端状态清理只能在真实 Windows Terminal + PowerShell 7 中验证；
- exact behavior to verify：按 `docs/windows-e2e-runbook.md` §4.4 的交互步骤逐项确认；
- prerequisite：Windows Terminal、PowerShell 7、当前 release binary；
- expected result：满足 Runbook §4.5 的完成判定；
- 说明：与 Q1 共用同一次 Windows 会话（先跑非交互 CLI，再进入交互复验），避免为零散功能分别安排 Windows 启动。

#### Q3 — 发布前 Windows 资产与启动 smoke

- related change：项目发布门禁（`docs/release/manual-release.md` §6.1、`docs/testing.md` §2.2 门禁 6）；
- relevant files / modules：`scripts/build_release_local.ps1`、`scripts/verify_release_assets.py`；
- why Windows validation is required：Windows 资产必须在 Windows 原生构建与验收，项目未配置交叉编译；
- exact behavior to verify：`CopyPolish.exe` 启动、一次真实格式化、设置保存与重启恢复、剪贴板、退出；`CopyPolish-tui.exe` 在 Windows Terminal 的 raw-mode / OSC 52 / 保存退出；`.7z` 根目录结构，以及 `--platform windows` / `--platform all` 与哈希校验；
- prerequisite：待发布 tag / 候选资产（当前开发基线为 `0.7.0-dev.1`，尚无候选 tag）；
- expected result：发布资产门禁全部通过；
- 触发时机：发布前（不是当前开发阶段的阻塞项）。

#### Q4 — GUI 中文布局与排版

- related change：`docs/roadmap.md` P1「GUI 中文布局与排版」；主界面重排、设置分类导航、模式文案修正、复制反馈优化；
- relevant files / modules：`frontend/src/App.tsx`、`frontend/src/index.css`、`frontend/src/components/SettingsDialog.tsx`、`frontend/src/components/HelpDialog.tsx`、`frontend/src/components/settings/*`；
- why Windows validation is required：桌面默认窗口 920×720、最小窗口 800×600、DPI 缩放、窗口拖动/最小化/最大化/关闭、真实系统剪贴板只能在 Windows 原生 WebView2 环境验证；
- exact behavior to verify：默认与最小窗口下左右对照与窄屏堆叠正常；实时/手动模式标题与立即排版动作正确；设置六个分类可切换且焦点可用；浅色/深色主题、80%–125% 界面缩放、中文与中英文混排无溢出；复制结果/复制并清空/清空输入语义保持；
- prerequisite：Windows 原生 checkout，并执行 `npm run build:app --prefix e2e` 构建当前 GUI；
- expected result：界面行为与 Linux 验证一致；出现 Computer Use 不可用时按 §4.5 标记 `BLOCKED`，进入人工队列复核。

#### Q5 — 简繁转换 feature 构建（opencc-fmmseg 0.12.x）

- related change：开放 PR #80（`opencc-fmmseg` 0.12.0 → 0.12.1）；`dev` 已由 #73 将依赖升级至 0.12.0；
- relevant files / modules：`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`scripts/verify.py`、简繁转换相关 Rust 模块；
- why Windows validation is required：`simplified-trad-conversion` 是可选 feature，`scripts/verify.py` 的 Rust 步骤只覆盖 default 与 `tui`，常规 CI 绿灯不覆盖该 feature；该 feature 引入 `zstd` 等 native 编译，Windows MSVC 与默认 Linux 构建的编译结果可能不同；
- exact behavior to verify：以 `--features simplified-trad-conversion` 构建 Tauri 应用与 TUI；在 GUI 中确认 capability 为 true、`t2s` / `s2t` 选项可用且输出正确；确认默认构建仍 capability 为 false 且选项禁用；
- prerequisite：Windows 原生 checkout，可用的 Rust MSVC 工具链；已执行启用该 feature 的 `cargo test`；
- expected result：feature 构建编译通过且转换输出正确；默认构建行为不变；不出现 native 链接或运行时错误；
- 备注：需先在 Linux 侧为该 feature 建立可执行的验证入口，否则本项无法在 Windows 上给出有效结论。

#### Q6 — Tauri 运行时与 CLI 补丁升级（#85、#79）

- related change：开放 PR #85（`@tauri-apps/cli` 2.11.4 → 2.11.5）、#79（Rust `tauri` 2.11.5 → 2.11.6）；
- relevant files / modules：`frontend/package-lock.json`、`src-tauri/Cargo.lock`、Tauri 打包配置；
- why Windows validation is required：Tauri 打包、MSVC 构建、WebView2 宿主行为与前端单测覆盖范围不同，补丁版本也可能在 Windows 侧引入差异；
- exact behavior to verify：Windows 原生 `cargo build` 与 Tauri 打包成功；应用启动、WebView2 加载、窗口控制与剪贴板行为正常；
- prerequisite：Windows 原生 checkout 与 MSVC 工具链；
- expected result：构建与运行行为与升级前一致。

#### Q7 — 发布链路 Actions 跨主版本升级（#69、#67、#65、#64）

- related change：开放 PR #69（upload-artifact 4.6.2 → 7.0.1）、#67（download-artifact 4.3.0 → 8.0.1）、#65（checkout 4.4.0 → 7.0.1）、#64（setup-node 4.4.0 → 7.0.0）；
- relevant files / modules：`.github/workflows/ci.yml`、`.github/workflows/release.yml`；
- why Windows validation is required：这四个 PR 修改发布工作流，普通 PR CI 只覆盖 `ci.yml`，不执行 `release.yml` 的资产上传、跨平台下载与汇总；跨主版本升级的 artifact 行为差异只能在真实发布流程中暴露；
- exact behavior to verify：Linux 与 Windows 资产分别上传成功；Windows smoke 步骤能下载对应资产；汇总步骤能取得完整发布 artifact 且哈希校验通过；
- prerequisite：候选 tag 与发布流程执行权限；
- expected result：上传、下载、汇总链路全部成功，资产完整；
- 备注：这四个 PR 当前 CI 的 Rust 失败由既有 `cli.rs` rustfmt 问题引起（已由 #77 在 `dev` 修复），**不能据此判断 Actions 升级本身有问题**；需更新基线并重跑 CI 后再评估，且必须验证 `release.yml` 实际执行。

### 4.3 集中验证计划（Windows Validation Preparation 输出）

本轮（2026-09-15）变更影响评估：final diff 只包含文档（`AGENTS.md`、`docs/**`、`docs/validation/**`），没有应用代码、构建配置或依赖变化。

| 类别 | 本轮验证项 | 说明 |
| --- | --- | --- |
| Build / Toolchain | 无新增 | 未改动 `src-tauri/Cargo.toml`、`package.json`、工具链或 CI 配置 |
| Runtime | Q1（`--preset` CLI）、Q3（启动 smoke） | Q1 来自 `6e8c034`；Q3 属发布前置门禁 |
| Filesystem | Q1 的输入 / 输出路径行为 | 覆盖 Windows 盘符路径与 Unicode 文件名 |
| Integration | 无新增 | 本轮无 IPC / 外部服务变更 |
| Packaging | Q3 | 便携版 `CopyPolish-windows-x64.7z` 与 `CopyPolish-tui-windows-x64.7z` |
| Regression | Q2（Windows Terminal 交互）；W2–W4（GUI E2E 门禁）本轮不触发 | GUI E2E 仅在应用代码变更时复跑，本轮为纯文档改动 |

合并说明：Q1 与 Q2 合并为同一次 Windows Terminal 会话；未为单个功能分别安排 Windows 启动。

Handoff 字段模板（每个测试项）：ID、test name、purpose、related changes、prerequisites、steps / command、expected result、priority、是否需要人工交互。

### 4.4 不需要 Windows 验证的项（判定记录）

| 项 | 判定 | 依据 |
| --- | --- | --- |
| `scripts/evaluate_corpus.py` 语料评测框架（`d4d20dc`） | `NOT_APPLICABLE` | 开发期本地 Python 工具，不随应用或 TUI 资产发布；规范见 `docs/source-corpus-spec.md`、`docs/roadmap.md` |
| 本轮文档改动（`AGENTS.md`、`docs/**`） | `NOT_APPLICABLE` | 纯文档，无平台运行时行为；Linux 侧由 `python3 scripts/verify.py --profile checks` 覆盖 |

### 4.5 Computer Use 不可用时的处理

Computer Use / GUI automation 不可用导致的无法执行**不是功能失败**：标记 `BLOCKED` 并记录 `Blocker: COMPUTER_USE_UNAVAILABLE`，不得记为 `FAIL` / `PASS` / `NOT_APPLICABLE`。处理规则见 [../development/cross-platform-validation.md](../development/cross-platform-validation.md) §17。要点：

- 疑似偶发故障允许**重试一次**，仍不可用即 `BLOCKED`，不长时间重复同一操作；
- 出现 `BLOCKED` 后重新检查依赖关系，继续执行不依赖 GUI automation 的项（build、`cargo test`、CLI、filesystem、配置、日志、进程/端口检查等）；
- 原目标可由可靠的非 GUI 方法等价验证时才可记为 `PASS`，否则保留 `BLOCKED` 并进入 §4.6；
- 不得因为一个 GUI 测试被阻塞就中止整个 Windows validation phase。

### 4.6 Manual Windows Validation Queue

在 Windows 自动验证结束后生成，收纳因 Computer Use unavailable、GUI-only workflow、required manual interaction 或不可用自动化能力而未完成的项。字段结构见 [../development/cross-platform-validation.md](../development/cross-platform-validation.md) §17.6。

| Test ID | Test name | Current status | Blocker | 来源队列项 |
| --- | --- | --- | --- | --- |
| `WIN-MANUAL-001` | Windows Terminal TUI 交互复验 | `BLOCKED`（待人工执行，用户反馈前不得记为 `PASS`） | `MANUAL_INTERACTION_REQUIRED`（raw-mode / OSC 52 / 终端状态清理无法自动化） | Q2 |
| `WIN-MANUAL-002` | 发布版 GUI 启动与剪贴板 smoke | `BLOCKED`（待人工执行） | `MANUAL_INTERACTION_REQUIRED` | Q3 |
| `WIN-MANUAL-003` | 真实 PDF/CAJ 来源文本人工验收（按需） | `BLOCKED`（待人工执行） | `MANUAL_INTERACTION_REQUIRED` + 前置条件缺失（尚无许可或脱敏语料） | 按需验收 |
| `WIN-MANUAL-004` | GUI 中文布局与排版人工复核 | `WINDOWS_VERIFICATION_PENDING`（待集中验证阶段执行） | `MANUAL_INTERACTION_REQUIRED`（默认/最小窗口、DPI、窗口控制、真实剪贴板） | Q4 |

#### WIN-MANUAL-001 — Windows Terminal TUI 交互复验

- **Purpose**：证明 Windows 原生 Windows Terminal + PowerShell 7 下的交互行为（raw-mode、规则面板、快捷键、Unicode grapheme、bracketed paste、复制 / OSC 52、保存与重启恢复、终端状态清理）在当前 binary 上成立。
- **Related change**：Q2（`docs/roadmap.md` P0「依赖与安全维护」条目；`6e8c034` 的 TUI/CLI 改动）。
- **Preconditions**：Windows 原生 checkout 已同步至待验证 revision；Windows Terminal 与 PowerShell 7 可用；已构建 `src-tauri\target\release\copypolish-tui.exe`；使用独立临时设置目录，不污染真实用户设置。
- **Manual test steps**：
  1. 在 PowerShell 7 中构建：`cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui`；
  2. 建立临时设置目录：`$settingsDir = Join-Path $env:TEMP ("copypolish-tui-" + [guid]::NewGuid()); New-Item -ItemType Directory -Path $settingsDir | Out-Null; $env:COPYPOLISH_E2E_SETTINGS_DIR = $settingsDir`；
  3. 在 **Windows Terminal**（不是 conhost）中启动：`& .\src-tauri\target\release\copypolish-tui.exe`；
  4. 在输入区逐个输入裸 `r`、`q`、`?`，确认字符进入正文，而不是误触规则、退出或帮助；
  5. 用方向键、Home、End、Delete 在 ASCII、中文、emoji 与多行文本上移动和编辑，重点确认最后一个 grapheme 的 Right/Delete 边界；
  6. `Ctrl+R` 打开规则面板，确认默认启用规则在默认关闭规则之前且同组顺序稳定；用 Space、`a`、`d`、`n` 切换，Esc 返回正文；
  7. `Ctrl+?` 打开帮助并核对说明；`Ctrl+Q` 退出，确认终端恢复普通输入；
  8. 执行 `Set-Clipboard -Value "第一行 r q ?`n第二行 中文🙂é"`，在 TUI 中粘贴，确认文本完整插入、换行正确、未误触快捷键；
  9. 在输出区/规则区导航并复制，用 `Get-Clipboard` 检查结果，记录 OSC 52 成功或被拦截时的状态栏提示；
  10. `Ctrl+S` 保存规则与最近输入，检查设置目录中的 `rules.yaml`、备份与 metadata；用同一目录重启，确认规则与最近输入恢复；
  11. 检查无残留进程、无监听端口、终端模式已恢复、无未预期控制序列。
- **Expected result**：所有步骤符合 `docs/windows-e2e-runbook.md` §4.4 与 §4.5 的预期；emoji 与组合字符不出现半个代理字符、占位错位或残片；OSC 52 不可用时应用给出使用 `--stdin/--output` 的提示。
- **Failure evidence to collect**：截图、输入按键序列、`stdout.txt` / `stderr.txt`、`rules.yaml` 前后副本、Windows Event Viewer 条目（如崩溃）、复现步骤。
- **Result**：`PASS` / `FAIL` / `BLOCKED`；**Notes**：___；**Error**：___；**Evidence**：___。

#### WIN-MANUAL-002 — 发布版 GUI 启动与剪贴板 smoke

- **Purpose**：证明发布资产在 Windows 原生可启动，并完成真实格式化、设置保存与重启恢复、剪贴板和退出检查。
- **Related change**：Q3（`docs/release/manual-release.md` §6.1、`docs/testing.md` §2.2 门禁 6）。
- **Preconditions**：存在发布候选 tag 与资产（`CopyPolish.exe`、`CopyPolish-windows-x64.7z`）；WebView2 Evergreen Runtime 可用（Windows 10/11 通常内置）；准备一个可写目录，或接受设置回退到 `%APPDATA%\CopyPolish`。
- **Manual test steps**：
  1. 将 `CopyPolish-windows-x64.7z` 解压到空目录，确认压缩包根目录直接包含 `CopyPolish.exe`；
  2. 双击 `CopyPolish.exe`（或在 PowerShell 中执行 `.\CopyPolish.exe`）；
  3. 等待主窗口出现，确认无 crash、无空白页；
  4. 在输入区粘贴 `在LeanCloud上，花了5000元`，确认输出按规则完成排版；
  5. 点击复制结果，在记事本中粘贴，确认内容与输出一致；
  6. 打开设置窗口，修改一项（主题或某个规则开关）并保存；
  7. 完全退出应用后重新启动，确认设置恢复为保存前状态；
  8. 检查程序目录：目录可写时应出现 `rules.yaml`；不可写时应回退到 `%APPDATA%\CopyPolish` 并在界面提示；
  9. 确认程序目录未混入测试 fixture、`node_modules`、日志或 E2E artifact。
- **Expected result**：启动正常、格式化与复制正确、设置保存与重启恢复正确、退出干净、程序目录无多余文件。
- **Failure evidence to collect**：截图、精确错误文本、`rules.yaml` 或 `%APPDATA%\CopyPolish` 内容、Windows Event Viewer 条目、复现步骤。
- **Result**：`PASS` / `FAIL` / `BLOCKED`；**Notes**：___；**Error**：___；**Evidence**：___。

#### WIN-MANUAL-003 — 真实 PDF/CAJ 来源文本人工验收（按需）

- **Purpose**：为 `cleanup.cjk-internal-space` 建立真实来源证据，并评估未来段内软换行规则。
- **Related change**：`docs/roadmap.md` 中 PDF/CAJ Spike 未完成项、`docs/acceptance/pdf-caj/`。
- **Preconditions**：取得许可或完成脱敏的真实 PDF/CAJ 文本；原件保存在 Windows 本地受控目录（建议 `E:\CopyPolish-private\pdf-caj\`）；准备好 `manifest.yaml` 与 `annotation-template.md`。
- **Manual test steps**：按 `docs/acceptance/pdf-caj/README.md` 的「操作步骤」1–6 执行：保存原件并记录许可 / hash / 来源类型 / 是否多栏 / 提取工具与日期 → 用 PDF 阅读器或 CAJViewer 复制文本（不让 CopyPolish 读取原件本体）→ 脱敏后保存为本地 `.txt` 并建立 manifest 记录 → 人工标注每个候选行边界为 `merge` / `keep` / `unknown`，并标注 CJK 空格应删除或保留及理由 → 在 E 盘 Windows checkout 通过 GUI/TUI/CLI 输入脱敏文本，记录规则关闭与开启的输出、差异、幂等性、换行风格和误删 / 漏删 → 只把脱敏后的最小片段与标注摘要加入评审包。
- **Expected result**：满足 `docs/acceptance/pdf-caj/README.md` 的验收门槛（每条样本有许可 / 脱敏说明与 hash；三类边界均覆盖；保护 Markdown、代码、URL、LaTeX、HTML、表格、公式与多栏顺序；报告 precision / recall、误删率、漏合并率与无法判断比例；输出幂等并保留 LF/CRLF）。
- **Failure evidence to collect**：脱敏后的输入 / 输出摘要、误删或漏改清单、规则开关差异、日志路径；原件、截图与临时 artifact 不入库。
- **Result**：`PASS` / `FAIL` / `BLOCKED`；**Notes**：___；**Error**：___；**Evidence**：___。

#### WIN-MANUAL-004 — GUI 中文布局与排版人工复核

- **Purpose**：证明本次中文重新布局在 Windows 原生 WebView2 下可用：默认/最小窗口布局、实时/手动模式、设置分类、主题/缩放、窗口控制与真实剪贴板。
- **Related change**：Q4（`docs/roadmap.md` P1「GUI 中文布局与排版」）。
- **Preconditions**：Windows 原生 checkout 已同步至待验证 revision；WebView2 Evergreen Runtime 可用；已执行 `npm run build:app --prefix e2e` 构建当前 GUI。
- **Manual test steps**：
  1. 启动 `CopyPolish.exe`，分别在默认窗口 920×720 与最小窗口 800×600 下确认主界面为左右对照可用布局；
  2. 缩小窗口或使用窄屏，确认输入/输出按自动布局上下堆叠且无内容被遮挡；
  3. 输入 `在LeanCloud上，花了5000元`，确认实时模式自动排版，切换手动模式后不再自动刷新并出现“立即排版”动作；
  4. 打开设置窗口，逐个切换六个分类（排版规则、替换与转换、编辑与输出、外观显示、快捷键、隐私与存储），确认内容可达且键盘焦点可见；
  5. 切换浅色/深色主题与 80%–125% 界面缩放，确认中文说明换行自然、按钮标签不拆行、长路径仍可悬停/复制；
  6. 用顶部标题栏拖动窗口，依次使用最小化、最大化/还原、关闭按钮，确认行为正常；
  7. 点击复制结果后在记事本粘贴，确认内容一致；再验证复制并清空与清空输入的原有语义保持。
- **Expected result**：布局、模式、设置、主题/缩放、窗口控制与复制语义均符合预期；中文无溢出、无遮挡、无重复反馈文案。
- **Failure evidence to collect**：截图、精确错误文本、窗口尺寸/DPI 设置、`rules.yaml` 相关内容（如涉及）、Windows Event Viewer 条目、复现步骤。
- **Result**：`PASS` / `FAIL` / `BLOCKED`；**Notes**：___；**Error**：___；**Evidence**：___。

### 4.7 阶段结束汇总（四组）

Windows 验证阶段结束时按以下四组汇报，Manual validation required 不得只给名称：

| 组 | 含义 | 当前（截至 2026-09-15） |
| --- | --- | --- |
| Automated PASS | 自动完成并通过的测试 | 无（本轮未执行 Windows 验证） |
| Automated FAIL | 实际执行但发现项目问题 | 无 |
| `BLOCKED` | 因 Computer Use 或其他环境能力问题无法得出结论 | 无（本轮未进入 Windows 阶段） |
| Manual validation required | 需要用户人工执行的测试 | `WIN-MANUAL-001` ~ `WIN-MANUAL-004`（见 §4.6，含可直接执行的步骤） |

在用户实际执行人工测试并反馈结果前，Manual validation required 项目不得标记为 `PASS`。

## 5. 验证记录

每次 Windows 验证在本文追加一个小节，格式固定为 Validation Environment / Validation Results / Errors / Not Executed / Blocked。

### 5.1 2026-09-15 — 流程与记录文档落地（未执行 Windows 验证）

本轮任务是把 Windows 平台验证工作流写入 `AGENTS.md` 与本文，未在 Windows 环境执行任何验证；以下状态是如实记录，不代表验证通过。

#### Validation Environment

| 项 | 值 |
| --- | --- |
| 验证日期 | 2026-09-15（未执行验证，仅落地流程与记录结构） |
| Windows version | `unknown`：本轮未进入 Windows 环境，未采集 |
| Architecture | `unknown`：同上 |
| Runtime / toolchain 版本 | `unknown`：同上（项目要求见 testing.md §7.0-1） |
| Linux source branch | `dev` |
| Linux source commit | `d4d20dc0e63cd222311100591c68c9b1fde49a4f`（2026-09-07） |
| 是否包含未提交修改 | 是（working tree 含 `docs/README.md`、`docs/testing.md`、`docs/roadmap.md`、`docs/source-corpus-spec.md`、`docs/decisions/pdf-soft-wrap-spike.md`、`docs/acceptance/pdf-caj/*`，以及本次新增的 `AGENTS.md`、`docs/development/cross-platform-validation.md`、`docs/validation/windows.md`） |
| Windows 工作副本位置 | `E:\Shiraishi\VSCode Workspace\chinese_copywriting_formatter` |
| Windows 工作副本状态 | `dev` 分支 checkout，HEAD `ac92636`；存在大量本地未提交改动；本轮未同步 |
| 本轮 artifact / 日志路径 | 无（未执行） |

#### Validation Results

| ID | 验证项目 | 状态 | 执行命令 | 简要结果 |
| --- | --- | --- | --- | --- |
| W1 | 依赖安装与 E2E 类型检查 | `NOT RUN` | — | 本轮任务范围为文档落地，未在 Windows 安装依赖 |
| W2 | 默认 embedded GUI 完整回归 | `NOT RUN` | — | 同上 |
| W3 | 简繁 feature GUI | `NOT RUN` | — | 同上 |
| W4 | 标准 W3C provider smoke | `NOT RUN` | — | 同上 |
| W5 | Windows MSVC Rust/TUI 测试 | `NOT RUN` | — | 同上 |
| W6 | 设置故障、ACL、artifact 与 transcript | `NOT RUN` | — | 同上 |
| W7 | 发布资产启动 smoke | `NOT RUN` | — | 本轮无待发布资产 |
| W8 | 发布资产校验 | `NOT RUN` | — | 本轮无待发布资产 |

#### Errors

本轮无新增错误（未执行验证）。历史失败记录不在此重复，见 §5.2。

#### Not Executed / Blocked

| 项目 | 原因类别 | 说明 |
| --- | --- | --- |
| W1–W6 | 任务范围 | 本轮请求是“把验证规则写入文档”，未要求也未触发新一轮 Windows 验证；执行它们需要进入 Windows 原生会话 |
| W7–W8 | 前置条件缺失 | 没有待发布的 tag / 候选资产（insufficient environment capability / no release candidate） |
| 同步动作本身 | 任务范围 | 本轮未执行 Linux → Windows 同步；目标目录当前含大量本地未提交改动，直接同步会覆盖人工状态，必须先在 Windows 侧确认后再按 `AGENTS.md` §5.7 执行 |

### 5.2 既往 Windows 证据（非本轮结果，供追溯）

以下记录产生于本流程落地之前，保留原始描述与来源，不套用 §2 状态标记，也不能替代新一轮验证：

| 时间 | 内容 | 来源 |
| --- | --- | --- |
| 2026-09-07 | 用户确认当前维护版本的 Windows 平台验证完成，覆盖默认 embedded、简繁 feature、W3C smoke、设置恢复/损坏/ACL/reparse point、MSVC/TUI、Windows Terminal 交互和发布前 Windows smoke。未提供机器版本、命令计数或 artifact 路径，因此只作为**用户确认**记录，不构成可复现证据。 | windows-e2e-runbook.md §1.1 / §1.2、testing.md §2.2 |
| 2026-09-06 | v0.6.2 S2-B 设置存储加固（reparse point/junction 拒绝、跨进程并发保存、唯一临时文件与失败清理）由用户在 Windows 原生环境手动验证通过；同样只有用户确认，无 artifact 路径。 | windows-e2e-runbook.md §1.2.2 |
| 2026-09-04 | 最近一次带命令计数与 artifact 描述的 Windows 原生复验：前端 101/101、E2E typecheck 通过、默认 embedded 3/3、简繁 feature 2/2（s2t/t2s）、W3C smoke 2/2、Windows MSVC `cargo test --features tui` 182+5+3、损坏设置 3/3、NTFS ACL 1/1、GUI 视觉 artifact 1/1、设置快捷键控制台 1/1、TUI transcript 4/4；GUI DPI 自动矩阵与 GitLab Windows stage 跳过。 | windows-e2e-runbook.md §13 / §14、archive/validation/windows-2026-09.md |
| 2026-09-04 | 历史失败与闭环：`restart-settings.spec.ts:53` 旧 selector 强制等待 `t2s` 导致失败，spec 已按 `simplifiedTradConversion` capability 分支修正，默认与 feature 的 write/read 各 2/2；W3C smoke 首次瞬时失败未能复现，单独重跑 2/2 通过。 | windows-e2e-runbook.md §13 / §14 |

## 6. Final Review 检查项

每次记录完成后逐项确认：

1. 小节包含 Validation Environment / Validation Results / Errors / Not Executed / Blocked 四段；
2. 所有适用清单项都有状态，且状态取自 §2；
3. `FAIL` / `BLOCKED` / `NOT RUN` 都写明了原因；
4. 没有把失败项、`finished=0` 或未执行项记为 `PASS`；
5. Windows 专属项（WebView2、MSVC、NTFS ACL/reparse point、DPI、Windows Terminal、剪贴板、Windows 进程）没有被 Linux/WSL 结果替代；
6. Linux 侧 Git diff 只包含预期文档改动，无业务代码修改；
7. artifact 与日志路径已记录，测试产物已按 `python3 scripts/clean.py --generated` 清理；
8. 没有写入未经采集的机器版本、命令计数或 artifact 路径（无法获得时写 `unknown` 并说明原因）；
9. Windows Validation Queue 与集中验证计划（§4）已反映本轮变更，并已合并高度相关的验证项；
10. 因 Computer Use / GUI automation 不可用而 `BLOCKED` 的项已记录 blocker、是否重试、是否存在非 GUI 替代验证与手动步骤（§4.5 / §4.6），且在用户反馈前未被标记为 `PASS`。

## 7. 维护规则

- 每次 Windows 验证只**追加**一个新的 §4 小节，不重写历史小节；失去追溯价值的历史内容迁入 `docs/archive/validation/`；
- 不复制 Runbook 的命令细节与完整日志，只保留关键错误、相关 stack trace 摘要、日志文件路径和必要上下文；
- 不在本文固定易失真的 “latest”、Pipeline ID、Release ID 等状态；
- 当项目的 Windows 门禁发生变化时，同步更新本文 §3、[../testing.md](../testing.md) §2.2 和 [../windows-e2e-runbook.md](../windows-e2e-runbook.md)，保持单一事实来源；
- 本文的职责登记在 [../README.md](../README.md) 的文档职责表中。