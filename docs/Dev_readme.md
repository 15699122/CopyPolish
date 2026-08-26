# 文案净排（CopyPolish）：开发说明

本文档面向后续维护者，记录当前项目结构、架构边界、开发运行方式、测试命令、打包方式和已知注意事项。普通用户使用说明请参阅 [README.md](README.md)。

## `v0.5.0` 正式发布计划

当前 `v0.5.0` 正式发布的开发、测试、Windows 验收和发布门槛统一记录在 [v0.5.0-release-plan.md](v0.5.0-release-plan.md)。后续开发应按该计划的阶段顺序执行；在黄金样例回归测试体系建立前，不新增格式化规则或大型 UI 功能。

其他文档入口：

- [roadmap.md](roadmap.md)：`v0.5.0` 发布后的中长期开发路线图（快捷键配置、Unicode 引擎升级、ICU4X 评估、E2E、性能基准、hooks 拆分等）；
- [manual-release.md](manual-release.md)：本地构建与手动上传 GitHub Release 的操作 Runbook（备用发布路径）。

> 历史说明：项目早期为 Python + customtkinter 桌面 GUI（入口 `chinese_copywriting_formatter.py`，曾使用 `rules.yaml` 保存设置）。该路线已于 2026-08 彻底移除，相关文件（`gui/`、`python/formatter_bridge.py`、`run.sh`、`packaging/`、PyInstaller 工作流等）均不再存在。当前 Rust 应用使用新的 `rules.yaml` 设置实现，并通过同目录旧版 `ccw-formatter-settings.json` 进行一次性迁移；不复用旧 Python 的读写逻辑。

## 当前架构

```text
Tauri 2
├── frontend/            React + TypeScript + Tailwind v4 + shadcn/ui
│   └── 无边框标题栏：拖动、最小化、最大化、关闭
│   └── src/lib/tauri.ts 受限 command 封装（前端唯一后端入口）
│   └── src/lib/fonts.ts 统一字体令牌（全 UI 共享 --app-font-family）
│   └── src/lib/shortcuts.ts 快捷键 schema、序列化、匹配与校验
│   └── src/hooks/useShortcuts.ts 快捷键监听启停与动作分发
└── src-tauri/
    ├── src/engine/            Rust 原生排版引擎
    │   ├── registry.rs        规则注册表（稳定 key / 元数据 / 默认启用 / key 迁移）
    │   ├── pipeline.rs        格式化主流程（保护 → 逐行规则 → 还原）
    │   ├── protection.rs      Markdown / LaTeX / URL / 邮箱保护层
    │   ├── tokenizer.rs       字符分类 + 化学式识别
    │   ├── unicode_boundaries.rs  UAX #29 grapheme 边界层（roadmap §5）
    │   ├── spans.rs            结构/语义 span 扫描与优先级仲裁脚手架
    │   ├── edit_plan.rs        UTF-8 安全 TextEdit 规划与冲突仲裁脚手架
    │   ├── rule_impls.rs      各条规则的纯函数实现
    │   ├── model.rs           RuleMeta / FormatRequest
    │   └── tests.rs           引擎单元测试
    ├── src/user_settings.rs   用户设置持久化（exe 同目录 rules.yaml，YAML）
    └── src/commands.rs        Tauri command 层
```

