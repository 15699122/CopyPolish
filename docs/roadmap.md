# CopyPolish 后续开发计划

本文只跟踪尚未完成的工作。当前实现见 [architecture.md](architecture.md) 和 [development.md](development.md)，测试策略见 [testing.md](testing.md)，发布操作见 [release/manual-release.md](release/manual-release.md)，历史计划见 [archive/](archive/)。

## 规划原则

1. 正确性和结构保护优先于新增规则；
2. 先建立可观测基线和回归测试，再替换实现；
3. 默认不改写可能影响语义的 Unicode 表示；
4. 大型依赖和跨平台能力先做独立 Spike；
5. 每个里程碑同时包含实现、测试、文档和可复现验证。

## 当前基线与风险

当前生产入口使用 span-aware 混合管线，规则注册表、阶段依赖、结构/语义 span 和 UTF-8 安全 TextEdit 已落地。桌面 GUI 与实验性 TUI 共用 Rust 引擎和 `rules.yaml`。

主要风险：

- Markdown/HTML/LaTeX 保护仍是保守扫描器，不是完整 CommonMark/HTML 解析器；
- 保护层仍部分使用内部 placeholder；
- 1 MB 级 Markdown/LaTeX 语料仍需进一步减少分配和重复扫描；
- 真实 Tauri 桌面链路已在 Linux/WSLg 和 Windows WebView2 建立；本次 TUI/GUI 修复后的双 provider 最小 E2E、设置恢复、损坏设置、NTFS ACL、GUI/TUI 手动回归、Rust 设置/TUI、TUI stdin 和双 provider 稳定性验证均已完成。统一 E2E artifact、受控失败诊断、GUI 主题/窄窗口 artifact 和 TUI 非交互 transcript 已完成；TUI-EDIT-DELETE-001 已通过 grapheme 边界修复和回归测试关闭；Windows 三档 DPI 原生记录、Terminal 交互 artifact、React 19 warning 闭环和 GitLab 可选 stage 仍待补齐；
- `App.tsx` 和设置组件仍有进一步降低编排复杂度的空间；
- TUI 跨终端手动 smoke、非交互 transcript artifact 和正式资产决策已完成；Windows Terminal raw-mode/OSC 52 交互 artifact 仍待补齐。

## P0：流程与桌面验证

### P0.1 工程基线

- [x] 建立贡献、架构和测试文档；
- [x] 统一 GitHub 分支 CI、GitLab tag pipeline 和本地发布流程说明；
- [x] 增加 CHANGELOG；
- [x] 明确记录 GitHub Actions 账户计费阻塞属于外部运维风险：远程 workflow 需在账户解除阻塞后确认，本地 `verify.py` 与 GitLab tag pipeline 为替代门禁；
- [x] 在干净工作区按文档完成完整验证（清空全部可再生目录后重新 `npm ci`）：前端 13 文件 / 57 用例通过、构建通过，Rust 127 用例通过、TUI 构建与性能门禁通过，Markdown 链接/安全/版本检查通过。

### P0.2 真实 Tauri E2E

- [x] E2E 路线选型（决策 5）：选定 WebdriverIO + `@wdio/tauri-service`（embedded provider）为主路线，分析与 Spike 清单见 [e2e-driver-options.md](e2e-driver-options.md)；
- [x] 按选型方案在 Linux/WSLg 执行真实 Spike（`e2e` feature flag 集成 wdio 插件、embedded provider 启动真实应用）；
- [x] 参考 `Choochmeque/tauri-plugin-webdriver` 建立并行标准 WebDriver provider PoC（固定 `0.2.1`，Linux/WSLg smoke 通过；不替换当前 provider）；
- [x] 覆盖启动、真实引擎输出和默认示例；
- [x] 覆盖全不选恒等、规则切换和快捷键开关（双 provider 自动化最小链路及修复后 GUI/TUI 人工回归已完成；TUI-EDIT-DELETE-001 已关闭）；
- [x] 覆盖 GUI 设置重启恢复、损坏设置 fixture 和不可写目录保存失败提示（修复后 Windows 手动回归已完成；三种损坏设置 fixture 和重启恢复已在两个 provider 自动化通过）；
- [x] 使用临时设置目录，禁止污染真实 `rules.yaml`；
- [x] Linux 和 Windows 各保留至少一条真实链路，稳定后再纳入合并门禁（Linux/WSLg 与 Windows WebView2 修复后最小链路、设置/ACL 故障注入、双 provider 5 次稳定性统计及人工回归已完成；GitLab 可选 stage 和 artifact 固化仍待执行；不使用 GitHub Actions，见决策 6）。

