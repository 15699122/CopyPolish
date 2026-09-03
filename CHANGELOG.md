# Changelog

本文件记录 CopyPolish 的重要用户可见变化、兼容性变化和工程维护变化。详细历史仍以 Git 提交和 GitHub Release 为准。

## [Unreleased]

### Added

- 增加默认关闭的 `spacing.numeric-punctuation` 规则：可选修复小数点、时间/比例冒号、数字分组逗号和数字斜线两侧的异常 ASCII 空格；保留版本号/IP 等连续点号数字链，并通过结构保护跳过 URL、代码和公式。
- 增加默认关闭的 `cleanup.kangxi-radicals` 规则：依据 Unicode 17.0.0 官方 `UnicodeData.txt` 的 214 项兼容分解映射修复康熙部首；不执行全文 NFKC，并通过结构保护跳过 URL、代码和公式。
- 增加 GUI 静态帮助入口和首次使用提示；帮助说明高风险清洗规则、结构保护、输出/复制动作及浏览器演示模式边界，首次提示可关闭或直接打开帮助，查看状态仅保存在前端 `localStorage`。

### Maintenance

- 记录 PDF/CAJ 段内软换行与 CJK 内部异常空格 Spike：当前仓库没有真实 PDF/CAJ 原文件或脱敏语料，因此只建立合成失败基线和实现前置条件，不引入自动清洗规则，也不将路线图项标记为完成。详情见 `docs/decisions/pdf-soft-wrap-spike.md`。
- 2026-09-03 复核 E2E 依赖：WebdriverIO 9.31.5、`@wdio/tauri-service` 1.3.0 已是当前直接依赖版本；审计仍报告 13 项 high，根因是 `@puppeteer/browsers@2.13.2` 引入的 `extract-zip@2.0.1`，上游暂无修复版本。未接受跨 major override 或 `npm audit fix --force` 的 WebdriverIO 8 降级，详情见 `docs/decisions/wdio-transitive-dependencies.md`。
- 增加 GUI“复制并清空”显式动作；复制结果仍保留输入/输出，复制并清空仅在剪贴板写入成功后清除内容，复制失败不会误清空，也不引入窗口失焦自动行为。
- 增加 GUI 实时/手动输出模式、自动/左右/上下布局和输入输出 Unicode 字符统计；新增设置字段 `output_mode` 与 `layout_mode`，旧设置缺失时回退为实时输出和自动布局，手动模式保留“立即排版”快捷键。
- 增加首批来源文本清洗规则：可选清理普通文本中的方括号/中文方括号引用角标、连续 ASCII 空格和连续空行；清洗规则默认关闭，并通过 span-aware TextEdit 保护 Markdown 链接、代码、URL 和其他结构。跨行空行清理使用独立的结构边界路径，不参与普通逐行规则循环。
- 为规则注册表增加用户说明、类型和风险元数据，并在 GUI/TUI 规则面板展示；保留 stable key、默认状态、phase、依赖和既有格式化行为不变，README 表格与注册表一致性测试已同步扩展。
- 完成简繁转换 Spike 并接入 `opencc-fmmseg`：以 `simplified-trad-conversion` 可选 feature 提供互斥的 T2S/S2T（MIT、OpenCC 风格词典 + FMM 分词，只改写可编辑区间、保护链接/代码/公式；实测 1 MB `s2t` ≈130 MB/s），默认构建不启用、保持占位；`scripts/generate_licenses.py` 新增 `--features` 参数以纳入可选依赖许可证。决策与语义边界见 `docs/decisions/simplified-trad-conversion-spike.md`。
- 增加自定义字面量替换：扩展 `FormatRequest` 为统一请求模型（有序、仅 active、span 保护前执行），随 `Preset` 模板一并落地；GUI 与 TUI 均支持添加、编辑、启停和删除替换项，并将列表顺序持久化到设置。
- 补齐 GUI 与 TUI 的替换/简繁转换交互回归：覆盖设置恢复、旧字段默认值、持久化、实时重排、TUI 请求面板和快捷键立即排版设置透传。
- 增加真实 GUI E2E 的替换/转换保存与重启恢复用例：默认构建验证 capability=false、设置保存/恢复、替换行为及不可用简繁选择归一化；`simplified-trad-conversion` feature 单独验证 capability=true 与真实双向转换。Windows 默认 embedded、feature embedded 和 W3C smoke 均已取得当前收尾证据；TUI/CLI 共享设置行为已接入。
- 新增 `build:app:simplified-trad` 与 feature 专用 GUI E2E spec，并在当前 Linux/WSL embedded provider 验证双向真实转换 2/2 通过（`s2t` / `t2s`）；默认 E2E 构建语义保持不变。
- 收敛 Windows 原生验证文档：明确默认 embedded 完整回归、简繁 feature embedded 双向验证、W3C 兼容性 smoke、NTFS ACL 和按需 GUI artifact 的执行顺序；移除已不存在的专项 `:webdriver` 命令引用，并标明 DPI 自动矩阵、GitLab stage 和已完成的 Terminal/TUI 项目状态。
- 修复设置保存状态的旧 Promise 覆盖问题，并增强 E2E 的真实输入事件、格式化请求/结果和失败诊断；Linux/WSL 默认 embedded 设置链路恢复为 3/3，简繁 feature GUI 双向转换为 2/2。Windows 需使用当前修复 binary 重新留证，Unix-only 权限测试已增加平台条件。
- 修复立即设置保存未取消旧输入防抖保存的问题，避免旧的简繁转换快照在 `t2s` 设置之后再次写入；新增保存时序回归和 feature E2E 的实际保存序号/转换值校验。Linux/WSL 默认 embedded 3/3、简繁 feature 2/2 通过；Windows 仍需使用当前 binary 重新留证。
- 增加构建 capability 查询：默认构建明确声明不包含简繁转换能力，GUI 禁用 T2S/S2T 并将不可用选择归一化为 `none`；`simplified-trad-conversion` feature 构建继续启用真实 OpenCC 转换。默认与 feature GUI 回归均覆盖 capability 边界。
- 使用提交 `6687c13` 在隔离 Windows 原生 checkout 刷新 capability 证据：默认 embedded 3/3、简繁 feature embedded 2/2、Windows MSVC TUI 167/167、W3C smoke 2/2（随机端口 51737）均通过；所有 runner 均有明确完成数，隔离 checkout 和生成物已清理。
- TUI 新增替换与字符转换请求设置面板（`Ctrl+E`）：支持有序替换项新增/编辑/启停/删除、`from`/`to` 字段切换和 `none`/`t2s`/`s2t` 模式循环；与 GUI 共用 `FormatRequest` 和 `rules.yaml`，默认构建将不可用简繁模式归一化为 `none`，feature 构建保留真实转换。
- 新增三个内置工作流预设：中文文案、PDF 清洗、技术文档。预设通过 `get_presets` 同时提供给 GUI 和 TUI，只展开为统一 `FormatRequest` 的规则选择、替换与转换字段；PDF 预设不解析 PDF 文件本体。
- 新增文本清洗与规范排版工作流决策（`docs/decisions/text-cleaning-workflow.md`），并将产品描述调整为本地优先的中文文本清洗与规范排版工具；更复杂的来源文本清洗、全角 ASCII 转半角和其他转换能力仍属于后续路线图，不代表本版本已经实现。
- 新增设置存储回退（ADR 方案 B，`docs/decisions/settings-storage-policy.md`）：程序目录不可写时自动改用平台应用数据目录（Windows `%APPDATA%\CopyPolish`、Linux/macOS `~/.config/CopyPolish`）保存 `rules.yaml`，主界面提示实际生效位置；便携用户行为不变。新增 6 项存储决策单测（同目录优先、双位置并存、只读回退等）。
- 新增统一本地清理入口 `scripts/clean.py`（白名单删除构建缓存、`e2e/artifacts/` 和 `e2e/settings-*` 临时设置目录，支持 `--dry-run`/`--deep`），并约定测试结果记录后清理本地 artifact、远程仅记录测试结论。
- 新增注册表与 README 规则表一致性自动检查（`src-tauri/tests/readme_registry.rs`），防止 stable key、展示名、分类和默认状态在两份数据间漂移。
- 新增引擎属性回归测试（`src-tauri/tests/properties.rs`，确定性伪随机语料，不引入新依赖）：幂等性、任意规则选择健壮性、emoji grapheme 边界、CRLF/CR 换行还原、保护结构不被改写和 legacy key 归一化幂等。
- 新增 W3C provider 兼容性 smoke（`specs/w3c/smoke.spec.ts`）：session 创建、主窗口发现、1 次真实格式化、1 次设置保存、正常退出与清理。
- 新增贡献指南、架构说明和测试指南。
- 新增项目变更记录。
- 新增独立 WebdriverIO + Tauri embedded provider E2E 工程，覆盖真实启动、Rust IPC 默认排版、全不选恒等和临时设置文件隔离。
- 新增基于 `tauri-plugin-webdriver` 0.2.1 的并行标准 WebDriver E2E provider，复用现有 smoke 并保持原 embedded provider 可回退。2026-09-01 已收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`），不再与 embedded provider 并行跑完整回归。

### Changed

- TUI 从「实验性」正式定位为 **Beta**：CLI 参数、规则选择和 `rules.yaml` 设置格式保持兼容；终端显示能力（emoji、OSC 52）依赖终端支持，SSH 不在验证范围。
- 重组开发文档、文档导航和后续路线图，明确当前事实、操作手册、计划和历史归档的职责边界。
- 统一 GitHub 分支 CI、GitLab tag 构建和本地验证流程的说明。
- E2E 构建增加 `custom-protocol`、条件 capability 和测试专用 `withGlobalTauri` 配置，生产构建不加载 WebDriver plugin。
- E2E 前端资源使用相对路径，并按 spec 启动独立 WDIO 进程，避免测试间共享 `rules.yaml` 状态。
- 标准 WebDriver provider 使用随机 localhost 端口、独立应用进程和运行 artifact；其前端不加载 `@wdio/tauri-plugin`。2026-09-01 已收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`），不再与 embedded provider 并行跑完整回归。
- 新增三种损坏 `rules.yaml` / `rules.yaml.bak` fixture 的双 provider 自动化入口，覆盖备份恢复、无备份降级和主备份同时损坏，并验证真实 Rust IPC 仍可用。
- 新增双 provider 设置重启恢复自动化入口，验证同一临时设置目录中的规则选择、最近输入和真实 Rust IPC 输出在第二次启动后恢复。
- 新增 Windows-only NTFS ACL 设置保存失败自动化入口，使用 `icacls.exe` 注入当前用户 deny ACE，并在 `finally` 中恢复权限和清理 fixture；非 Windows 环境显式跳过。
- 统一 embedded 与标准 W3C provider 的 E2E artifact 目录、manifest/result、失败截图、page source 和设置 fixture 收集；manifest 仅记录白名单环境摘要，不写入完整环境变量。2026-09-01 标准 W3C provider 已收敛为兼容性 smoke，不再与 embedded provider 并行跑完整回归。
- 新增 embedded/W3C 受控失败 artifact probe：先验证真实 Rust IPC，再按预期失败，并自动校验失败退出码、截图、page source、日志、manifest、result 和设置 fixture。
- 新增 embedded/W3C GUI 视觉 artifact 入口，采集正常/窄窗口及浅色/深色设置窗口的 screenshot、page source 和状态 metadata；Windows 三档 DPI 仍需原生环境执行。
- 新增 TUI 非交互 transcript artifact 入口，覆盖默认格式化、`--rules none` 恒等、未知规则 warning 和缺失输入文件错误，并保存输入、stdout、stderr、退出码及环境摘要；不替代 Windows Terminal raw-mode/OSC 52 交互。
- 新增 Windows 原生 E2E 留证 Runbook，单独整理 GUI 100%/125%/150% DPI artifact、Windows Terminal 交互 artifact 和 GitLab Windows 可选 E2E stage 的前置条件、执行步骤、失败诊断、清理要求与完成门槛；文档不将尚未接入的 Windows CI stage 误记为已完成。
- 记录并完成 Windows 原生验证：Node 24.19.0、npm 11.17.0、Rust 1.98.0 MSVC、WebView2 Runtime 151.0.4129.107、Windows Terminal 1.24.11911.0、PowerShell 7.6.5；前端 57/57、设置 Rust 测试 16/16、Rust/TUI 148/148 通过，TUI MSVC release/stdin smoke 通过。
- embedded 与标准 W3C WebDriver provider 的真实 WebView2/Rust IPC 普通用例各 3/3、设置重启 write/read 各 1/1、三种损坏设置 fixture 各 3/3、NTFS ACL 各 1/1 通过；统一 artifact、受控失败 probe、GUI 主题/窄窗口 artifact 和 TUI 非交互 transcript 也已验证。Windows GUI/TUI 手动回归和双 provider 稳定性验证均已完成。后续补项已闭环或按项目决策跳过：Windows 三档 DPI 人工验证已完成（自动矩阵决定不执行）、Windows Terminal 交互 artifact 已由用户确认通过、React 19 `act` warning 已由设置控制台 runner 复核为 0 并保留硬件键兼容性说明、GitLab Windows 可选 E2E stage 已决定跳过。2026-09-01 标准 W3C provider 已收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`），不再与 embedded provider 并行跑完整回归。
- 修复 TUI 规则面板展示顺序、输入区可打印字符误触全局快捷键、bracketed paste 以及最后一个 grapheme 的 Delete/Right 边界；修复后的 Windows Terminal raw-mode、规则排序、`Get-Clipboard` 完整粘贴、新快捷键语义和编辑器边界已完成手动回归。
- 修复 Windows Terminal 自动换行后的多行显示缺陷（WT-TUI-001 额外行绘制/状态栏重叠、WT-TUI-002 光标不可见）：新增与 ratatui 渲染等价的视觉换行布局（`src-tauri/src/tui/wrap.rs`），光标与滚动改按视觉行而非逻辑行计算，并新增 10 项 Rust/UI 回归（含 CJK、emoji grapheme 与 ratatui 渲染等价校验）。Windows MSVC 完整 TUI 测试已增至 158/158 通过；WT-TUI-003（emoji 显示）一并纳入修复范围，WT-TUI-001/002/003 均已在真实 Windows Terminal 中由用户确认复验通过并关闭。
- 2026-09-01 Windows 复验：前端 57/57、TUI 158/158、TUI transcript 4/4、embedded provider 普通 E2E 3/3、重启设置 write/read、三种损坏设置、NTFS ACL、GUI artifact 和受控失败 probe 全部达到预期。100%/125%/150% DPI 人工 GUI 验证已完成；GitLab Windows 可选 E2E stage 已决定跳过（不执行）。标准 W3C provider 已收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`）。
- 统一桌面 GUI 输出框边框阴影、设置标题间距、恢复按钮、长路径中间省略和主题/快捷键复选框样式；Linux/WSLg 前端测试与构建及 Windows WebView2 下的 DPI、窄窗口和视觉回归均已完成。
- 抽离前端规则目录加载（`useRuleCatalog`）和清空输入反馈（`useClearFeedback`），降低 `App.tsx` 编排复杂度。
- 拆分设置界面为独立分区组件（主题、显示、快捷键、规则、状态 Footer），`SettingsDialog.tsx` 负责编排。
- 抽取设置提醒文案与判定到 `frontend/src/lib/settingsLoadNotices.ts`，主界面与设置 Footer 共用，消除重复并精简 `App.tsx`。
- 将前端业务 hook 编排集中到 `frontend/src/hooks/useAppController.ts`，使 `App.tsx` 仅负责页面渲染和组件组合；不引入全局状态库。
- 浏览器预览增加醒目的「演示模式」标识，明确其最小化 fallback 不代表桌面版 Rust 引擎的完整行为。
- 新增 `useAppController` 独立编排契约测试，覆盖设置恢复、清空后的空输入保存、输入格式化/保存调度连接和浏览器演示模式。
- 修复 E2E 测试链的 `serialize-javascript` 和 `deepmerge-ts` 高危告警：分别通过 npm `overrides` 固定到 `7.1.1` 和 `8.0.2`，保留 WebdriverIO 9/Mocha 10；干净安装、E2E 类型检查、运行时导入和依赖审计均通过。审计告警由 16 项降为 13 项 high，剩余告警属于其他 WebdriverIO/浏览器工具传递依赖，记录见 `docs/decisions/wdio-transitive-dependencies.md`。
- 统一 E2E runner 的临时设置目录清理：通用 embedded、W3C smoke 和损坏设置入口在子进程异常或失败时也会执行 `finally`，调试保留设置的显式例外保持不变。
- 重新生成第三方许可证清单：改为以 `frontend/package-lock.json` 的完整 `packages` 条目为准，并写入生成日期；当前 Rust 431 条、npm 294 条，许可证字段缺失 0 条。
- 新增真实文档语料回归集 `src-tauri/tests/fixtures/real-world-corpus.yaml`，覆盖产品文案、技术文档、README、HTML/LaTeX、单位/化学式、emoji、未闭合结构和 CRLF。
- 扩展有限单位词典以覆盖真实技术语料中的二进制容量（`KiB/MiB/GiB/TiB`）和比特速率（`bps/kbps/Mbps/Gbps/Tbps`），并保留 `bit/bytes` 普通单词反例。
- 完成 Placeholder 重构设计 Spike：基于 1 MB 分阶段性能基线和真实语料兼容性评估，暂不进行大规模重构，继续采用“TextEdit + 受控 placeholder”的混合管线；决策与重新评估条件见 `docs/decisions/placeholder-migration.md`。
- 优化 Markdown 行内代码扫描，降低大量反引号文本的重复查找开销，并保持多长度 delimiter 与未闭合 delimiter 行为。
- 优化结构 span 仲裁，减少复杂 Markdown/LaTeX 文本中的 O(n²) 重叠检查开销，并保持优先级与嵌套结构语义。
- 缩小可编辑规则阶段的保护预扫描范围，仅扫描不透明结构与化学式，减少重复语义扫描开销并保持保护边界。
- 优化专有名词规则的批量替换，减少重复全文遍历，并保持相邻词、前缀词和 ASCII 单词边界语义。
- 优化中英文/中文数字空格规则，改用流式 Unicode 判定单位遍历，减少中间数组分配并保持 Grapheme 边界语义。
- 优化 HTML block 结束标签查找，减少大小写不敏感扫描中的无效窗口比较，并保持 UTF-8 内容与跨行保护语义。
- 合并 LaTeX `\(...\)` / `\[...\]` 定界符扫描，减少重复全文遍历并保持混合定界符和未闭合保护行为。
- 合并普通 Markdown 链接与引用式链接的候选扫描，减少重复全文遍历并保持图片链接、嵌套目标和引用式链接边界语义。
- 优化美元数学候选扫描，避免对连续反斜杠进行重复回扫，并保持奇偶转义、单/双美元和跨行边界语义。
- 合并 URL 与邮箱候选扫描，减少一次全文遍历，并保持 URL 优先、邮箱边界和大小写语义。
- 优化表格分隔线扫描，避免为每个候选行创建临时单元格数组，并保持对齐标记与合法性边界。


