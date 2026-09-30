# 跨平台开发与 Windows 验证工作流

本文是 CopyPolish 跨平台开发与 Windows 验证的**规则正文**，说明 Linux 开发与 Windows 验证的分工、同步规则、状态模型、结果分类和记录要求。

| 文档 | 职责 |
| --- | --- |
| [../../AGENTS.md](../../AGENTS.md) 第 5 节 | 代理必须遵守的操作规则（速查版，与本文一致） |
| 本文 | 规则正文与本项目落地参数 |
| [../validation/windows.md](../validation/windows.md) | Windows 验证的结果记录（含验证清单） |
| [../windows-e2e-runbook.md](../windows-e2e-runbook.md) | 命令级步骤、通过条件、artifact 与清理要求 |
| [../testing.md](../testing.md) §2.2 / §2.3 | Windows 平台门禁矩阵与执行顺序 |

## 1. Purpose

本项目主要在 Linux 环境进行开发。Windows 环境主要用于：

- Windows-specific build verification；
- Windows runtime verification；
- Windows packaging / installer verification；
- filesystem / path / process / sidecar 等平台相关验证；
- 项目文档定义的其他 Windows 验证。

Linux 项目目录（本仓库根）是主要开发工作区和项目事实来源；Windows 项目目录是验证工作副本，位置与同步约定见本文 §7。

## 2. Source of Truth

Linux 项目目录是以下内容的主要事实来源：

- source code；
- project configuration；
- project documentation；
- implementation state；
- Plan / task state（本项目为 [../roadmap.md](../roadmap.md)）；
- validation documentation。

Windows 工作副本主要用于验证，不作为主要开发源：

- 代码同步方向默认必须为 Linux → Windows；
- 除验证文档结果外，不应将 Windows 工作副本中的代码修改自动反向同步到 Linux。

## 3. Development / Validation Cycle

标准工作流：

1. Linux implementation；
2. Linux verification；
3. Mark Windows verification requirements（在 roadmap / 验证文档中标记 `WINDOWS_VERIFICATION_PENDING`）；
4. Sync Linux project to Windows（见 §7）；
5. Windows validation（见 §8）；
6. Write Windows validation results back to Linux documentation（见 §11）；
7. Linux reads and reconciles Windows results（见 §9）；
8. Continue implementation or fixes（见 §10）；
9. Repeat Windows validation when required。

不得将“Linux 已修复”视为“Windows 已验证通过”。

开发阶段的执行方式采用**批量开发、集中验证**：不要按 `Feature A -> Windows test -> Feature B -> Windows test` 的方式交替执行，而应连续完成所有可在 Linux 环境完成的工作，再进入一次集中的 Windows 验证阶段（见 §16）。

## 4. Validation State Model

平台相关任务应尽可能使用以下状态：

| 状态 | 含义 | 典型使用位置 |
| --- | --- | --- |
| `IMPLEMENTED` | 代码已实现，尚未在 Linux 完成验证 | roadmap 任务、PR 描述 |
| `LINUX_VERIFIED` | 已在 Linux 平台完成适用验证 | roadmap 任务、PR 描述 |
| `WINDOWS_VERIFICATION_PENDING` | 等待下一轮 Windows 验证 | roadmap 任务、验证文档 |
| `WINDOWS_PASS` | 已在 Windows 实际执行并通过 | 验证文档、roadmap 任务 |
| `WINDOWS_FAIL` | 已在 Windows 实际执行但失败 | 验证文档、roadmap 任务 |
| `WINDOWS_BLOCKED` | 因前置条件缺失无法在 Windows 验证 | 验证文档、roadmap 任务 |
| `NOT_RUN` | 本轮未执行，但理论上应执行 | 验证文档 |
| `NOT_APPLICABLE` | 根据当前项目或平台状态明确不适用 | 验证文档 |

典型状态流：

```text
IMPLEMENTED
  -> LINUX_VERIFIED
  -> WINDOWS_VERIFICATION_PENDING
  -> WINDOWS_PASS
```