## P1：引擎和长文本体验

### P1.1 保护层和语义扩展

- [x] 单位词典扩展策略（决策 3）：以互联网真实语料（技术博客/文档中英文混排段落）驱动候选收集，逐项评估误判/漏判后再入词典；
- [ ] 继续按真实语料扩展有限单位词典；
- [ ] Placeholder 重构（决策 2）：已批准启动；profiling 基线（2026-08-30）显示占位符全链路仅约 7.4% 耗时，重构性能收益有限，优先级低于 `editable_rules`/`scan_spans` 优化；重构本身以语义清晰度与边界健壮性为目标逐步推进，保持保护优先级和现有输出兼容；
- [x] 补充复杂 Markdown、HTML、LaTeX 和化学式真实样本（括号化学式分组、LaTeX 命令/定界/未闭合定界符、HTML 属性、中文混排均已入 fixture；未闭合 `\(`/`\[` 开定界符不再被标点规则改写为全角）；
- [x] 对未闭合结构继续坚持“宁漏格式化，不破坏结构”：现有 fixture 和引擎回归覆盖未闭合 Markdown 链接/反引号、LaTeX 定界符、HTML block/inline，并验证后续文本不会被吞并或破坏；
- [x] 所有新规则遵守注册表、fixture、幂等性、迁移和文档准入流程：已有全规则 fixture 覆盖、幂等性检查、stable key/legacy alias/依赖图测试；新增 alias 唯一性和不遮蔽 stable key 检查；

### P1.2 性能和响应性

