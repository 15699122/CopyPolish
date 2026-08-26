# CopyPolish 后续开发路线图

本文档跟踪 `v0.5.0` 正式发布后的中长期开发工作；`v0.5.0` 发布门槛与验收仍以 [v0.5.0-release-plan.md](v0.5.0-release-plan.md) 为准，本地构建与手动发布操作见 [manual-release.md](manual-release.md)。

## 1. 优先级原则

1. 先完成 `v0.5.0` 发布闭环（Windows 真机验收、正式 Release 复核），再启动功能扩展；
2. 每项改动先补测试与文档，再改实现；
3. 大型依赖（如 ICU4X）先做 Spike 评估，确认体积/性能/跨平台收益后才正式接入。

## 2. P0：v0.5.0 发布闭环

- Windows 10/11 真机人工验收（清单见 v0.5.0-release-plan.md 第 12 节）；
- 正式 Release 资产、Release Notes 与 latest 标记复核；
- 未闭环前不新增大型规则或 UI 重构。

## 3. P1：本地构建与手动发布能力

- GitHub Actions 保持为标准 CI 与可复现构建路径；
- 本地构建 + 手动上传为正式支持的备用发布方式（Runbook：[manual-release.md](manual-release.md)）；
- 已提供本地发布自动化脚本：
  - `scripts/build_release_local.ps1`（Windows 版本同步、构建、DLL 收集、`.7z` 打包）；
  - `scripts/build_release_local.sh`（Linux 构建与资产命名）;
  - `scripts/verify_release_assets.py`（校验 tag、版本一致性、资产存在性与命名、7z 目录结构）。
- 脚本约束：要求干净发布工作区；默认不创建 tag、不推送、不上传；产物写入被忽略的 `dist/`。

## 4. P1：快捷键配置与冲突规避

快捷键配置已于 2026-08 合入 `dev`：原硬编码于 `frontend/src/App.tsx` 的全局 keydown 监听已重构为 `lib/shortcuts.ts` + `hooks/useShortcuts.ts`，支持总开关、自定义绑定、冲突校验与 IME 防护。以下为设计约束的最终落地记录：

### 阶段 A：快捷键总开关 ✅ 已完成

- 新增持久化设置字段 `shortcuts.enabled: bool`（Rust `UserSettings` + YAML 序列化 + 前端类型 + localStorage 回退同步）；
- 关闭时不注册/不执行应用自定义快捷键；
- 保留 Radix Dialog 原生 `Esc` 关闭行为；
- 仅在窗口聚焦且事件确实匹配已启用动作时调用 `preventDefault()`。

### 阶段 B：自定义快捷键 ✅ 已完成（Comma 作为默认值的历史兼容例外，自定义录制按白名单限制）

| 动作 key | 默认值 |
| --- | --- |
| `format_now` | `CtrlOrCmd+Enter` |
| `copy_output` | `CtrlOrCmd+Shift+C` |
| `open_settings` | `CtrlOrCmd+Comma` |

设计约束：

- 存储语义组合键 `CtrlOrCmd`，用 `KeyboardEvent.code` 识别按键；
- 自定义绑定必须包含 `CtrlOrCmd`（可额外附加 Shift/Alt）；禁止单字母/数字/标点绑定；`CtrlOrCmd+Comma` 作为默认值的历史兼容例外保留；
- 动作间禁止重复绑定，冲突给出明确校验错误；
- 维护高风险系统组合键黑名单；
- 输入法组合态（`event.isComposing`，兼容 `keyCode === 229`）不触发；
- 提供恢复默认按钮；冲突/保存状态通过 `aria-live` 反馈。

代码落点：

```text
frontend/src/lib/shortcuts.ts     # schema、序列化、冲突判断、默认值
frontend/src/hooks/useShortcuts.ts # 监听、启停、IME 防护、动作分发
```

### 测试要求

