# CopyPolish 后续开发计划

本文只跟踪**尚未完成**的工作。当前架构与验证方式见 [development.md](development.md)，发布操作见 [release/manual-release.md](release/manual-release.md)，已完成版本的计划与验收记录见 [archive/](archive/)。

## 1. 规划原则

1. 正确性和结构保护优先于新增规则；
2. 先建立可观测基线与回归测试，再替换实现；
3. 默认不改写可能影响语义的 Unicode 表示；
4. 大型依赖和跨平台能力先做独立 Spike，记录体积、性能和维护成本；
5. 每个里程碑必须同时包含实现、测试、文档和可复现验证结果。

## 2. 当前基线与主要风险

当前生产入口已经使用 span-aware 混合管线，规则注册表、阶段依赖、结构/语义 span 和 UTF-8 安全 TextEdit 基础设施均已落地。桌面 GUI 与实验性 TUI 共用 Rust 引擎和 `rules.yaml`。

主要风险：

- 保护层仍使用内部 placeholder 承载不可编辑 span（TextEdit 应用层已覆盖全部规则阶段，剩余工作是减少占位符依赖、让语义边缘空格完全脱离占位符路径）；
- Markdown/HTML 保护仍是保守扫描器与有限正则的组合，不是完整语法解析器；
- 长文本基准曾暴露旧保护正则的回溯上限；当前已移除 `fancy-regex` 保护依赖并建立数量级性能门禁，但复杂 Markdown/LaTeX 语料仍需继续优化；
- 真实 Tauri 桌面链路缺少自动化 E2E；
- 前端核心状态仍集中在 `App.tsx`，异步格式化和设置保存逻辑可维护性有限；
- 依赖审计和真实 Tauri 桌面链路仍未闭环。

## 3. P0：工程门禁与发布可靠性

- [x] 增加常规开发分支 CI：GitHub Actions（`.github/workflows/ci.yml`）在 push/PR 到 `dev`/`master` 时运行 Rust fmt/clippy/test（默认 + TUI feature 与 TUI 构建）、前端 vitest/build、secret/SOPS 安全扫描和 Markdown 链接检查；`git diff --check` 在本地 Runbook 中保留。**当前阻塞**：GitHub Actions 账户存在计费阻塞，workflow 会在启动后立即失败（与本仓库内容无关）；需在账户设置中解除计费阻塞后 CI 才能真正生效，在此之前本地 Runbook 仍是权威门禁；
- [x] Dependabot 的 GitHub Actions 生态配置已随常规 CI 恢复而重新生效；
- [x] 增加 Markdown 相对链接检查：`scripts/check_md_links.py`（已在 CI `checks` job 中启用），阻止删除或移动文档后留下死链；
- [x] 将发布前检查封装为单一入口 `scripts/verify.py`，按 `rust` / `frontend` / `checks` / `security` / `audit` / `release` profile 供本地、GitHub Actions、GitLab job 和发布脚本复用；
- [x] 完成一次离线 age 私钥恢复演练并记录结果；演练使用临时测试密钥和临时 `SOPS_AGE_KEY`，未读取或输出生产令牌，验证了备份私钥可独立恢复 SOPS 解密能力；

验收：在干净 clone 中可以用文档命令完成全部检查，失败信息能定位到具体门禁。

## 4. P0：完成 Span/Edit 迁移

- [x] 审阅 `structure-precedence.yaml`，确认 span-aware 输出是产品预期；
- [x] 将结构优先级样例迁入稳定黄金回归，不再作为 pending 差异观察；
- [x] 把剩余结构边界与文本边界规则迁移到 TextEdit 策略；全部规则阶段（标点/名词/结构边界/文本边界/清理）现均经 `edit_plan.rs` 的 TextEdit 应用层执行，保护层行循环已删除；
- [x] 移除普通/数学多套 placeholder 编号和旧 placeholder pipeline；
- [x] 删除仅用于新旧路径等价对照的测试，改为生产路径行为测试；
- [x] 验证稳定 fixture、换行风格、幂等性和未知规则 key 行为不变。

验收：代码中只保留一条可执行格式化管线，结构保护优先级由 span 仲裁统一解释；剩余 placeholder 仅作为内部保护实现，不再存在第二套生产 pipeline。

## 5. P1：复杂排版与 Unicode 能力

### 5.1 Markdown/HTML 状态机收敛

- [x] 反引号、平衡括号链接和 HTML block 已由 span 扫描器覆盖，并补齐嵌套括号、任意长度 delimiter、未闭合 delimiter 与大小写混合 HTML 闭合标签回归；
- [x] 移除旧 `fancy-regex` 保护路径依赖：结构 span 扫描器已覆盖生产保护入口，链接间距规则改用标准 `regex`，并通过 86 项 Rust 测试、Clippy 和 1 MB 性能门禁验证边界行为与性能；
- [x] 明确未闭合 fenced code、嵌套链接、引用式链接、HTML 可见文本和 LaTeX 嵌套环境的产品策略；统一遵循“宁漏格式化、不破坏结构”，具体契约见 `docs/development.md` § Rust 引擎要点。
- [x] 补齐 Unicode 域名 URL、括号包裹 URL 和复杂化学式歧义回归；生产测试验证 URL/化学式内容不被破坏，周边空格仍由既有规则决定；
- [x] 保持“宁漏格式化、不破坏结构”，不把整篇 Markdown 文档全部冻结；现有结构扫描器、保护层和生产回归共同验证该策略。

