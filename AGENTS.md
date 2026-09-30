# AGENTS.md — CopyPolish 代理工作约定

本文件是 AI 代理与自动化工具在本仓库工作的入口约定，只描述**流程、边界和约束**，不重复记录技术事实。

- 当前技术事实：[README.md](README.md)、[docs/architecture.md](docs/architecture.md)、[docs/development.md](docs/development.md)；
- 测试策略与平台门禁：[docs/testing.md](docs/testing.md)；
- 未完成工作：[docs/roadmap.md](docs/roadmap.md)（待办只写在这里）；
- Windows 平台验证流程见本文件第 5 节，结果记录见 [docs/validation/windows.md](docs/validation/windows.md)。

## 1. 事实来源优先级

判断“项目当前要求什么”时，按以下顺序取信；禁止凭空创造项目中不存在的命令、门槛、资产或平台能力：

1. 本文件与项目文档（`AGENTS.md`、`docs/`、`CONTRIBUTING.md`）；
2. CI / 构建 / 测试配置（`.github/workflows/*.yml`、`.gitlab-ci.yml`、`scripts/verify.py`）；
3. `package.json`、`src-tauri/Cargo.toml`、`scripts/*` 等脚本与配置；
4. 当前代码实现。

## 2. 项目速览

- 技术栈：Tauri 2 + React 19 + TypeScript + Vite + Tailwind v4 + Rust 2021；
- 目录：`frontend/`（React 前端）、`src-tauri/`（Rust/Tauri，排版引擎在 `src-tauri/src/engine/`）、`e2e/`（WebdriverIO E2E）、`scripts/`（验证、清理与发布脚本）、`docs/`（维护者文档）；
- 工具链固定：Node 见 `.nvmrc`，Rust 见 `rust-toolchain.toml`（含 `rustfmt` / `clippy`）；
- 前端只能通过 `frontend/src/lib/tauri.ts` 访问 Tauri command，不在组件或 hook 中直接 `invoke`；
- Rust `engine` 是格式化行为的唯一事实来源，TUI 只做交互状态，不复制规则实现；
- 设置读写测试使用系统临时目录，不写入仓库根目录的 `rules.yaml`；
- 不提交可再生目录与本地状态：`frontend/node_modules/`、`frontend/dist/`、`e2e/node_modules/`、`e2e/artifacts/`、`src-tauri/target/`、`src-tauri/gen/`、明文凭据与用户设置。

## 3. 常用验证入口

| 目的 | 命令 |
| --- | --- |
| 文档 / 密钥 / diff 检查 | `python3 scripts/verify.py --profile checks` |
| 前端依赖 + Vitest + 构建 | `python3 scripts/verify.py --profile frontend` |
| Rust fmt/clippy/test + TUI + 性能门禁 | `python3 scripts/verify.py --profile rust` |
| 可选 feature（简繁转换）clippy/test/build | `python3 scripts/verify.py --profile feature` |
| 依赖安全审计 | `python3 scripts/verify.py --profile audit` |
| 与常规 CI 对齐 | `python3 scripts/verify.py --profile ci` |
| E2E 类型检查 | `npm run typecheck --prefix e2e` |
| 生成物与测试产物清理 | `python3 scripts/clean.py --generated` |

测试产物、截图、日志和临时设置目录在结果写入文档或 `CHANGELOG.md` 后清理，不入库、不上传远程。

## 4. 平台边界

Linux/WSL 可以完成前端、Rust、E2E typecheck 与业务语义验证，但不能替代 Windows 原生验收：WebView2、MSVC/Windows SDK 构建、NTFS ACL 与 reparse point、桌面 DPI、剪贴板、Windows Terminal raw-mode/OSC 52 以及 Windows 进程清理必须由 Windows 原生环境验证。完整矩阵见 [docs/testing.md](docs/testing.md) §2.2 与 [docs/windows-e2e-runbook.md](docs/windows-e2e-runbook.md)。

平台验证不是代码开发任务的默认步骤：仅在项目文档、变更范围或发布流程要求时执行，并按第 5 节流程留下记录。

## 5. 跨平台开发与 Windows 验证工作流

