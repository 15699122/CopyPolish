# 文案净排（CopyPolish）

> 项目名称：**文案净排**（英文：**CopyPolish**）。源码仓库：<https://github.com/15699122/CopyPolish>

文案净排（CopyPolish）是一款本地优先的中文文本清洗与规范排版工具，适用于网页文案、技术文档以及从 PDF、CAJ、Zotero 等来源复制的文本。它按照 [chinese-copywriting-guidelines](https://github.com/sparanoid/chinese-copywriting-guidelines) 的简体中文文案规范，整理中文、英文、数字、单位和标点之间的格式，并提供首批来源文本清洗、自定义字面量替换和可选简繁转换能力。

应用采用 Tauri 2 + React + shadcn/ui 桌面界面，排版引擎由 Rust 实现。左侧输入原文，右侧实时显示规范化结果；规则可逐条启用或关闭。

## 功能亮点

- **实时排版**：输入或粘贴文本后自动生成格式化结果。
- **左右双栏编辑**：输入区和输出区并排显示，适合对照检查。
- **规则可配置**：设置窗口支持逐条启用/停用规则，也支持全选、全不选和恢复默认。
- **文本替换与转换可配置**：设置窗口支持按顺序管理自定义字面量替换，并选择不转换、繁体转简体或简体转繁体。
- **深色 / 浅色 / 跟随系统主题**：设置中可切换主题，选择会持久化，下次启动自动恢复。
- **界面字体可配置**：设置中可选择常用系统字体预设，也可以恢复默认字体；未安装的字体会自动回退。
- **争议规则默认关闭**：例如链接之间增加空格、简体中文使用直角引号，可按个人习惯开启。
- **Markdown / LaTeX 保护**：尽量避免误改代码块、行内代码、链接、图片链接、URL、邮箱和公式内容。
- **Unicode 边界安全**：排版按 grapheme cluster 判定中英/中数边界，emoji 组合序列（如 ZWJ 家庭 emoji）与组合附加符不会被拆开；CJK 扩展区汉字（如 `𠀀`）与普通汉字同样参与插空。
- **本地优先**：文本处理在本机完成，不依赖远程服务。
- **用户设置持久化**：默认保存在程序同目录的 `rules.yaml`；程序目录不可写时自动回退到平台应用数据目录，启动时自动恢复。

### 功能分层

当前版本已经提供**规范排版**、首批**来源文本清洗**、自定义字面量替换和 GUI 设置中的简繁转换选项；更复杂的来源清洗和字符转换能力仍是后续扩展方向：

- **规范排版**：中文、英文、数字、单位和标点的上下文相关格式化，并保护 Markdown、LaTeX、URL、邮箱、代码和化学式；
- **来源文本清洗**：可选清理普通文本中的方括号引用角标、连续 ASCII 空格和连续空行；PDF/CAJ 软换行、圆括号引用和更复杂的异常字符清洗仍在规划中；
- **字符转换**：GUI 设置支持互斥的简繁转换（T2S/S2T，基于 OpenCC 风格词典，只改写可编辑区间、保护链接/代码/公式）；实际转换依赖 `simplified-trad-conversion` feature。默认构建会明确显示能力不可用、禁用 T2S/S2T，并将不可用选择归一化为“不转换”；更完整的全角/半角处理与有限 Unicode 等价字符仍在规划中。

应用不解析 PDF/DOCX 文件本体，只处理粘贴或文本文件中的文本；也不提供翻译、AI 写作或在线语法检查服务。

## 下载与运行

请在 GitHub Releases 页面下载对应平台产物。

源码开发、Issue、Pull Request、版本 tag 和正式 Release 均以 GitHub 仓库为准；
GitLab 仅作为 Linux/Windows Release 构建服务，不作为日常开发或公开下载入口。

| 平台 | 产物 | 说明 |
| --- | --- | --- |
| Windows | `CopyPolish.exe` | 便携版，直接运行 |
| Windows | `CopyPolish-windows-x64.7z` | 压缩包，根目录直接包含 exe |
| Linux | `.deb` / `.rpm` / `.AppImage` | 根据发行版选择 |

Windows 版依赖系统的 WebView2 Evergreen Runtime。Windows 10/11 通常已内置；如无法启动，请从微软官网安装 WebView2 Runtime。

> 注意：Windows 版为便携版，不提供安装器。程序目录可写时设置保存在同目录；若目录不可写，应用会自动回退到 `%APPDATA%\CopyPolish`。

## 基本使用

1. 启动应用。
2. 在左侧输入框粘贴或输入中文文案。
3. 右侧输出框会实时显示规范化结果，例如：`在 LeanCloud 上，花了 5000 元`。
4. 点击 **复制结果** 将输出复制到剪贴板并保留当前内容；或点击 **复制并清空**，在复制成功后同时清除输入和输出。
5. 点击 **清空输入** 仅清除当前文本和输出，不会自动复制。
6. 首次使用时可从提示条打开 **帮助**；之后也可随时点击底部 **帮助** 查看静态说明。帮助会说明高风险规则、结构保护、输出/复制动作以及浏览器演示模式边界。
7. 点击 **设置** 打开设置窗口：
   - 勾选或取消勾选单条规则（默认开启的规则显示在上方，默认关闭的规则显示在下方）；
   - 使用全选、全不选、恢复默认；
   - 在“快捷键”分区启用或关闭应用快捷键，修改各动作的组合键或恢复默认；
   - 勾选 **跟随系统** 让主题自动跟随操作系统的浅色/深色模式；取消勾选后可手动选择浅色或深色（取消时会先切换为当前系统对应的主题）；
   - 通过下拉框选择界面缩放（80%–125%）与编辑器字号（小/标准/大/特大，同时作用于输入框与输出框）；
   - 选择界面字体或恢复默认字体；
   - 在“文本替换与转换”分区按顺序添加、编辑、启停或删除字面量替换，并选择简繁转换模式；
   - 将鼠标悬停在任一规则上可查看该规则的作用示例；
   - 在底部查看当前完整应用版本（预发布构建会带 pre 后缀）与设置保存状态；
   - 查看设置文件 `rules.yaml`：悬停可查看完整路径，点击可复制完整路径；
   - 点击 **完成** 保存并返回主界面。

无边框窗口可通过顶部标题栏拖动，右上角按钮可控制最小化、最大化和关闭。

## 快捷键

默认快捷键（`Ctrl` 在 macOS 上对应 `Cmd`）：

| 动作 | 默认组合键 |
| --- | --- |
| 立即排版 | `Ctrl/Cmd + Enter` |
| 复制结果 | `Ctrl/Cmd + Shift + C` |
| 打开设置 | `Ctrl/Cmd + ,` |

- 设置窗口的“快捷键”分区可关闭全部应用快捷键，或逐个修改组合键（点击“修改”后按下新组合键，Esc 取消）；
- 输入法组合中的按键不会触发快捷键；未匹配的按键不会被拦截，不影响正常输入；
- 动作之间不允许重复绑定；系统/窗口高风险组合键会被拒绝；
- “恢复默认快捷键”一键还原所有绑定与总开关。

## 当前支持的规则

规则由后端注册表统一管理，当前内置 21 条（文本清洗规则和「专有名词使用正确的大小写」「不要使用不地道的缩写」「链接之间增加空格」「简体中文使用直角引号」「统一等价 Unicode 单位字符」「ASCII 字符使用半角形式」「修复数值标点异常空格」「修复康熙部首」「清理中文之间异常空格」默认关闭）。其中 `cleanup.cjk-internal-space` 目前仍是未经真实 PDF/CAJ 语料验收的保守试实现。

| 分类 | 类型 | 风险 | 规则 | 稳定 key |
| --- | --- | --- | --- | --- |
| 文本清洗 | 清洗 | 需复核 | 删除方括号引用角标，默认关闭 | `cleanup.reference-square` |
| 文本清洗 | 清洗 | 需复核 | 折叠连续空格，默认关闭 | `cleanup.collapse-horizontal-spaces` |
| 文本清洗 | 清洗 | 需复核 | 限制连续空行，默认关闭 | `cleanup.limit-blank-lines` |
| 文本清洗 | 清洗 | 需复核 | 修复康熙部首，默认关闭 | `cleanup.kangxi-radicals` |
| 文本清洗 | 清洗 | 需复核 | 清理中文之间异常空格，默认关闭 | `cleanup.cjk-internal-space` |
| 标点符号 | 排版 | 需复核 | 不重复使用标点符号 | `punctuation.no-repetition` |
| 全角和半角 | 排版 | 需复核 | 使用全角中文标点 | `punctuation.fullwidth-cjk` |
| 全角和半角 | 转换 | 低风险 | 数字使用半角字符 | `text.halfwidth-digits` |
| 全角和半角 | 转换 | 需复核 | ASCII 字符使用半角形式，默认关闭 | `text.halfwidth-ascii` |
| 全角和半角 | 排版 | 需复核 | 遇到完整的英文整句、特殊名词，其内容使用半角标点 | `text.ascii-punct-in-latin` |
| 全角和半角 | 转换 | 需复核 | 统一等价 Unicode 单位字符，默认关闭 | `text.unicode-equivalents` |
| 名词 | 排版 | 需复核 | 专有名词使用正确的大小写，默认关闭 | `naming.proper-nouns` |
| 名词 | 排版 | 需复核 | 不要使用不地道的缩写，默认关闭 | `naming.expand-abbreviations` |
| 争议 | 排版 | 需复核 | 链接之间增加空格，默认关闭 | `spacing.around-links` |
| 争议 | 排版 | 需复核 | 简体中文使用直角引号，默认关闭 | `punctuation.corner-quotes` |
| 空格 | 排版 | 低风险 | 中英文之间需要增加空格 | `spacing.cjk-latin` |
| 空格 | 排版 | 低风险 | 中文与数字之间需要增加空格 | `spacing.cjk-number` |
| 空格 | 排版 | 需复核 | 数字与单位之间需要增加空格 | `spacing.number-unit` |
| 空格 | 清洗 | 需复核 | 修复数值标点异常空格，默认关闭 | `spacing.numeric-punctuation` |
| 空格 | 排版 | 需复核 | 摄氏度/华氏度符号与中文之间加空格 | `spacing.temperature-cjk` |
| 空格 | 排版 | 低风险 | 全角标点与其他字符之间不加空格 | `spacing.no-space-around-fw-punct` |

> 说明：排版规则尽量贴近规范，但自然语言文本存在上下文差异。建议在重要文案发布前人工复核一次。

> `spacing.numeric-punctuation` 默认关闭，只修复明确的数字内部异常 ASCII 空格；版本号/IP 等连续点号数字链会保留原样，重要数据仍建议人工复核。

> 规划中的 PDF 文本清洗规则可能改变空格、断行或引用标记，默认会保持关闭，并在实现和测试完成后单独记录风险与适用场景。

## 文本保护范围

格式化过程中会优先保护以下内容，降低误改概率：

- Markdown fenced code block；
- Markdown 行内代码；
- Markdown 链接和图片；
- 自动链接形式的 URL / 邮箱；
- 普通 URL 和邮箱；
- LaTeX 行内公式、展示公式、常见环境和命令；
- YAML front matter、HTML block/注释、表格分隔行、引用式链接定义；
- 任意长度反引号行内代码、嵌套括号链接/图片、行内 HTML 标签、转义 Markdown 标记和硬换行；
- 美元定界的行内/展示数学公式，以及有限词典识别的特殊单位（如 `μm`、`µm`、`Å`、`Å`、`Ω`、`mg/mL`、`mM`、`μM`、`mmol`、`mAh`、`kWh`）；
- 化学式：包含 Unicode 上下标、电荷标记或水合物连接符（`·`）的片段，如 `Fe²⁺`、`SO₄²⁻`、`FeCl₂·4H₂O`、`CuSO₄·5H₂O`。

例如 `$E=mc^2$`、代码块中的符号、链接地址、化学式通常不会被规则拆开或替换。

> 仍在增强：Markdown 保护目前是保守子集，不等同于完整 CommonMark/HTML 解析器；后续工作与优先级见 [docs/roadmap.md](docs/roadmap.md)。当前版本继续遵循「宁漏格式化、不破坏结构」原则。

## 用户设置

用户设置的默认保存路径为：

```text
rules.yaml
```

保存内容包括：

- 已启用规则；
- 最近输入；
- 主题模式（`system` / `light` / `dark`）。
- 界面字体预设（`system`、微软雅黑、苹方、Noto Sans CJK、宋体或黑体）。
- 编辑器字号（小、标准、大或特大），同时应用于输入框与输出框。
- 主界面缩放（80%、90%、100%、110% 或 125%）。
- 有序自定义字面量替换（空来源项会被忽略）。
- 简繁转换模式（`none` / `t2s` / `s2t`）。
- 输出模式（`realtime` / `manual`）：手动模式下使用“立即排版”快捷键刷新输出。
- 输入/输出布局（`auto` / `horizontal` / `vertical`）：自动模式在宽屏左右排列、小屏上下排列。
- 内置工作流预设不写入设置文件：中文文案、PDF 清洗、技术文档；应用预设会同步当前规则、替换和转换设置。PDF 清洗只处理从 PDF/CAJ 复制出的文本，不解析文件本体。
- 快捷键：总开关与各动作绑定：

```yaml
shortcuts:
  enabled: true
  bindings:
    format_now: CtrlOrCmd+Enter
    copy_output: CtrlOrCmd+Shift+KeyC
    open_settings: CtrlOrCmd+Comma
```

旧版设置文件缺少 `shortcuts` 字段时，自动回退为启用并使用默认组合键。

旧版设置文件缺少 `replacements` 或 `conversion` 字段时，分别回退为 `[]` 和 `none`。替换按列表顺序执行，仅支持字面量，不支持用户正则。

旧版设置文件缺少 `output_mode` 或 `layout_mode` 字段时，分别回退为实时输出和自动布局。输入/输出统计按 Unicode code point 计数，emoji 等多 code point 字符不会按 UTF-16 code unit 重复计数。

文件缺失或损坏时，应用会使用内置默认规则集。若程序放在只读目录（如 `Program Files`），应用会自动改用平台应用数据目录（Windows `%APPDATA%\CopyPolish`，Linux/macOS `~/.config/CopyPolish` 或 `$XDG_CONFIG_HOME`）保存设置，并在主界面提示；实际生效路径可在设置窗口底部查看（显示为 `rules.yaml`，悬停可见完整路径，点击可复制）。程序目录与应用数据目录同时存在时，优先使用程序目录设置。

如果检测到旧版本设置文件、主设置文件损坏或备份文件损坏，应用会在主界面和设置窗口显示对应提醒；主设置损坏时会优先尝试从 `rules.yaml.bak` 恢复。

## 终端版（Beta）

除桌面 GUI 外，项目还提供基于 Ratatui 的终端界面 `copypolish-tui`，与桌面版共用同一 Rust 排版引擎和 `rules.yaml` 规则设置。TUI 定位为 **Beta**：CLI 参数、规则选择和设置文件格式保持兼容；终端显示能力（emoji 宽度、OSC 52 剪贴板）取决于终端支持，SSH 环境不在验证范围内。需从源码构建：

```bash
# 交互式终端界面
cargo run --manifest-path src-tauri/Cargo.toml --features tui --bin copypolish-tui

# 非交互模式（stdin → stdout）
printf '在LeanCloud上，花了5000元' | copypolish-tui --stdin --no-config

# 文件输入输出
copypolish-tui --input article.md --output formatted.md --rules all

# 查看全部参数
copypolish-tui --help
```

非交互模式常用参数：`--rules <all|defaults|none>` 覆盖规则集；`--enable <key>` / `--disable <key>` 微调单条规则；`--no-config` 完全跳过共享设置。交互界面支持多行编辑、实时预览、规则开关、复制输出（OSC 52，依赖终端支持）、替换/字符转换请求设置和工作流预设。按 `Ctrl+E`（非输入区也可用 `e`）打开请求设置面板；按 `Ctrl+P`（非输入区也可用 `p`）打开预设面板，使用 `↑/↓` 选择、Enter 应用。默认构建会将不可用的 T2S/S2T 归一化为“不转换”，启用 `simplified-trad-conversion` feature 的 TUI 才执行真实转换。终端模块和开发命令详见 [docs/architecture.md](docs/architecture.md) 与 [docs/development.md](docs/development.md)。

## 开发文档

版本管理、开发环境、测试方法、CI / Release、打包细节和实现约束请参阅：

- [docs/README.md](docs/README.md)：开发者文档导航与阅读顺序；
- [CONTRIBUTING.md](CONTRIBUTING.md)：分支、提交、PR、验证和完成标准；
- [docs/architecture.md](docs/architecture.md)：架构、模块职责、数据流和修改入口；
- [docs/testing.md](docs/testing.md)：测试层次、功能映射和 fixture 规范；
- [docs/development.md](docs/development.md)：工具链、启动命令、验证入口和工程约束；
- [docs/roadmap.md](docs/roadmap.md)：按优先级整理的后续开发计划；
- [docs/release/manual-release.md](docs/release/manual-release.md)：构建、验收与手动发布 Runbook；
- [docs/secrets-management.md](docs/secrets-management.md)：维护者凭据的 SOPS/age 管理与恢复指南；
- [CHANGELOG.md](CHANGELOG.md)：版本和重要变更记录。

已完成版本的发布计划与验收记录保存在 `docs/archive/`，不作为当前开发任务清单。

## 参考

CopyPolish 的产品规则、实现方式和工程工具参考了以下项目、规范与官方资料。这里列出的是项目文档或实现中明确采用、对照或依赖的主要来源；第三方传递依赖的完整许可证信息以 [docs/licenses.md](docs/licenses.md) 为准。

- [CopyPolish](https://github.com/15699122/CopyPolish)：当前项目的公开源码仓库；实现和行为以仓库中的源码、配置和测试为准。

### 规范与数据来源

- [chinese-copywriting-guidelines](https://github.com/sparanoid/chinese-copywriting-guidelines)：简体中文文案排版规范的主要参考。
- [Unicode Character Database](https://www.unicode.org/ucd/)：康熙部首兼容分解映射、Unicode 字符属性和相关边界语义的数据来源。
- [Unicode Standard Annex #29](https://www.unicode.org/reports/tr29/)：grapheme cluster 边界处理的规范参考。

### 产品与功能参考

- [paper-assistant](https://github.com/laorange/paper-assistant)：来源文本清洗、引用角标和 PDF/CAJ 复制文本场景的功能对照参考。CopyPolish 不复制其源码，也不解析 PDF/DOCX 文件本体。
- [Grammarly](https://www.grammarly.com/)：用于产品边界对照；CopyPolish 不依赖 Grammarly 或在线服务。

### 应用框架与界面生态

- [Tauri](https://v2.tauri.app/)：桌面应用壳、IPC、窗口和构建配置。
- [React](https://react.dev/)：GUI 组件和状态交互。
- [Vite](https://vite.dev/)：前端开发服务器和生产构建。
- [Tailwind CSS](https://tailwindcss.com/)：界面样式工具链。
- [shadcn/ui](https://ui.shadcn.com/) 与 [Radix UI](https://www.radix-ui.com/)：设置弹窗、表单控件和无障碍交互基础。
- [Lucide](https://lucide.dev/)：界面图标。
- [Rust](https://www.rust-lang.org/)：排版引擎、设置模型和 TUI 实现语言。
- [Ratatui](https://ratatui.rs/) 与 [Crossterm](https://github.com/crossterm-rs/crossterm)：终端版 `copypolish-tui` 的界面渲染和终端事件处理。

### 文本转换、测试与工程工具

- [OpenCC](https://github.com/BYVoid/OpenCC)：简繁转换词典和语义边界的参考；实际可选转换实现使用 [opencc-fmmseg](https://github.com/laisuk/opencc-fmmseg)。
- [WebdriverIO](https://webdriver.io/)：真实 Tauri GUI E2E 测试框架和 provider 配置参考。
- [Tauri WebDriver 文档](https://v2.tauri.app/develop/tests/webdriver/)：embedded 与标准 WebDriver 测试路线参考。
- [tauri-plugin-webdriver](https://github.com/Choochmeque/tauri-plugin-webdriver)：标准 W3C WebDriver provider 的 PoC 和兼容性 smoke 参考，不是生产 GUI 的唯一测试路线。
- [sops](https://github.com/getsops/sops) 与 [age](https://age-encryption.org/)：加密凭据文件和密钥管理工具。

参考项目只用于规范、架构、功能边界或测试方案对照；CopyPolish 的实际行为以仓库中的 Rust 引擎、前端实现、配置文件和测试结果为准。