- **应用为纯 Rust 实现**：`format_text` / `get_rules` / `get_enabled_defaults` / 设置读写全部由 Rust 提供，构建与打包不依赖 Python。
- **规则注册表驱动**：规则的唯一事实来源是 `src-tauri/src/engine/registry.rs`。每条规则有稳定的机器 key（如 `spacing.cjk-latin`），展示名/分组仅存于元数据；新增规则只需在注册表追加一个 `RuleDef`，command 层、pipeline 与前端均无需改动。历史规则已迁移为独立注册项，当前共 13 条规则（既有规则的效果与默认开关保持不变）。
- **用户设置**：保存在 exe 相同目录的 `rules.yaml`（YAML；见下文），首次运行自动迁移旧版 `ccw-formatter-settings.json`；读取与保存时通过 `normalize_rule_keys` 把旧版中文 key 迁移为稳定 key 并丢弃未知 key。
- **化学式识别**：tokenizer 保守识别含 Unicode 上下标、电荷标记或水合物连接符的片段（`Fe²⁺`、`SO₄²⁻`、`FeCl₂·4H₂O` 等），在规则处理前转为占位符整体保护，为后续新规则提供可靠判定单元。
- **Unicode 边界层**（roadmap §5）：`unicode_boundaries.rs` 基于 `unicode-segmentation` 提供 extended grapheme cluster 切分与保守分类（`Han / Latin / Digit / Other`）。中英插空与中数插空两条规则以 grapheme 为判定单位——emoji ZWJ 序列、肤色修饰符、组合附加符不会被切断；Han 范围表集中维护并已覆盖 CJK Extension B。`BoundaryStrategy::LegacyChars` 仅供新旧策略对比测试，生产固定使用 Graphemes；化学式检测不经过该层，仍沿用保守正则 + 字节区间。Kana/Hangul 首期归为 `Other` 不触发插空，行为由 `tests/fixtures/unicode-boundaries.yaml` 冻结；性能基线见 [unicode-baseline.md](unicode-baseline.md)。
- **规则调度与迁移基础设施**：`registry.rs` 已使用 `RulePhase` 与 `before/after` 依赖进行稳定拓扑排序，并拒绝未知依赖、重复 key 和循环依赖。`spans.rs` 已提供结构/语义 span 扫描与优先级仲裁；`edit_plan.rs` 已提供 UTF-8 安全的 `TextEdit` 创建、冲突仲裁、逆序应用和单位/数学边界规划。两者当前均为迁移脚手架，尚未替换生产 placeholder pipeline。

## 当前开发状态

Tauri 2 迁移与 Rust 主引擎已完成，当前关键状态如下：

- `frontend/`：React + Vite + TypeScript + Tailwind v4 + shadcn/ui 界面已落地。
- `src-tauri/src/engine/`：纯 Rust 可扩展引擎（注册表 + 保护层 + 化学式识别），无 Python/PyO3 运行时依赖，也不受固定规则数量约束。
- `reference/`：历史 Python 引擎与规则目录仅作归档保留，不参与构建、打包与测试门禁。
- 设置 Dialog 使用稳定响应式布局：在视口安全边距内居中并限制最大宽高，固定 header/footer + 原生 `overflow-y-auto` 内容滚动；不再模拟 Dialog 内部拖动/缩放，主 Tauri 窗口仍可拖动和 resize。
- 设置 Dialog 支持主题切换（“跟随系统”为勾选框：勾选时浅色/深色单选项禁用，取消勾选时按 `prefers-color-scheme` 立即回退到显式 light/dark）、主界面缩放与编辑器字号下拉框、界面字体预设（含恢复默认）、Footer 显示完整应用版本 / 保存状态 / 设置文件路径（点状下划线仅作用于路径文本，“设置文件：”标签无下划线；路径截断时悬停或键盘聚焦展示完整值）。
- 主窗口最小尺寸为 `800×600`，用于避免布局过度压缩。
- 输入框 placeholder 固定为「请在这里粘贴或输入文字」（字号更小、颜色更淡）；输出框已有真实空状态提示。
- Linux 打包目标：`.deb` / `.rpm` / `.AppImage`。
- Windows 仅提供无边框便携版：`CopyPolish.exe` 与 `CopyPolish-windows-x64.7z`，不提供安装器。

## 编码说明

全链路统一使用 UTF-8：Rust `String` 原生 UTF-8，Tauri command 走 JSON（UTF-8），设置文件 `rules.yaml`（YAML）以 UTF-8 写入/读取；中文输入输出、emoji、CJK 扩展区字符均有自动化回归测试覆盖。

## 目录结构

