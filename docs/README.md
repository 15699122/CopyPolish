# 文档导航

根目录 [README.md](../README.md) 面向使用者；`docs/` 面向维护者。文档按“当前事实、操作手册、开发计划、历史归档”分层，避免把已经完成的发布进度混入长期文档。

## 推荐阅读顺序

1. [development.md](development.md)：架构、目录职责、开发环境、验证命令和工程约束；
2. [roadmap.md](roadmap.md)：仅包含未完成工作、优先级和建议里程碑；
3. [release/manual-release.md](release/manual-release.md)：构建、资产校验、人工验收和公开发布 Runbook；
4. [secrets-management.md](secrets-management.md)：age + SOPS 凭据管理、轮换和恢复；
5. [benchmarks/unicode-baseline.md](benchmarks/unicode-baseline.md)：Unicode 边界层的历史性能与体积基线；
6. [upgrade-runbook.md](upgrade-runbook.md)：Node、Rust、Tauri、React/Vite 升级和回滚流程；
7. [development/gitlab-mcp.md](development/gitlab-mcp.md)：可选的 GitLab Build Service 诊断工具说明。

## 文档职责

| 文档 | 职责 | 更新时机 |
| --- | --- | --- |
| `../README.md` | 用户功能、规则、使用方式、限制和下载说明 | 用户可见行为变化时 |
| `development.md` | 当前实现、开发流程、测试命令、CI 和工程约束 | 架构或开发流程变化时 |
| `roadmap.md` | 尚未完成的开发工作及其验收标准 | 优先级或任务状态变化时 |
| `release/manual-release.md` | 与版本无关、可重复执行的发布步骤 | 发布流程、脚本或资产变化时 |
| `secrets-management.md` | 加密凭据、接收者、轮换与灾难恢复 | 凭据结构或恢复流程变化时 |
| `upgrade-runbook.md` | Node、Rust、Tauri、React/Vite 升级、验收和回滚 | 工具链、依赖或升级门禁变化时 |
| `benchmarks/` | 可重复测量的方法和结果 | 基准重新测量时 |
| `archive/` | 已完成版本的计划、验收和历史决策 | 只追加必要更正，不承载新任务 |

## 维护规则

- 当前状态只写在 `development.md`，待办只写在 `roadmap.md`，避免双重维护；
- 发布完成后删除本地 Release Notes 草稿，正式说明以发布平台为准；
- 一次性迁移过程在完成后从现行文档移除，仍有价值的约束合并到开发说明或 Runbook；
- 不在长期文档中固定测试数量、Pipeline ID、Release ID 或“latest”状态；这些信息容易失真，应从命令输出或发布平台查询；
- 历史实现和已删除文件通过 Git 历史追溯，不在当前目录保留重复说明。