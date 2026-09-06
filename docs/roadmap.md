# CopyPolish 后续开发计划

本文只跟踪尚未完成的工作。当前实现见 [architecture.md](architecture.md) 和 [development.md](development.md)，测试策略见 [testing.md](testing.md)，发布操作见 [release/manual-release.md](release/manual-release.md)，历史计划与验证记录见 [archive/](archive/)。

## 规划原则

1. 正确性和结构保护优先于新增规则；
2. 先建立可观测基线和回归测试，再替换实现；
3. 默认不改写可能影响语义的 Unicode 表示；
4. 大型依赖和跨平台能力先做独立 Spike；
5. 每个里程碑同时包含实现、测试、文档和可复现验证。

## 当前基线

span-aware 混合管线、规则注册表、阶段依赖、结构/语义 span 和 UTF-8 安全 TextEdit 已落地；桌面 GUI 与 TUI 共用 Rust 引擎和 `rules.yaml`。Windows 原生验证（E2E、设置损坏/ACL、GUI DPI 人工三档、Windows Terminal TUI 交互）均已完成或按项目决策跳过；默认构建重启 spec 已按 capability=false 语义修正并完成验证，记录见 [windows-e2e-runbook.md](windows-e2e-runbook.md) 与归档。标准 W3C provider 已于 2026-09-01 收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`），不再与 embedded provider 并行跑完整回归。

## v0.6.2 维护版本范围（功能冻结）

v0.6.2 定义为**隐私、安全、依赖和供应链维护版本**，不增加用户功能、不新增排版规则、不改变现有规则默认行为。除非维护者明确指定，当前不得构建或发布 v0.6.2 Pre-Release/正式版。

在 v0.6.2 安全维护完成并发布前，v0.7.0 的新功能开发冻结；不得提前实现格式化 diff/撤销、规则搜索、用户预设、PDF/CAJ 清洗、正则替换或其它产品功能。

### v0.6.2 维护阶段

1. **S2-A 基线与资源边界**：合并安全审计模型和输入/设置资源限制；已完成 PR #35、PR #36，覆盖 GUI/TUI/CLI 共用请求模型、设置加载/保存和审计门禁。
2. **S2-B 设置存储安全**：完成 Unix `0600` 私有权限、Unix symlink 拒绝、进程内并发保存串行化、原子计数器保证唯一临时文件名、临时文件失败清理和备份恢复测试；Windows reparse point/junction 与跨进程并发已由用户于 2026-09-06 在原生环境手动确认通过，审计级 artifact 复现仍按需补充。
3. **S2-C IPC 错误边界**：建立稳定错误 code，避免错误消息泄露正文、凭据和非必要内部路径；补齐 command 级边界测试。
4. **S2-D E2E 供应链**：完成复核——WebdriverIO 9.31.5 → 9.31.6 小版本升级未能覆盖 `GHSA-jmr9-qjv8-65gv`，`npm audit fix --force` 仅提供破坏性的 WebdriverIO 8 降级，均未采用；维持 `docs/decisions/e2e-audit-policy.json` 限期风险登记（review_after 2026-10-06），在兼容 WebdriverIO 升级或 provider 替换前按季度复核。
5. **S2-E CI 与 Actions**：固定 GitHub Actions commit SHA、收紧 job permissions、校验 workflow 输入并加入 artifact/privacy scan。
6. **S2-F 发布完整性**：生成生产 CycloneDX SBOM（`scripts/generate_sbom.py`，已接入 `verify.py --profile audit` 与 release workflow 的 assemble/发布），审查 Release 资产内容并确保 checksum/license 清单可重复生成；许可证清单现由 `scripts/generate_licenses.py --check` 纳入 audit 门禁；provenance/attestation 与签名机制留待后续评估接入。
7. **S2-G 最终审计**：完成全量 CI、依赖审计、隐私扫描、设置安全矩阵和跨平台 smoke，形成 v0.6.2 安全维护审计报告。

### v0.6.2 发布门槛

- 生产依赖无未解释的 high/critical 漏洞；
- 用户正文默认不落盘，主文件、备份、临时文件和测试 artifact 均不泄露正文；
- IPC、格式化请求和设置文件均有资源边界；
- 设置权限、symlink、并发和备份恢复行为完成验证；
- E2E advisory 已修复，或有负责人、缓解措施和明确到期日期的正式风险接受；
- Actions SHA、job 权限、SBOM、provenance、checksum、license 和 Release 资产检查完成；
- 完成一次独立的 v0.6.2 安全维护审计；
- 维护者明确指定前，不创建 v0.6.2 tag、不构建、不发布。

### v0.7.0 启动条件

仅在 v0.6.2 安全维护完成并按发布门槛发布后，才允许从 `dev` 启动 `v0.7.0-dev.1`，再恢复新功能开发。

## P0：仓库卫生与事实来源收敛

- [x] 增加统一的安全清理入口（`scripts/clean.py`，白名单删除 `src-tauri/target/`、`frontend/dist/`、`src-tauri/gen/`、`scripts/__pycache__/`、`e2e/artifacts/` 与 `e2e/settings-*`，支持 `--dry-run`/`--deep`）；
- [x] 将「测试后清理本地 artifact、远程仅记录测试结论」写入 `docs/development.md` 与 `CONTRIBUTING.md`；
- [x] 推送前复核未推送提交不含截图、日志、临时设置或 artifact 路径（2026-09-01 复核：`origin/dev..HEAD` 全部为源码/文档，无二进制、日志或设置文件）。

## P0：依赖与安全维护

- [x] 运行当前依赖审计（`verify.py --profile audit` 口径）：frontend 生产依赖 0 漏洞；e2e 测试链在 2026-09-01 复核后报告 13 项 high，涉及 `@wdio`、`puppeteer`、`extract-zip` 等传递依赖；Cargo 0 漏洞、20 项允许的 unsound 警告（`lru` 等传递依赖）；
- [x] 修复 e2e 测试链 `serialize-javascript` 高危告警：在 `e2e/package.json` 增加 npm `overrides` 固定到 `7.1.1`，保留 WebdriverIO 9/Mocha 10；`npm ci`、类型检查和审计验证通过，决策记录见 [decisions/wdio-serialize-javascript.md](decisions/wdio-serialize-javascript.md)。
- [x] 修复 E2E 传递依赖 `deepmerge-ts` high 告警：在 `e2e/package.json` 增加 override 固定到 `8.0.2`，保留 WebdriverIO 9；干净安装、动态导入、类型检查和审计验证通过，记录见 [decisions/wdio-transitive-dependencies.md](decisions/wdio-transitive-dependencies.md)。
- [ ] 持续跟随 WebdriverIO/@wdio 及浏览器工具升级，处理剩余 13 项 E2E 传递依赖 high 告警；当前由 `scripts/verify.py --profile audit` 透明登记 `GHSA-jmr9-qjv8-65gv`，`@puppeteer/browsers`/`extract-zip` 暂不覆盖，等待完整 provider 回归和工具链升级窗口；
- [x] 对 `serde_yaml`（上游 deprecated）迁移做独立 Spike：结论为**暂不迁移、保持观察**（无漏洞告警、使用面仅 2 处；若迁移首选 API 兼容的 `serde-yaml-ng` 并跑全量 round-trip 对照），记录见 [decisions/serde-yaml-migration.md](decisions/serde-yaml-migration.md)；
- [x] 重新生成并审阅 `docs/licenses.md`（2026-09-01：生成脚本改为读取 `frontend/package-lock.json` 的完整 `packages` 条目；Rust 431 条、npm 294 条、许可证字段缺失 0 条；重复生成结果稳定）。

## P1：引擎正确性

- [x] 建立真实语料 corpus（`src-tauri/tests/fixtures/real-world-corpus.yaml`，覆盖技术文档、产品文案、Markdown README、HTML/LaTeX 科研文本、单位/化学式/emoji、未闭合结构和 CRLF，8 条样本）；每条样本记录结构保护要求与当前预期输出。
- [x] 增加属性测试（`src-tauri/tests/properties.rs`，确定性伪随机语料：幂等性、任意规则选择不 panic、emoji grapheme 不拆分、CRLF/CR 换行还原、fenced code/行内代码/URL/邮箱/LaTeX/化学式保护、legacy key 归一化幂等，5 项测试）；
- [x] 增加注册表与 README 规则表一致性自动检查（`src-tauri/tests/readme_registry.rs`：表格行数、分类、展示名、「，默认关闭」后缀、stable key 与 `enabled_defaults` 一致性，2 项测试）；
- [x] 按真实技术语料扩展有限单位词典：补充二进制容量 `KiB/MiB/GiB/TiB` 和比特速率 `bps/kbps/Mbps/Gbps/Tbps`，并增加 `bit/bytes` 普通单词反例。
- [x] 完成 Placeholder 重构设计 Spike：比较受控 placeholder、全程 span/TextEdit、分段 rope 三方案，并用现有 fixture、真实语料和 1 MB `profile-stages` 基线做兼容性/性能归因；结论为暂不大规模重构，保持当前混合管线，记录见 [decisions/placeholder-migration.md](decisions/placeholder-migration.md)。

## P0：文本清洗工作流基线

- [x] 完成参考 `paper-assistant` 的功能对照和产品边界决策：CopyPolish 扩展为“文本清洗与规范排版”工具，但不解析 PDF/DOCX 文件本体、不加入翻译/AI/Grammarly；记录见 [decisions/text-cleaning-workflow.md](decisions/text-cleaning-workflow.md)。
- [x] 扩展规则元数据以区分清洗、字符转换、规范排版及风险等级，并保持 README、GUI、TUI 和 CLI 的稳定 key 兼容；`RuleMeta` 的说明、类型和风险字段已接入 Rust、GUI、TUI、README 一致性测试，stable key 和设置兼容性保持不变；
- [x] 设计并实现清洗/转换/排版的统一请求模型：`FormatRequest` 扩展 `replacements` 与 `conversion` 字段；预设只展开为统一请求模型，不复制规则实现；自定义替换在 span 保护前执行，简繁转换互斥并由请求模型保证；核心 phase/依赖顺序不变，用户不能拖拽覆盖；决策记录见 [decisions/unified-request-model.md](decisions/unified-request-model.md)。

## P1：来源文本清洗

- [x] 增加方括号、中文方括号引用角标清理：默认关闭，不能误删 Markdown 链接、代码、公式或 URL；
- [x] 增加连续空行和普通文本重复空格清理：保护代码、公式、表格和硬换行；当前已拆为 `cleanup.limit-blank-lines` 与 `cleanup.collapse-horizontal-spaces` 两个稳定 key；
- [x] 增加数值标点异常空格修复：显式覆盖小数点、时间/比例、数字分组场景，保留版本号/IP 连续点号数字链，并补充误改反例；规则默认关闭；
- [x] 增加康熙部首修复：采用 Unicode 17.0.0 `UnicodeData.txt` 的 214 项兼容分解映射，默认关闭并记录数据来源；
- [ ] 增加结构感知的 PDF 段内软换行修复和 CJK 内部异常空格清理：默认关闭；当前仅完成未经真实 PDF/CAJ 语料验收的 `cleanup.cjk-internal-space` 保守试实现，段内软换行、多栏顺序及正式 CJK 空格验收仍待真实语料、人工标注和失败率基线；

## P1：字符转换与用户工作流

- [x] 完成全角 ASCII 转半角 Spike/实现：不使用全文 NFKC；新增默认关闭的 `text.halfwidth-ascii`，仅转换全角 ASCII 字母/标点，全角数字继续由 `text.halfwidth-digits` 负责，并通过现有结构保护跳过链接、代码、公式和化学式；
- [x] 完成简繁转换 Spike 并用 `opencc-fmmseg`（MIT、OpenCC 风格词典 + FMM 分词）接入：确认许可证/纯 Rust/性能/体积（1 MB `s2t` ≈130 MB/s、字节增量约 2.1 MB），结构感知只转可编辑区间；以 `simplified-trad-conversion` 可选 feature 落地（默认构建不启用，保持占位）。决策与语义边界见 `docs/decisions/simplified-trad-conversion-spike.md`；
- [x] 增加自定义字面量替换（有序、仅 active、span 保护前执行、首版不支持正则）：作为请求层阶段随统一请求模型落地；
- [x] 在桌面 GUI 中接入替换列表和简繁转换选择器，完成请求透传、实时重排与 `rules.yaml` 持久化；
- [x] 补齐 GUI 替换/转换交互回归：覆盖组件操作、设置恢复、旧字段默认值、持久化、实时请求以及快捷键立即排版设置透传；
- [x] 增加并验证 GUI E2E 的替换/转换保存与重启恢复 spec：Linux/WSL 环境 环境 embedded provider 的设置保存与替换输出 3/3、Windows 当前 binary 3/3、重启恢复 write/read 各 1/1 通过；
- [x] 增加并验证简繁转换 feature 专用 GUI E2E 构建与 spec：Linux/WSL 环境 环境 与 Windows 独立 feature binary 均双向真实转换 2/2 通过；标准 W3C provider 不运行该 feature spec；
- [x] 确认简繁转换构建能力与 GUI 语义决策：默认构建不静默承诺 t2s/s2t，feature 构建通过 capability 明确启用；实现与正式发布 feature 是否默认开启分别按后续验收推进，记录见 [decisions/simplified-trad-capability.md](decisions/simplified-trad-capability.md)；
- [x] 在 TUI 中接入替换列表和简繁转换控件：通过 `Ctrl+E` 请求设置面板复用 `FormatRequest` 与共享 `rules.yaml`，支持替换增删/编辑/启停、转换模式循环、默认构建 capability 归一化和 feature 构建真实转换；
- [x] 增加中文文案、PDF 清洗和技术文档预设：通过 `get_presets` 同时接入 GUI/TUI，预设只展开为统一请求模型，不复制规则实现；PDF 预设只处理复制出的文本，不解析文件本体；
- [x] 增加实时/手动输出模式、自动/左右/上下布局和输入输出统计：手动模式保留显式“立即排版”动作，统计按 Unicode code point 计算并随设置持久化；
- [x] 增加复制后的显式动作（保留/复制并清空），不使用窗口失焦自动复制或自动清空；复制失败时不清空内容；
- [x] 增加静态帮助和首次使用提示，明确高风险清洗规则、结构保护和浏览器演示模式边界；首次提示状态仅保存在前端 localStorage，不改变 Rust/TUI 设置。

## P1：设置存储策略决策

- [x] 确认存储策略 ADR（[decisions/settings-storage-policy.md](decisions/settings-storage-policy.md)）并按方案 B 落地：exe 目录优先，不可写时回退平台应用数据目录并提示 `UsingAppDataFallback`；TUI/GUI 共用；6 项决策单测覆盖（同目录优先/双位置并存/只读回退/均不可读/探针/legacy 固定）。

## P2：E2E 收敛

- [x] Embedded provider 保留完整回归；标准 W3C provider 缩减为兼容性 smoke（session、主窗口、一次真实格式化、一次设置保存、退出清理）；`specs/w3c/smoke.spec.ts` 已建立，`wdio.webdriver.conf.ts` 与 `run-webdriver-specs.ts` 已同步指向 `specs/w3c/`，`package.json` 中各 `:webdriver` 专项脚本已移除。
- [x] 处置 GUI DPI 自动矩阵脚本（`run-gui-dpi-pair.ts`、`validate-gui-dpi-matrix.ts` 已删除，`test:gui-dpi` 命令移除；`run-gui-visual-artifacts.ts` 保留并记录 DPI 环境元数据）：DPI 采用发布前人工检查。
- [x] 所有会创建临时设置目录的 runner 均使用统一 `try/finally` 清理路径，并将非零子进程状态写入 `process.exitCode` 后退出循环；测试结果只记录摘要，不提交 artifact。

## P2：TUI 产品定位

- [x] 在「实验性 / Beta / 正式支持」中明确定位：**Beta**（CLI 参数、规则选择与 `rules.yaml` 格式保持兼容；终端显示能力取决于终端支持，SSH 不在验证范围）。README 终端版章节、发布资产说明（TUI 独立资产沿用既有 7 资产发布）已同步。

## P2：前端编排收敛

- [x] 将 `App.tsx` 的业务 hook 编排与设置组装抽出到 `useAppController`，不引入全局状态库；`App.tsx` 仅负责页面渲染和组件组合。
- [x] 为 `useAppController` 增加独立编排契约测试（`frontend/src/hooks/useAppController.test.ts`）：覆盖设置恢复、清空后的空输入保存、输入格式化/保存调度连接和浏览器演示模式。
- [x] 为浏览器 fallback 增加醒目的「演示模式」标识（`data-testid="demo-mode-banner"`），明确浏览器预览不代表桌面版 Rust 引擎的完整行为。

## P2：发布持续维护

- [x] 将 Rust、frontend 和 E2E 依赖审计统一纳入 `scripts/verify.py --profile audit`；E2E 已接受风险必须登记在 [decisions/e2e-audit-policy.json](decisions/e2e-audit-policy.json)，未登记 high/critical 或审计网络/JSON 失败仍阻断。
- [ ] 持续执行依赖审计、许可证清单更新和工具链升级 Runbook；该维护项属于 v0.6.2 安全维护周期，完成前不启动 v0.7.0 功能开发。

## 规则扩展准入

每条新规则必须同时具备：

1. `registry.rs` 中的稳定 key、展示名、默认状态和 legacy 策略；
2. 纯函数实现及明确执行阶段/依赖；
3. 单规则、组合、保护层和幂等性测试；
4. 争议性和默认开关说明；
5. README 规则表同步；
6. 设置迁移、TUI 和前端动态元数据兼容验证；
7. `CHANGELOG.md` 中的用户可见变化说明。

每完成一项，只更新本文件的待办状态；若架构、命令或流程发生变化，再同步更新对应权威文档和 `CHANGELOG.md`。

## 历史验证记录

2026-09 的 Windows 原生复验与收尾记录已迁入 [archive/validation/windows-2026-09.md](archive/validation/windows-2026-09.md)；当前 Windows 自动验证无待闭环失败，后续仅在相关代码、工具链或诊断范围变化时按 [windows-e2e-runbook.md](windows-e2e-runbook.md) 复跑。
