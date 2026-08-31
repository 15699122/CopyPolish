# Changelog

本文件记录 CopyPolish 的重要用户可见变化、兼容性变化和工程维护变化。详细历史仍以 Git 提交和 GitHub Release 为准。

## [Unreleased]

### Added

- 新增贡献指南、架构说明和测试指南。
- 新增项目变更记录。
- 新增独立 WebdriverIO + Tauri embedded provider E2E 工程，覆盖真实启动、Rust IPC 默认排版、全不选恒等和临时设置文件隔离。
- 新增基于 `tauri-plugin-webdriver` 0.2.1 的并行标准 WebDriver E2E provider，复用现有 smoke 并保持原 embedded provider 可回退。

### Changed

- 重组开发文档、文档导航和后续路线图，明确当前事实、操作手册、计划和历史归档的职责边界。
- 统一 GitHub 分支 CI、GitLab tag 构建和本地验证流程的说明。
- E2E 构建增加 `custom-protocol`、条件 capability 和测试专用 `withGlobalTauri` 配置，生产构建不加载 WebDriver plugin。
- E2E 前端资源使用相对路径，并按 spec 启动独立 WDIO 进程，避免测试间共享 `rules.yaml` 状态。
- 标准 WebDriver provider 使用随机 localhost 端口、独立应用进程和运行 artifact；其前端不加载 `@wdio/tauri-plugin`。
- 新增三种损坏 `rules.yaml` / `rules.yaml.bak` fixture 的双 provider 自动化入口，覆盖备份恢复、无备份降级和主备份同时损坏，并验证真实 Rust IPC 仍可用。
- 新增双 provider 设置重启恢复自动化入口，验证同一临时设置目录中的规则选择、最近输入和真实 Rust IPC 输出在第二次启动后恢复。
- 新增 Windows-only NTFS ACL 设置保存失败自动化入口，使用 `icacls.exe` 注入当前用户 deny ACE，并在 `finally` 中恢复权限和清理 fixture；非 Windows 环境显式跳过。
- 统一 embedded 与标准 W3C provider 的 E2E artifact 目录、manifest/result、失败截图、page source 和设置 fixture 收集；manifest 仅记录白名单环境摘要，不写入完整环境变量。
- 记录并完成 Windows 原生验证：Node 24.19.0、npm 11.17.0、Rust 1.98.0 MSVC、WebView2 Runtime 151.0.4129.107、Windows Terminal 1.24.11911.0、PowerShell 7.6.5；前端 57/57、设置 Rust 测试 16/16、Rust/TUI 148/148 通过，TUI MSVC release/stdin smoke 通过。
- embedded 与标准 W3C WebDriver provider 的真实 WebView2/Rust IPC 普通用例各 3/3、设置重启 write/read 各 1/1、三种损坏设置 fixture 各 3/3、NTFS ACL 各 1/1 通过；Windows GUI/TUI 手动回归和双 provider 稳定性验证均已完成。后续仅补 GUI/TUI 自动 artifact、受控失败完整诊断包、React 19 `act` warning 闭环和 GitLab Windows 可选 E2E。
- 修复 TUI 规则面板展示顺序、输入区可打印字符误触全局快捷键、bracketed paste 以及最后一个 grapheme 的 Delete/Right 边界；修复后的 Windows Terminal raw-mode、规则排序、`Get-Clipboard` 完整粘贴、新快捷键语义和编辑器边界已完成手动回归。
- 统一桌面 GUI 输出框边框阴影、设置标题间距、恢复按钮、长路径中间省略和主题/快捷键复选框样式；Linux/WSLg 前端测试与构建及 Windows WebView2 下的 DPI、窄窗口和视觉回归均已完成。
- 抽离前端规则目录加载（`useRuleCatalog`）和清空输入反馈（`useClearFeedback`），降低 `App.tsx` 编排复杂度。
- 拆分设置界面为独立分区组件（主题、显示、快捷键、规则、状态 Footer），`SettingsDialog.tsx` 负责编排。
- 抽取设置提醒文案与判定到 `frontend/src/lib/settingsLoadNotices.ts`，主界面与设置 Footer 共用，消除重复并精简 `App.tsx`。
- 优化 Markdown 行内代码扫描，降低大量反引号文本的重复查找开销，并保持多长度 delimiter 与未闭合 delimiter 行为。
- 优化结构 span 仲裁，减少复杂 Markdown/LaTeX 文本中的 O(n²) 重叠检查开销，并保持优先级与嵌套结构语义。
- 缩小可编辑规则阶段的保护预扫描范围，仅扫描不透明结构与化学式，减少重复语义扫描开销并保持保护边界。
- 优化专有名词规则的批量替换，减少重复全文遍历，并保持相邻词、前缀词和 ASCII 单词边界语义。
- 优化中英文/中文数字空格规则，改用流式 Unicode 判定单位遍历，减少中间数组分配并保持 Grapheme 边界语义。
- 优化 HTML block 结束标签查找，减少大小写不敏感扫描中的无效窗口比较，并保持 UTF-8 内容与跨行保护语义。
- 合并 LaTeX `\(...\)` / `\[...\]` 定界符扫描，减少重复全文遍历并保持混合定界符和未闭合保护行为。
- 合并普通 Markdown 链接与引用式链接的候选扫描，减少重复全文遍历并保持图片链接、嵌套目标和引用式链接边界语义。
- 优化美元数学候选扫描，避免对连续反斜杠进行重复回扫，并保持奇偶转义、单/双美元和跨行边界语义。
- 合并 URL 与邮箱候选扫描，减少一次全文遍历，并保持 URL 优先、邮箱边界和大小写语义。
- 优化表格分隔线扫描，避免为每个候选行创建临时单元格数组，并保持对齐标记与合法性边界。

### Removed

- 移除已失效的 VS Code `reference` Python 分析路径配置。

## [0.5.0] - 2026-08-28

### Added

- 完成 CopyPolish 0.5.0 正式发布基线。
- 提供 Tauri 2 + React 桌面界面、Rust 排版引擎和实验性 Ratatui TUI。
- 支持规则开关、主题、字体、字号、界面缩放、快捷键和用户设置持久化。
- 支持 Markdown、LaTeX、URL、邮箱、化学式和 Unicode grapheme 边界保护。

### Changed

- 排版规则由 Rust 注册表统一管理，并使用稳定机器 key。
- 格式化生产流程收敛到 span-aware TextEdit 管线。
- 发布流程采用本地或 GitLab 构建、人工校验和 GitHub Release 发布。

### Security

- 使用 SOPS/age 管理发布相关凭据，并加入明文凭据扫描和发布前安全检查。