### 5.2 单位与数学语义

- [ ] 继续按真实语料扩展有限单位词典；当前已补齐 `km`、`kHz`、`kPa`、`kW`、`mM`、`μM`、`mmol`、`μmol`、`mAh`、`kWh` 的稳定回归，仍禁止用宽泛 `\p{L}+` 代替语义词典；
- [x] 明确数学表达式与中文之间的空格策略并纳入 `mathematical-symbols.yaml`：表达式内部保持完整，仅在与 Han 直接相邻时插空，不在全角标点后额外插空；浏览器 fallback 仍是非等价的最小演示实现。
- [x] 评估温度相关规则 key：保留独立 stable key `spacing.temperature-cjk`，默认启用且当前没有历史 legacy alias；注册表元数据回归已冻结该契约。
- [ ] 新规则必须遵循第 9 节准入流程。

### 5.3 Unicode 等价识别

- [x] 允许识别层把 `µ/μ`、`Å/Å` 等视为等价语义：`SemanticToken` 提供 canonical unit key，识别等价不改变原始输出字符；
- [x] 输出规范化已作为独立且默认关闭的规则 `text.unicode-equivalents` 落地，仅显式选择时执行 `µ→μ`、`Å→Å`；
- [x] 禁止默认全文 NFKC：默认格式化不改写数学字母、兼容字符和受保护结构；后续只评估更多有限映射。

## 6. P1：ICU4X 技术验证

仅评估纯 Rust ICU4X，不引入 ICU C/C++ FFI。

- [x] 在仓库外隔离 Spike 中评估 ICU4X 2.3.0 的 Script/General Category API 是否能替代手写 Unicode 区间；
- [x] 记录二进制体积、编译时间和扫描性能：手写版 467,920 bytes / 1.00 s / 64.7 ms，ICU4X 版 512,328 bytes / 7.56 s / 91.6 ms；跨平台打包未执行，因成本数据已足以支持暂不引入的决策；
- [x] 对比 Spike 样本分类汇总值与手写实现一致；未修改正式黄金样例或默认行为；
- [x] 结论：ICU4X 当前没有解决已确认的产品问题，且构建、体积和运行时成本更高，因此不进入正式依赖；重新引入必须先用 feature flag/独立分支完成完整差异和跨平台评审。

## 7. P1：真实 Tauri E2E

- [ ] Spike `tauri-driver` 或当前 Tauri 2 支持的 WebDriver 路径；
- [ ] 覆盖启动、真实引擎输出、全不选恒等、规则切换、快捷键开关、设置保存与重启恢复；
- [ ] 使用临时设置目录，禁止污染真实 `rules.yaml`；
- [ ] 覆盖损坏设置恢复和不可写目录错误；
- [ ] Linux 与 Windows 至少各保留一条真实链路，稳定后纳入合并门禁。

## 8. P1：性能基准与长文本体验

- [x] 重跑并扩展 `benchmarks/unicode-baseline.md` 的 10 KB/100 KB/1 MB 五类语料（TextEdit 迁移与热点修复后的完整实测数据已记录）；
- [x] 记录耗时与正则热点：热点为词级规则每片段重新编译正则（已修复：预编译缓存）、占位符巨型拼接正则（已修复：单一通用模式 + 成员集合）；峰值 RSS 已测量：release 基准进程约 63.0 MiB，详见 `benchmarks/unicode-baseline.md`；
- [x] 可控处理正则上限：占位符正则编译超限 panic 已消除（1 MB 文本不再 `CompiledTooBig`）；GUI 侧已有请求序号守卫，格式化错误只呈现错误状态、不会用旧结果覆盖新输出；
- [ ] 优化 1 MB 级 Markdown/LaTeX 密集语料（当前约 1.32–1.38 s，已较此前约 4.9 s 改善）：已完成结构扫描共享单次行范围表，减少块级扫描器重复分割文本；端到端尚未证明稳定加速，后续继续减少重复字符串分配和嵌套结构扫描（与 §5 合并推进）；
- [ ] 评估 worker thread、可取消任务和动态 debounce；
- [x] 基于实测数据建立数量级性能回归门禁：`scripts/check_performance.py` 对 1 MB 五类语料执行宽松阈值检查（普通语料 500 ms、Markdown/LaTeX 5 s），并接入 GitHub Actions；详细基准仍保留在 `benchmarks/unicode-baseline.md`。