```text
├── README.md                      # 用户文档
├── docs/
│   ├── Dev_readme.md              # 开发者文档（本文件）
│   ├── v0.5.0-release-plan.md     # v0.5.0 发布计划与验收门槛
│   ├── roadmap.md                 # v0.5.0 后的中长期开发路线图
│   └── manual-release.md          # 本地构建与手动上传 Release Runbook
├── reference/                     # 历史参考资产（不参与构建、打包与测试门禁）
│   ├── ccw_engine.py              # 历史 Python 实现
│   └── rule_catalog.yaml          # 历史规则元数据
├── frontend/                      # React/Vite/TS/Tailwind v4/shadcn-ui 界面
│   └── src/App.tsx                # 主界面：双栏编辑、设置 Dialog、防抖实时排版
├── scripts/
│   ├── check_version.py           # 版本一致性校验（CI 与本地共用）
│   └── prepare_release_version.py # 发布时把 tag 完整版本写入构建配置
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── src/engine/                # 可扩展 Rust 排版引擎
│   ├── src/user_settings.rs       # 设置持久化 + 单元测试（临时文件）
│   ├── src/commands.rs            # format_text / get_rules / get_enabled_defaults
│   │                              # / get_user_settings / save_user_settings
│   └── src/lib.rs                 # command 注册
```

## Rust 引擎要点（改动前必读）

1. 规则 key 使用稳定的英文点分标识（如 `spacing.cjk-latin`）；中文展示名不参与内部寻址。旧版设置中的中文 key 由 `normalize_rule_keys` 迁移。
2. 保护层正则依赖 lookbehind/backreference，必须使用 `fancy-regex`（`regex` crate 不支持）。
3. 占位符格式为 `\u{E000}CCWPROTECTED{n}\u{E001}`；保护范围包括文档开头 YAML front matter、表格分隔行、HTML block、行内 HTML 标签、常见转义 Markdown 标记、硬换行、引用式链接定义、fenced code block、HTML 注释、LaTeX 环境/display/inline/command、支持嵌套括号目标的 Markdown 图片/链接/autolink、任意长度同 delimiter 的行内代码、URL、邮箱、缩进代码行及化学式。
4. 行内占位符补空格时须过滤跨行值（fenced block 不补空格）。
5. 规则选择由 `FormatRequest.selection` 显式表达：`all` 执行全部规则，`defaults` 执行默认规则，`only` 执行指定 key，`none` 不执行任何规则。用户设置文件继续保存 `enabled: string[]`，其中空数组表示“全不选”，由前端转换为 `none`。
6. 保护规则或 tokenizer 新增/调整后，必须补充 Markdown / LaTeX / URL / 邮箱 / 代码 / 化学式边界测试。
7. `spacing.cjk-latin` 除直接中英相邻外，还处理 Markdown 单星强调片段 `*word*`（word 仅含 ASCII 字母）与 CJK 或比较运算符 `<`/`>`/`=` 相邻的边界；与英文字母/数字相邻（如 `a*b*c`）及 `**粗体**` 不受影响。调整边界字符集时须同步更新 spacing.yaml 黄金样例与幂等用例。
8. `spacing.cjk-latin` 另处理以 Unicode 上标结尾的科学单位片段（如 `mg·mL⁻¹`：字母开头、可含 `·` 连接段、以上标字符结尾）与相邻中文之间的空格；片段内部不改写，化学式在保护层已转为占位符不会进入该规则。浏览器预览的 `fallbackFormat` 须同步对应边界行为。
9. 复杂排版增强（多行文本、特殊单位 `μm/µm`、`Å/Å`、`Ω`、数学符号 `∂/±/≤`、Markdown 结构、Unicode 边界）按 `docs/roadmap.md` §5 的阶段推进，遵循「测试先行、保守保护、能力分层、不默认改写原文」；阶段 A（测试先行）优先于任何新规则或保护层改动落地。
10. 单位识别采用有限词典 + 复合语法（`unit_lexicon.rs` / `semantic_tokens.rs`），当前覆盖 `cm`、`cL`、`hPa` 等显式常用单位；不把正则直接扩展为 `\p{L}+`，避免把自然语言英文、变量名、产品名误判为单位。
11. 阶段 C 第一批已将 `spacing.number-unit` 接入有限单位词典：支持 `μm/µm`、`Å/Å`、`Ω/kΩ`、`°C/°F` 与常见复合科学单位；`semantic_tokens.rs` 同时提供明确数学表达式的保守扫描（`∂f/∂x`、`x≤y`、`a≈b`、`3±0.5`、`2×3`），表达式内部保护、仅在 Han 直接边界补空格，不在全角标点后添加额外空格。单位扫描不使用 look-around，边界通过 Rust 字节区间检查完成。
12. Markdown 安全处理：检测到明显 Markdown 标记时默认启用安全模式（宁漏格式化、不破坏结构）；当前采用“手写扫描器 + 有限 `fancy-regex` + 占位符”的混合保护层，块级（front matter / fenced、缩进代码 / HTML 注释 / 表格分隔行 / 引用式链接定义）与行内（任意长度反引号 / 链接平衡括号 / HTML 标签 / 美元定界行内与展示数学 / 转义字符）保护保持「保护 → 仅格式化可编辑区间 → 还原」的管线。后续重构目标是统一为可仲裁的 span / edit plan，而不是继续堆叠正则。
13. `registry.rs` 的 `RulePhase` 与 `before/after` 是规则调度的显式元数据：pipeline 通过 `execution_rules()` 做稳定拓扑排序，同 phase 使用注册表顺序作为 tie-break；未知依赖、重复 key 和循环依赖会被拒绝。`rules()` 仍用于稳定的 UI 展示顺序。
14. `spans.rs` 提供统一 `SpanKind` / `SpanPriority` / `TextSpan` 与重叠仲裁；当前已汇总化学式、有限单位、Unicode 数学 token，以及 fenced code、front matter、HTML block/comment、引用式链接定义、缩进代码、表格分隔行、行内代码、Markdown 链接和美元数学等结构 span，但仍不改变现有 placeholder pipeline。`edit_plan.rs` 提供 UTF-8 安全的 `TextEdit` 创建、冲突仲裁、逆序应用和语义边界编辑规划；其中单位/数学边界规划只在测试路径验证，仍未接管生产 pipeline。
 15. Unicode 等价识别与输出规范化分离：识别层把 `µ/μ`、`Å/Å` 视为等价语义，输出层默认不改写用户原文；如提供统一表示，须作为独立、默认关闭的规范化规则并评估 NFKC 影响。
 16. 复杂输入测试应至少覆盖结构保护、语义 token、普通文本规则三层同时命中的场景；新增规则或保护层改动必须补充复杂组合 fixture、规则选择组合、优先级和幂等性测试。当前 `structure-precedence.yaml` 作为 pending 基线记录行内代码与单位/数学扫描的优先级冲突，不能把该差异误标为稳定行为。