本项目主要在 Linux 环境开发；Windows 环境用于平台相关的构建、运行、打包和兼容性验证。本节是代理必须遵守的规范；规则正文与本项目落地参数见 [docs/development/cross-platform-validation.md](docs/development/cross-platform-validation.md)，验证结果记录见 [docs/validation/windows.md](docs/validation/windows.md)，命令级步骤、通过条件、artifact 与清理要求见 [docs/windows-e2e-runbook.md](docs/windows-e2e-runbook.md)。本节规定平台验证的规范；**开发阶段的执行方式（批量开发、集中验证）见第 6 节**。

> **核心原则（规则 §15）**：始终区分三个不同事实 —— `Code implemented`、`Linux verified`、`Windows verified`。只有在 Windows 平台**实际执行并通过**相应验证后，才能声明 `Windows verified`；不得把“Linux 已修复”视为“Windows 已验证通过”。

### 5.1 Purpose（规则 §1）

本项目主要在 Linux 环境进行开发。Windows 环境主要用于：

- Windows-specific build verification；
- Windows runtime verification；
- Windows packaging / installer verification；
- filesystem / path / process / sidecar 等平台相关验证；
- 项目文档定义的其他 Windows 验证。

Linux 项目目录是主要开发工作区和项目事实来源；Windows 项目目录是验证工作副本。

### 5.2 Source of Truth（规则 §2）

Linux 项目目录是 source code、project configuration、project documentation、implementation state、Plan / task state、validation documentation 的主要事实来源。

- Windows 工作副本主要用于验证，不作为主要开发源；
- 代码同步方向默认必须为 Linux → Windows；
- 除验证文档结果外，不应将 Windows 工作副本中的代码修改自动反向同步到 Linux。

### 5.3 Development / Validation Cycle（规则 §3）

标准工作流：

1. Linux implementation；
2. Linux verification；
3. Mark Windows verification requirements；
4. Sync Linux project to Windows；
5. Windows validation；
6. Write Windows validation results back to Linux documentation；
7. Linux reads and reconciles Windows results；
8. Continue implementation or fixes；
9. Repeat Windows validation when required。

不得将“Linux 已修复”视为“Windows 已验证通过”。

### 5.4 Validation State Model（规则 §4）

平台相关任务应尽可能使用以下状态：

| 状态 | 含义 |
| --- | --- |
| `IMPLEMENTED` | 代码已实现，尚未在 Linux 完成验证 |
| `LINUX_VERIFIED` | 已在 Linux 平台完成适用验证 |
| `WINDOWS_VERIFICATION_PENDING` | 等待下一轮 Windows 验证 |
| `WINDOWS_PASS` | 已在 Windows 实际执行并通过 |
| `WINDOWS_FAIL` | 已在 Windows 实际执行但失败 |
| `WINDOWS_BLOCKED` | 因前置条件缺失无法在 Windows 验证 |
| `NOT_RUN` | 本轮未执行，但理论上应执行 |
| `NOT_APPLICABLE` | 根据当前项目或平台状态明确不适用 |

典型状态流：`IMPLEMENTED` → `LINUX_VERIFIED` → `WINDOWS_VERIFICATION_PENDING` → `WINDOWS_PASS`。

若 Windows 验证失败：`WINDOWS_FAIL` → Linux fix → `LINUX_VERIFIED` → `WINDOWS_VERIFICATION_PENDING` → Windows re-validation。

Linux 端不得在 Windows 实际重新验证之前，将修复后的项目标记为 `WINDOWS_PASS`。

### 5.5 Linux Development Responsibilities（规则 §5）

Linux 侧负责：阅读当前仓库和项目文档；执行主要开发工作；执行 Linux 平台适用的验证；分析 Windows 验证结果；修复真正属于项目代码的问题；更新 Plan 和项目文档；标记需要下一轮 Windows 验证的内容。

Linux 侧不应：假装执行了 Windows-only 验证；将 Linux 测试成功等同于 Windows 验证成功；为纯 Windows 环境配置问题修改项目代码；无依据扩大当前任务范围。

### 5.6 Windows Validation Responsibilities（规则 §6）

Windows 侧负责：读取 Linux 项目及项目文档；将需要验证的内容从 Linux 单向同步至 Windows 工作副本；根据仓库、文档、构建配置和脚本确定验证范围；执行适用的 Windows 验证；记录 `PASS` / `FAIL` / `BLOCKED` / `NOT RUN` / `NOT APPLICABLE`；分析失败原因；将实际验证结果写回 Linux 项目的验证文档。

