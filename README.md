# 文案净排

> 项目名称：**文案净排**（英文：**CopyPolish**）。

文案净排（CopyPolish）是一款本地桌面端中文文案排版工具，用于按照 [chinese-copywriting-guidelines](https://github.com/sparanoid/chinese-copywriting-guidelines) 的简体中文文案规范，自动整理中文、英文、数字、单位和标点之间的格式。

应用采用 Tauri 2 + React + shadcn/ui 桌面界面，排版引擎由 Rust 实现。左侧输入原文，右侧实时显示规范化结果；规则可逐条启用或关闭。

## 功能亮点

- **实时排版**：输入或粘贴文本后自动生成格式化结果。
- **左右双栏编辑**：输入区和输出区并排显示，适合对照检查。
- **规则可配置**：设置窗口支持逐条启用/停用规则，也支持全选、全不选和恢复默认。
- **深色 / 浅色 / 跟随系统主题**：设置中可切换主题，选择会持久化，下次启动自动恢复。
- **界面字体可配置**：设置中可选择常用系统字体预设，也可以恢复默认字体；未安装的字体会自动回退。
- **争议规则默认关闭**：例如链接之间增加空格、简体中文使用直角引号，可按个人习惯开启。
- **Markdown / LaTeX 保护**：尽量避免误改代码块、行内代码、链接、图片链接、URL、邮箱和公式内容。
- **Unicode 边界安全**：排版按 grapheme cluster 判定中英/中数边界，emoji 组合序列（如 ZWJ 家庭 emoji）与组合附加符不会被拆开；CJK 扩展区汉字（如 `𠀀`）与普通汉字同样参与插空。
- **本地优先**：文本处理在本机完成，不依赖远程服务。
- **用户设置持久化**：规则开关、主题与最近输入保存在程序同目录的 `rules.yaml`，启动时自动恢复。

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

> 注意：Windows 版为便携版，不提供安装器。建议将程序放在有写权限的目录中运行，以便保存 `rules.yaml` 用户设置。

## 基本使用

1. 启动应用。
2. 在左侧输入框粘贴或输入中文文案。
3. 右侧输出框会实时显示规范化结果，例如：`在 LeanCloud 上，花了 5000 元`。
4. 点击 **复制结果** 将输出复制到剪贴板。
5. 点击 **清空输入** 清除当前文本。
6. 点击 **设置** 打开设置窗口：
   - 勾选或取消勾选单条规则（默认开启的规则显示在上方，默认关闭的规则显示在下方）；
   - 使用全选、全不选、恢复默认；
   - 在“快捷键”分区启用或关闭应用快捷键，修改各动作的组合键或恢复默认；
   - 勾选 **跟随系统** 让主题自动跟随操作系统的浅色/深色模式；取消勾选后可手动选择浅色或深色（取消时会先切换为当前系统对应的主题）；
   - 通过下拉框选择界面缩放（80%–125%）与编辑器字号（小/标准/大/特大，同时作用于输入框与输出框）；
   - 选择界面字体或恢复默认字体；
   - 在底部查看当前完整应用版本（预发布构建会带 pre 后缀）与设置保存状态；
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

规则由后端注册表统一管理，当前内置 14 条（「专有名词使用正确的大小写」「不要使用不地道的缩写」「链接之间增加空格」「简体中文使用直角引号」「统一等价 Unicode 单位字符」默认关闭）。每条规则有稳定的机器 key（如 `spacing.cjk-latin`），展示名仅用于界面：

| 分类 | 规则 | 稳定 key |
| --- | --- | --- |
| 标点符号 | 不重复使用标点符号 | `punctuation.no-repetition` |
| 全角和半角 | 使用全角中文标点 | `punctuation.fullwidth-cjk` |
| 全角和半角 | 数字使用半角字符 | `text.halfwidth-digits` |
| 全角和半角 | 遇到完整的英文整句、特殊名词，其内容使用半角标点 | `text.ascii-punct-in-latin` |
| 全角和半角 | 统一等价 Unicode 单位字符，默认关闭 | `text.unicode-equivalents` |
| 名词 | 专有名词使用正确的大小写，默认关闭 | `naming.proper-nouns` |
| 名词 | 不要使用不地道的缩写，默认关闭 | `naming.expand-abbreviations` |
| 争议 | 链接之间增加空格，默认关闭 | `spacing.around-links` |
| 争议 | 简体中文使用直角引号，默认关闭 | `punctuation.corner-quotes` |
| 空格 | 中英文之间需要增加空格 | `spacing.cjk-latin` |
| 空格 | 中文与数字之间需要增加空格 | `spacing.cjk-number` |
| 空格 | 数字与单位之间需要增加空格 | `spacing.number-unit` |
| 空格 | 摄氏度/华氏度符号与中文之间加空格 | `spacing.temperature-cjk` |
| 空格 | 全角标点与其他字符之间不加空格 | `spacing.no-space-around-fw-punct` |

> 说明：排版规则尽量贴近规范，但自然语言文本存在上下文差异。建议在重要文案发布前人工复核一次。

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

用户设置保存在程序相同目录下的：

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

文件缺失或损坏时，应用会使用内置默认规则集。若程序放在只读目录（如 `Program Files`），设置将无法保存；建议将便携版解压到有写权限的目录。

如果检测到旧版本设置文件、主设置文件损坏或备份文件损坏，应用会在主界面和设置窗口显示对应提醒；主设置损坏时会优先尝试从 `rules.yaml.bak` 恢复。

## 终端版（实验性）

除桌面 GUI 外，项目还提供基于 Ratatui 的终端界面 `copypolish-tui`，与桌面版共用同一 Rust 排版引擎和 `rules.yaml` 规则设置。需从源码构建：

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

非交互模式常用参数：`--rules <all|defaults|none>` 覆盖规则集；`--enable <key>` / `--disable <key>` 微调单条规则；`--no-config` 完全跳过共享设置。交互界面支持多行编辑、实时预览、规则开关与复制输出（OSC 52，依赖终端支持）。终端模块和开发命令详见 [docs/architecture.md](docs/architecture.md) 与 [docs/development.md](docs/development.md)。

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