## 当前开发进度摘要（2026-08-26）

- **已完成**：阶段 A 测试分层、阶段 B Unicode grapheme 边界、阶段 C 第一批有限单位词典/语义 token、阶段 D 首批 Markdown 保护能力。
- **已完成**：快捷键总开关与自定义绑定（roadmap §4 阶段 A/B；持久化、冲突校验、IME 防护、恢复默认）。
- **已完成的调度重构基础**：`RulePhase`、`before/after` 依赖、稳定拓扑排序、依赖图错误检测。
- **已完成的 Span/Edit 基础**：结构与语义 span 扫描、重叠仲裁、UTF-8 安全 `TextEdit`、非重叠编辑逆序应用、单位/数学边界编辑规划。
- **未完成**：Span/Edit 尚未接管 `format_text`；现有生产路径仍是 placeholder + 逐行 `fn(&str) -> String` 规则；`structure-precedence.yaml` 仍是 pending 基线。编辑计划与旧 placeholder 路径的逐例 diff 对照已建立（`tests.rs::edit_plan_path_matches_placeholder_pipeline_on_semantic_fixtures`）；文本边界（Han↔Latin/Han↔Digit）已纳入编辑计划并消除 10 例差异，剩余 5 例（selection 门控、温标/‰ 规则、`$…$` 结构保护）冻结在测试内 `PENDING_DIFFS` 清单。
- **未完成**：完整单位词典、温度规则独立 stable key、Unicode 等价识别/默认关闭规范化、1 MB 长文本性能基准、真实 Tauri E2E、Windows 10/11 真机验收和正式 `v0.5.0` 发布。
- **当前验证基线**：Rust 单元测试 70 项、前端 Vitest 33 项；fmt、Clippy、前端构建和 diff 检查均纳入本地/CI 验证。