总开关关闭后全部失效；默认/自定义组合键触发；重复绑定拒绝；单键拒绝；IME 组合中不触发；未匹配组合键不阻止文本输入；配置重启恢复；恢复默认持久化。

## 5. P1/P2：复杂排版与 Unicode 基础能力增强

复杂排版增强分多个阶段推进，涵盖多行文本、Markdown 结构、特殊单位（`μm` / `Å` / `Ω` 等）、数学符号（`∂` / `±` / `≤` 等）、标点与 Unicode 边界。整体遵循「测试先行、保守保护、能力分层、不默认改写原文」原则，新增规则须符合 §10 准入流程，且保护层改动须同步 §7 的 fixture 覆盖要求。

### 5.1 现状限制

- 阶段 B 已解决 grapheme cluster 边界问题；当前剩余限制是语义分类仍分散在 tokenizer、unit lexicon 和规则实现中；
- 单位词典已覆盖首批 Unicode、SI、温标和复合单位，但仍是有限词典，不是完整计量单位语法；
- Markdown 保护已覆盖路线图首批结构，但仍是“扫描器 + 有限正则 + 占位符”的保守子集，不等同于完整 Markdown/HTML 语法解析；
- pipeline 已按 `RulePhase` + `before/after` 依赖做稳定拓扑排序，同 phase 使用注册表顺序作为 tie-break；Span/Edit 的基础优先级与冲突仲裁已完成，但尚未接管生产保护层或格式化 pipeline；
- `spans.rs` 已提供统一语义/结构 span 与重叠仲裁基础，`edit_plan.rs` 已提供 UTF-8 安全的 TextEdit 仲裁/应用及语义边界规划基础；两者尚未接管保护层或 pipeline；
- 复杂输入的组合 fixture、规则容斥矩阵和长文本性能基线仍需继续补齐。

### 5.2 阶段总览

| 阶段 | 优先级 | 内容 | 状态 |
| --- | --- | --- | --- |
| A | P0 | 测试先行：补齐复杂排版 fixture，并区分稳定回归与待实现基线 | ✅ 已完成（稳定/待实现基线已分离） |
| B | P1 | `unicode-segmentation` 与统一字符边界层 | ✅ 已完成（见 5.4） |
| C | P1 | 单位词典与语义 token（特殊单位 / 温度 / 数学符号分类） | 🚧 进行中 |
| D | P2 | Markdown 块级扫描器与行内保护扩展 | 🚧 进行中（已完成 YAML/front matter、HTML block/注释、表格分隔行、引用式链接、反引号、嵌套括号链接、行内 HTML、转义标记、硬换行和美元数学保护） |
| E | P2 | Unicode 等价识别与输出规范化（默认关闭） | 规划 |
| F | P2 | 性能基准与边界回归纳入 CI | 规划 |

当前开发主线为 `dev`。阶段 A–B 已完成，阶段 C–D 的第一批功能已落地；R2/R3 是在不改变现有用户输出的前提下，为后续 Span/Edit 迁移建立调度和编辑基础设施。

### 5.3 阶段 A：测试先行（P0，先行于任何新规则/保护层改动）

不引入新依赖，先沉淀行为基线：

1. 新增失败型黄金样例，覆盖以下场景并明确「应保护」还是「应格式化」：
   - Markdown：多反引号行内代码、表格分隔行、引用式链接定义、YAML front matter、HTML 注释、硬换行（行尾两空格 / 反斜杠）；
   - 特殊单位：`μm/µm`、`Å/Å`、`Ω/kΩ`、`°C/°F`、`‰`、`mg·mL⁻¹`、`kg·m⁻³`；
   - 数学符号：`∂f/∂x`、`x≤y`、`±`、`×`、`≈`；
   - Unicode 边界：CJK Extension B 及后续、ZWJ emoji、组合附加符、`U+00A0` / `U+202F` 空白、`U+2028` / `U+2029` 分隔符。
