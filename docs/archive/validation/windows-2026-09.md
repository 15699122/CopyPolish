# Windows 原生验证历史记录（2026-09）

> 本文自 `docs/roadmap.md` 迁入，仅作为已完成验收的历史快照保留，不再是当前任务清单。当前测试策略见 [testing.md](../../testing.md)，Windows 执行入口见 [windows-e2e-runbook.md](../../windows-e2e-runbook.md)。

## 2026-09-02 Windows 复验补充

本轮 Windows 原生环境原生验证完成环境、构建、W3C smoke、设置恢复/损坏/ACL、GUI 视觉 artifact 和 TUI transcript；旧 binary 的 embedded `selection-and-persistence.spec.ts` 替换输出 case 失败，当前修复已在 Linux/WSL 环境 环境 定向回归 3/3 通过，Windows 需重新留证；简繁 feature spec 在正确 feature binary 下 2/2 通过（s2t、t2s）；Unix-only 权限测试已隔离，Windows `cargo test --features tui` 需重新执行，详见 `docs/windows-e2e-runbook.md`。

## 2026-09-03 Windows 收尾执行计划

当前仍需在 Windows 原生环境完成并记录的新鲜证据：

- [x] 使用当前修复 binary 串行运行 embedded `selection-and-persistence.spec.ts`，3/3 通过，replacement case 输出 `待办`；
- [x] 先执行 `npm run build:app:simplified-trad --prefix e2e` 后运行 `simplified-trad-conversion.spec.ts`：2/2 通过（s2t、t2s）；
- [x] 在 Windows MSVC toolchain 执行 `cargo test --manifest-path src-tauri/Cargo.toml --features tui`：166 passed/0 failed；
- [x] 每项结果已记录环境、退出状态、实际完成数和日志/artifact；`finished=0` 的 runner 未计为通过；
- [x] 已按 Runbook 完成串行测试后的进程、端口、ACL、临时设置目录和生成物清理。
本轮保存竞态修复已在 Linux/WSL 环境 环境 默认 embedded 3/3、简繁 feature 2/2 通过；W3C smoke、重启/损坏设置、NTFS ACL、GUI 视觉 artifact、设置快捷键控制台和 TUI transcript 已有独立通过记录，只有代码或诊断范围变化时按需复跑；GUI DPI 自动矩阵和 GitLab Windows stage 继续按项目决定跳过。完整步骤见 [`docs/windows-e2e-runbook.md` 第 2.5 节](../../windows-e2e-runbook.md#25-2026-09-03-当前必须执行的-windows-收尾流程)。


## 2026-09-03 Windows 复验结论

默认 embedded GUI、设置恢复/损坏、NTFS ACL、GUI 视觉、快捷键控制台、TUI transcript、简繁 feature 2/2、标准 W3C smoke 2/2 和 Windows MSVC Rust/TUI 回归均已取得当前证据。GUI DPI 自动矩阵与 GitLab Windows 可选 stage 维持跳过，人工 DPI 与 Windows Terminal 交互 artifact 维持已完成。

## 2026-09-03 提交 6687c13 capability 刷新结论

已在隔离 Windows 原生 checkout `<isolated-windows-checkout>` 刷新当前 capability 实现的证据：默认 embedded 3/3、简繁 feature embedded 2/2、Windows MSVC `cargo test --features tui` 167/167、标准 W3C smoke 2/2（随机 localhost 端口）均通过；所有 runner 均有明确 `finished > 0` 和失败数为 0。原有旧 checkout 未修改，隔离目录和 artifact 已清理。简繁转换 capability 决策及 Windows 收尾验证不再阻塞后续 P1；TUI 替换/简繁控件已完成，下一项未完成任务为中文文案、PDF 清洗和技术文档预设。

## 2026-09-04 当前 checkout Windows 复验

2026-09-04 的 Windows 原生复验记录显示：前端 101/101、E2E typecheck、默认 embedded selection 3/3、简繁 feature 2/2、W3C/WebDriver 2/2、Rust/TUI MSVC 182/182、损坏设置 3/3、NTFS ACL 1/1、GUI artifact 1/1、设置快捷键 1/1、TUI transcript 4/4 均通过；125%/150% 人工 GUI 与 Windows Terminal 交互 artifact 保持已完成，GUI DPI 自动矩阵和 GitLab Windows stage 按项目决定跳过。该段是历史验收记录，不要求使用特定本机 checkout 路径。

默认 binary 的 `test:restart-settings` 曾因旧 spec 在 `restart-settings.spec.ts:53` 强制等待 `t2s` 而失败；当前 spec 已按 `simplifiedTradConversion` capability 分支断言，默认构建验证禁用/归一化为 `conversion: none`，feature 构建验证 `t2s` 恢复。2026-09-04 Windows 原生复验中默认与 feature 的 write/read 均为 2/2，历史失败已闭环。
## 2026-09-04 Windows 验证闭环

重启恢复 spec 的 selector 已修正并同步至 Windows 原生环境；默认 capability=false 与简繁 feature 的 write/read 均 2/2 通过。W3C smoke 单独重跑 2/2，损坏设置 3/3、ACL 1/1、GUI artifact 1/1、快捷键 1/1、feature 转换 2/2、Rust/TUI 182+5+3、TUI transcript 4/4 全部通过。GUI DPI 自动矩阵和 GitLab Windows stage 维持跳过，人工 DPI 与 Windows Terminal 交互 artifact 维持完成；Windows 自动验证当前无待闭环失败。