## 用户设置持久化

- 文件位置：**exe 相同目录**（`std::env::current_exe()` 的父目录；无法定位时回退 cwd）下的 `rules.yaml`（YAML）：
  ```yaml
  enabled:
    - spacing.cjk-latin
  last_input: ...
  theme: system
  font: system
  editor_font_size: normal
  ui_scale: normal
  shortcuts:
    enabled: true
    bindings:
      format_now: CtrlOrCmd+Enter
      copy_output: CtrlOrCmd+Shift+KeyC
      open_settings: CtrlOrCmd+Comma
  ```
- 字段：`enabled`（启用规则）、`last_input`（最近输入）、`theme`（`system` / `light` / `dark`）、`font`（字体预设）、`editor_font_size`（`small` / `normal` / `large` / `x-large`）、`ui_scale`（`compact` / `small` / `normal` / `large` / `x-large`）、`shortcuts`（快捷键总开关与绑定，缺省回退为启用 + 默认组合键）。旧版设置文件缺少新增字段时分别回落为默认值。
- 迁移：`rules.yaml` 不存在但同目录存在旧版 `ccw-formatter-settings.json` 时，自动读取旧 JSON 并转换写入新 YAML。
- 保存策略：写临时文件 `rules.yaml.tmp` 并 `sync_all` 后原子 rename 到 `rules.yaml`；替换前将上一份有效文件轮换为 `rules.yaml.bak`，避免中途退出产生半截文件，并为主文件损坏提供恢复来源。目标是 exe 同目录，目录不存在或无写权限时返回带完整路径的诊断错误（前端在设置弹窗中展示，提示把便携版放到可写目录）。
- 实现：`user_settings.rs` 提供 `load_from/save_to`（可注入路径）、`load_from_dir_with_status`（区分旧版本迁移、主设置损坏、备份损坏和恢复状态）、`load_from_dir`（兼容返回设置内容）与 `load_with_status`（exe 目录）；command 层暴露 `get_user_settings`（返回 `notices` 提醒列表）、`save_user_settings`（保存前过滤未知规则 key并保存字号/缩放）与 `get_settings_path`（返回设置文件完整路径，供界面显示）。
- 前端行为：启动恢复；规则开关/全选/恢复默认/清空/主题/字体即时保存，输入防抖（160ms）保存；浏览器预览回退 localStorage。字体使用固定跨平台预设与 CSS fallback 栈，不尝试枚举系统已安装字体。
- 快捷键：`frontend/src/lib/shortcuts.ts` 集中维护动作 key、默认绑定（`CtrlOrCmd` + `KeyboardEvent.code` 序列化）、事件匹配、IME 防护与校验（必须含 Ctrl/Cmd、动作间禁止重复、系统黑名单、允许按键白名单，`Comma` 作为默认值兼容例外）；`frontend/src/hooks/useShortcuts.ts` 负责监听启停（总开关关闭时不注册监听器）与动作分发；录制交互在 SettingsDialog 的“快捷键”分区完成，冲突/保存反馈通过 `aria-live` 输出；Esc 始终交给 Radix Dialog。
- 长文本排版：普通文本使用 160ms 防抖，达到 50,000 字符使用 450ms，达到 200,000 字符使用 900ms；界面显示排版中、长文本和最近一次耗时提示，并通过序列号丢弃过期结果。
- 设置 Dialog 的版本号通过 `getAppVersion()` 读取：打包环境使用 Tauri `getVersion()`，浏览器预览使用 Vite 从 `frontend/package.json` 注入的 `__APP_VERSION__` 回退值，避免重复维护版本常量。
- 设置弹窗 Footer 保持与主界面一致的 `py-4` 纵向内边距；桌面端单行显示版本/保存状态/设置路径与操作按钮，小屏或较长保存错误时采用 flex-wrap 响应式换行。
- 预发布版本一致性：release 工作流在测试校验后调用 `scripts/prepare_release_version.py <tag>`，把 tag 的完整版本（如 `v0.5.0-pre1` → `0.5.0-pre1`）写入 package.json / package-lock.json / tauri.conf.json / Cargo.toml / Cargo.lock，仅作用于构建工作区（CI runner 或本地隔离发布 worktree），不回写源码提交；`check_version.py` 同时接受源码基础版本与同步后的完整版本两种状态。
- 主界面输入框与输出框共享 `--app-font-family`、`--editor-font-size` 和 `--editor-line-height`；textarea 基础组件不再自带 `text-base`/`md:text-sm` 字号工具类，保证 `.editor-text` 的字号变量对输入框与输出框同时生效。主界面内容通过 `--app-ui-scale` 缩放。设置文件路径在 Footer 中单行截断，保留 `title` 和 `aria-label`，不再显示自定义悬停浮层；灰色点状下划线仅作用于路径文本。输入框 placeholder 固定为「请在这里粘贴或输入文字」。
- 设置窗口的“字体”部分通过下拉框支持字号预设（13/14/16/18px），“主题”部分通过下拉框支持主界面缩放预设（80/90/100/110/125%）。旧 `rules.yaml` 缺少新字段时回退为标准字号与 100% 缩放。
- 设置窗口中的规则列表仅做展示排序（默认开启在上、默认关闭在下，组内保持注册表顺序），不改变 Rust 注册表数组顺序与 pipeline 执行顺序。
- 设置加载提醒包括旧版本设置迁移、旧设置损坏、主设置损坏后的备份恢复、主/备份均损坏以及备份损坏但主设置可用等状态；提醒会显示在主界面提示条和设置文件区域。
- 测试约定：所有设置读写测试一律使用系统临时目录中的唯一随机文件（PID + 计数器），禁止写仓库内固定路径。
- 该文件已加入 `.gitignore`（根目录 `/rules.yaml`）。

