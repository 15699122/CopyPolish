# Tauri 2 真实 E2E 路线选型分析（决策记录，已归档）

> **归档说明**：本决策已关闭。选定方案 A（embedded）为主路线、方案 E（标准 W3C WebDriver）为并行 provider；后续执行状态统一记录在 [e2e-development.md](../../e2e-development.md) 与 [windows-e2e-runbook.md](../../windows-e2e-runbook.md)。Windows 原生验证已全部完成或按项目决策跳过；本文仅保留选型依据，不维护进度状态。
>
> 状态：方案 A 已建立并通过 Linux/WSLg 基线；方案 E 已完成 Linux/WSLg 并行 PoC，并于 2026-08-31 在 Windows WebView2 上完成最小对照 smoke。TUI 事件路由、规则排序、编辑器边界和 GUI 样式随后完成修复；修复后的双 provider 最小回归、设置重启恢复、损坏设置、NTFS ACL、Windows GUI/TUI 手动回归、连续稳定性、统一 artifact、受控失败 probe、GUI 主题/窄窗口 artifact 和 TUI 非交互 transcript 均已完成。Windows 三档 DPI 人工验证已完成（自动矩阵决定不执行）、Terminal 交互 artifact 已由用户确认通过、React 19 告警已由设置控制台 runner 复核为 0、GitLab Windows 可选 E2E 已决定跳过（不执行）。
> 前置阅读：[roadmap.md](../../roadmap.md)。

Windows 原生验证的详细矩阵、artifact 规范和 GitLab runner 接入要求见 [windows-e2e-runbook.md](../../windows-e2e-runbook.md)。provider 选型本文只记录 A/E 的技术差异；DPI、Terminal 和 CI 结果不得在两个文档中分别维护。

## 1. 候选方案

| 方案 | 机制 | 平台 | 依赖 | 维护活跃度 |
| --- | --- | --- | --- | --- |
| A. WebdriverIO + `@wdio/tauri-service`（embedded） | 测试专用插件在应用内嵌 WebDriver server，无需外部 driver | Windows / Linux / **macOS** | 两个 Tauri 插件（`tauri-plugin-wdio-webdriver`、`tauri-plugin-wdio`） | WebdriverIO 官方维护，文档完整 |
| B. WebdriverIO + `@wdio/tauri-service`（tauri-driver） | 服务自动管理 `tauri-driver` + 平台原生 driver（Windows: EdgeDriver；Linux: WebKitWebDriver） | Windows / Linux | `webkit2gtk-driver`（Linux）或 Edge（Windows）；服务自动同步 EdgeDriver 版本 | 同上 |
| C. 直接驱动 `tauri-driver`（自建 harness，无 Node） | Tauri 官方 CLI + WebDriver 协议（Selenium 或任意客户端） | 仅 Windows / Linux（macOS 无 WKWebView driver） | `tauri-driver` crate + 平台 driver | Tauri 官方，但生态示例少 |
| D. 浏览器模式（renderer-only） | Tauri 前端跑在普通 Chrome + Vite dev server，`invoke()` 被拦截可 mock | 全平台（不含原生行为） | 仅 Node + Chrome | 同 A |
| E. `tauri-plugin-webdriver` + 标准 WebdriverIO client | 应用内直接启动 W3C WebDriver HTTP server，WDIO 通过 `127.0.0.1:<port>` 连接 | Linux / Windows / macOS（上游另有移动端实现） | 单一 Rust 插件；不需要 `@wdio/tauri-service` 或 `@wdio/tauri-plugin` | 上游 release `0.2.1`；main 分支持续维护，需固定版本并自行验证 |

## 2. 评估要点