2. 建议将现有混合 fixture 拆分为独立文件并新增：
   ```text
   src-tauri/tests/fixtures/
   ├── markdown-blocks.yaml
   ├── markdown-inline.yaml
   ├── unicode-boundaries.yaml
   ├── measurements.yaml
   ├── mathematical-symbols.yaml
   ├── punctuation-contexts.yaml
   └── regressions.yaml
   ```
3. 每个样例同时覆盖：单规则、默认规则组合、Markdown 保护组合、幂等性、LF / CRLF / CR 保留、长文本性能回归。

阶段 A 的测试分层约束：

- 当前已实现的行为进入稳定黄金回归集，必须通过 `cargo test` 与 CI；阶段 C 第一批计量单位案例已从 pending 基线迁移到稳定集；
- 阶段 C/D 尚未实现但已经确认目标的行为进入 pending 基线，只要求 fixture 可解析并记录当前差异，不得让 CI 长期失败；阶段 C 第一批数学表达式案例已迁移到稳定黄金集；
- 数学表达式与中文之间的精确空格、HTML block 内可见文本是否完全冻结等尚未完成产品决策的行为，不在决策前作为唯一正确输出；
- 阶段 C/D 完成对应实现后，pending 案例必须迁移到稳定黄金回归集，并补充幂等性断言；
- 阶段 A 已完成：上述 fixture 已补齐，稳定黄金样例与 pending 基线已在 `src-tauri/src/engine/tests.rs` 中分离，并已加入稳定样例幂等性测试；Rust、前端和 diff 检查均已纳入验证流程。
- 阶段 A 后续维护已新增复杂组合、规则交互、结构优先级及换行/幂等性 fixture；其中结构优先级样例暂作为 pending 基线，已记录“行内代码应优先于单位/数学扫描”的当前差异，待 span/edit 重构后迁移到稳定黄金回归集。

#### 阶段 A 当前验收结果

- 稳定黄金 fixture 与 pending baseline 已在 `engine/tests.rs` 分离；
- 已增加 `complex-compositions.yaml`、`rule-interactions.yaml`、`structure-precedence.yaml`、`newline-and-idempotence.yaml`；
- 已覆盖默认规则、单规则、复杂规则组合、换行风格、幂等性和结构优先级差异；
- 当前 Rust 全量测试为 66 项，pending 基线不阻断 CI，但必须持续保留可观测差异。

### 5.4 阶段 B：`unicode-segmentation` 与统一字符边界层（P1）✅ 已在 `unicode-boundaries` 分支实现

- 轻量 Rust crate（UAX #29 Grapheme / Word / Sentence 边界，MIT/Apache-2.0）；
- 新建独立封装层 `src-tauri/src/engine/unicode_boundaries.rs`，不立即全面替换 tokenizer；
- 提供语义 API（`is_han_grapheme`、`script_of`、`is_latin_or_greek_letter`、`is_numeric`），规则不各自引入边界判断；
- 适用场景：
  - 按 grapheme cluster 遍历，避免切断 emoji 组合序列与组合字符；
  - 为选区格式化、光标映射提供稳定边界；
  - 补充 Unicode 边界回归样例（emoji ZWJ、CJK Extension B、Kana/Hangul 混排、组合附加符等）。
- 改造原则：
  1. 保护层优先顺序不变：化学式 → Markdown/LaTeX/URL/邮箱 → Unicode 边界 → 规则管线 → 还原；
  2. 既有黄金 fixture 必须全部通过，新行为只新增样例不改旧预期；
  3. 通过内部策略开关保留新旧实现对比期；
  4. 引入前后记录编译时间、二进制体积与 10 KB / 100 KB / 1 MB 性能基线。

范围说明：UAX #29 是通用边界规则，不等于中文语义分词，不替代现有格式化规则系统。

### 5.5 阶段 C：单位词典与语义 token（P1）