- 新增 GUI DPI 自动 artifact 采集与矩阵校验、设置快捷键控制台检查和 Windows Terminal TUI artifact 准备器。GUI DPI 自动矩阵随后按项目决定跳过（不执行，三档人工验证保留）；设置控制台 spec 1/1 且无 React `act` warning（EdgeDriver 逗号键码使用 UI 回退，硬件级快捷键保留兼容性说明）；Windows Terminal TUI 完整交互 artifact 已由用户在真实 Windows Terminal 中确认通过（raw-mode、跨行编辑、emoji、规则面板、bracketed paste、OSC 52、保存/重启和终端清理），WT-TUI-001/002/003 已关闭；早期普通命令会话运行完整 Terminal artifact 因缺少 `WT_SESSION` 被安全阻止，属历史记录。2026-09-01 标准 W3C provider 已收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`）。

### Removed

- 移除 GUI DPI 自动矩阵脚本与命令（`run-gui-dpi-pair.ts`、`validate-gui-dpi-matrix.ts`、`test:gui-dpi*`）：项目维持「DPI 采用发布前人工检查」决策；GUI 视觉 artifact 入口保留。
- 移除已失效的 VS Code `reference` Python 分析路径配置。
- 移除 `package.json` 中各 `:webdriver` 专项脚本（`test:corrupt-settings:webdriver`、`test:restart-settings:webdriver`、`test:acl-settings:webdriver`、`test:artifact-probe:webdriver`、`test:gui-visual-artifacts:webdriver`、`test:settings-shortcut-console:webdriver`）：标准 W3C provider 已收敛为兼容性 smoke，不再与 embedded provider 并行跑完整回归。

## [0.5.0] - 2026-08-28

### Added

- 完成 CopyPolish 0.5.0 正式发布基线。
- 提供 Tauri 2 + React 桌面界面、Rust 排版引擎和实验性 Ratatui TUI。
- 支持规则开关、主题、字体、字号、界面缩放、快捷键和用户设置持久化。
- 支持 Markdown、LaTeX、URL、邮箱、化学式和 Unicode grapheme 边界保护。

### Changed

- 排版规则由 Rust 注册表统一管理，并使用稳定机器 key。
- 格式化生产流程收敛到 span-aware TextEdit 管线。
- 发布流程采用本地或 GitLab 构建、人工校验和 GitHub Release 发布。

### Security

- 使用 SOPS/age 管理发布相关凭据，并加入明文凭据扫描和发布前安全检查。