Windows 侧默认执行验证，而不是功能开发。除机器本地配置、构建产物或验证所需的临时变化外，不应为了让验证通过而自行修改业务代码。若发现代码问题，应记录 failure、reproduction、likely root cause、relevant code location、suggested follow-up，并交由 Linux 开发阶段处理。

### 5.7 Windows Synchronization Rules（规则 §7；对应原 Phase 1–3）

**同步前检查**：在执行任何同步或验证之前先检查 Linux 项目，自动确定 Git branch / commit / working tree 状态、项目目录结构、语言/框架/构建系统、Windows 相关文档与脚本、测试/lint/typecheck/build/package 命令、已有平台兼容性要求、已有验证文档及其格式、`AGENTS.md` 或其他 project instructions，以及 Windows 验证依赖的软件、环境变量、外部服务和工具。事实来源优先级按第 1 节执行，不要凭空创造项目不存在的验证要求。

同时检查 Linux branch / commit / working tree、Windows target directory，以及是否存在 Windows 本地需要保留的配置；不要无条件删除未知文件，不要覆盖属于 Windows 本地配置、凭据、缓存或机器特定设置的内容。

**同步与核对**：代码同步方向必须为 Linux → Windows 单向。同步后至少确认：关键源代码已经更新；Windows 工作副本对应当前需要验证的 Linux 项目状态；没有因路径分隔符、大小写、符号链接或权限问题造成明显缺失；Windows 本地专用配置未被意外覆盖。

**默认不要跨平台同步**：`.git`（除非验证流程需要）、`node_modules`、Python virtual environments、Rust `target`、build caches、IDE caches、temporary files、secrets、machine-specific configuration；判定优先依据 `.gitignore`、项目文档、构建配置和已有同步脚本。

**优先使用项目已有的** sync scripts、checkout procedures、worktree workflow、build preparation scripts，而不是自行实现另一套同步逻辑。

如果不能获得可靠的 commit / hash，或 Linux 侧包含未提交改动，必须记录“包含 working tree changes”，不得假装对应一个纯 Git commit。

### 5.8 Windows Validation Scope（规则 §6 / §13；对应原 Phase 4）

读取项目文档和仓库配置，自行确定本次 Windows 平台应执行的验证项目。候选范围包括但不限于：依赖/环境准备、代码生成、formatter check、lint、typecheck、单元测试、集成测试、Windows 专属测试、build、打包、应用启动、CLI 行为、文件系统/路径行为、子进程/sidecar 行为、网络/服务集成、安装器/包校验。

只执行与当前项目实际相关的项目，并在开始验证前形成内部验证清单，区分 Required（项目文档明确要求）、Applicable（根据当前项目配置应执行）、Not applicable（当前 Windows 环境或项目不适用）。本项目现成的清单与命令映射见 [docs/validation/windows.md](docs/validation/windows.md) 第 3 节和 [docs/testing.md](docs/testing.md) §2.2。

### 5.9 Windows Validation Result Classification（规则 §8；对应原 Phase 5）

在 Windows 工作副本中执行验证。原则：使用项目已有命令和脚本（优先 `scripts/verify.py` 与 `e2e/package.json` 中已定义的 runner）；优先使用项目规定的软件包管理器和工具链（`.nvmrc`、`rust-toolchain.toml`）；不因为命令失败就随意更换工具或改变项目配置；不为了“让测试通过”而修改功能代码；不跳过失败项目并将其标记为成功。

每个验证项目至少使用以下结果之一：

| 状态 | 判定 | 必须记录 |
| --- | --- | --- |
| `PASS` | 验证实际执行，并满足预期 | 命令、结果摘要；有 artifact 时记录路径 |
| `FAIL` | 验证实际执行，但项目行为或输出不符合要求 | command、failure point、relevant error、likely cause、whether Windows-specific、follow-up recommendation |
| `BLOCKED` | 由于前置条件缺失无法继续（依赖/证书/外部服务/凭据不可用、前置 test/build 失败、环境不受支持） | 阻塞原因与依赖关系 |
| `NOT RUN` | 本轮未执行，但理论上应执行 | 未执行原因 |
| `NOT APPLICABLE` | 根据当前项目或平台状态明确不适用 | 判定依据 |

