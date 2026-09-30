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

## 版本与维护历史

- v0.6.2 是隐私、安全、依赖和供应链维护版本，已于 **2026-09-07** 正式发布，tag 指向 `master` 的 `0.6.2` commit。当前开发基线已提升为 `0.7.0-dev.1`。
- 详细维护阶段、发布门槛、Windows 原生验收步骤与已完成的验证记录见：
  - [archive/releases/v0.6.2.md](archive/releases/v0.6.2.md)
  - [archive/validation/windows-2026-09.md](archive/validation/windows-2026-09.md)
  - [release/manual-release.md](release/manual-release.md)
  - [windows-e2e-runbook.md](windows-e2e-runbook.md)

## P0：仓库卫生与事实来源收敛

- [x] 增加统一的安全清理入口（`scripts/clean.py`，白名单删除 `src-tauri/target/`、`frontend/dist/`、`src-tauri/gen/`、`scripts/__pycache__/`、`e2e/artifacts/` 与 `e2e/settings-*`，支持 `--dry-run`/`--deep`）；
- [x] 将「测试后清理本地 artifact、远程仅记录测试结论」写入 `docs/development.md` 与 `CONTRIBUTING.md`；
- [x] 推送前复核未推送提交不含截图、日志、临时设置或 artifact 路径（2026-09-01 复核：`origin/dev..HEAD` 全部为源码/文档，无二进制、日志或设置文件）。

## P0：依赖与安全维护

- [x] 运行当前依赖审计（`verify.py --profile audit` 口径）：frontend 生产依赖 0 漏洞；e2e 测试链在 2026-09-01 复核后报告 13 项 high，涉及 `@wdio`、`puppeteer`、`extract-zip` 等传递依赖；Cargo 0 漏洞、20 项允许的 unsound 警告（`lru` 等传递依赖）；
- [x] 修复 e2e 测试链 `serialize-javascript` 高危告警：在 `e2e/package.json` 增加 npm `overrides` 固定到 `7.1.1`，保留 WebdriverIO 9/Mocha 10；`npm ci`、类型检查和审计验证通过，决策记录见 [decisions/wdio-serialize-javascript.md](decisions/wdio-serialize-javascript.md)。
- [x] 修复 E2E 传递依赖 `deepmerge-ts` high 告警：在 `e2e/package.json` 增加 override 固定到 `8.0.2`，保留 WebdriverIO 9；干净安装、动态导入、类型检查和审计验证通过，记录见 [decisions/wdio-transitive-dependencies.md](decisions/wdio-transitive-dependencies.md)。
- [x] 处置 Dependabot `lru` low 告警（GHSA-rhfx-m35p-ff5j）：随 ratatui 0.29.0 → 0.30.2（配套 crossterm 0.28 → 0.29）升级消除，lockfile 中 `lru` 已达 0.18.4（≥0.16.3 修复版）；Rust 全量验证（含 TUI feature clippy/test/build 与性能门禁）与 audit 通过。Windows Terminal 原生 TUI 交互回归仍需在发布演练时按既有 Runbook 复验。
- [x] 处置 Dependabot `glib` medium 告警（GHSA-wrw7-89jp-8q8g）：修复版本 0.20.0 需要 Tauri/Wry GTK 0.20 代系迁移，超出 v0.6.2 安全维护边界；建立限期风险接受（owner maintainers，复核期限 2026-12-31），见 [decisions/glib-0.18-soundness-risk.md](decisions/glib-0.18-soundness-risk.md)。
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

## P1：CLI 非交互模式补齐（v0.7.0 优先）

- [x] 增加非交互 CLI 的 `--preset <copywriting|pdf-cleaning|technical-docs>` 参数：复用内置预设展开为统一 `FormatRequest`，与 `--rules`/`--enable`/`--disable` 组合时，预设作为基础、显式规则参数在其上微调；自定义字面量替换与简繁转换参数当前不实现，因此 `--preset` 仅展开规则选择，不含替换与转换。

## P1：来源文本清洗语料框架与评测基线

