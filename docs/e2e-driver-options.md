# Tauri 2 真实 E2E 路线选型分析（决策记录）

> 状态：方案已评审选定，待真实桌面环境执行 Spike。
> 前置阅读：[roadmap.md](roadmap.md) P0.2 节。

## 1. 候选方案

| 方案 | 机制 | 平台 | 依赖 | 维护活跃度 |
| --- | --- | --- | --- | --- |
| A. WebdriverIO + `@wdio/tauri-service`（embedded） | 测试专用插件在应用内嵌 WebDriver server，无需外部 driver | Windows / Linux / **macOS** | 两个 Tauri 插件（`tauri-plugin-wdio-webdriver`、`tauri-plugin-wdio`） | WebdriverIO 官方维护，文档完整 |
| B. WebdriverIO + `@wdio/tauri-service`（tauri-driver） | 服务自动管理 `tauri-driver` + 平台原生 driver（Windows: EdgeDriver；Linux: WebKitWebDriver） | Windows / Linux | `webkit2gtk-driver`（Linux）或 Edge（Windows）；服务自动同步 EdgeDriver 版本 | 同上 |
| C. 直接驱动 `tauri-driver`（自建 harness，无 Node） | Tauri 官方 CLI + WebDriver 协议（Selenium 或任意客户端） | 仅 Windows / Linux（macOS 无 WKWebView driver） | `tauri-driver` crate + 平台 driver | Tauri 官方，但生态示例少 |
| D. 浏览器模式（renderer-only） | Tauri 前端跑在普通 Chrome + Vite dev server，`invoke()` 被拦截可 mock | 全平台（不含原生行为） | 仅 Node + Chrome | 同 A |

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

## 3. 选定结论

**选定方案 A（WebdriverIO + @wdio/tauri-service，embedded provider）为主路线**，理由：

1. 单一工具链覆盖 Linux 与 Windows（未来含 macOS），与现有 Node 前端工具链（npm/vitest）天然一致；
2. embedded provider 不依赖平台 driver 版本同步，CI 环境最稳；
3. IPC mock 与日志开箱即用，直接服务"损坏设置恢复"等故障注入用例；
4. WebdriverIO 官方维护、文档与示例应用齐全。

**Spike 验证清单**（真实桌面环境，Windows Terminal + PowerShell 7 / Linux 桌面）：

- [ ] `e2e` feature flag 下集成两个 wdio 插件，release 构建不受影响；
- [ ] `npm create wdio` 脚手架 + `driverProvider: 'embedded'` 启动真实应用；
- [ ] 用例 1：启动 → 默认样例格式化（`在LeanCloud上，花了5000元`）；
- [ ] 用例 2：设置保存 → 重启恢复（读写 `settings.json` 真实路径）；
- [ ] 用例 3：注入损坏的 `settings.json` → 应用降级默认值并提示；
- [ ] 用例 4：不可写目录错误提示；
- [ ] 确认前端 React 19 `act` warning 是否随真实用户流消失（P0.2 顺带项）。

**门禁策略**：Spike 通过后，E2E 先作为 GitLab `master` tag pipeline 的可选 stage（不阻塞 tag 发布），稳定后再考虑纳入 `dev` 合并门禁；不使用 GitHub Actions（见决策 6）。

## 4. 参考链接

- Tauri WebDriver 指南：<https://v2.tauri.app/develop/tests/webdriver/>
- WebdriverIO Tauri Quick Start：<https://webdriver.io/docs/desktop-testing/tauri/quick-start>
- 示例应用：<https://github.com/webdriverio/desktop-mobile>