对每一个验证项目记录：验证项目名称、实际执行命令、工作目录、结果、必要时记录关键版本信息、关键输出或错误摘要。

如果某项验证失败：保存关键错误信息；分析最可能的原因；判断它属于 Windows 平台兼容性问题、项目代码问题、环境配置问题、缺失依赖、外部服务问题、测试自身问题，还是暂时无法确定；在合理情况下继续执行不依赖该失败项的其他验证。不要因为一个非致命验证项目失败就自动停止全部验证；依赖失败项的后续验证标记为 `BLOCKED` 并说明依赖关系。本项目历史教训：`exitCode=0` 但 `finished=0` 的 runner 必须记为未完成，不得记为通过。

### 5.10 Modification Policy（规则 §6 / §10）

本流程的主要目的为验证，而不是开发。除非项目文档明确说明某些 Windows 本地配置需要生成或修改，否则：

- 不修改业务代码；
- 不修改 Linux 项目的功能实现；
- 不为了通过测试自行修复代码；
- 不进行与验证无关的重构；
- 不升级依赖；
- 不改变项目架构。

可以在 Windows 工作副本产生正常的 build artifacts、dependency caches、test artifacts、logs、temporary files 和 machine-local configuration。如果发现需要代码修改才能解决 Windows 问题：不直接将该修改作为本次验证任务的一部分实施，而在验证文档中记录问题、可能根因和建议修复位置，交由 Linux 开发阶段处理。

### 5.11 Reconciliation of Windows Results on Linux（规则 §9）

当 Windows 验证结果写回 Linux 项目后，Linux 侧必须先重新读取：`git diff`、modified documentation、current Plan、validation result documents、相关的 `AGENTS.md` 指令，然后重新评估当前 Plan，并区分：

- 已经 Windows 验证完成的事项；
- Windows 失败且属于代码问题的事项；
- 环境问题；
- `WINDOWS_BLOCKED` 项目；
- 尚未执行（`NOT_RUN`）项目；
- 已不再适用（`NOT_APPLICABLE`）的项目；
- 需要下一轮 Windows re-validation 的项目。

不得机械继续旧 Plan；当前仓库和最新验证结果优先于旧任务假设。

### 5.12 Handling Windows Failures（规则 §10）

Windows `FAIL` 后，Linux 侧首先判断问题属于：project code、Windows compatibility、dependency、environment、credential、external service、test infrastructure、unknown。只有真正属于项目代码或必要兼容性问题时才修改代码。

修复后：

1. 执行 Linux 适用的 regression tests；
2. 执行相关 Linux verification；
3. 更新文档；
4. 将 Windows 状态改为 `WINDOWS_VERIFICATION_PENDING`，而不是 `WINDOWS_PASS`；
5. 记录下一轮 Windows 应重新执行哪些验证。

### 5.13 Documentation Rules（规则 §11；对应原 Phase 6）

Windows 验证结束后，将结果写回 Linux 项目中已有的相应验证文档：本项目使用 [docs/validation/windows.md](docs/validation/windows.md)，命令级细节与历史证据保留在 [docs/windows-e2e-runbook.md](docs/windows-e2e-runbook.md)。优先沿用当前文档的文件位置、Markdown 结构、表格格式、状态标记和命名规则；不要在已有合适文档的情况下创建重复的验证报告。

记录至少包括：

- **Validation Environment**：Windows version（如果可获得）、architecture、相关 runtime/toolchain 版本、Linux source branch、Linux source commit、是否包含未提交修改、Windows 工作副本位置、验证日期；
- **Validation Results**：每项的验证项目、状态、执行命令、简要结果；
- **Errors**：失败步骤、关键错误信息、最可能原因、是否为 Windows-specific、是否阻塞其他验证、建议后续处理方式；
- **Not Executed / Blocked**：所有未执行项目必须说明原因（prerequisite failed、required external service unavailable、missing credential、hardware unavailable、test not applicable on Windows、project documentation does not currently define this validation、insufficient environment capability 等），不能只写 “not tested”。

Windows 实际验证历史必须保留，不要为了反映最新代码状态而删除历史失败记录：

```text
Previous Windows validation:
FAIL

Issue:
...

Linux fix:
...

Linux verification:
PASS

Current Windows status:
WINDOWS_VERIFICATION_PENDING
```