- [x] 建立本地语料收集规范（`docs/source-corpus-spec.md`）和评测脚本（`scripts/evaluate_corpus.py`），支持 TP/FP/FN 统计与误改率计算。
- [ ] 收集至少 20 条真实脱敏 PDF/CAJ/Zotero 语料（覆盖单栏/多栏/表格/公式），放入 `src-tauri/tests/fixtures/corpus-local/`（不提交仓库）。
- [ ] 基于真实语料完成 `cleanup.cjk-internal-space` 的正式验收，误改率 < 2%。
- [ ] 根据语料结果决定是否进入段内软换行实现（Go/No-Go 决策）。

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

## P1：GUI 中文布局与排版（v0.7.0 进行中）

- [x] 按中文阅读习惯重排主界面：保留“原始文本 → 排版结果”工作流，突出编辑区层级，统一标题/正文/辅助说明字号与行距；
- [x] 将设置长列表改为按任务分类（排版规则、替换与转换、编辑与输出、外观显示、快捷键、隐私与存储），规则批量操作放回规则分类；
- [x] 修正模式相关文案：主界面标题、说明与空状态跟随实时/手动输出模式，不再固定宣称实时生成；
- [x] 优化操作位置与反馈：输入区对应清空动作，结果区对应复制动作，主操作使用明确的复制成功状态；
- [ ] 覆盖默认窗口（920×720）与最小窗口（800×600）、窄屏堆叠、浅色/深色主题、80%–125% 界面缩放与中英文混排渲染检查（Linux 侧已通过 Vitest 回归，渲染验收待 Windows 原生执行）；
- [x] 补充组件回归测试并执行 `frontend` + `checks` 验证；Windows 原生（WebView2/DPI/窗口控制/剪贴板）进入集中验证队列 Q4。

## P1：设置存储策略决策

- [x] 确认存储策略 ADR（[decisions/settings-storage-policy.md](decisions/settings-storage-policy.md)）并按方案 B 落地：exe 目录优先，不可写时回退平台应用数据目录并提示 `UsingAppDataFallback`；TUI/GUI 共用；6 项决策单测覆盖（同目录优先/双位置并存/只读回退/均不可读/探针/legacy 固定）。

## P0：开放 PR 评估修复（进行中）

本节记录对仓库全部开放 PR 的评估结论与后续修复计划。评估范围为 12 个开放 PR（含 1 个功能 PR 与 11 个 Dependabot PR）。

### 评估结论要点

- **#86（GUI 中文布局）存在两处必须修复的问题**：设置分类化后，现有 GUI E2E 未同步分类切换，导致 `selection-and-persistence`、`restart-settings`、`gui-visual-artifacts` 等 spec 访问未挂载控件；且设置底栏「恢复默认」在非规则分类仍调用规则恢复回调，语义误导。
- **#80（opencc-fmmseg 0.12.1）无版本不一致问题**：该 PR 分支的 `Cargo.toml` 已为 `0.12.0` 且与 `Cargo.lock` 一致，`dev` 已通过 #73 升级到 0.12.0。此前评估基于落后分支得出的「manifest/lock 不一致」结论不成立。
- **#80 的真实缺口是验证覆盖**：`scripts/verify.py` 的 Rust 步骤覆盖 default 与 `tui`，不覆盖 `simplified-trad-conversion`，因此 CI 绿灯不能证明简繁转换链路可用。
- **#69 / #67 / #65 / #64（Actions 跨主版本升级）CI 失败的直接原因是既有 `cli.rs` rustfmt 不合规**，而非 Actions 新版本本身；`dev` 已由 #77 修复该格式问题，这四个 PR 需更新基线后重新验证。

### 修复计划

