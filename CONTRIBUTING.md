# CopyPolish 贡献指南

本文档说明 CopyPolish 的日常开发、测试、提交和评审流程。项目面向用户的说明见根目录 [README.md](README.md)，架构说明见 [docs/architecture.md](docs/architecture.md)，测试策略见 [docs/testing.md](docs/testing.md)。

## 1. 开发环境

- Node.js：使用 `.nvmrc` 固定版本；
- Rust：使用 `rust-toolchain.toml` 固定版本，并包含 `rustfmt` / `clippy`；
- 前端依赖：`frontend/package-lock.json`；
- Rust 依赖：`src-tauri/Cargo.lock`；
- Linux 桌面构建需要 Tauri/WebKitGTK 系统依赖。

初始化：

```bash
nvm use
npm ci --prefix frontend
```

启动桌面开发环境：

```bash
npm run tauri --prefix frontend -- dev
```

## 2. 分支策略

| 分支 | 用途 |
| --- | --- |
| `master` | 已验证的稳定代码和正式发布基线 |
| `dev` | 集成开发分支 |
| `feature/<topic>` | 新功能 |
| `fix/<topic>` | 缺陷修复 |
| `refactor/<topic>` | 行为不变重构 |
| `docs/<topic>` | 文档维护 |
| `chore/<topic>` | 工具链和工程维护 |

日常开发从 `dev` 创建短期分支，完成后通过 Pull Request 合并回 `dev`。稳定发布前再创建 `dev` → `master` 的 Pull Request。不要在共享分支上重写历史。

## 3. 开发步骤

1. 从最新 `dev` 创建分支；
2. 先确认受影响模块和现有测试；
3. 实现最小范围变更；
4. 为新行为或缺陷补充测试；
5. 根据变更影响更新 README、开发文档、路线图和 `CHANGELOG.md`；
6. 运行对应验证 profile；
7. 检查 diff、敏感信息和 Markdown 链接；
8. 提交 Pull Request。

## 4. 验证分级

统一入口是 `scripts/verify.py`：

```bash
# 文档、密钥和 diff 检查
python3 scripts/verify.py --profile checks

# 前端测试与构建
python3 scripts/verify.py --profile frontend

# Rust、TUI 和性能门禁
python3 scripts/verify.py --profile rust

# 依赖安全审计
python3 scripts/verify.py --profile audit

# 与常规 CI 对齐
python3 scripts/verify.py --profile ci
```

建议按变更范围执行：

- 仅文档或注释：`checks`；
- 前端：`frontend` + `checks`；
- Rust 引擎、规则或 TUI：`rust` + `checks`；
- 依赖升级：`rust`、`frontend`、`audit`、许可证清单生成和 `checks`；
- 发布：在隔离发布工作区执行 `python3 scripts/verify.py --profile release --tag vX.Y.Z`。

## 5. 代码和规则约定

- 前端只能通过 `frontend/src/lib/tauri.ts` 访问 Tauri command，不在组件或 hook 中直接调用 `invoke`；
- Rust `engine` 是格式化行为的唯一事实来源；
- 新规则必须加入 `registry.rs`，使用稳定英文 key，并明确阶段、依赖、默认状态和 legacy alias 策略；
- 新规则必须有单规则、组合、保护层和幂等性测试；
- 保护层遵循“宁漏格式化，不破坏结构”；
- 不用宽泛字母正则替代有限语义词典；
- 设置读写测试使用唯一临时目录，不写入仓库根目录的 `rules.yaml`。

## 6. Commit 规范

使用 Conventional Commits 风格：

```text
feat: add ...
fix: correct ...
refactor: extract ...
perf: improve ...
test: cover ...
docs: update ...
build: ...
ci: ...
chore: ...
```

一个提交只包含一个逻辑主题。依赖升级、行为变更、纯格式化和文档整理不要无理由混在同一提交中。破坏性变化使用 `!` 或正文中的 `BREAKING CHANGE:` 标记。

## 7. Pull Request 要求

PR 描述应说明：

- 变更目的和影响模块；
- 用户可见变化；
- 风险、兼容性和迁移影响；
- 新增或更新的测试；
- 已运行的验证命令；
- 文档和 CHANGELOG 是否同步；
- 是否涉及发布资产、设置格式、规则 key、CSP 或凭据。

## 8. Definition of Done

- 实现符合现有模块边界；
- 相关测试通过且没有未解释 warning；
- 文档已更新，或明确确认无需更新；
- `CHANGELOG.md` 的 `Unreleased` 已反映重要变化；
- Markdown 链接、密钥扫描和 `git diff --check` 通过；
- 没有提交 `node_modules`、`dist`、`target`、本地设置或明文凭据；
- 涉及桌面能力时完成 Tauri smoke；
- 涉及规则时验证默认状态、稳定 key、迁移、GUI/TUI 兼容性和幂等性。

## 9. 发布和安全

正式发布只从 `master` 创建 `vX.Y.Z` tag，预发布版本可从 `dev` 创建带后缀的 tag。GitHub Actions 负责 `dev` / `master` 的常规分支 CI；GitLab 只接收合法 `v*` tag，用于 Linux/Windows 构建和资产汇总；公开 Release 由维护者人工审阅和发布。

详细步骤见 [docs/release/manual-release.md](docs/release/manual-release.md)，依赖升级见 [docs/upgrade-runbook.md](docs/upgrade-runbook.md)，凭据操作见 [docs/secrets-management.md](docs/secrets-management.md)。任何 token、age 私钥和 SOPS 明文都不得进入提交、命令参数或日志。