验证文档应区分四类信息：Windows validation result、Linux implementation/fix、Linux verification、pending Windows re-validation。

不要将巨大完整日志直接复制进主验证文档；优先保存关键错误、最相关 stack trace、日志文件路径和必要上下文。

### 5.14 Plan Management（规则 §12）

Windows 验证可能影响原 Plan。Linux 侧读取结果后，应对 Plan 中每个剩余步骤判断：completed、still required、modified、obsolete、blocked、requires Windows re-validation。

Plan 的总体目标不应因为验证结果自动扩大；只有当前仓库、明确需求或实际验证事实证明必要时，才能新增工作。

### 5.15 Verification Principles（规则 §13）

应优先从仓库自动确定验证命令，来源包括：`AGENTS.md`、project documentation、CI configuration（`.github/workflows/*.yml`、`.gitlab-ci.yml`）、package scripts 与 `package.json`、`src-tauri/Cargo.toml`、build scripts（`scripts/*`）、test configuration。不得凭空创造项目不存在的验证流程。

### 5.16 Final Diff Review（规则 §14；对应原 Final Review）

每轮 Linux 开发结束时检查最终 git diff，确保：

- 没有无关修改；
- Plan / validation documentation 已更新；
- Windows 状态没有被错误标记；
- 需要重新验证的项目已经明确列出；
- Windows 工作副本确实对应本次 Linux 源状态（若本轮涉及验证）；
- 所有适用的验证项目都有明确状态，`FAIL` / `BLOCKED` / `NOT RUN` 均有原因；
- 没有把失败项目误记为成功，没有遗漏明显的 Windows-specific 验证。

每轮 Windows 验证结束时同样检查：

- Windows 工作副本没有意外成为新的代码事实来源；
- Linux 端只接收预期的验证文档更新；
- 没有未经确认的代码反向同步。

本项目的常用检查入口为 `python3 scripts/verify.py --profile checks`。

### 5.17 Core Principle（规则 §15）

始终区分三个不同事实：`Code implemented`、`Linux verified`、`Windows verified`。只有在 Windows 平台实际执行并通过相应验证后，才能声明 `Windows verified`。

### 5.18 最终报告要求（对应原 Final Response）

向用户报告：

1. 本次验证对应的 Linux branch / commit / working tree 状态；
2. Windows 同步结果；
3. `PASS` 项目；
4. `FAIL` 项目；
5. `BLOCKED` / `NOT RUN` 项目及原因；
6. 发现的 Windows 平台问题；
7. 更新了哪些 Linux 文档；
8. 是否存在需要后续开发任务处理的问题，以及相关任务当前的 `WINDOWS_VERIFICATION_PENDING` 状态。

不要仅报告“验证完成”。如果仍有失败、阻塞或未执行项目，应明确指出。

## 6. 开发与延后 Windows 验证（批量开发、集中验证）

本项目主要开发环境为 Linux；Windows 环境仅用于无法在 Linux 环境充分完成的平台相关验证。规则正文见 [docs/development/cross-platform-validation.md](docs/development/cross-platform-validation.md) §16；验证队列与集中验证计划记录在 [docs/validation/windows.md](docs/validation/windows.md) §4。

### 6.1 核心执行规则

开发阶段采用“批量开发、集中验证”。不要按 `Feature A -> Windows test -> Feature B -> Windows test -> Feature C -> Windows test` 的方式交替执行；应连续完成所有当前可在 Linux 环境完成的开发工作，执行 Linux verification，汇总所有需要 Windows 环境验证的项目，最后进入一次集中的 `Windows validation phase`。

除非某个 Windows 验证结果是继续开发的硬性前置条件，否则不要在开发过程中提前进入 Windows 验证阶段。

### 6.2 Development Phase

优先连续完成当前 Plan 中所有满足以下条件的工作：可以在 Linux 环境实现；不依赖新的 Windows 验证结果才能继续；可以通过代码、静态分析、Linux 测试或已有文档合理确定实现方式；与当前任务范围一致。

完成一个功能后不要因为“该功能最终需要 Windows 验证”而暂停整个 Plan，而应：完成功能实现 → 执行 Linux 环境适用的测试 → 将需要 Windows 后续确认的内容加入 Windows Validation Queue → 继续下一个可执行开发项。