### 与本项目约束的匹配
- **测试专用 binary**：A/B 均通过 `appBinaryPath` 指向 `cargo build` 产物；无需改动现有 `tauri.conf.json`。C 需要按官方指引创建测试专用 binary 与 capabilities。
- **插件侵入性**：A 的两个插件需加入 `Cargo.toml` 并注册到 Builder。可放在 feature flag（如 `e2e`）后面，发布构建不受影响；B/C 不需要插件。
- **IPC mock**：A/B/D 支持 `browser.tauri.execute()` 与命令 mock，便于覆盖"设置保存/重启恢复/损坏设置"链路中的后端错误注入；C 用 Selenium 需自行实现等价能力。
- **日志捕获**：A/B 提供前后端日志捕获，利于 CI 失败诊断；C 需自建。
- **macOS**：仅 A（embedded）支持。本项目当前无 macOS 资产（GitLab pipeline 只出 Linux/Windows），非决定因素，但 A 保留了未来扩展性。

### 风险
- A 的 `tauri-plugin-wdio` 相对较新，Tauri 2 小版本升级可能需要同步跟进；
- B 依赖 Linux `webkit2gtk-driver` 与 CI 镜像的 WebKitGTK 版本严格一致（本项目 GitLab 镜像为 `rust:1.98-bookworm`，需加装 `webkit2gtk-driver`）；
- C 生态最少，失败排查成本最高；
- D 无法覆盖 Tauri 原生窗口行为（无边框拖动、最小尺寸等），不能替代真实链路。
- E 的上游 release 与 main 分支版本能力可能不同；插件暴露 HTTP 自动化接口，必须保持测试 feature 隔离；标准 WebDriver 不自动提供当前方案的前后端日志聚合和 `browser.tauri.execute()` 能力。

## 3. 选定结论

**选定方案 A（WebdriverIO + @wdio/tauri-service，embedded provider）为主路线**，理由：

1. 单一工具链覆盖 Linux 与 Windows（未来含 macOS），与现有 Node 前端工具链（npm/vitest）天然一致；
2. embedded provider 不依赖平台 driver 版本同步，CI 环境最稳；
3. IPC mock 与日志开箱即用，直接服务"损坏设置恢复"等故障注入用例；
4. WebdriverIO 官方维护、文档与示例应用齐全。

**方案 A Spike 验证清单**（真实桌面环境，Windows Terminal + PowerShell 7 / Linux 桌面）：

- [x] `e2e` feature flag 下集成两个 wdio 插件，release 构建不受影响；
- [x] `npm create wdio` 脚手架 + `driverProvider: 'embedded'` 启动真实应用；
- [x] 用例 1：启动 → 默认样例格式化（`在LeanCloud上，花了5000元`）；
- [x] 用例 2：设置保存 → 重启恢复（读写 `rules.yaml` 真实路径，Windows 人工回归已完成）；
- [x] 用例 3：注入损坏的 `rules.yaml` → 应用降级默认值并提示（Windows 人工回归已完成）；
- [x] 用例 4：不可写目录错误提示（Windows NTFS ACL 人工回归已完成）；
- [ ] 确认前端 React 19 `act` warning 是否随真实用户流消失（当前仍保留为 jsdom 环境告警跟踪项）。

Windows 原生验证状态（按最新结论）：三档 DPI 人工验证已完成（自动矩阵决定不执行）；Windows Terminal 交互 artifact 已由用户确认通过、WT-TUI-001/002/003 已关闭；可选 GitLab Windows E2E stage 已决定跳过（不执行）。

### 3.1 参考插件方案 E 的并行 PoC 结果

截至 2026-08-31，当前仓库已在不删除方案 A 的前提下增加 `e2e-webdriver` feature 和独立 WDIO 配置，固定使用 `tauri-plugin-webdriver = 0.2.1`。该方案：

- 可与当前 Tauri `2.11.5`、Rust `1.98` 和 Linux WebKitGTK 环境编译；
- 通过随机 localhost 端口启动应用内 W3C WebDriver server；
- 通过真实 WebView 和 Rust IPC 完成默认示例格式化；
- 通过全不选恒等、临时设置目录和真实 `rules.yaml` 保存 smoke；
- 不加载 `@wdio/tauri-plugin`、不使用 E2E capability 和 `withGlobalTauri`；
- Windows WebView2 最小启动、session、真实 IPC、设置保存和清理，以及 GUI 重启恢复、损坏设置、ACL 保存失败、规则/快捷键、窗口/DPI 和 Terminal TUI 的修复后人工回归、双 provider 各 5 次稳定性统计均已完成；设置恢复、损坏设置、ACL 故障注入、受控失败诊断、GUI 主题/窄窗口 artifact 和 TUI 非交互 transcript 也已自动化通过。TUI-EDIT-DELETE-001 已通过 Rust 回归测试和 Windows 定向复验关闭；Windows Terminal 交互 artifact 已由用户确认通过、WT-TUI-001/002/003 已关闭；Windows DPI 三档人工验证已完成（自动矩阵决定不执行）、CI stage 已决定跳过。