当前进度：已完成第一批有限单位词典与语义 token 基础设施，并将既有 `spacing.number-unit` 迁移到该层。当前实现覆盖 Unicode 微米/埃/欧姆、常见 ASCII/SI 单位、厘米/厘升/百帕、温标与复合科学单位，且保留 `μ/µ`、`Å/Å` 的原始输出写法；普通英文单词、变量名和已由保护层处理的化学式不会被当作计量单位。数学表达式已完成首批保守扫描与保护。

- 已实现：`src-tauri/src/engine/unit_lexicon.rs`、`semantic_tokens.rs`；
- 已迁移：`spacing.number-unit` stable key 继续保留，内部改用有限词典扫描；
- 已覆盖：`μm/µm`、`Å/Å`、`Ω/kΩ`、`cm/cL/hPa`、`°C/°F`、`mg·mL⁻¹`、`kg·m⁻³` 及普通英文/化学式反例；
- 待完成：继续扩充完整单位词典、评估是否需要独立的温度规则 stable key；当前已支持有限范围的 `/` 复合单位（如 `mg/mL`、`m/s`、`kg/m³`），已将第一批 `measurements.yaml` 与 `mathematical-symbols.yaml` 案例迁移到稳定黄金回归集，并完成明确数学表达式的首批保守识别。
- 约束：继续禁止使用 `\p{L}+` 作为通用单位识别；不默认做 Unicode 等价字符规范化；不改变化学式保护层优先级。

#### 阶段 C 未完成项

- 完整单位词典和更完整复合单位语法；
- 温度表示的独立 stable key 评估及与 `spacing.temperature-cjk` 的最终兼容方案；
- `MathExpression` 的更完整语法边界；
- 将语义边界从当前规则函数迁移到统一 TextEdit 计划，并补充旧/新路径输出对照；——混合管线对照骨架已建立（`pipeline.rs::format_text_span_aware`：span 划分可编辑区间 + 复用纯函数规则），全部稳定 fixture 上 span 化混合管线与生产高度一致（初始 23 例差异 → 屏蔽 OpaqueStructure 后仅剩 7 例 protection 细节未对齐），对照测试 `tests::span_aware_pipeline_matches_production_on_stable_fixtures` 当前标 `#[ignore]` 作为开发基线，7 例缺口（化学式边缘空格、`×3cm²` 单位拆分、数学符号全角替换、硬换行、引用后链接定义空格等）须逐一补齐 span 覆盖或边缘规则；

- 不使用「任意 Unicode 字母都可当单位」的宽泛 regex，改用**有限词典 + 复合语法**：
  - 基础单位：`m` / `g` / `s` / `L` / `mol` / `K` / `Pa` / `Hz` / `N` / `J` / `W` / `V` / `A` / `Ω` / `dB` / `rad` / `rpm` / `px` 等；显式常用项包括 `cm` / `cL` / `hPa`；
  - 常用前缀：`k` / `M` / `G` / `m` / `μ` / `µ` / `n` / `p`；
  - 非 SI 单位：`℃` / `℉` / `°C` / `°F` / `Å` / `Å` / `mmHg` / `eV`；
  - 复合连接：`/`、`·`、`⋅`、Unicode 上下标。
- 新增 `src-tauri/src/engine/semantic_tokens.rs` 与 `unit_lexicon.rs`，识别不可拆的语义片段：
  - `Measurement`：`10 μm`、`20 kΩ`、`5 Å`；
  - `Temperature`：`4℃`、`4 °C`、`32℉`；
  - `ScientificUnit`：`mg·mL⁻¹`、`mol·L⁻¹`、`kg·m⁻³`；
  - `MathExpression`：保守识别 `∂f/∂x`、`x≤y`、`a≈b`、`3±0.5`、`2×3`，避免一般文本被过量保护。
- 新规则建议（默认开关须按 §10 流程评估）：
  | 规则 key | 名称 | 建议默认 |
  | --- | --- | --- |
  | `spacing.measurement-boundaries` | 数值、计量单位与中文之间使用正确空格 | 开启 |
  | `spacing.scientific-unit-boundaries` | 科学复合单位与中文之间使用正确空格 | 开启 |
  | `spacing.temperature-notation` | 温度表示与中文之间使用正确空格 | 开启 |
  | `text.unicode-unit-equivalence` | 统一等价单位字符表示 | 关闭 |
