# CopyPolish 架构说明

本文记录当前实现事实。开发流程见根目录 [CONTRIBUTING.md](../CONTRIBUTING.md)，测试策略见 [testing.md](testing.md)。

## 1. 总体结构

```text
React + TypeScript UI
        │
        │ frontend/src/lib/tauri.ts
        ▼
Tauri commands
        │
        ├── Rust engine::format_text
        └── user_settings（rules.yaml）

Ratatui TUI ───────────────┘
```

应用是 Tauri 2 桌面程序。前端负责界面状态和交互，Rust 负责排版行为、设置读写和 TUI。GUI 与 TUI 共用同一个 Rust 排版引擎，不维护两套规则实现。

## 2. 目录职责

```text
frontend/src/
├── App.tsx                    主界面编排
├── components/                标题栏、设置窗口和 UI 基础组件
├── hooks/                     格式化、设置、主题、快捷键和窗口控制
├── lib/tauri.ts               前端唯一的 Tauri IPC 封装
└── test/                      Vitest 全局测试环境

src-tauri/src/
├── commands.rs                Tauri command 边界
├── user_settings.rs           rules.yaml、备份和旧 JSON 迁移
├── engine/
│   ├── registry.rs             稳定 key、元数据、默认状态和依赖
│   ├── pipeline.rs             唯一生产格式化入口
│   ├── spans.rs                结构/语义 span 扫描与仲裁
│   ├── protection.rs           Markdown、LaTeX、URL、邮箱和化学式保护
│   ├── tokenizer.rs            字符分类和化学式识别
│   ├── semantic_tokens.rs      单位与数学语义识别
│   ├── unicode_boundaries.rs   grapheme 边界和字符分类
│   ├── edit_plan.rs            UTF-8 安全 TextEdit 规划与应用
│   ├── rule_impls.rs            规则纯函数
│   └── tests.rs                 引擎测试入口
└── tui/                        Ratatui/Crossterm 终端界面
```

工程脚本在 `scripts/`，平台专属 CI 脚本在 `scripts/ci/`。可再生目录 `frontend/node_modules`、`frontend/dist`、`src-tauri/target` 和 `src-tauri/gen` 不属于源码。

## 3. 请求和格式化数据流

### GUI

```text
用户输入
  → App/hooks
  → lib/tauri.ts
  → commands::format_text
  → engine::format_text
  → registry + span scanner + TextEdit + protection
  → 输出结果
```

浏览器预览模式只提供最小 JS fallback，用于脱离 Tauri 开发 UI；它不代表完整 Rust 引擎行为。

### Rust 格式化管线

1. 统一换行符并记录原始换行风格；
2. 在可编辑区间执行标点和名词规则；
3. 扫描结构/语义 span；
4. 将不可编辑结构转换为内部保护占位符；
5. 在受保护文本上执行结构边界、文本边界和清理规则；
6. 处理占位符边界空格；
7. 还原原文并恢复换行风格。

所有生产规则阶段都经过 `edit_plan.rs` 的 TextEdit 应用层。保护层仍可使用内部 placeholder，但不存在第二套生产 pipeline。

## 4. 规则注册表

`src-tauri/src/engine/registry.rs` 是规则的唯一事实来源。每个 `RuleDef` 包含稳定机器 key、展示元数据、默认启用状态、执行阶段、依赖、legacy key 别名和规则实现函数。

前端通过 `get_rules` 动态取得元数据，因此新增规则通常不需要修改前端。新增规则必须同步测试、README 规则表、设置迁移兼容性和 CHANGELOG。

## 5. 设置和兼容性

桌面端设置默认保存在程序同目录的 `rules.yaml`，损坏时尝试 `.bak`，首次发现旧版 `ccw-formatter-settings.json` 时进行迁移。读取和保存会把旧规则 key 归一化为稳定 key，并丢弃未知 key。程序目录不可写时（ADR 已采纳方案 B，见 `docs/decisions/settings-storage-policy.md`），启动时一次性决策回退到平台应用数据目录并通过 `UsingAppDataFallback` 提醒前端；实际路径经 `get_settings_path` 展示。程序目录与应用数据目录同时存在时，优先使用程序目录设置。

TUI 通过自己的设置门面复用同一文件，但只修改规则选择和最近输入。前端浏览器预览使用 localStorage fallback，不代表桌面持久化实现。

## 6. 常见修改入口

| 任务 | 首选文件 |
| --- | --- |
| 修改规则行为 | `engine/rule_impls.rs`、`engine/registry.rs`、Rust fixture/测试 |
| 修改保护边界 | `engine/spans.rs`、`engine/protection.rs`、保护 fixture |
| 优化 inline-code 扫描 | `engine/spans.rs::scan_inline_code_spans`、结构 span 回归测试 |
| 优化 span 仲裁 | `engine/spans.rs::arbitrate_spans`、span 优先级回归测试 |
| 修改单位/数学识别 | `engine/semantic_tokens.rs`、`engine/unit_lexicon.rs` |
| 修改前端格式化状态 | `frontend/src/hooks/useFormatter.ts`、`useInputFormatting.ts` |
| 修改前端页面编排 | `frontend/src/hooks/useAppController.ts`、`frontend/src/App.tsx` |
| 修改规则目录加载 | `frontend/src/hooks/useRuleCatalog.ts` |
| 修改清空输入反馈 | `frontend/src/hooks/useClearFeedback.ts` |
| 修改设置提醒文案/判定 | `frontend/src/lib/settingsLoadNotices.ts` |
| 修改设置行为 | `user_settings.rs`、相关 settings hook、`SettingsDialog.tsx`、`components/settings/` |
| 修改 IPC | `commands.rs` 和 `frontend/src/lib/tauri.ts` |
| 修改 TUI | `src-tauri/src/tui/`，不得复制引擎规则 |
| 修改发布流程 | `scripts/verify.py`、`scripts/ci/`、`docs/release/manual-release.md` |

## 7. 不应违反的边界

1. 前端不直接调用任意 Tauri command；窗口控制经 `@tauri-apps/api/window`（非 IPC 后端命令），Tauri 环境判断仍来自 `lib/tauri.ts`；
2. TUI 不复制格式化规则；
3. 不以历史 Python 实现作为当前行为基准；
4. 不使用默认全文 NFKC 改写用户文本；
5. 保护策略优先避免破坏 Markdown、LaTeX、URL、邮箱、代码和化学式；
6. 变更发布配置时必须同步版本、资产校验和 Runbook。