若 Windows 验证失败：

```text
WINDOWS_FAIL
  -> Linux fix
  -> LINUX_VERIFIED
  -> WINDOWS_VERIFICATION_PENDING
  -> Windows re-validation
```

Linux 端不得在 Windows 实际重新验证之前，将修复后的项目标记为 `WINDOWS_PASS`。

> 本节状态是**任务级**生命周期标记；单次验证项目的 `PASS` / `FAIL` / `BLOCKED` / `NOT RUN` / `NOT APPLICABLE`（见 §8）是**项目级**结果标记，两者不互相替代。

## 5. Linux Development Responsibilities

Linux 侧负责：

- 阅读当前仓库和项目文档；
- 执行主要开发工作；
- 执行 Linux 平台适用的验证；
- 分析 Windows 验证结果；
- 修复真正属于项目代码的问题；
- 更新 Plan 和项目文档；
- 标记需要下一轮 Windows 验证的内容。

Linux 侧不应：

- 假装执行了 Windows-only 验证；
- 将 Linux 测试成功等同于 Windows 验证成功；
- 为纯 Windows 环境配置问题修改项目代码；
- 无依据扩大当前任务范围。

本项目的 Linux 验证入口见 §13。

## 6. Windows Validation Responsibilities

Windows 侧负责：

- 读取 Linux 项目及项目文档；
- 将需要验证的内容从 Linux 单向同步至 Windows 工作副本；
- 根据仓库、文档、构建配置和脚本确定验证范围；
- 执行适用的 Windows 验证；
- 记录 `PASS` / `FAIL` / `BLOCKED` / `NOT RUN` / `NOT APPLICABLE`；
- 分析失败原因；
- 将实际验证结果写回 Linux 项目的验证文档。

Windows 侧默认执行验证，而不是功能开发。除机器本地配置、构建产物或验证所需临时变化外，不应为了让验证通过而自行修改业务代码。

若发现代码问题，应记录 failure、reproduction、likely root cause、relevant code location、suggested follow-up，并交由 Linux 开发阶段处理（见 §10）。

## 7. Windows Synchronization Rules

同步方向固定为 **Linux → Windows**，单向执行。

| 角色 | 本项目位置 |
| --- | --- |
| 同步源 | Linux 项目目录（本仓库根） |
| 同步目标 | `E:\Shiraishi\VSCode Workspace\chinese_copywriting_formatter`（WSL 侧 `/mnt/e/Shiraishi/VSCode Workspace/chinese_copywriting_formatter`） |

同步前应检查：Linux branch、Linux commit、Linux working tree、Windows target directory、是否存在 Windows 本地需要保留的配置。

默认不要跨平台同步：`.git`（除非验证流程需要）、`node_modules`、Python virtual environments（Windows 副本存在 `.venv/`）、Rust `target`、build caches（`frontend/dist`、`src-tauri/gen`、`e2e/artifacts`）、IDE caches（`.vscode/`、`.codex/`）、temporary files、secrets（`secrets/` 与明文凭据）、machine-specific configuration。

判定依据优先级：仓库 [`.gitignore`](../../.gitignore) → 项目文档 → 构建配置 → 已有同步脚本。

优先使用项目已有的 sync scripts、checkout procedures、worktree workflow、build preparation scripts，而不是自行实现另一套同步逻辑：

- 隔离发布工作区：`docs/release/manual-release.md` §3（git worktree）；
- WSL + Windows 主机协作方式：`docs/release/manual-release.md` §7。

若 Linux 侧包含未提交改动，或无法获得可靠的 commit / hash，必须在验证记录中写明“包含 working tree changes”，不得假装对应一个纯 Git commit。

## 8. Windows Validation Result Classification

每个验证项目至少应使用以下结果之一：

### PASS

验证实际执行，并满足预期。记录命令、结果摘要；有 artifact 时记录路径。

### FAIL

验证实际执行，但项目行为或输出不符合要求。应记录：

