# 文档导航

根目录 [README.md](../README.md) 面向使用者；`docs/` 面向维护者。文档按“当前事实、操作手册、开发计划、历史归档”分层，避免把已经完成的发布进度混入长期文档。

## 推荐阅读顺序

1. [../CONTRIBUTING.md](../CONTRIBUTING.md)：开发、测试、提交和 PR；
2. [architecture.md](architecture.md)：架构、模块职责和数据流；
3. [testing.md](testing.md)：测试层次、功能地图和 fixture 约定；
4. [development.md](development.md)：工具链、启动、验证和工程约束；
5. [roadmap.md](roadmap.md)：仅包含未完成工作、优先级和建议里程碑；
6. [release/manual-release.md](release/manual-release.md)：构建、资产校验、人工验收和公开发布 Runbook；
7. [secrets-management.md](secrets-management.md)：age + SOPS 凭据管理、轮换和恢复；
8. [upgrade-runbook.md](upgrade-runbook.md)：Node、Rust、Tauri、React/Vite 升级和回滚流程；
9. [benchmarks/unicode-baseline.md](benchmarks/unicode-baseline.md)：Unicode 边界层的历史性能与体积基线；
10. [benchmarks/icu4x-spike.md](benchmarks/icu4x-spike.md)：ICU4X 技术验证及不引入结论；
11. [development/gitlab-mcp.md](development/gitlab-mcp.md)：可选的 GitLab Build Service 诊断工具说明。
12. [e2e-development.md](e2e-development.md)：真实 Tauri GUI E2E 的开发步骤、环境边界和验收标准。
13. [windows-e2e-runbook.md](windows-e2e-runbook.md)：Windows 原生 DPI、Windows Terminal 交互 artifact 和 GitLab Windows E2E stage。
14. [decisions/settings-storage-policy.md](decisions/settings-storage-policy.md)：设置存储策略决策（Accepted，方案 B 已落地）。
15. [decisions/serde-yaml-migration.md](decisions/serde-yaml-migration.md)：serde_yaml 迁移 Spike（结论：暂不迁移）。
16. [decisions/placeholder-migration.md](decisions/placeholder-migration.md)：Placeholder 重构 Spike（结论：暂不重构，保持当前混合管线）。
17. [decisions/wdio-serialize-javascript.md](decisions/wdio-serialize-javascript.md)：E2E `serialize-javascript` 修复记录（采用 npm override，保留 WebdriverIO 9）。
18. [archive/decisions/e2e-provider-selection.md](archive/decisions/e2e-provider-selection.md)：E2E provider 选型决策（已归档，仅保留选型依据）。

## 文档职责

| 文档 | 职责 | 更新时机 |
| --- | --- | --- |
| `../README.md` | 用户功能、规则、使用方式、限制和下载说明 | 用户可见行为变化时 |
| `../CONTRIBUTING.md` | 分支、提交、PR、验证和完成标准 | 开发流程变化时 |
| `architecture.md` | 当前架构、模块边界和修改入口 | 架构变化时 |
| `testing.md` | 测试策略、功能映射和测试规范 | 测试结构或门禁变化时 |
| `development.md` | 开发快速入口、工具链、命令和工程约束 | 工具链或常用命令变化时 |
| `roadmap.md` | 尚未完成的开发工作及其验收标准 | 优先级或任务状态变化时 |
| `release/manual-release.md` | 与版本无关、可重复执行的发布步骤 | 发布流程、脚本或资产变化时 |
| `secrets-management.md` | 加密凭据、接收者、轮换与灾难恢复 | 凭据结构或恢复流程变化时 |
| `upgrade-runbook.md` | Node、Rust、Tauri、React/Vite 升级、验收和回滚 | 工具链、依赖或升级门禁变化时 |
| `decisions/placeholder-migration.md` | Placeholder 重构方案比较、性能基线和重新评估条件 | 重构 Spike 重新测量或决策变化时 |
| `decisions/wdio-serialize-javascript.md` | E2E `serialize-javascript` 修复方案、验证结果和后续升级约束 | E2E 依赖升级或审计结果变化时 |
| `e2e-development.md` | 真实 Tauri GUI E2E 的实现、测试和跨平台环境边界 | E2E 工程或桌面验证流程变化时 |
| `windows-e2e-runbook.md` | 必须依赖 Windows 原生环境的 DPI、Terminal 交互和 GitLab Windows E2E 留证流程 | Windows 验证矩阵、artifact 或 Windows runner 流程变化时 |
| `benchmarks/icu4x-spike.md` | ICU4X 技术验证、成本数据和依赖决策 | Spike 重新测量或依赖决策变化时 |
| `benchmarks/` | 可重复测量的方法和结果 | 基准重新测量时 |
| `archive/` | 已完成版本的计划、验收和历史决策 | 只追加必要更正，不承载新任务 |

## 维护规则

- 当前架构事实写在 `architecture.md` / `development.md`，待办只写在 `roadmap.md`，避免双重维护；
- 发布完成后删除本地 Release Notes 草稿，正式说明以发布平台为准；
- 一次性迁移过程在完成后从现行文档移除，仍有价值的约束合并到开发说明或 Runbook；
- 不在长期文档中固定测试数量、Pipeline ID、Release ID 或“latest”状态；这些信息容易失真，应从命令输出或发布平台查询；
- 历史实现和已删除文件通过 Git 历史追溯，不在当前目录保留重复说明。