对应入口：

```bash
npm run build:app:webdriver --prefix e2e
npm run test:webdriver --prefix e2e
```

- 当前结论：方案 E 与方案 A 均已达到本次修复后的 Windows WebView2 smoke（各 3 个普通用例、重启 write/read、3 个损坏 fixture 和 1 个 ACL 用例通过），并在当前环境完成受控失败诊断、GUI 主题/窄窗口 artifact 和 TUI 非交互 transcript；修复后 GUI/TUI 人工回归和双 provider 稳定性验证也已完成。方案 A 仍为主路线。TUI-EDIT-DELETE-001 已关闭；Windows Terminal 交互 artifact 已由用户确认通过、WT-TUI-001/002/003 已关闭；Windows DPI 三档人工验证已完成（自动矩阵决定不执行）；GitLab Windows 可选 E2E stage 已决定跳过。React 19 告警闭环已由设置控制台 runner 复核为 0，硬件级快捷键保留 EdgeDriver 兼容性说明。

#### Windows 原生对照记录模板

Windows 验证必须对方案 A 和方案 E 使用同一 commit、同一 Node/Rust/WebView2 环境和等价临时设置 fixture。记录以下结果：

| 项目 | 方案 A：embedded | 方案 E：标准 W3C WebDriver | 备注 |
| --- | --- | --- | --- |
| binary 构建 | 通过 | 通过 | Node 24.19.0、MSVC、Rust MSVC |
| WebView2 启动 | 通过 | 通过 | WebView2 Runtime 151.0.4129.107 |
| session 创建 | 通过 | 通过 | embedded session；W3C 随机端口 55755/53010 |
| 主窗口发现 | 通过 | 通过 | 真实打包前端窗口 |
| 默认格式化与 Rust IPC | 通过 | 通过 | `在LeanCloud上，花了5000元` |
| 设置保存 | 通过 | 通过 | 临时 `rules.yaml` |
| 重启恢复 | 自动通过 | 自动通过 | write/read 阶段各 1/1 |
| 损坏设置恢复 | 自动通过 | 自动通过 | 三种 fixture 各 3/3 |
| ACL 不可写目录 | 自动通过 | 自动通过 | 各 1/1；拒写/恢复/删除及 GUI 保存失败提示已确认 |
| 失败诊断 | 自动 probe 通过 | 自动 probe 通过 | manifest/result、日志、截图、page source 和 fixture 已验证 |
| 进程和端口清理 | 通过 | 通过 | 无残留进程/监听端口 |

方案 E 已达到方案 A 的本次修复后 Windows WebView2 smoke 和设置/ACL 故障注入覆盖，方案 A 仍为主路线。只有当后续 artifact 收集证明标准协议 provider 在诊断能力上没有明显劣势时，才重新评估主路线。

TUI-EDIT-DELETE-001 已关闭：`TextEditor` 的最后一个 grapheme 边界已修复，并由 Rust 编辑器/TUI 事件回归和 Windows Terminal 定向复验覆盖。

**门禁策略**：Spike 通过后，E2E 先作为 GitLab `master` tag pipeline 的可选 stage（不阻塞 tag 发布），稳定后再考虑纳入 `dev` 合并门禁；不使用 GitHub Actions（见决策 6）。

## 4. 参考链接

- Tauri WebDriver 指南：<https://v2.tauri.app/develop/tests/webdriver/>
- WebdriverIO Tauri Quick Start：<https://webdriver.io/docs/desktop-testing/tauri/quick-start>
- 示例应用：<https://github.com/webdriverio/desktop-mobile>
