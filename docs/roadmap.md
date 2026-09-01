# CopyPolish 后续开发计划

本文只跟踪尚未完成的工作。当前实现见 [architecture.md](architecture.md) 和 [development.md](development.md)，测试策略见 [testing.md](testing.md)，发布操作见 [release/manual-release.md](release/manual-release.md)，历史计划与验证记录见 [archive/](archive/)。

## 规划原则

1. 正确性和结构保护优先于新增规则；
2. 先建立可观测基线和回归测试，再替换实现；
3. 默认不改写可能影响语义的 Unicode 表示；
4. 大型依赖和跨平台能力先做独立 Spike；
5. 每个里程碑同时包含实现、测试、文档和可复现验证。

## 当前基线

span-aware 混合管线、规则注册表、阶段依赖、结构/语义 span 和 UTF-8 安全 TextEdit 已落地；桌面 GUI 与 TUI 共用 Rust 引擎和 `rules.yaml`。Windows 原生验证（双 provider E2E、设置恢复/损坏/ACL、GUI DPI 人工三档、Windows Terminal TUI 交互）均已完成或按项目决策跳过，记录见 [windows-e2e-runbook.md](windows-e2e-runbook.md) 与归档。

## P0：仓库卫生与事实来源收敛

- [ ] 增加统一的安全清理入口（清理 `src-tauri/target/`、`frontend/dist/`、`src-tauri/gen/`、`scripts/__pycache__/`、`e2e/artifacts/` 等可再生目录，禁止宽泛递归删除）；
- [ ] 将「测试后清理本地 artifact、远程仅记录测试结论」写入 `docs/development.md` 与 `CONTRIBUTING.md`；
- [ ] 推送前复核未推送提交不含截图、日志、临时设置或 artifact 路径。

## P0：依赖与安全维护

- [ ] 运行当前依赖审计（`verify.py --profile audit`），分类处理：生产漏洞 / 仅测试依赖漏洞 / 无可用修复 / 升级破坏兼容；
- [ ] 优先处理 E2E 工具链（`e2e/package.json`）的高危依赖；
- [ ] 对 `serde_yaml`（上游 deprecated）迁移做独立 Spike；
- [ ] 重新生成并审阅 `docs/licenses.md`。

## P1：引擎正确性

- [ ] 建立真实语料 corpus（技术文档、产品文案、Markdown README、HTML/LaTeX 科研文本、单位/化学式/emoji、未闭合结构），记录应保持结构与已知争议点；
- [ ] 增加属性测试：幂等性、非法 UTF-8 边界、受保护 span 不变、CRLF/LF round trip、任意规则选择不 panic、legacy key 归一化稳定、GUI/TUI 输出一致；
- [ ] 增加注册表与 README 规则表一致性自动检查（数量、stable key、默认状态、展示名）；
- [ ] 继续按真实语料扩展有限单位词典；
- [ ] Placeholder 重构（决策 2）先做小型设计 Spike：比较受控 placeholder、全程 span/TextEdit、分段 rope 三方案，用现有 fixture 与 1 MB corpus 做输出兼容和性能对照；可维护性收益明确后再实施。

## P1：设置存储策略决策

- [ ] 形成产品决策文档（纯便携 / 便携优先 + 应用数据目录回退 / portable.flag 标记），确定后再改存储行为；当前继续加强只读目录提示。

## P2：E2E 收敛

- [ ] Embedded provider 保留完整回归；标准 W3C provider 缩减为兼容性 smoke（session、主窗口、一次真实格式化、一次设置保存、退出清理）；
- [ ] 处置 GUI DPI 自动矩阵脚本（`run-gui-dpi-pair.ts`、`validate-gui-dpi-matrix.ts`）：确认长期跳过后删除并记录「DPI 采用发布前人工检查」；
- [ ] 所有 runner 增加统一 finally 清理；测试结果只记录摘要，不提交 artifact。

## P2：TUI 产品定位

- [ ] 在「实验性 / Beta / 正式支持」中明确定位（建议 Beta），同步 README、发布资产说明和支持矩阵。

## P2：前端编排收敛

- [ ] 评估将 `App.tsx` 的设置组装与保存调度抽出为 controller hook；不引入全局状态库；
- [ ] 为浏览器 fallback 增加醒目的「演示模式」标识。

## P2：发布持续维护

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

每完成一项，只更新本文件的待办状态；若架构、命令或流程发生变化，再同步更新对应权威文档和 `CHANGELOG.md`。