- [x] 对 `spans.rs`、`protection.rs` 和 `edit_plan.rs` 做 profiling：新增 `profile-stages` feature 与 `examples/profile_stages.rs` 分阶段计时基线（1 MB，5 轮平均）：Markdown/LaTeX 语料 `editable_rules` 47.7% + `scan_spans` 40.2% 为主热点，占位符全链路仅约 7.4%；数据见 `docs/benchmarks/unicode-baseline.md`；
- [x] 二级归因（`per_rule_timings` / `scan_split_timings`，`profile-stages` feature）：Markdown 密集语料下 `scan_structure` 占扫描的约 508 ms（语义仅 13 ms）；`editable_rules` 内嵌的行区间判定本身也执行一次全文扫描，管线共两次全文扫描；规则侧前三热点为 `naming.proper-nouns`/`spacing.cjk-latin`/`spacing.cjk-number`；数据与优化方向见 `docs/benchmarks/unicode-baseline.md`；
- [x] 优化 inline-code 结构扫描：按行一次性收集反引号 run，并按 delimiter 长度索引，避免每个开定界符从当前位置重复扫描到行尾；1 MB profiling 中 `inline_code` 从此前约 279 ms 降至约 1.6 ms，现有 129 个 Rust/TUI 测试通过；
- [x] 优化 span 仲裁：按起点维护有序 accepted 列表，每个候选只检查相邻前驱/后继，避免 O(n²) 全量重叠检查；1 MB profiling 总耗时观测由约 805 ms 降至约 283 ms，`scan_structure` 由约 183 ms 降至约 7 ms；
- [x] 削减可编辑阶段的重复语义扫描：新增 `scan_editable_protection_spans`，仅扫描不透明结构与化学式，避免首次 `scan_all_spans` 的测量/单位/数学语义扫描；最新 profiling 总耗时观测约 218 ms，现有 Rust/TUI 测试与保护行为回归通过；
- [x] 优化 `naming.proper-nouns`：将 40+ 次逐词全文替换合并为单次词候选扫描，保留 ASCII 单词边界、前缀词和大小写不敏感语义；规则 profiling 由约 60 ms 降至约 2.5 ms，并新增相邻/嵌入词回归测试；阶段总耗时最新观测约 155 ms；
- [x] 优化 `spacing.cjk-latin` / `spacing.cjk-number`：生产路径改用与 `units()` 语义一致的流式相邻单位遍历，避免构造中间 `Vec<TextUnit>`；规则 profiling 观测约 19–30 ms / 17–36 ms，并新增 Graphemes/LegacyChars 一致性测试；
- [x] 优化 HTML block 结束标签查找：先定位可能的 ASCII 首字节，再执行大小写不敏感定长比较，降低重复窗口扫描的最坏情况成本；补充 UTF-8 前缀、大小写混合与未命中回归测试；当前基准语料中 `html_block` 约 0 ms，属于局部微优化；
- [x] 合并 LaTeX `\(...\)` / `\[...\]` 定界符扫描：单次遍历反斜杠候选并按定界符类型维护消费边界，保持混合定界符和按类型未闭合行为；新增 2 个回归测试，profiling 中 `latex_delimited` 约 0.04 ms；
- [x] 合并普通 Markdown 链接与引用式链接的 `[` 候选遍历：共享 label 扫描，保留普通链接的图片前缀/圆括号目标和引用链接的方括号目标语义；新增引用式图片边界回归测试，profiling 中合并后的 `markdown_links` 约 0.41 ms；
- [x] 优化美元数学候选扫描：前向遍历中维护连续反斜杠计数，避免每个 `$` 候选向前重复回扫；保持奇偶转义、单/双美元及单美元跨行边界；新增 2 个边界测试，当前 `dollar_math` 约 0.60 ms，未宣称平均耗时收益；
- [x] 合并 URL 与邮箱候选扫描：使用一次正则遍历并按匹配前缀分类，保持 URL 优先和邮箱边界语义；新增 URL/邮箱类型回归测试，`url_email` 单轮观测由约 4.90 ms 降至约 3.21 ms，端到端总耗时未宣称稳定收益；
- [x] 优化表格分隔线扫描：以流式 fold 替代每个候选行的临时 `Vec`，保持空单元格过滤、冒号对齐标记和最小横线长度语义；新增有效/无效行回归测试，局部耗时处于 profiling 抖动范围内，不宣称稳定毫秒级收益；
- [x] 继续优化 1 MB Markdown/LaTeX 密集语料中的字符串分配和嵌套扫描：已完成 inline-code、span 仲裁、URL/邮箱、美元数学和表格分隔线等低风险靶点；HTML 扫描器当前约 0.00–0.08 ms，未发现值得承担边界风险的进一步改动；
- [x] 评估 worker thread、可取消任务和动态 debounce：当前前端已具备 160/450/900 ms 分级 debounce、请求序列号过期结果保护、清空/卸载清理；Tauri `format_text` IPC 尚不接受取消令牌，因此暂不引入无法真正终止后端请求的 `AbortController` 或 Worker 改造；
- [x] 保持 `scripts/check_performance.py` 的数量级回归门禁：已纳入 `verify.py --profile ci`，当前 1 MB 语料全部低于宽松上限；
- [x] 长文本优化必须同时补充性能数据和行为回归：已有长文本提示、分级 debounce、端到端前端测试及 Rust 性能门禁覆盖；

## P1：前端可维护性

- [x] 让 `App.tsx` 主要承担组件编排：剩余内联逻辑已抽出（设置提醒文案/判定抽到 `frontend/src/lib/settingsLoadNotices.ts`，主界面与设置 Footer 共用消除重复）；
- [x] 抽离规则目录加载与设置恢复触发，新增 `frontend/src/hooks/useRuleCatalog.ts`；
- [x] 抽离清空输入、取消排版、清理错误、空输入持久化和完成反馈定时器，新增 `frontend/src/hooks/useClearFeedback.ts`；
- [x] 按主题、显示、规则、快捷键和状态 Footer 拆分设置界面，新增 `frontend/src/components/settings/`（`ThemeSection`、`DisplaySection`、`ShortcutsSection`、`RulesSection`、`SettingsFooter`），`SettingsDialog.tsx` 退化为编排容器；
- [ ] 清理 React 19 快捷键测试中的异步 `act` warning：已定位为“打开设置”快捷键触发的 Radix Dialog 异步挂载在 React 19 + jsdom 下的环境告警，非产品缺陷；简单包装 `act` 会产生更多 warning，需在真实 Tauri E2E（P0.2）链路中验证和闭环；
- [x] 保持 hook 不直接调用 `invoke`，继续使用 `lib/tauri.ts`（已核对：`invoke` 仅存在于 `lib/tauri.ts`；`useWindowControls` 使用 Tauri 窗口 API，不属于 IPC 后端命令）。

