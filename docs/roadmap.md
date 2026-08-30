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
- 真实 Tauri 桌面链路缺少稳定的自动化 E2E；
- `App.tsx` 和设置组件仍有进一步降低编排复杂度的空间；
- TUI 尚未完成跨终端 smoke 和正式资产决策。

## P0：流程与桌面验证

### P0.1 工程基线

- [x] 建立贡献、架构和测试文档；
- [x] 统一 GitHub 分支 CI、GitLab tag pipeline 和本地发布流程说明；
- [x] 增加 CHANGELOG；
- [x] 明确记录 GitHub Actions 账户计费阻塞属于外部运维风险：远程 workflow 需在账户解除阻塞后确认，本地 `verify.py` 与 GitLab tag pipeline 为替代门禁；
- [x] 在干净工作区按文档完成完整验证（清空全部可再生目录后重新 `npm ci`）：前端 13 文件 / 57 用例通过、构建通过，Rust 127 用例通过、TUI 构建与性能门禁通过，Markdown 链接/安全/版本检查通过。

### P0.2 真实 Tauri E2E

- [ ] Spike 当前 Tauri 2 可用的 WebDriver/driver 路线；
- [ ] 覆盖启动、真实引擎输出和默认示例；
- [ ] 覆盖全不选恒等、规则切换和快捷键开关；
- [ ] 覆盖设置保存、重启恢复、损坏设置和不可写目录；
- [ ] 使用临时设置目录，禁止污染真实 `rules.yaml`；
- [ ] Linux 和 Windows 各保留至少一条真实链路，稳定后再纳入合并门禁。

## P1：引擎和长文本体验

### P1.1 保护层和语义扩展

- [ ] 继续按真实语料扩展有限单位词典；
- [ ] 减少 placeholder 依赖，保持保护优先级和现有输出兼容；
- [ ] 补充复杂 Markdown、HTML、LaTeX 和化学式真实样本；
- [ ] 对未闭合结构继续坚持“宁漏格式化，不破坏结构”；
- [ ] 所有新规则遵守注册表、fixture、幂等性、迁移和文档准入流程。

### P1.2 性能和响应性

- [ ] 对 `spans.rs`、`protection.rs` 和 `edit_plan.rs` 做 profiling；
- [ ] 优化 1 MB Markdown/LaTeX 密集语料中的字符串分配和嵌套扫描；
- [ ] 评估 worker thread、可取消任务和动态 debounce；
- [ ] 保持 `scripts/check_performance.py` 的数量级回归门禁；
- [ ] 长文本优化必须同时补充性能数据和行为回归。

## P1：前端可维护性

- [x] 让 `App.tsx` 主要承担组件编排：剩余内联逻辑已抽出（设置提醒文案/判定抽到 `frontend/src/lib/settingsLoadNotices.ts`，主界面与设置 Footer 共用消除重复）；
- [x] 抽离规则目录加载与设置恢复触发，新增 `frontend/src/hooks/useRuleCatalog.ts`；
- [x] 抽离清空输入、取消排版、清理错误、空输入持久化和完成反馈定时器，新增 `frontend/src/hooks/useClearFeedback.ts`；
- [x] 按主题、显示、规则、快捷键和状态 Footer 拆分设置界面，新增 `frontend/src/components/settings/`（`ThemeSection`、`DisplaySection`、`ShortcutsSection`、`RulesSection`、`SettingsFooter`），`SettingsDialog.tsx` 退化为编排容器；
- [ ] 清理 React 19 快捷键测试中的异步 `act` warning：已定位为“打开设置”快捷键触发的 Radix Dialog 异步挂载在 React 19 + jsdom 下的环境告警，非产品缺陷；简单包装 `act` 会产生更多 warning，需在真实 Tauri E2E（P0.2）链路中验证和闭环；
- [ ] 保持 hook 不直接调用 `invoke`，继续使用 `lib/tauri.ts`。

## P2：TUI 产品化和持续维护

- [ ] 在 Windows Terminal、Linux 终端和 SSH 环境完成 smoke；
- [ ] OSC 52 不可用时提供明确降级提示；
- [ ] 评估大文本后台任务；
- [ ] 决定是否发布独立 `CopyPolish-TUI-*` 资产；
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