- 兼容既有 `spacing.temperature-cjk`（`℃`/`℉`）：新规则纳入 `°C`/`°F` 时，通过 legacy key 映射或稳定 key 保持用户设置兼容。

### 5.6 阶段 D：Markdown 块级扫描器与行内保护扩展（P2）

- 在保留占位符机制前提下，将管线从「所有非空行均规则处理」调整为「只格式化可编辑文本区间」；
- 块级扫描器首批识别：YAML front matter、fenced / indented code block、HTML 注释与 HTML block、表格分隔行、引用式链接定义；生产 placeholder 保护已覆盖这些首批结构，Span 层也已完成对应只读扫描，统一迁移仍待 R3 后续阶段；
- 行内保护扩展：
  - 任意长度反引号 delimiter（`` ` `` / `` `` ` `` / `` ``` `` 等）；当前已完成同长度 delimiter 的行内代码保护；
  - Markdown 链接的平衡括号与引用式链接；当前已完成嵌套括号链接/图片保护；
  - HTML inline tag；当前已完成常见行内标签和自闭合标签保护；
  - 行内数学与转义字符；当前已完成美元定界行内/展示数学、常见转义 Markdown 标记与硬换行保护；
- 用小型状态机替代堆叠 `fancy-regex`（尤其反引号与括号嵌套）；
- 产品策略：
  - 检测到明显 Markdown 标记时默认启用「Markdown 安全模式」（宁漏格式化、不破坏结构）；
  - 把「识别等价性」与「输出改写」分离：识别可视为等价，改写默认关闭。

#### 阶段 D 当前限制

- 当前保护实现仍是手写扫描器、有限 `fancy-regex` 与 placeholder 的混合模式；
- `spans.rs` 已能扫描首批块级/行内结构并与语义 span 仲裁，但尚未接管 `protection.rs` 或 `pipeline.rs`；
- `edit_plan.rs` 已能规划并应用测试用 TextEdit，但尚未替换现有逐行规则管线；
- 行内代码优先于单位/数学扫描的目标行为仍由 `structure-precedence.yaml` 作为 pending 基线冻结；
- 不将当前能力宣称为完整 CommonMark/HTML 解析器。

### 5.7 R2/R3：规则调度与 Span/Edit 迁移基础（P1/P2，进行中）

#### 已完成

- `RulePhase`：标点规范化、名词规范化、结构边界、文本边界、最终清理；
- `RuleDef.before/after` 依赖元数据；
- 稳定拓扑排序，同 phase 使用注册表顺序作为 tie-break；
- 未知依赖、重复 key、循环依赖测试；
- `SpanKind` / `SpanPriority` / `TextSpan`；
- 语义 span：化学式、Measurement、Temperature、ScientificUnit、MathExpression；
- 结构 span：fenced code、front matter、HTML block/comment、引用式链接定义、缩进代码、表格分隔行、行内代码、Markdown 链接、美元数学；
- span 重叠仲裁：结构 > 语义原子 > 可编辑文本；
- `TextEdit` UTF-8 边界校验、编辑冲突仲裁、逆序应用；
- 单位和数学边界的测试用编辑计划。

#### 未完成

- 将结构/语义 span 正式接入现有保护层；
- 结构保护还原迁移的三类已确认差异（2026-08-26 对照输出）：
  a) inline placeholder 周边 CJK 空格：生产管线对行内代码 / 链接 / 图片 / 行内 HTML / 化学式占位符两侧补空格；
  b) HTML block 的 span 覆盖缺口：✅ 已修复——`scan_html_block_spans` 此前计算 span 终点时漏掉中间行长度与换行符，导致块内正文漏出 span；已修正并新增回归测试 `spans.rs::html_block_span_covers_interior_lines`；
  c) 未闭合结构的特殊还原（如 `[文档]（` 全角括号替换）；
- 将 `TextEdit` 正式接入 `format_text`；
- 将 `spacing.number-unit`、温标、数学边界和全角标点清理迁移为 span-aware edits；
- 移除普通/数学多套 placeholder 编号约定；
- 解决所有结构优先级 pending 案例并迁移为稳定黄金 fixture；
- 增加编辑计划与旧 placeholder 路径的逐例 diff 对照；✅ 已完成并扩展至**全部稳定黄金 fixture**（`tests.rs::edit_plan_path_matches_placeholder_pipeline_on_stable_fixtures`）：语义边界、`cn_en_space` 扩展边界（强调/上标单位）、inline placeholder 边缘补空格、未闭合反引号特例双路径输出完全一致；HTML block span 覆盖缺口已修复；剩余 14 例差异**收敛为非边界逐行规则一类**（标点规范化、名词规范化、全角括号替换、全选组合等），按案例冻结在测试内 `PENDING_DIFFS` 清单，建议以「span 划定可编辑区间 + 复用现有纯函数规则」的混合模式随生产接管一并消除；
- 建立 10 KB/100 KB/1 MB 性能和内存基线。

### 5.8 阶段 E：Unicode 等价识别与输出规范化（P2）

- 识别阶段可把 `µ/μ`、`Å/Å` 视为等价语义；
- 输出阶段不擅自用 NFKC 改写用户原文（如 `µm` → `μm`、`Å` → `Å`）；
- 若需统一表示，须作为独立、默认关闭的 Unicode 规范化规则，并评估对数学字母、全角符号的影响（与 §6 ICU4X 规范化评估对齐）。

### 5.9 阶段 F：性能基准与边界回归（P2）

- 基准输入：10 KB / 100 KB / 1 MB × {纯中文、中英数混排、Markdown / URL / LaTeX 密集、emoji 与组合字符密集、CJK 扩展区密集}；
- 记录耗时、峰值内存、保护层正则占比与规则数增长退化趋势；
- 重点剖析 `protection.rs` fancy-regex 调用次数、占位符替换复杂度；
- 目标：1 MB 文本不阻塞 UI，无旧结果覆盖，有明确处理反馈（与 §8 对齐）。

### 5.10 不建议直接采用的做法

1. 不把单位正则直接扩展为 `\p{L}+`——会把自然语言英文、变量名、产品名误判为单位；
2. 不对全文默认做 NFKC——可能改变 `Å`、兼容字符、数学字母、全角符号等原文语义；
3. 不通过继续堆叠 `fancy-regex` 解析完整 Markdown——嵌套 / 跨行 / 表格 / HTML 适合状态机或 AST；
4. 不把所有特殊符号都当作「中英文之间加空格」依据——`∂` / `±` / `≤` / `×` 等数学符号间距规范与计量单位不同；
5. 不为「Markdown 安全」而保护整篇文档——只保护确定的语法块与行内不可改写区域，让普通叙述文本继续接受排版。

## 6. P2：ICU4X 技术验证

- **不引入 ICU C/C++**（FFI、原生依赖、打包复杂度不可接受）；仅评估纯 Rust 的 ICU4X；
- 以独立分支 / feature flag 做 Spike，候选模块：
  - Script / General Category 属性（替代手写 Unicode 区间）；
  - Unicode 规范化（NFC/NFKC——可能改变用户原文，必须显式规则化，绝不默认启用）；
  - 分词与断行（仅在行为与产品规则一致时采用）。
- 评估记录项：二进制体积增量、编译时间、长文本性能、Windows/Linux 打包影响、新旧输出 diff、locale 数据加载方式；
- 只有明确解决现存问题的模块才进入正式依赖。

## 7. P2：真实 Tauri E2E 测试

- 优先评估 tauri-driver (WebDriver) 方案，测试真实桌面链路而非 mock；
- 首批场景：启动可用性、真实引擎输出、"全不选"输出恒等、规则切换即时重排、设置临时目录写入与重启恢复、损坏设置提醒、全部快捷键（含开关关闭态）；
- E2E 使用注入的临时设置目录，禁止污染真实 `rules.yaml`；
- 初期作为 nightly / 手动工作流，稳定后纳入 PR 门禁；Linux 与 Windows 至少各保一条真实链路。

## 8. P2：性能基准与优化

- Rust 侧 benchmark：10 KB / 100 KB / 1 MB × {纯中文、中英数混排、Markdown/URL/LaTeX 密集、emoji 与组合字符密集、CJK 扩展区密集}；
- 记录耗时、峰值内存、保护层正则占比、规则数增长的退化趋势；
- 重点剖析 `protection.rs` fancy-regex 调用次数、占位符替换复杂度、前端防抖积压；
- 第一阶段目标：1 MB 文本不阻塞 UI、无旧结果覆盖、有明确处理反馈；SLA 在取得基线数据后再定。

## 9. P2：前端 hooks 重构

拆分顺序（行为不变前提下）：

```text
frontend/src/hooks/
├── useFormatter.ts       # 输入/输出、动态防抖、请求序号竞态防护、耗时统计
├── useUserSettings.ts    # 初始化读取、保存队列、提醒状态、五类设置模型
├── useThemeAndFont.ts    # system/light/dark、matchMedia、CSS variables
├── useShortcuts.ts       # 快捷键监听与分发（随路线图 §4 一并落地）
└── useWindowControls.ts  # Tauri 窗口控制，浏览器预览安全回退
```

`App.tsx` 只保留组件编排；hooks 不直接 `invoke`，继续经由 `lib/tauri.ts`。

## 10. P2：规则边界扩展准入流程

在黄金样例体系之上新增规则时，每条必须同时具备：

1. `registry.rs` 注册稳定 key / 展示名 / 默认状态 / legacy 别名策略；
2. `rule_impls.rs` 纯函数实现；
3. 单规则黄金样例 + 与其他规则及保护层的组合回归样例；
4. 幂等性验证（二次格式化不再变化）；
5. 争议性评估与默认开关依据；
6. README 规则表同步。

优先补齐的保护层边界：嵌套 Markdown 链接/引用、反引号数量不一致、LaTeX 嵌套环境、Unicode 域名 URL、括号包裹 URL、更复杂化学式歧义。

## 11. P2：依赖、安全与发布维护

- `cargo audit`、`npm audit`（明确高危阻断阈值）、许可证清单；
- 审查并评估收紧 `tauri.conf.json` 中当前为 `null` 的 CSP（改动需桌面端 smoke 回归）；
- Dependabot 已覆盖 npm / Cargo / GitHub Actions（周更，target dev 分支），保持合并前 CI 必过；
- 建立 Node/Rust/Tauri/React 升级 runbook 与预发布 tag 验证流程。

## 12. 推荐执行顺序

```text
P0  §2 发布闭环
 ↓
P1  §3 本地构建脚本（可选） → §4 快捷键开关 → §4 自定义快捷键
 ↓
P1  §5
     阶段 A（测试先行：复杂排版/Markdown/单位/数学/Unicode 边界样例）
     → 阶段 B（unicode-segmentation 封装与小范围替换）
     → 阶段 C（单位词典与语义 token / 温度 / 数学符号分类）
 ↓
P2  §5 阶段 D（Markdown 块级扫描器）→ 阶段 E（Unicode 等价规范化，默认关）
 → §8 性能基准 ⇄ §7 E2E → §6 ICU4X Spike → §9 hooks 重构 → §10/§11 持续维护
```

> 注：§5 阶段 A（测试先行）优先于任何新规则或保护层改动落地，以守住既有的黄金样例回归体系。

每完成一项，更新本文档对应章节的状态标记，并按 `Dev_readme.md` 的文档同步约定更新 README / Dev_readme。
