# Changelog

本文件记录 CopyPolish 的重要用户可见变化、兼容性变化和工程维护变化。详细历史仍以 Git 提交和 GitHub Release 为准。

## [Unreleased]

### Added

- 新增贡献指南、架构说明和测试指南。
- 新增项目变更记录。

### Changed

- 重组开发文档、文档导航和后续路线图，明确当前事实、操作手册、计划和历史归档的职责边界。
- 统一 GitHub 分支 CI、GitLab tag 构建和本地验证流程的说明。
- 抽离前端规则目录加载（`useRuleCatalog`）和清空输入反馈（`useClearFeedback`），降低 `App.tsx` 编排复杂度。
- 拆分设置界面为独立分区组件（主题、显示、快捷键、规则、状态 Footer），`SettingsDialog.tsx` 负责编排。
- 抽取设置提醒文案与判定到 `frontend/src/lib/settingsLoadNotices.ts`，主界面与设置 Footer 共用，消除重复并精简 `App.tsx`。
- 优化 Markdown 行内代码扫描，降低大量反引号文本的重复查找开销，并保持多长度 delimiter 与未闭合 delimiter 行为。
- 优化结构 span 仲裁，减少复杂 Markdown/LaTeX 文本中的 O(n²) 重叠检查开销，并保持优先级与嵌套结构语义。
- 缩小可编辑规则阶段的保护预扫描范围，仅扫描不透明结构与化学式，减少重复语义扫描开销并保持保护边界。
- 优化专有名词规则的批量替换，减少重复全文遍历，并保持相邻词、前缀词和 ASCII 单词边界语义。

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