- command；
- failure point；
- relevant error；
- likely cause；
- whether Windows-specific；
- follow-up recommendation。

### BLOCKED

由于前置条件缺失无法继续。例如：required dependency unavailable、certificate unavailable、external service unavailable、credential unavailable、prerequisite test/build failed、unsupported environment。必须记录阻塞原因。

### NOT RUN

本轮未执行，但理论上应执行。必须说明原因。

### NOT APPLICABLE

根据当前项目或平台状态明确不适用。

> 附加约束：runner 出现 `exitCode=0` 但 `finished=0` 时只能记为未完成，不得记为 `PASS`。

本项目的验证范围（Required / Applicable / NOT APPLICABLE）、执行命令与通过条件见 [../validation/windows.md](../validation/windows.md) §3。

## 9. Reconciliation of Windows Results on Linux

当 Windows 验证结果写回 Linux 项目后，Linux 侧必须先重新读取：

- `git diff`；
- modified documentation；
- current Plan（[../roadmap.md](../roadmap.md)）；
- validation result documents（[../validation/windows.md](../validation/windows.md)）；
- 相关的 `AGENTS.md` 指令。

然后重新评估当前 Plan。应区分：

- 已经 Windows 验证完成的事项；
- Windows 失败且属于代码问题的事项；
- 环境问题；
- BLOCKED 项目；
- 尚未执行项目；
- 已不再适用的项目；
- 需要下一轮 Windows re-validation 的项目。

不得机械继续旧 Plan；当前仓库和最新验证结果优先于旧任务假设。

## 10. Handling Windows Failures

Windows `FAIL` 后，Linux 侧首先判断问题属于：project code、Windows compatibility、dependency、environment、credential、external service、test infrastructure、unknown。只有真正属于项目代码或必要兼容性问题时才修改代码。

修复后：

1. 执行 Linux 适用的 regression tests；
2. 执行相关 Linux verification；
3. 更新文档；
4. 将 Windows 状态改为 `WINDOWS_VERIFICATION_PENDING`，而不是 `WINDOWS_PASS`；
5. 记录下一轮 Windows 应重新执行哪些验证。

## 11. Documentation Rules

Windows 实际验证历史必须保留。例如：

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

不要为了反映最新代码状态而删除历史失败记录。验证文档应区分：

- Windows validation result；
- Linux implementation / fix；
- Linux verification；
- pending Windows re-validation。

记录位置与结构沿用 [../validation/windows.md](../validation/windows.md)；不要将巨大完整日志复制进主验证文档，优先保存关键错误、最相关 stack trace、日志文件路径和必要上下文。

## 12. Plan Management

Windows 验证可能影响原 Plan（本项目为 [../roadmap.md](../roadmap.md)）。Linux 侧读取结果后，应对 Plan 中每个剩余步骤判断：

- completed；
- still required；
- modified；
- obsolete；
- blocked；
- requires Windows re-validation。

Plan 的总体目标不应因为验证结果自动扩大。只有当前仓库、明确需求或实际验证事实证明必要时，才能新增工作。

## 13. Verification Principles

应优先从仓库自动确定验证命令，包括：`AGENTS.md`、project documentation、CI configuration、package scripts、`src-tauri/Cargo.toml`、build scripts、test configuration。不得凭空创造项目不存在的验证流程。

本项目当前可用的验证来源：

| 来源 | 用途 |
| --- | --- |
| `scripts/verify.py --profile checks` / `frontend` / `rust` / `audit` / `security` / `ci` / `release` | 统一验证入口（Linux / Windows 通用部分） |
| `e2e/package.json` 中的 `build:app*` 与 `test*` runners | 真实 Tauri GUI E2E 与 Windows 专项验证 |
| `cargo test --manifest-path src-tauri/Cargo.toml [--features tui]` | Rust / TUI 编译与测试 |
| `python3 scripts/verify_release_assets.py <tag> [--platform windows]` | 发布资产结构与版本校验 |
| `python3 scripts/clean.py --generated` | 验证结束后清理本地生成物 |
| `.github/workflows/*.yml`、`.gitlab-ci.yml` | CI 门禁与 Windows 构建参考 |

