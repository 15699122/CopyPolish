# 文案净排（CopyPolish）：开发入口

本文是维护者的快速入口，记录当前工具链、常用命令、工程约束和流程索引。详细架构见 [architecture.md](architecture.md)，测试策略见 [testing.md](testing.md)，贡献规范见 [../CONTRIBUTING.md](../CONTRIBUTING.md)。

## 当前基线

- 技术栈：Tauri 2、React 19、TypeScript、Vite、Tailwind v4、shadcn/ui、Rust 2021；
- 前端目录：`frontend/`；
- Rust/Tauri 目录：`src-tauri/`；
- Rust 排版引擎：`src-tauri/src/engine/`；
- TUI：启用 Cargo feature `tui` 的 `copypolish-tui`；
- 当前行为事实来源：Rust engine；浏览器 fallback 仅用于 UI 预览；
- 当前产品边界：已交付规范排版、首批来源文本清洗、自定义字面量替换、中文文案/PDF 清洗/技术文档内置预设、GUI 设置接线以及 TUI 替换/字符转换/预设面板；简繁转换随 `simplified-trad-conversion` 可选 feature 提供，默认构建保持占位；全角 ASCII 转半角和更复杂的清洗/转换能力仍按 [roadmap.md](roadmap.md) 规划，未实现能力不得写成当前行为；
- 当前后续任务：见 [roadmap.md](roadmap.md)。

## 工具链和初始化

Node 版本由 `.nvmrc` 固定，Rust 版本由 `rust-toolchain.toml` 固定：

```bash
nvm use
npm ci --prefix frontend
```

前端类型检查和构建：

```bash
npx tsc -p frontend/tsconfig.app.json --noEmit
npm run build --prefix frontend
```

## 启动和构建

```bash
# Tauri 开发模式
npm run tauri --prefix frontend -- dev

# Linux bundle
npm run tauri --prefix frontend -- build

# 不生成安装器的便携构建
npm run tauri --prefix frontend -- build --no-bundle
```

构建缓存和前端产物是可再生目录，不提交：`frontend/node_modules/`、`frontend/dist/`、`src-tauri/target/`、`src-tauri/gen/`。

## 清理本地生成目录

测试运行产物和构建缓存只在本地留存；远程仓库仅记录测试结论。使用统一的安全清理入口（白名单删除，不做宽泛通配）：

```bash
python3 scripts/clean.py --dry-run     # 预览
python3 scripts/clean.py --generated   # 构建缓存 + e2e artifact/临时设置目录
python3 scripts/clean.py --deep        # 额外删除 node_modules（下次验证需重新 npm ci）
```

约定：每轮 Windows/Linux 测试在把结果摘要写入文档或 `CHANGELOG.md` 后，执行 `--generated` 清理本地 artifact；截图、日志、临时设置 fixture 不入库、不上传远程。

## TUI

```bash
cargo run --manifest-path src-tauri/Cargo.toml --features tui --bin copypolish-tui
printf '在LeanCloud上，花了5000元' | cargo run --manifest-path src-tauri/Cargo.toml --features tui --bin copypolish-tui -- --stdin --no-config
cargo run --manifest-path src-tauri/Cargo.toml --features tui --bin copypolish-tui -- --input article.md --output formatted.md --rules all
```

TUI 只负责交互状态和展示，格式化行为必须复用 `engine::format_text`。

交互 TUI 的 `Ctrl+E` 请求设置面板直接维护 `FormatRequest` 的 `replacements` 与 `conversion` 字段，并通过同一个 `rules.yaml` 与 GUI 共享设置；TUI 不复制规则实现。默认构建会在请求和持久化前将不可用简繁模式归一化为 `none`。

## 验证入口

```bash
python3 scripts/verify.py --profile checks
python3 scripts/verify.py --profile frontend
python3 scripts/verify.py --profile rust
python3 scripts/verify.py --profile audit
python3 scripts/verify.py --profile ci
```

`rust` profile 包含 Rust 默认/TUI 检查和性能门禁；`frontend` profile 包含 `npm ci`、Vitest 和前端构建；`checks` 包含 diff、密钥/SOPS 和 Markdown 链接检查；`audit` 执行 npm/Cargo 依赖审计。发布验证使用 `release` profile，必须在隔离 worktree 中执行，见 [release/manual-release.md](release/manual-release.md)。

## 工程约束

1. 前端只能通过 `frontend/src/lib/tauri.ts` 访问后端，不直接调用 `invoke`；
2. 新规则必须加入 `src-tauri/src/engine/registry.rs`，使用稳定 key、明确阶段/依赖/默认状态和 legacy 策略；
3. 需要运行时参数的预设、自定义替换和转换模式不得伪装成静态规则，必须经过明确的数据模型和 IPC/CLI 设计；
4. 规则、保护层、设置和 TUI 变更必须补充对应测试；
5. 保护层遵循“宁漏格式化，不破坏结构”；
6. 不使用默认全文 NFKC 改写文本；
7. 设置测试使用系统临时目录，不污染仓库内 `rules.yaml`；
8. 不提交可再生目录、用户设置、明文 SOPS 文件或 age 私钥。

## CI、发布和安全

- `.github/workflows/ci.yml`：`dev` / `master` 的 push 和 PR 常规 CI；
- `.gitlab-ci.yml`：合法 `v*` tag 的安全检查、Linux/Windows 构建、资产汇总和内部 Release；
- 本地发布脚本：对应平台的隔离 worktree 构建和资产整理；
- GitHub：源码、协作、tag 和公开 Release；
- GitLab：构建服务，不接收日常分支同步。

如果 GitHub Actions 因账户或平台问题不可用，本地 `verify.py` 和 GitLab tag pipeline 是替代门禁，不改变代码要求。发布步骤见 [release/manual-release.md](release/manual-release.md)，升级见 [upgrade-runbook.md](upgrade-runbook.md)，凭据见 [secrets-management.md](secrets-management.md)。

## 相关文档

- [../CONTRIBUTING.md](../CONTRIBUTING.md)：分支、Commit、PR 和完成标准；
- [architecture.md](architecture.md)：模块、数据流和修改入口；
- [testing.md](testing.md)：测试层次、功能矩阵和 fixture 规范；
- [roadmap.md](roadmap.md)：只包含未完成事项；
- [release/manual-release.md](release/manual-release.md)：本地/GitLab 构建和手动发布；
- [upgrade-runbook.md](upgrade-runbook.md)：工具链和依赖升级；
- [decisions/text-cleaning-workflow.md](decisions/text-cleaning-workflow.md)：文本清洗工作流和架构边界；
- [secrets-management.md](secrets-management.md)：SOPS/age 凭据管理；
- [../CHANGELOG.md](../CHANGELOG.md)：版本变化记录；
- [windows-e2e-runbook.md](windows-e2e-runbook.md)：Windows 原生 DPI、Windows Terminal 交互 artifact 和可选 GitLab Windows E2E stage。