## P2：TUI 产品化和持续维护

- [x] 在本次 TUI 事件路由修复后重新完成 Windows Terminal（PowerShell 7）交互回归：规则默认状态排序、裸可打印字符输入、`Event::Paste`/bracketed paste、Ctrl 快捷键、OSC 52、保存和重启恢复；已记录 TUI-EDIT-DELETE-001，SSH 环境不在范围内；
- [x] 在本次 GUI 样式修复后重新完成 Windows WebView2 视觉回归：输入/输出框表面、设置间距、恢复按钮、长路径中间省略、checkbox 样式以及 100%/125%/150% DPI 和窄窗口；
- [x] OSC 52 不可用时提供明确降级提示：复制成功后状态栏说明“若粘贴为空，说明终端不支持或禁用了 OSC 52，请改用 --stdin/--output”；
- [x] 评估大文本后台任务：前端已有 50 KB/200 KB 分级 debounce、序列号过期结果保护和清理机制；TUI 已验证约 1.29 MB 非交互输入；真正可取消的后台任务仍需 Tauri IPC 取消协议与真实桌面 E2E 支持；
- [x] TUI 独立资产决策（决策 1）：发布 `CopyPolish-tui-windows-x64.7z`（内含 `CopyPolish-tui.exe`）与 `CopyPolish-tui-linux-x86_64.7z`（内含 `copypolish-tui`），与桌面版共享 Release、tag、SHA256SUMS 与发布方式；`verify_release_assets.py` 与 GitLab pipeline 已支持七资产校验；
- [x] 为 embedded 与标准 W3C provider 各连续运行至少 5 次，记录 flaky、启动耗时、端口冲突、残留进程和 artifact 完整性；
- [x] 修复 TUI-EDIT-DELETE-001：补齐最后一个 grapheme 的 Right/Delete 边界，并覆盖 ASCII、Unicode、emoji、组合字符和 TUI Delete 事件回归；
- [x] 执行并记录 Windows NTFS ACL 故障注入 spec：embedded 与标准 W3C provider 各 1/1 通过，权限恢复和 fixture 删除成功；
- [ ] 固化 GUI 浅色/深色、100%/125%/150% DPI 和窄窗口的自动截图、page source 与环境清单；主题/窄窗口场景和统一 artifact 基础设施已完成，待补 Windows 三档 DPI 原生执行与记录；
- [ ] 固化 Windows Terminal TUI raw-mode、规则面板、粘贴、OSC 52、保存/退出/重启恢复的自动 artifact；非交互 transcript 已由 `test:tui-transcript` 覆盖，待补真实终端交互 artifact；
- [x] 增加 embedded/W3C 受控失败探针，验证 stdout/stderr、WDIO log、manifest、exit status、截图、page source 和设置 fixture 的完整诊断包；统一 artifact 基础设施和双 provider 自检已完成；
- [ ] 接入并连续验证 GitLab Windows 可选 E2E stage，使用 `allow_failure: true` 和 `artifacts: when: always` 收集稳定性数据；统一 artifact 和受控失败 probe 已具备接入条件。
- [ ] 持续执行依赖审计、许可证清单更新和工具链升级 Runbook。

## 规则扩展准入

每条新规则必须同时具备：

1. `registry.rs` 中的稳定 key、展示名、默认状态和 legacy 策略；
2. 纯函数实现及明确执行阶段/依赖；
3. 单规则、组合、保护层和幂等性测试；
4. 争议性和默认开关说明；
5. README 规则表同步；
6. 设置迁移、TUI 和前端动态元数据兼容验证；
7. `CHANGELOG.md` 中的用户可见变化说明。

## 推荐执行顺序

```text
A：文档/流程基线与远程 CI 状态确认
  ↓
B：真实 Tauri E2E 与设置链路
  ↓
C：保护层、单位语义和长文本性能
  ↓
D：前端编排简化与 warning 清理
  ↓
E：TUI 跨终端验证和产品化决策
```

每完成一项，只更新本文件的待办状态；若架构、命令或流程发生变化，再同步更新对应权威文档和 `CHANGELOG.md`。