Windows 侧仍需按 [../testing.md](../testing.md) §2.2 判断哪些项目属于 Windows 原生专属，不能被 Linux 结果替代。

## 14. Final Diff Review

每轮 Linux 开发结束时检查最终 git diff。确保：

- 没有无关修改；
- Plan / validation documentation 已更新；
- Windows 状态没有被错误标记；
- 需要重新验证的项目已经明确列出。

每轮 Windows 验证结束时同样检查：

- Windows 工作副本没有意外成为新的代码事实来源；
- Linux 端只接收预期的验证文档更新；
- 没有未经确认的代码反向同步。

本项目的常用检查入口：`python3 scripts/verify.py --profile checks`（包含 `git diff --check`、密钥/SOPS 扫描和 Markdown 链接检查）。

## 15. Core Principle

始终区分三个不同事实：

1. `Code implemented`；
2. `Linux verified`；
3. `Windows verified`。

只有在 Windows 平台实际执行并通过相应验证后，才能声明 `Windows verified`。

## 16. 开发与延后 Windows 验证（Deferred Windows Validation Workflow）

本项目主要开发环境为 Linux；Windows 环境仅用于无法在 Linux 环境充分完成的平台相关验证。本节规定“批量开发、集中验证”的执行方式，避免开发过程被零散的 Windows 验证打断。代理操作规则见 [../../AGENTS.md](../../AGENTS.md) §6；Windows 验证队列与集中验证计划记录在 [../validation/windows.md](../validation/windows.md) §4。

### 16.1 Core execution rule

开发阶段采用“批量开发、集中验证”。不要按以下模式执行：

```text
Feature A -> Windows test -> Feature B -> Windows test -> Feature C -> Windows test
```

应按照以下模式执行：

```text
Feature A -> Feature B -> Feature C
  -> 完成所有当前可在 Linux 环境执行的开发工作
  -> Linux verification
  -> 汇总所有需要 Windows 环境验证的项目
  -> Windows validation phase
```

除非某个 Windows 验证结果是继续开发的硬性前置条件，否则不要在开发过程中提前进入 Windows 验证阶段。

### 16.2 Development Phase

在当前 Plan（[../roadmap.md](../roadmap.md)）中，优先连续完成所有满足以下条件的工作：

- 可以在 Linux 环境实现；
- 不依赖新的 Windows 验证结果才能继续；
- 可以通过代码、静态分析、Linux 测试或已有文档合理确定实现方式；
- 与当前任务范围一致。

完成一个功能后，不要因为该功能最终需要 Windows 验证而暂停整个 Plan。应：

1. 完成功能实现；
2. 执行 Linux 环境适用的测试；
3. 将需要 Windows 后续确认的内容加入 Windows Validation Queue；
4. 继续下一个可执行开发项。

### 16.3 Windows Validation Queue

开发过程中维护一个累计的 Windows 验证清单（本项目记录在 [../validation/windows.md](../validation/windows.md) §4）。每当发现某项修改需要 Windows 验证时，将其记录到该清单，但不要立即切换到 Windows 环境。

每个验证项至少记录：

- validation item；
- related feature / change；
- relevant files / modules；
- why Windows validation is required；
- exact behavior to verify；
- prerequisite；
- expected result；
- priority；
- whether it blocks further Linux development。

状态可使用 `WINDOWS_VERIFICATION_PENDING` 或 `WINDOWS_VERIFICATION_BLOCKING`。默认使用 `WINDOWS_VERIFICATION_PENDING`；只有当缺少 Windows 验证结果会导致后续实现无法可靠继续时，才标记为 `WINDOWS_VERIFICATION_BLOCKING`。

### 16.4 When Windows validation may interrupt development

只有以下情况可以在开发阶段提前要求 Windows 验证：

