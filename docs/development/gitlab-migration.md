# CopyPolish GitLab Build Service：状态与 CI/CD 说明

本文记录 GitHub 主平台与 GitLab Build Service 之间的构建编排、当前进度、已定决策、CI/CD 运行方式与后续待办。
仅面向维护者；Cline 接入 GitLab MCP 见 [gitlab-mcp.md](gitlab-mcp.md)。

> 现状速览（2026-08-27）：GitHub 是源码、开发协作、tag 和公开 Release 的主平台；
> GitLab 仅接收 GitHub Release workflow 推送的 `v*` tag，并负责 Linux/Windows 构建与内部构建 Release。
> GitLab build-only CI、GitHub tag bridge 和 GitHub Release 编排已落地到 `dev`；
> 尚待使用新的验证 tag 完成完整 GitHub → GitLab → GitHub 链路验收。

## 1. 架构与目标

```text
开发者 / Cline
        │
        ▼
GitHub（主平台）
  ├── 开发 / Issue / Pull Request
  ├── GitHub Actions 普通 CI
  ├── 创建 v* tag
  └── 编排并创建公开 GitHub Release
              │ 精确同步同一 tag
              ▼
GitLab Build Service
  ├── Linux 构建
  ├── Windows SaaS 构建
  ├── Generic Package Registry
  └── 内部 GitLab Release
```

原则：

- GitHub 是唯一日常写入源；开发、Issue、Pull Request、tag 和公开 Release 均在 GitHub 完成；
- GitLab 只接收 GitHub Release workflow 推送的 `v*` tag，不接收 `dev` / `master` 日常同步；
- GitLab CI 只负责跨平台构建和内部构建 Release，GitHub Actions 负责发布编排和最终公开 Release；
- GitLab MCP 只用于 Build Service 状态和日志诊断，不进入发布关键路径（见 gitlab-mcp.md）。

## 2. 远程仓库现状

| 远程 | URL | 角色 |
| --- | --- | --- |
| `origin` | `https://github.com/15699122/chinese_copywriting_formatter.git` | 主平台 |
| `gitlab` | `https://gitlab.com/olivaceum-group/chinese_copywriting_formatter.git` | Build Service |

分支：

- `dev`：GitHub 默认开发分支，本地 upstream 为 `origin/dev`；
- `master`：稳定分支，与既有流程不变；
- GitLab 不维护日常开发分支，只保留 GitHub workflow 推送的 Release tag。

## 3. Tag 差异记录

GitLab 比 GitHub 多以下内容（这些 tag 在 GitHub 从未存在）：

- `v0.5.0`
- `v0.5.0-pre5`
- `backup/pre-merge-ub-into-dev-20260825-142543`

即本地 / GitLab 拥有既有版本历史的完整来源，GitHub 只有到 `v0.5.0-pre4` 的记录。
是否补推 GitHub 属待办决策（见 §7）。

## 4. 已定技术决策

1. **项目迁移至 `olivaceum-group`（Ultimate 试用）**：个人 namespace（Free）无法使用 Windows SaaS 共享 runner；迁移后可用。
2. **Windows 构建使用 GitLab.com SaaS Windows runner**，标签 `saas-windows-medium-amd64`（已确认两台在线且可调度）。
3. **Windows 环境探测结论**（`windows-probe` job 日志 2026-08-27）：
   - 镜像为 Windows Server VM（Packer），每 job 全新拉起；
   - Node.js 预装 `v21.7.3`（**非项目固定的 24.19.0**）；
   - **Rust 未预装**（`rustc` / `cargo` 不存在）；
   - Git for Windows 已预装（`2.51.2.windows.1`）；7-Zip 需要 job 内确认/安装。