- [x] #86：新增 E2E 设置分类切换辅助函数，更新所有跨分类访问的 spec（`selection-and-persistence`、`restart-settings`、`gui-visual-artifacts`、`simplified-trad-conversion`），并验证关闭重开设置时的分类状态行为；
- [x] #86：非规则分类移除底栏「恢复默认」，或在实现分类级恢复后使用明确命名的独立回调，禁止复用规则恢复回调（已改为「恢复默认规则」且仅在规则分类渲染）；
- [x] #80：为 `simplified-trad-conversion` feature 补充 Linux 可执行的构建/测试验证入口（新增 `--profile feature`），并登记 Windows 原生转换回归到集中验证队列（Q5）；
- [x] 复核低风险依赖 PR（#83、#81、#84、#82）并在合并后确认剩余 PR 状态：四项均已 squash 合并到 `dev`；
- [x] Tauri 相关 PR（#85、#79）汇总原生构建、打包与运行验证项：两项均已 rebase 到最新 `dev`、CI 通过并合并；原生构建与打包验证仍归入 [validation/windows.md](validation/windows.md) Q6；
- [x] #80（opencc-fmmseg 0.12.1）：确认 manifest `0.12.0` 约束与 lockfile `0.12.1` 一致（此前「版本不一致」结论不成立），以 `--features simplified-trad-conversion` 执行 clippy 与测试通过后合并；
- [x] #65、#64 更新基线后重跑 CI：两项已 rebase 到最新 `dev` 且 CI 全部通过；
- [x] #69、#67 联合验证发布链路上传/下载组合：见「P0：v0.7.0-pre1 发布准备」；
- [ ] 全部相关 GUI 与 Tauri 变更合并后，执行**一次集中 Windows 原生验证**，不按 PR 交替切换平台。

## P0：v0.7.0-pre1 发布准备（进行中）

本节记录 PR #86、#64、#65、#67、#69 的合并计划、必要安全审计与 `v0.7.0-pre1` 发布候选准备。**完成「准备」不等于授权发布**；实际 `publish=true` 与公开 Release 需在最终候选的构建与 Windows 验证通过后另行确认。

### 阶段一：安全审计

- [x] 在最终依赖基线上重新执行 `python3 scripts/verify.py --profile audit`，取新鲜结果，不沿用历史输出（`--profile audit` 通过：Rust 0 漏洞、前端 0 high/critical、E2E 残余 2 个已登记 advisory、SBOM 716 组件、许可证清单一致）
- [x] 核实前端 `undici` 等告警是否随依赖升级实际消除：确认 `npm audit --prefix frontend` 为 0 vulnerabilities，`undici@8.11.2` 实际消除，**不沿用此前「缓存旧结果」的未验证解释**
- [x] 分别核实 E2E `extract-zip` 的两个 advisory（GHSA-jmr9-qjv8-65gv、GHSA-7pqw-9j4j-h8q3）：影响范围、可用修复版本、依赖链与可利用条件：`extract-zip@2.0.1` 已是 npm latest，两个 advisory 均无 patched version
- [x] 复核 Rust `glib`（GHSA-wrw7-89jp-8q8g）与 [decisions/glib-0.18-soundness-risk.md](decisions/glib-0.18-soundness-risk.md) 的暴露范围（Linux 桌面构建路径，Windows/macOS 不涉及）：维持既有限期接受（复核期限 2026-12-31），确认仅影响 Linux GUI 资产
- [x] 无兼容修复时，在 [decisions/e2e-audit-policy.json](decisions/e2e-audit-policy.json) 与 [decisions/wdio-transitive-dependencies.md](decisions/wdio-transitive-dependencies.md) 登记**有期限、范围明确**的风险接受；不得为使审计变绿而扩大忽略范围或降级 WebdriverIO：另以同 major 版本区间 override 修复 `undici`/`brace-expansion`/`ip-address`（消除 14 个新 advisory）；`extract-zip` 的 GHSA-7pqw-9j4j-h8q3 新增限期登记，期限 2026-10-06
- [x] 审计结果区分：已修复 / 已接受的残余风险 / 仍阻塞发布的问题。

### 阶段二：合并 PR