1. 后续设计依赖某个 Windows-specific 行为是否成立；
2. Windows API / filesystem / process / installer 行为无法从代码或文档可靠判断；
3. 一个关键兼容性假设如果错误，会使后续大量开发失效；
4. 当前失败只能在 Windows 上复现，并且阻塞继续开发；
5. 用户明确要求立即进行 Windows 验证。

除此之外，应继续完成剩余 Linux 开发工作。不要仅因为“这个功能需要最终在 Windows 测试”“Windows 上可能存在兼容性问题”“这是跨平台代码”就中断当前开发阶段。

### 16.5 Linux verification during development

Windows 验证可以延后，但 Linux 侧验证不能全部延后。每个开发阶段仍应执行适用的：unit tests、integration tests that run on Linux、regression tests、lint、typecheck、formatter、build、static checks。

能够在 Linux 环境发现的问题，应尽量在进入 Windows 验证前解决。目标是：进入 Windows 验证阶段时，剩余问题应尽可能只包含真正的平台相关验证。本项目在 Linux 侧的适用入口见 §13。

- [../../AGENTS.md](../../AGENTS.md) §5：代理操作规则与最终报告要求；
- [../validation/windows.md](../validation/windows.md)：Windows 验证清单与结果记录；
- [../windows-e2e-runbook.md](../windows-e2e-runbook.md)：Windows 原生执行步骤、artifact 与清理要求；
- [../testing.md](../testing.md)：测试层次、平台门禁与执行顺序；
- [../release/manual-release.md](../release/manual-release.md)：隔离工作区、WSL/Windows 协作与发布资产验收；
- [../development.md](../development.md)：工具链、常用命令和工程约束；
- [../roadmap.md](../roadmap.md)：当前 Plan（只写未完成工作）；
### 16.6 End of Development Phase

只有在以下条件基本满足后，才结束当前 Linux 开发阶段：

- 当前 Plan 中所有不依赖 Windows 环境的开发项均已完成；
- 所有可在 Linux 环境执行的验证均已完成；
- 已知 Linux 侧错误已经处理；
- 所有 Windows-required verification items 已被累计记录；
- 没有遗漏明显的平台相关验证要求。

此时不要继续零散执行功能开发，而是进入 `Windows Validation Preparation` 阶段。

### 16.7 Windows Validation Preparation

开发阶段结束后，统一分析整个最终 diff，而不是只按单个功能分别考虑测试。根据以下信息生成完整 Windows 验证计划：

- final git diff；
- current Plan；
- changed modules；
- Windows-related code paths；
- project documentation；
- build / CI configuration；
- previous Windows validation history；
- Windows Validation Queue。

合并重复或高度相关的验证项目。例如不要分别列出 Feature A / B / C 的启动测试；如果一次完整应用启动即可覆盖，应合并成一个验证场景。本项目的样例记录见 [../validation/windows.md](../validation/windows.md) §4.3。

### 16.8 Windows validation summary

最终输出一份集中式 Windows 验证清单，按以下类别组织：

1. **Build / Toolchain**：Windows build、compiler / toolchain compatibility、native dependency resolution；
2. **Runtime**：application startup、subprocess behavior、sidecar startup、environment detection；
3. **Filesystem**：Windows path handling、drive-letter paths、Unicode paths、file locking、permissions；
4. **Integration**：IPC、external services、browser extension ↔ desktop communication、native APIs；
5. **Packaging**：installer、bundled resources、sidecar packaging、application upgrade；
6. **Regression**：列出因本轮修改而需要重新确认的已有 Windows 行为。

上述条目为通用类别；本项目实际适用的子集、以及明确不适用的项（无安装器、无 sidecar、无 browser extension 等）见 [../validation/windows.md](../validation/windows.md) §3.3。

### 16.9 Final Windows Validation Handoff

最终 Windows 验证计划中的每个测试项应包含：ID、test name、purpose、related changes、prerequisites、steps / command、expected result、priority，以及 whether manual interaction is required。

优先级建议：

- `P0`：必须验证，失败意味着任务不能完成；
- `P1`：重要的平台兼容性验证；
- `P2`：建议验证，但不阻塞主要功能。