### 6.3 Windows Validation Queue

开发过程中累计维护一份 Windows 验证清单（本项目位置：`docs/validation/windows.md` §4）。每当某项修改需要 Windows 验证时记录到该清单，但不要立即切换到 Windows 环境。

每项至少记录：validation item、related feature / change、relevant files / modules、why Windows validation is required、exact behavior to verify、prerequisite、expected result、priority，以及 whether it blocks further Linux development。

状态默认 `WINDOWS_VERIFICATION_PENDING`；只有当缺少 Windows 验证结果会导致后续实现无法可靠继续时，才标记为 `WINDOWS_VERIFICATION_BLOCKING`。

### 6.4 允许中断开发的情况

只有以下情况可以在开发阶段提前要求 Windows 验证：后续设计依赖某个 Windows-specific 行为是否成立；Windows API / filesystem / process / installer 行为无法从代码或文档可靠判断；关键兼容性假设如果错误会使后续大量开发失效；当前失败只能在 Windows 上复现且阻塞继续开发；用户明确要求立即验证。

不要仅因为“这个功能需要最终在 Windows 测试”“Windows 上可能存在兼容性问题”“这是跨平台代码”就中断当前开发阶段。

### 6.5 Linux 侧验证不可延后

Windows 验证可以延后，但 Linux 侧验证不能全部延后：每个开发阶段仍应执行适用的 unit tests、Linux 可运行的 integration tests、regression tests、lint、typecheck、formatter、build 和 static checks（入口见第 3 节）。能在 Linux 环境发现的问题，应尽量在进入 Windows 验证前解决。

### 6.6 阶段退出与 Windows Validation Preparation

只有在 Plan 中所有不依赖 Windows 环境的开发项均已完成、所有可在 Linux 环境执行的验证均已完成、已知 Linux 侧错误已处理、所有 Windows-required verification items 已累计记录后，才结束当前 Linux 开发阶段，并进入 `Windows Validation Preparation`：统一分析**整个最终 diff**（而不是逐功能分别考虑测试），结合 current Plan、changed modules、Windows-related code paths、project documentation、build / CI configuration、previous Windows validation history 和 Windows Validation Queue 生成完整验证计划，并合并重复或高度相关的验证项（例如一次完整应用启动即可覆盖多个功能）。

集中清单按 **Build / Toolchain、Runtime、Filesystem、Integration、Packaging、Regression** 六类组织。每项包含 ID、test name、purpose、related changes、prerequisites、steps / command、expected result、priority，以及是否需要人工交互；优先级为 `P0`（必须验证，失败意味着任务不能完成）、`P1`（重要的平台兼容性验证）、`P2`（建议验证，但不阻塞主要功能）。

### 6.7 开发阶段结束报告

报告以下 9 项：已完成的开发工作；Linux 端执行的验证；Linux 端剩余问题；为什么现在可以结束 Linux development phase；完整的 Windows Validation Queue；合并整理后的 Windows 验证计划；哪些测试是 `P0` / `P1` / `P2`；是否存在 `WINDOWS_VERIFICATION_BLOCKING` 项目；下一步 Windows 环境应一次性执行哪些验证。

除非存在明确的 blocking Windows validation，否则不要在完成整个 Linux development phase 前要求切换到 Windows 环境。

## 7. Computer Use 故障处理

Windows 验证过程中 Computer Use / GUI automation 可能偶发或持续不可用；它本身不应导致整个 Windows validation phase 终止。规则正文见 [docs/development/cross-platform-validation.md](docs/development/cross-platform-validation.md) §17；人工队列记录在 [docs/validation/windows.md](docs/validation/windows.md) §4.6。

### 7.1 核心规则

如果某项测试因为 Computer Use 不可用而无法继续：标记为 `BLOCKED` → 记录阻塞原因 → 不反复无休止重试 → 跳过当前测试 → 继续执行所有不依赖该能力的其他验证项目 → 阶段结束后统一汇总未完成项目 → 为每个需要人工完成的项目提供可直接执行的详细手动测试步骤。

不要因为一个 GUI / Computer Use 测试被阻塞就停止 build、CLI tests、unit tests、integration tests、filesystem checks、configuration checks、log inspection、service / process verification，以及其他不依赖 GUI automation 的测试。

### 7.2 Classification