4. **不能「像 GitHub 一样随身完整工具缓存」的原因**：SaaS runner 每 job 新建临时 VM（Custom executor + autoscaler，job 结束即销毁），工具链与 `.cargo/` 无法跨 job 持久化；且镜像未预装 Rust。因此 `build:windows` 必须在 job 内显式自装 Rust + 对齐 Node 24.19.0，不依赖缓存，按「全新 VM」设计。Windows job 还必须把 `CARGO_HOME` / `RUSTUP_HOME` 放在 `%TEMP%`，不能放在仓库目录，否则 rustup 生成的 `.cargo` 会使发布脚本的干净工作区检查失败。
5. GitLab SaaS Windows runner 默认以 Windows PowerShell 5.1 执行 job。当前通过 `PYTHONIOENCODING=utf-8` 解决 Python 中文输出问题；如继续遇到 5.1 兼容性问题，再在 job 内安装 PowerShell 7 并用 `pwsh -File` 执行独立脚本，但不能通过项目 YAML 修改 SaaS runner 的底层 shell。

## 5. GitLab Build Service CI/CD 说明

主配置：仓库根 `.gitlab-ci.yml`。按 `workflow:rules` 分流：

- **tag**（`vX.Y.Z` 或含 `-` 的预发布）：`build:linux` 与 `build:windows` 并行，随后 `package:assemble` 和 `release:gitlab`；
- GitLab 不响应 `dev` / `master` push、GitLab MR 或普通开发流水线。

stage 顺序：`build → package → release`。

- `build:linux`：GitLab Linux runner 安装 Tauri GTK/WebKit 依赖，构建并校验 deb/rpm/AppImage；
- `build:windows`：Windows SaaS runner 通过 `scripts/ci/build_windows_gitlab.ps1` 自装并核验 MSVC/Rust/Node/Python/7-Zip，构建并校验 exe/.7z；
- `package:assemble`：合并两平台 artifacts，运行 `verify_release_assets.py --platform all` 并生成 SHA256SUMS；
- `release:gitlab`：上传六个构建文件到 Generic Package Registry，创建内部 GitLab Release；公开 Release 仍由 GitHub Actions 创建。

版本脚本复用既有：`check_version.py` / `prepare_release_version.py` / `verify_release_assets.py`。

## 6. GitHub → GitLab 构建编排（代码已落地，待外部 Secret 与首轮验收）

GitHub 为主，GitLab 为 Build Service，不做双向写：

- `release.yml` 通过 `scripts/ci/push_tag_to_gitlab.sh` 推送精确 tag；
- `wait_for_gitlab_pipeline.py` 验证 GitLab pipeline SHA 与 GitHub tag SHA 相同并等待成功；
- `download_gitlab_release_assets.py` 下载并验证 GitLab 内部 Release 资产；
- GitHub Windows runner 对 GitLab 生成的 exe 执行 GUI smoke，随后创建公开 GitHub Release；
- `release-fallback.yml` 仅手动触发，在 GitLab Build Service 不可用时由 GitHub 全平台构建。
- [x] `dev` 本地 upstream 已切换为 `origin/dev`（GitHub）；
- [x] GitLab CI 已改为仅响应 Release tag；
- [x] GitLab Linux/Windows 构建与内部 Release job 已落地；
- [x] GitHub 主 Release workflow 已加入 tag bridge、pipeline 等待、资产下载、SHA 校验和 Windows smoke；
- [x] `release-fallback.yml` 已改为仅手动触发；
- [ ] 按 gitlab-mcp.md 完成 Build Service 只读验收；
- [ ] 创建 `v0.5.0-pre8`，验收 GitHub tag bridge、GitLab 双平台构建和 GitHub 公开 Release；
- [ ] 核对 GitHub 下载的五项资产与 GitLab SHA256SUMS 一致；
- [ ] 完成真实 Windows GUI/DPI/WebView2 人工验收。

## 8. Windows 人工验收（保持既有约束）

即便 SaaS runner 构建成功，正式发布前仍须在真实 Windows 10/11 完成人工验收（清单见 `../v0.5.0-release-plan.md` 第 12 节）：WebView2、无边框窗口、DPI、默认样例 `在LeanCloud上，花了5000元` → `在 LeanCloud 上，花了 5000 元`、规则 / 设置持久化。

## 9. 风险与备注

- SaaS Windows runner 属 beta，可能随试用计划变化；`build:windows` 始终按「全新 VM、自装工具」编写；
- `prepare_release_version.py` 只应在干净发布工作区 / CI 内运行，禁止在待提交开发工作区直接执行；
- GitHub Release token 只允许在同步 job 内使用，MCP 及其他 job 不得接触（见 gitlab-mcp.md 安全规则）。