## 开发环境

常用变量：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

依赖：Node.js/npm、Rust 工具链、Linux 下 Tauri/WebKitGTK 系统依赖。应用构建与测试无需 Python。

工具链版本通过仓库文件固定：

- Node.js：`.nvmrc`（当前 24.19.0，与 GitHub Actions 一致）；
- Rust：`rust-toolchain.toml`（当前 1.98.0，包含 rustfmt / clippy）；
- 前端依赖：`frontend/package-lock.json` + `npm ci`；
- Rust 依赖：`src-tauri/Cargo.lock`。

建议本地开发前执行：

```bash
nvm use
npm ci --prefix frontend
```

依赖更新不在普通 CI 构建中自动执行；Dependabot 会为 npm、Cargo 和 GitHub Actions 依赖创建独立 PR，合并前必须通过 CI。

## 分支开发流程

默认在 `dev` 分支开发，`master` 只保留已验证的稳定代码：

使用 Cline Act Mode 修改本地文件时，完成实现与必要验证后应自动同步本次修改结果至 Markdown 文档，并同步到 GitHub 远程仓库的相应分支：

1. 先检查 `git status --short --branch`，确认当前分支、远程跟踪分支与待提交文件；
2. 查看 diff，确保提交范围只包含本次任务相关更改；
3. 审阅仓库 Markdown 文档并按本次修改的影响范围同步：用户可见的功能、用法、限制、下载/运行方式或设置行为有变化时更新 `README.md`；架构、开发流程、实现约束、测试/构建命令、CI 或发布流程有变化时更新 `Dev_readme.md`；
4. 若本次修改不改变已有文档描述，不做无意义改写，但必须在最终结果中明确说明“已审阅 Markdown 文档，确认无需更新”；文档更新应与代码修改一起纳入本次 diff、验证、提交与推送；
5. 运行与本次修改相关的必要验证；
6. 使用清晰的 commit message 提交；
7. 推送到当前分支对应的远程分支（通常为 `origin/<当前分支>`，例如 `dev -> origin/dev`）；
8. 推送后再次检查 `git status --short --branch`，确认本地分支与远程分支已同步。