### 16.10 Final development report

当所有非必须 Windows 环境的开发工作完成后，报告：

1. 已完成的开发工作；
2. Linux 端执行的验证；
3. Linux 端剩余问题；
4. 为什么现在可以结束 Linux development phase；
5. 完整的 Windows Validation Queue；
6. 合并整理后的 Windows 验证计划；
7. 哪些测试是 `P0` / `P1` / `P2`；
8. 是否存在 `WINDOWS_VERIFICATION_BLOCKING` 项目；
9. 下一步 Windows 环境应一次性执行哪些验证。

除非存在明确的 blocking Windows validation，否则不要在完成整个 Linux development phase 前要求切换到 Windows 环境。

## 17. Computer Use 故障处理（Computer Use Failure Handling）

Windows 验证过程中，Computer Use / GUI automation 可能偶发或持续不可用。**Computer Use 不可用本身不应导致整个 Windows validation phase 终止。** 代理操作规则见 [../../AGENTS.md](../../AGENTS.md) §7；人工队列记录在 [../validation/windows.md](../validation/windows.md) §4.6。

### 17.1 Core rule

如果某项测试因为 Computer Use 不可用而无法继续：

1. 将该测试标记为 `BLOCKED`；
2. 记录阻塞原因；
3. 不反复无休止重试；
4. 跳过当前测试；
5. 继续执行所有不依赖该能力的其他验证项目；
6. 在整个 Windows 验证阶段结束后，统一汇总所有未完成项目；
7. 为每个需要人工完成的项目提供可直接执行的详细手动测试步骤。

不要因为一个 GUI / Computer Use 测试被阻塞，就停止：build、CLI tests、unit tests、integration tests、filesystem checks、configuration checks、log inspection、service / process verification，以及其他不依赖 GUI automation 的测试。

### 17.2 Classification

如果测试由于以下原因无法执行：

- Computer Use unavailable；
- GUI automation unavailable；
- screen interaction unavailable；
- application window cannot be controlled；
- automation session lost；
- temporary UI-control failure；

应标记为 `BLOCKED`，并进一步记录 `Blocker: COMPUTER_USE_UNAVAILABLE`。**不要**将其标记为 `FAIL`、`PASS` 或 `NOT_APPLICABLE`。`BLOCKED` 表示测试尚未得到结论，而不是功能本身失败。

### 17.3 Retry policy

对于疑似偶发的 Computer Use 故障，可以进行有限次数的合理重试：

- 首次失败后允许重新尝试一次；
- 如果仍然无法正常使用，则将相关测试标记为 `BLOCKED`；
- 不要为了恢复 Computer Use 而长时间重复同一操作；
- 不要因此阻塞其他独立验证项目。

如果当前项目已有更明确的 retry policy，以项目规范为准；本项目当前没有额外的 Computer Use retry 规范，因此适用上述「一次重试」。

### 17.4 Continue independent validation

出现 `BLOCKED` 后，应重新检查剩余测试之间的依赖关系。如果后续测试不依赖当前被阻塞测试的结果、不依赖 Computer Use，且可以通过 CLI、脚本、日志或其他方式完成，则继续执行：

```text
GUI startup check       -> BLOCKED
cargo test              -> continue
cargo build             -> continue
CLI integration test    -> continue
filesystem test         -> continue
installer GUI test      -> BLOCKED
log validation          -> continue
```

如果后续测试确实依赖被阻塞项目，则标记 `BLOCKED` 并说明依赖关系。

### 17.5 Prefer non-GUI alternatives

如果原验证目标可以通过可靠的非 GUI 方法完成，优先使用已有的非 GUI 验证方法，例如：CLI、项目脚本、test APIs、logs、filesystem state、process status、configuration files、generated artifacts、HTTP endpoints、automated tests。

但**不要为了绕过 Computer Use 而自行改变测试语义**。只有当替代方法能够验证与原测试相同的行为时，才可以用替代方法将测试标记为 `PASS`；否则仍然应保留 `BLOCKED` 并进入人工测试清单。