- [x] **#86**：复核最终差异、检查状态与冲突后合并到 `dev`，不使用管理员绕过；Windows GUI 验证保持 `WINDOWS_VERIFICATION_PENDING`（已于 2026-09-30 squash 合并，三项 CI 检查 SUCCESS）；
- [x] **#65 → #64 → #69 → #67**：先在基于最新 `dev` 的临时集成分支上组合四项改动，核对官方升级说明、runner 要求、输入参数、artifact 名称/路径/权限与下载语义。

  组合分支 `integrate/actions-bumps` 合并四项时，`#65` 与 `#64` 在 `ci.yml` 的 Frontend job 产生冲突（修改同一 `checkout` 行），已按「新版本优先」取 `checkout` v7.0.1 + `setup-node` v7.0.0 解决，无冲突残留。

  逐版本核对官方 release notes 的结论：

  | Action | 升级 | 关键变更 | 对本仓库的影响 |
  | --- | --- | --- | --- |
  | `actions/checkout` | v4.4.0 → v7.0.1 | v5 持久化凭据改为独立文件；v6 要求 Node 24；v7 阻断 `pull_request_target` / `workflow_run` 的 fork checkout 并迁移 ESM | 全部使用 GitHub 托管 runner（`ubuntu-latest` / `windows-latest`），仓库无 `pull_request_target` / `workflow_run` 触发，**不受影响** |
  | `actions/setup-node` | v4.4.0 → v7.0.0 | v5–v6 转向 Node 24；v7 迁移 ESM、升级 `@actions/cache` 到 5.1.0，并**移除 dummy `NODE_AUTH_TOKEN` 导出** | 三个 job 仅用 `node-version-file` + `cache: npm`，仓库未使用 `NODE_AUTH_TOKEN` / `registry-url` / `always-auth`，**不受影响** |
  | `actions/upload-artifact` | v4.6.2 → v7.0.1 | v5–v6 转向 Node 24；v7 新增 `archive: false` 直传并迁移 ESM | 三处调用仍用 `name` + `path` + `retention-days: 7`，未使用新增的 `archive` 参数，输入语义未变 |
  | `actions/download-artifact` | v4.3.0 → v8.0.1 | v5 修正按 ID 单artifact下载的路径不一致；v6–v7 转向 Node 24；v8 默认在哈希不匹配时报错，并新增 `skip-decompress` | 四处调用**全部按 `name` 下载**（非 `artifact-ids`），v5 的路径变更不适用；下载物均为 zip 资产，v8 的跳过解压逻辑不影响；v8 的哈希失败即报错属于**更严格**的安全默认值，符合发布门禁需求 |

  YAML 解析校验通过（`ci.yml` 3 个 job、`release.yml` 6 个 job 结构完整）。

- [ ] 在集成分支执行 `publish=false` 发布演练，验证 checkout、Node 初始化、Linux/Windows 构建、Windows 自动 smoke、artifact 上传/下载/汇总与哈希校验；
- [ ] 组合演练成功后按序合并，每步确认基线与必需检查；合并后对最终基线**再次演练**，确保验证基线与交付基线一致。

### 阶段三：预发布支持

- [ ] `release.yml` 支持预发布：严格区分 `vX.Y.Z`（`--latest`）与 `vX.Y.Z-preN`（`--prerelease` 且不占用 latest），替换宽松 shell 通配符版本判断；
- [ ] 保留 `publish=false` 演练模式、分支限制与最小权限；发布时要求非空 `expected_sha` 并严格核对 HEAD；
- [ ] 补充版本/发布脚本测试，覆盖 `0.7.0-pre1` 在 `frontend/package.json`、`package-lock.json`、`src-tauri/tauri.conf.json`、`Cargo.toml`、`Cargo.lock` 与资产元数据中的一致性；
- [ ] 新增 `docs/archive/releases/v0.7.0-pre1.md` 发布说明（新 GUI、依赖更新、测试范围、已知限制、残余安全风险、简繁转换构建能力），不擅自改变正式发布资产的 feature 配置。

### 阶段四：最终候选验证

- [ ] 在独立发布工作区执行 `python3 scripts/verify.py --profile release --tag v0.7.0-pre1`、`--profile feature`、`--profile audit` 与 `npm run typecheck --prefix e2e`；
- [ ] 按项目分支流程将候选整合至 `master`，对**最终候选 SHA**执行完整演练；
- [ ] 集中 Windows 验证：`P0` 原生构建、资产校验、候选 GUI/TUI 启动 smoke、发布上传/下载链路；`P1` 中文 GUI 分类与布局、真实剪贴板、DPI/缩放、窗口控制、设置持久化、转换能力与 TUI 交互回归；`P2` 主题、字体与长文本视觉复核；
- [ ] 汇总 `PASS / FAIL / BLOCKED / NOT RUN`，给出「可发布」或「仍被哪些门禁阻塞」的明确结论。

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
- [x] 持续执行依赖审计、许可证清单更新和工具链升级 Runbook；v0.6.2 安全维护周期已结束，该维护项转为 v0.7.0 持续维护。

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
