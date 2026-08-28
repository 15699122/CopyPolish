# 文档导航

本目录按读者和文档生命周期组织。根目录 [README.md](../README.md) 是面向用户的产品说明；本目录内容面向维护者、发布者和后续开发工作。

## 推荐阅读顺序

1. [development.md](development.md)：当前架构、目录职责、开发环境、验证命令和实现约束；
2. [roadmap.md](roadmap.md)：发布后的持续开发路线图和未完成事项；
3. [release/manual-release.md](release/manual-release.md)：本地构建并手动上传 GitHub Release 的操作 Runbook；
4. [release/v0.5.0-release-notes-draft.md](release/v0.5.0-release-notes-draft.md)：`v0.4.0` → `v0.5.0` 正式版 Release Notes 本地草稿；
5. [benchmarks/unicode-baseline.md](benchmarks/unicode-baseline.md)：Unicode 边界层引入前后的编译、体积和性能基线；
6. [development/gitlab-mcp.md](development/gitlab-mcp.md)：Cline 接入 GitLab Build Service MCP Server 的配置、验收与安全规则。
7. [development/gitlab-migration.md](development/gitlab-migration.md)：GitLab Build Service、Windows SaaS runner 与 GitHub Release 编排状态。
8. [secrets-management.md](secrets-management.md)：age + sops 密钥管理、令牌轮换与灾难恢复。

## 文档职责

| 文档 | 唯一职责 | 更新时机 |
| --- | --- | --- |
| `../README.md` | 用户功能、使用方式、规则、限制和下载说明 | 用户可见行为变化时 |
| `development.md` | 当前实现、开发流程、测试、CI 和工程约束 | 架构或开发流程变化时 |
| `archive/release-plans/v0.5.0-release-plan.md` | `v0.5.0` 发布门槛、验收和发布结果归档 | 正式发布完成后只追加更正 |
| `roadmap.md` | 未完成的中长期开发工作 | 规划或任务状态变化时 |
| `release/manual-release.md` | 本地发布的可操作步骤与故障排查 | 发布流程或脚本变化时 |
| `release/v0.5.0-release-notes-draft.md` | `v0.5.0` 正式版 Release Notes 草稿 | 正式发布说明变化时 |
| `development/gitlab-mcp.md` | GitLab Build Service MCP Server 在 Cline 中的接入、验收与安全边界 | GitLab MCP 或构建服务变化时 |
| `development/gitlab-migration.md` | GitLab Build Service、GitHub Release 编排、进度和技术决策 | 构建服务或 Release 编排变化时 |
| `benchmarks/unicode-baseline.md` | 可重复的性能与体积测量记录 | 基准重新测量时 |
| `secrets-management.md` | SOPS 加密凭据、age 接收者、令牌轮换与灾难恢复 | 凭据结构或恢复流程变化时 |

`v0.5.0` 发布计划已先行迁入 `archive/release-plans/` 保存历史上下文；正式 Release 仍处于 Draft，待补齐 AppImage、完成 Release Notes 复核并将其发布为 latest 后，再将路线图中的 P0 发布闭环标记完成。历史 Python 实现已从当前工作树移除；如需追溯，请通过 Git 历史查看，不参与当前构建、测试或行为定义。