## 9. P1：规则扩展准入

每条新规则必须同时具备：

1. `registry.rs` 中稳定 key、展示名、默认状态和 legacy 别名策略；
2. 纯函数实现及明确执行阶段/依赖；
3. 单规则黄金样例、组合样例、保护层样例和幂等性断言；
4. 争议性与默认开关说明；
5. `README.md` 规则表同步；
6. 对设置迁移、TUI 和前端动态元数据的兼容验证。

## 10. P1：前端可维护性

按行为不变原则逐步拆分：

```text
frontend/src/hooks/
├── useFormatter.ts
├── useUserSettings.ts
├── useSettingsLoader.ts
├── useSettingsPersistence.ts
├── useSettingsActions.ts
├── useThemeAndFont.ts
├── useShortcuts.ts       # 已存在
└── useWindowControls.ts
```

- [x] 抽离格式化请求序号、竞态防护、耗时和错误状态：新增 `frontend/src/hooks/useFormatter.ts` 及独立测试，App 集成测试与 hook 测试共 36 项通过；
- [x] 抽离设置初始化和恢复提醒：新增 `frontend/src/hooks/useSettingsLoader.ts` 及独立测试；设置保存队列已抽离为 `frontend/src/hooks/useSettingsPersistence.ts`，主题/字体副作用已抽离为 `frontend/src/hooks/useThemeAndFont.ts`；
- [ ] `App.tsx` 最终只保留组件编排；窗口控制、设置加载/保存、设置动作和剪贴板反馈已抽离，规则加载、输入调度、清空和页面编排仍保留在 App；
- [x] hooks 不直接调用 `invoke`，`useFormatter` 继续通过 `frontend/src/lib/tauri.ts` 的 `formatText` 封装访问后端；
- [ ] 清理当前前端测试中的 React `act` warning；当前仅剩快捷键测试中的 React 19 异步 document listener 告警，行为测试本身 36/36 通过；
- [x] 清理 Node localStorage warning：测试 setup 直接使用进程内内存存储，避免 Node 24+ `--localstorage-file` experimental warning；
- [x] 处理 Vite 配置中 `__dirname` 对未来 native config loader 的兼容警告：改用 `import.meta.dirname`。

## 11. P2：依赖与安全维护

- [x] 确定 `cargo audit`、`npm audit` 的高危阻断策略：`critical` / `high` 级别阻断发布，`moderate` / `low` 仅记录并进入维护队列；统一入口为 `python3 scripts/verify.py --profile audit`，工具缺失或审计源不可访问时失败，不将未完成审计误报为通过；本次 npm 审计为 0 vulnerabilities，Rust 审计完成并发现 20 个已允许的 unmaintained / unsound warning，纳入后续依赖维护队列；
- [x] 生成并维护第三方许可证清单：`docs/licenses.md` 由 `scripts/generate_licenses.py` 从 Cargo metadata 和前端已安装依赖元数据生成，当前覆盖 432 个 Rust 依赖条目和 164 个 npm 依赖条目，缺失许可证字段为 0；
- [x] 收紧 `tauri.conf.json` CSP，并完成 Linux 桌面 smoke：生产环境仅允许本地资源与 Tauri IPC，开发环境单独允许 Vite localhost/HMR；前端测试 33 项、前端构建和 Tauri `--no-bundle --ci` release 构建均通过；
- [x] 建立 Node/Rust/Tauri/React 升级 Runbook：`docs/upgrade-runbook.md` 覆盖版本固定、lockfile、Dependabot、分层升级、统一验收、Tauri smoke 和回滚流程；
- [x] 建立并验证 `scripts/security_check.py --require-sops` 和令牌轮换流程：真实仓库强制 SOPS 校验及 `load_tokens.sh` 变量存在性检查通过，临时 SOPS 文件轮换演练验证新值覆盖旧值；真实 GitLab token 的平台侧吊销仍需维护者按 `docs/secrets-management.md` 执行并留存审计记录；

## 12. P2：TUI 后续工作

TUI MVP 已完成，后续仅在不分散桌面主线资源时推进：

- [ ] Windows Terminal、Linux 终端和 SSH 环境 smoke；
- [ ] OSC 52 不可用时的明确降级提示；
- [ ] 大文本 debounce/后台任务；
- [ ] 评估是否发布独立 `CopyPolish-TUI-*` 资产。

## 13. 推荐执行顺序

```text
里程碑 A：§3 工程门禁 + §4 Span/Edit 单路径
    ↓
里程碑 B：§8 性能基线 + §5 Markdown/语义收敛
    ↓
里程碑 C：§7 真实 E2E + §10 前端状态拆分
    ↓
里程碑 D：§6 ICU4X Spike + §9/§11/§12 持续维护
```

每完成一项，只更新本文件的待办状态；若架构或命令发生变化，再同步更新 `development.md`。已完成且只具历史意义的过程记录移入 `archive/` 或由 Git 历史保留。