Computer Use unavailable、GUI automation unavailable、screen interaction unavailable、application window cannot be controlled、automation session lost、temporary UI-control failure 等原因导致的无法执行，一律标记 `BLOCKED` 并记录 `Blocker: COMPUTER_USE_UNAVAILABLE`；**不得**标记为 `FAIL` / `PASS` / `NOT_APPLICABLE`。`BLOCKED` 表示测试尚未得到结论，而不是功能本身失败。

### 7.3 Retry policy

疑似偶发故障：首次失败后允许重试一次；仍不可用即 `BLOCKED`。不要为了恢复 Computer Use 长时间重复同一操作，也不要因此阻塞其他独立验证项目。项目已有更明确的 retry policy 时以项目规范为准（本项目当前没有额外规范，适用一次性重试）。

### 7.4 继续独立验证

出现 `BLOCKED` 后重新检查剩余测试的依赖关系：不依赖被阻塞结果、不依赖 Computer Use、可通过 CLI / 脚本 / 日志完成的测试继续执行。若后续测试确实依赖被阻塞项目，同样标记 `BLOCKED` 并说明依赖关系。

### 7.5 优先非 GUI 替代

原验证目标可由可靠的非 GUI 方法完成时，优先使用已有方法（CLI、项目脚本、test APIs、logs、filesystem state、process status、configuration files、generated artifacts、automated tests）。不得为了绕过 Computer Use 改变测试语义；只有替代方法能验证与原测试相同的行为时才可记为 `PASS`，否则保留 `BLOCKED` 并进入人工队列。

### 7.6 Manual Windows Validation Queue

Windows 自动验证结束后生成人工队列，包含因 Computer Use unavailable、GUI-only workflow、required manual interaction、unavailable automation capability 而未完成的项。每项必须包含：Test ID（如 `WIN-MANUAL-001`）、Test name、Current status、Blocker、Purpose、Related change、Preconditions、Manual test steps（用户可直接照做，不能只写“人工测试应用是否正常”）、Expected result、Failure evidence to collect（screenshot、exact error message、relevant log、Windows Event Viewer entry、generated file、console output、reproduction steps）、Result field（`PASS` / `FAIL` / `BLOCKED` 及 Notes / Error / Evidence）。

### 7.7 阶段结束汇总

结果至少分四组：Automated PASS、Automated FAIL、`BLOCKED`、Manual validation required。Manual validation required 项目不能只给名称，必须给足够详细、可独立执行的步骤。

### 7.8 文档回写

自动验证结果写回 Linux 验证文档；因 Computer Use 导致的 `BLOCKED` 项记录：原计划验证项目、自动验证状态、blocker、是否尝试过 retry、是否存在非 GUI 替代验证、手动验证方法、当前最终状态。**在用户实际执行并反馈结果前不得标记为 `PASS`**。

### 7.9 关键区分与最终原则

始终区分 `Automation unavailable` 与 `Product functionality failed`：Computer Use unavailable → `BLOCKED`；实际功能执行后行为错误 → `FAIL`；实际执行并符合预期 → `PASS`。Computer Use 故障应降低自动化覆盖率，而不应降低其余 Windows 验证工作的完成度：尽可能完成所有可自动完成的测试，再把无法自动完成的部分转换成清晰、可复现、可人工执行的测试计划。

## 8. 文档与提交约束

- 变更需按影响范围同步文档，或明确确认无需更新；重要变化写入 `CHANGELOG.md` 的 `Unreleased`；
- Windows 验证完成后更新 [docs/validation/windows.md](docs/validation/windows.md)；命令细节只在 [docs/windows-e2e-runbook.md](docs/windows-e2e-runbook.md) 维护，避免双重维护；
- 测试产生的本地 artifact 在结果记录到文档或 `CHANGELOG.md` 后用 `python3 scripts/clean.py --generated` 清理，不入库、不上传远程；
- 提交使用 Conventional Commits，一个提交只包含一个逻辑主题；分支策略、PR 要求和 Definition of Done 见 [CONTRIBUTING.md](CONTRIBUTING.md)；
- 提交前运行 `python3 scripts/verify.py --profile checks`（按变更范围叠加 `frontend` / `rust` / `audit`），确认 Markdown 链接检查、密钥扫描和 `git diff --check` 通过。