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
- 后续补充自动化脚本：
  - `scripts/build_release_local.ps1`（Windows 版本同步、构建、DLL 收集、`.7z` 打包）；
  - `scripts/build_release_local.sh`（Linux 构建与资产命名）;
  - `scripts/verify_release_assets.py`（校验 tag、版本一致性、资产存在性与命名、7z 目录结构）。
- 脚本约束：要求干净发布工作区；默认不创建 tag、不推送、不上传；产物写入被忽略的 `dist/`。

## 4. P1：快捷键配置与冲突规避

当前快捷键硬编码于 `frontend/src/App.tsx` 的全局 keydown 监听（`Ctrl/Cmd+Enter` 格式化、`Ctrl/Cmd+Shift+C` 复制、`Ctrl/Cmd+,` 打开设置），存在与输入法、系统或其他软件冲突的可能。分两阶段实施：

### 阶段 A：快捷键总开关

- 新增持久化设置字段 `shortcuts.enabled: bool`（Rust `UserSettings` + YAML 序列化 + 前端类型 + localStorage 回退同步）；
- 关闭时不注册/不执行应用自定义快捷键；
- 保留 Radix Dialog 原生 `Esc` 关闭行为；
- 仅在窗口聚焦且事件确实匹配已启用动作时调用 `preventDefault()`。

### 阶段 B：自定义快捷键

| 动作 key | 默认值 |
| --- | --- |
| `format_now` | `CtrlOrCmd+Enter` |
| `copy_output` | `CtrlOrCmd+Shift+C` |
| `open_settings` | `CtrlOrCmd+Comma` |

设计约束：

- 存储语义组合键 `CtrlOrCmd`，用 `KeyboardEvent.code` 识别按键；
- 必须至少一个修饰键；禁止单字母/数字/标点绑定；
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

## 5. P1：Unicode 文本引擎基础升级

现状限制：`tokenizer.rs` 使用手写 Unicode 区间判定 CJK/Latin/Digit，规则实现大量基于逐 `char` 遍历与 ASCII 判断，对 emoji ZWJ 序列、组合附加符、CJK 扩展区的处理不够健壮。

### 第一步：引入 `unicode-segmentation`

- 轻量 Rust crate（UAX #29 Grapheme / Word / Sentence 边界，MIT/Apache-2.0）；
- 先建独立封装层 `src-tauri/src/engine/unicode_boundaries.rs`，不立即全面替换 tokenizer；
- 适用场景：
  - 按 grapheme cluster 遍历，避免切断 emoji 组合序列与组合字符；
  - 为选区格式化、光标映射提供稳定边界；
  - 补充 Unicode 边界回归样例（emoji ZWJ、CJK Extension B、Kana/Hangul 混排等）。

### 改造原则

1. 保护层优先顺序不变：化学式 → Markdown/LaTeX/URL/邮箱 → Unicode 边界 → 规则管线 → 还原；
2. 既有黄金 fixture 必须全部通过，新行为只新增样例不改旧预期；
3. Unicode 能力统一封装在 tokenizer 层（提供 `is_han_grapheme` / `script_of` 等语义 API），规则不各自引入边界判断；
4. 通过内部策略开关保留新旧实现对比期；
5. 引入前后记录编译时间、二进制体积与 10 KB/100 KB/1 MB 性能基线。

范围说明：UAX #29 是通用边界规则，不等于中文语义分词，不替代现有格式化规则系统。

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
P1  §5 unicode-segmentation 封装与小范围替换
 ↓
P2  §8 性能基准 ⇄ §7 E2E → §6 ICU4X Spike → §9 hooks 重构 → §10/§11 持续维护
```

每完成一项，更新本文档对应章节的状态标记，并按 `Dev_readme.md` 的文档同步约定更新 README / Dev_readme。
