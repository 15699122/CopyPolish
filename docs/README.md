# 文档导航

本目录按读者和文档生命周期组织。根目录 [README.md](../README.md) 是面向用户的产品说明；本目录内容面向维护者、发布者和后续开发工作。

## 推荐阅读顺序

1. [development.md](development.md)：当前架构、目录职责、开发环境、验证命令和实现约束；
2. [v0.5.0-release-plan.md](v0.5.0-release-plan.md)：当前仍在进行的 `v0.5.0` 正式发布门槛与 Windows 验收清单；
3. [roadmap.md](roadmap.md)：发布后的持续开发路线图和未完成事项；
4. [release/manual-release.md](release/manual-release.md)：本地构建并手动上传 GitHub Release 的操作 Runbook；
5. [benchmarks/unicode-baseline.md](benchmarks/unicode-baseline.md)：Unicode 边界层引入前后的编译、体积和性能基线；
6. [development/gitlab-mcp.md](development/gitlab-mcp.md)：GitLab 迁移后 Cline 接入 GitLab MCP Server 的配置、验收与安全规则。

## 文档职责

| 文档 | 唯一职责 | 更新时机 |
| --- | --- | --- |
| `../README.md` | 用户功能、使用方式、规则、限制和下载说明 | 用户可见行为变化时 |
| `development.md` | 当前实现、开发流程、测试、CI 和工程约束 | 架构或开发流程变化时 |
| `v0.5.0-release-plan.md` | `v0.5.0` 发布门槛、验收和发布状态 | 当前版本发布流程中持续更新 |
| `roadmap.md` | 未完成的中长期开发工作 | 规划或任务状态变化时 |
| `release/manual-release.md` | 本地发布的可操作步骤与故障排查 | 发布流程或脚本变化时 |
| `development/gitlab-mcp.md` | GitLab MCP Server 在 Cline 中的接入、验收与安全边界 | GitLab 迁移或 MCP 配置变化时 |
| `benchmarks/unicode-baseline.md` | 可重复的性能与体积测量记录 | 基准重新测量时 |

`v0.5.0` 正式发布完成后，版本计划文档应迁入 `archive/release-plans/`，并从路线图中移除已完成的 P0 发布闭环。历史 Python 实现见 [`reference/README.md`](../reference/README.md)，不参与当前构建、测试或行为定义。