```bash
git switch dev
git pull --ff-only origin dev

# 开发、测试、提交
git push origin dev

# 功能确认后创建 PR：dev → master
gh pr create --base master --head dev
```

稳定发布仅从 `master` 创建 `v*` tag；后端引擎或其他重大功能变更应先从 `dev` 推送预发布 tag（如 `v0.5.0-pre2`，tag 可指向 `dev` 提交），Release 会自动标记为 pre-release、不占用 latest。Release 名称应直接使用 tag 名称（例如 `v0.5.0-pre2`），不要额外添加 `CopyPolish` 前缀；Release Notes 由 GitHub 自动生成后再人工审阅：

```bash
git switch dev
git pull --ff-only origin dev
git tag v0.5.0-pre2
git push origin v0.5.0-pre2
```

## 启动 / 构建

```bash
cd frontend && npm run tauri dev       # 开发运行
cd frontend && npm run tauri build     # Linux deb/rpm/AppImage
cd frontend && npm run tauri build -- --no-bundle  # Windows 便携 exe（不生成安装器）
```

构建产物位置：

- Linux bundle：`src-tauri/target/release/bundle/`
- Windows 便携 exe：`src-tauri/target/release/chinese-copywriting-formatter.exe`，发布流水线会重命名为 `CopyPolish.exe`

Windows `.7z` 压缩包约定：根目录直接包含 `CopyPolish.exe` 及构建目录中存在的旁置 DLL 依赖（如有），不得包含 `dist`、`windows` 或其他上级目录。

## 验证命令

开发提交前建议至少运行：

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml          # Rust 引擎与设置测试
npm test --prefix frontend                                # vitest 组件测试（jsdom，10 项）
npm run build --prefix frontend                           # tsc + vite
```

与 CI 对齐的快速验证命令：

```bash
npm ci --prefix frontend
npm test --prefix frontend -- --run
npm run build --prefix frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

## 重要实现约束

1. 前端只能通过 `frontend/src/lib/tauri.ts` 的封装访问后端，不直接 `invoke`。
2. `reference/` 仅作历史参考，不定义 Rust 引擎行为。
3. 改动规则定义时，只需更新 `src-tauri/src/engine/registry.rs`、对应规则实现及 Rust 回归测试；前端通过 command 动态读取 metadata。
4. 打包资源变更需同步 `tauri.conf.json` 的 `bundle.resources` 并做安装态 smoke。
5. Tailwind 优先使用间距刻度类（如 `min-h-130` = 130 × 0.25rem），仅当数值不在刻度上时才用 `[...]` 任意值写法，避免 lint 警告。

## 后续计划

1. 真实 Windows 机器人工验收：下载 CI 的 `windows-portable` artifact，运行 `.exe`，输入 `在LeanCloud上，花了5000元` → 应输出 `在 LeanCloud 上，花了 5000 元`；验证设置弹窗 13 条规则、开关即时重排、设置文件持久化、高 DPI 显示。
2. 正式发布 `v0.5.0`：人工复核 Release 资产、Release Notes、版本号和 latest 标记。
3. `v0.5.0` 发布后的中长期工作（复杂排版与 Unicode 基础能力增强（多行/Markdown/特殊单位/数学符号）、Unicode 引擎升级、ICU4X 评估、E2E、性能基准、其余 hooks 拆分等）统一在 [roadmap.md](roadmap.md) 跟踪，其中复杂排版增强的详细阶段计划见其 §5。本地构建自动化脚本与快捷键总开关/自定义绑定已完成并合入 `dev`。

## 图标

完整桌面图标集（ico/各尺寸 PNG）由 Tauri CLI 从 `icons/icon.png` 生成：
`./frontend/node_modules/.bin/tauri icon src-tauri/icons/icon.png -o src-tauri/icons`（生成后删除 `icon.icns`——项目仅支持 Linux/Windows）。更换设计稿后重跑该命令即可。

## 持续集成

`.github/workflows/ci.yml` 与 `.github/workflows/release.yml`：