### 17.6 Manual Validation Queue

Windows 自动验证结束后，统一生成 `Manual Windows Validation Queue`，其中包含所有因为 Computer Use unavailable、GUI-only workflow、required manual interaction 或 unavailable automation capability 而未能完成的测试。每个项目必须包含：

- **Test ID**：唯一标识，例如 `WIN-MANUAL-001`；
- **Test name**：简洁描述需要验证的行为；
- **Current status**：通常为 `BLOCKED`；
- **Blocker**：例如 `COMPUTER_USE_UNAVAILABLE`；
- **Purpose**：说明此测试要证明什么；
- **Related change**：与哪个功能、模块或修改相关；
- **Preconditions**：详细列出前置条件（应用已成功 build、Windows 工作副本已同步至指定 revision、服务已启动、测试数据已准备、需要管理员权限、需要浏览器扩展、需要网络连接等）；
- **Manual test steps**：从开始到结束的具体步骤，必须写成用户可以直接照做的形式，不能只写“人工测试应用是否正常”；
- **Expected result**：明确写出每一步或最终应出现的结果；
- **Failure evidence to collect**：失败时应收集的内容（screenshot、exact error message、relevant log、Windows Event Viewer entry、generated file、console output、reproduction steps）；
- **Result field**：留出人工填写 `PASS` / `FAIL` / `BLOCKED`，以及 Notes、Error、Evidence。

本项目的队列实例见 [../validation/windows.md](../validation/windows.md) §4.6。

### 17.7 End-of-validation summary

Windows 验证阶段结束时，应将结果分成至少四组：

1. **Automated PASS**：自动完成并通过的测试；
2. **Automated FAIL**：测试实际执行，但发现项目问题；
3. **BLOCKED**：由于 Computer Use 或其他环境能力问题无法得出结论；
4. **Manual validation required**：需要用户人工执行的测试。

对于 Manual validation required 项目，不要仅提供名称；必须提供足够详细的操作步骤，使用户可以在不了解代理内部推理的情况下独立完成验证。

### 17.8 Documentation update

将自动验证结果正常写回 Linux 项目验证文档。对于因 Computer Use 导致的 `BLOCKED` 项目，记录：

- 原计划验证项目；
- 自动验证状态：`BLOCKED`；
- blocker；
- 是否尝试过 retry；
- 是否存在非 GUI 替代验证；
- 手动验证方法；
- 当前最终状态。

在用户实际执行手动测试并反馈结果前，**不得**将其标记为 `PASS`。

### 17.9 Important distinction

始终区分 `Automation unavailable` 与 `Product functionality failed`：Computer Use 失败本身不是项目测试失败。

```text
Computer Use unavailable        -> BLOCKED
实际功能执行后行为错误           -> FAIL
实际执行并符合预期               -> PASS
```

### 17.10 Final principle

Computer Use 故障应降低自动化覆盖率，而不应降低其余 Windows 验证工作的完成度。尽可能完成所有可自动完成的测试，然后把无法自动完成的部分转换成清晰、可复现、可人工执行的测试计划。

## 18. 相关文档

- [../../AGENTS.md](../../AGENTS.md) §5 / §6：代理操作规则、延后 Windows 验证与最终报告要求；
- [../validation/windows.md](../validation/windows.md)：Windows 验证队列、清单与结果记录；
- [../windows-e2e-runbook.md](../windows-e2e-runbook.md)：Windows 原生执行步骤、artifact 与清理要求；
- [../testing.md](../testing.md)：测试层次、平台门禁与执行顺序；
- [../release/manual-release.md](../release/manual-release.md)：隔离工作区、WSL/Windows 协作与发布资产验收；
- [../development.md](../development.md)：工具链、常用命令和工程约束；
- [../roadmap.md](../roadmap.md)：当前 Plan（只写未完成工作）；
- [../archive/validation/windows-2026-09.md](../archive/validation/windows-2026-09.md)：历史 Windows 验收快照。