- `ci.yml` 在 `dev`、`master` 与 PR 上运行快速验证：cargo fmt + cargo clippy（`-D warnings`）+ 纯 Rust cargo test（当前包含黄金样例、设置备份恢复和 UTF-8 多字节回归）+ `git diff --check` + 前端 vitest 组件测试 + tsc/vite 构建；配置了 concurrency（同一 PR / 分支新 commit 自动取消过时 run）与 `timeout-minutes: 20`；纯 Rust 测试不安装 GTK/WebKitGTK 系统依赖；
- `release.yml` 在推送 `v*` tag 时运行完整发布流水线，也支持 workflow_dispatch 手动指定既有 tag 并可选标记 pre-release（tag 名含 `-` 时自动识别为预发布）；各 job 均设置 timeout 上限；
- 构建阶段拆分为 `linux-build` 与 `windows-build` 两个独立 job 并行执行：Linux 构建 deb/rpm/AppImage 并上传 `bundle-ubuntu-latest`；Windows 以 `--no-bundle` 构建无边框便携 `.exe`，以临时根目录打包为只含 exe/DLL 的 `.7z` 后上传 `windows-portable`（不生成任何安装器——WiX MSI 无法处理中文产品名，且产品定位为免安装便携版）。项目不支持 macOS。发布类 artifact 设置了短保留期（Linux bundle 与 Windows portable 均为 3 天、Windows 失败日志 1 天）并关闭二次压缩（`compression-level: 0`），长期下载一律以 GitHub Release assets 为准；
- `windows-smoke` job（windows-latest）：直接下载 `windows-build` 上传的同一份 `CopyPolish.exe` 做 GUI 冒烟——启动进程 → 轮询等待主窗口句柄出现（最长 60 秒）→ 保持 10 秒验证稳定性 → 强制结束进程；不再重复安装工具链或重新编译 Tauri。

- 存储治理：GitHub Actions artifacts 仅作为 job 间传递与短期人工验收的临时副本（保留 1–3 天），长期可下载来源一律以 GitHub Release assets 为准；本地 `src-tauri/target/` 是可随时删除的可再生构建缓存（受 `.gitignore` 覆盖），不提交、不视为源码。系统保留各 workflow 最近一次成功与失败 run 用于诊断，其余已完成的旧 run 可安全删除。

除 GitHub Actions 自动构建外，项目同样支持**本地构建 + 手动上传 GitHub Release** 的备用发布路径；两种模式共享相同的验证门槛、版本脚本、资产命名与人工验收标准，操作步骤见 [manual-release.md](manual-release.md)。`prepare_release_version.py` 可在 CI runner 或隔离的本地发布工作区（独立 clone / Git worktree）执行，禁止在待提交的日常开发工作区直接运行。

CI/Release 会打印 Node/npm/Rust/Cargo/系统版本，便于核对本地与 Runner 环境差异。

发布产物命名约定：

| 平台 | Artifact | Release 资产 |
| --- | --- | --- |
| Linux | `bundle-ubuntu-latest` | `CopyPolish_linux_amd64.deb` / `CopyPolish-linux-x86_64.rpm` / `CopyPolish_linux_amd64.AppImage` |
| Windows | `windows-portable` | `CopyPolish.exe` / `CopyPolish-windows-x64.7z` |

Release Notes 由 GitHub 自动生成后，必须在正式发布前重新审查并按需编辑；Release 标题保持与 tag 一致，不添加产品名前缀：

- 确认自动生成内容覆盖本次更新的大致范围（功能、修复、规则调整、构建/发布流程变化等），不要只保留 commit/PR 标题而遗漏用户可感知的变化；
- 对比上一版 Release Notes，删除或改写与既有版本重复的描述，避免把旧版本已发布的内容再次列为“本次更新”；
- 保留并按需更新固定说明（Windows 便携版命名、设置迁移、规则调整、已知限制等）；
- 最终发布前再检查资产列表与 Release Notes 是否一致，特别是 Linux `.deb` / `.rpm` / `.AppImage` 与 Windows `CopyPolish.exe` / `.7z` 是否均已说明清楚。
