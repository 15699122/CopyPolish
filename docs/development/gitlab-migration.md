# CopyPolish → GitLab 迁移：状态与 CI/CD 说明

本文记录本仓库从 GitHub 迁移至 GitLab 的整体计划、当前进度、已定决策、GitLab CI/CD 运行方式与后续待办。
仅面向维护者；Cline 接入 GitLab MCP 见 [gitlab-mcp.md](gitlab-mcp.md)。

> 现状速览（2026-08-27）：GitLab 已是主仓库与主 CI，GitHub 为只读镜像与第二个下载入口；
> 普通 CI（test:rust / test:frontend）已在 GitLab 全绿；Windows SaaS runner 已成功调度；
> `v0.5.0-pre6` 的 Linux 构建已成功；Windows SaaS 已完成 Rust/Node/Python/7-Zip 初始化，
> 独立构建脚本已修正 Tauri `--no-bundle` 参数转发和 Windows 工具链隔离；Linux 已切换为本地构建后上传。
> 当前验证 tag 为 `v0.5.0-pre7`：`.deb` / `.rpm` 已上传，`.AppImage` 因网络上传异常暂待手动上传；
> 在五项资产齐全前不执行 `release:finalize`。Push mirror 与 GitHub Release 同步尚未配置。

## 1. 架构与目标

```text
开发者 / Cline
        │
        ▼
GitLab（主仓库：olivaceum-group/chinese_copywriting_formatter）
  ├── GitLab CI/CD（test → build → package → release）
  ├── Generic Package Registry（长期二进制存储）
  ├── GitLab Release（权威发布）
  │
  ├── [待办] Push Mirror ───────────────► GitHub refs（只读镜像）
  └── [待办] Release Sync Job ──────────► GitHub Release assets
```

原则：

- GitLab 是**唯一写入源**；所有 push / MR / tag / Release 在 GitLab 完成；
- GitLab CI 是唯一自动构建路径（对应原 GitHub `ci.yml` / `release.yml`）；
- GitHub 仓库只接收被推送的 refs 与 Release assets，不接受直接开发提交；
- GitLab MCP 属于辅助控制面，不进入发布关键路径（见 gitlab-mcp.md）。

## 2. 远程仓库现状

| 远程 | URL | 角色 |
| --- | --- | --- |
| `origin` | `https://github.com/15699122/chinese_copywriting_formatter.git` | 镜像 |
| `gitlab` | `https://gitlab.com/olivaceum-group/chinese_copywriting_formatter.git` | 主仓库 |

分支：

- `dev`：默认开发分支，三端（本地 / GitHub / GitLab）已同步；
- `master`：稳定分支，与既有流程不变；
- `ci/windows-probe`：Windows SaaS Runner 探测临时分支，已完成探测并删除，不推 GitHub。

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

## 5. GitLab CI/CD 说明

主配置：仓库根 `.gitlab-ci.yml`。按 `workflow:rules` 分流：

- **分支 / MR**（`dev` / `master`）：`test:rust`、`test:frontend`；
- **tag**（`vX.Y.Z` 或含 `-` 的预发布）：`build:windows` → `publish:windows` → 手动 `release:finalize`；Linux 资产由本地脚本上传后参与 finalize。

stage 顺序：`test → build → package → release`。

- `build:windows`：Windows SaaS runner，独立脚本 `scripts/ci/build_windows_gitlab.ps1` 自装并核验 MSVC/Rust/Node/Python/7-Zip，使用已验证的 `npm run tauri build -- --no-bundle`，产出两个 Windows 资产；
- `publish:windows`：使用 `CI_JOB_TOKEN` 上传 Windows 资产至 Generic Package Registry；
- Linux 资产：在 Node 24.19.0 和 Linux Tauri 依赖齐全的本地隔离 worktree 中运行 `scripts/build_release_local.sh`，再运行 `scripts/upload_gitlab_linux_assets.sh` 上传三个 Linux 资产；
- `release:finalize`：手动下载同一 tag 的五个资产，运行 `verify_release_assets.py --platform all`，生成 SHA256SUMS，上传摘要并创建 GitLab Release；tag 含 `-` 即 prerelease；`resource_group` 防同 tag 并发。

版本脚本复用既有：`check_version.py` / `prepare_release_version.py` / `verify_release_assets.py`。

## 6. 同步 GitHub 的方式（待实施）

GitLab 为主，GitHub 为镜像，不做双向写：

- refs 同步：GitLab **push mirror** 到 GitHub（Settings → Repository → Mirroring repositories → Push）；
- Release assets：由 GitLab CI 后续新增的 `mirror-release` job（需 CI/CD Variable `GITHUB_RELEASE_TOKEN`、`GITHUB_REPOSITORY`）把 `release:finalize` 生成的同一批资产发布到 GitHub Release；
- GitHub Actions：迁移稳定后降级为手动 / 备用，避免两边重复构建与重复创建 Release。

## 7. 当前待办（按优先级）

- [x] **`build:windows` 自装工具链基础部分**（rustup 装 Rust + MSVC，装 Node 24.19.0、Python 3 shim、7-Zip）已落地到 dev；SaaS runner 已成功执行工具链安装；
- [ ] 修正后的独立 Windows 脚本在 SaaS runner 上完成 Tauri exe / `.7z` 产物验证（当前待用新验证 tag 重新执行）；
- [x] 本地 Linux 资产完成构建和平台校验；`v0.5.0-pre7` 的 `.deb` / `.rpm` 已上传；
- [ ] 手动上传 `v0.5.0-pre7` 的 `CopyPolish_linux_amd64.AppImage`，再执行 `release:finalize`；
- [x] 清理临时分支 `ci/windows-probe`（本地 + GitLab）及其临时 `workflow:rules` 放行；
- [ ] tag 对齐决策：是否将 GitLab 独有的 `v0.5.0` / `v0.5.0-pre5` / backup tag 补推 GitHub；
- [ ] 配置 GitLab → GitHub push to；
- [ ] 新增 `release:github` 同步 job，并配置 `GITHUB_RELEASE_TOKEN` / `GITHUB_REPOSITORY` CI/CD Variable；
- [ ] GitHub Actions 降级为手动 / 备用；
- [ ] 按 gitlab-mcp.md 完成 MCP 只读验收；
- [ ] 更新 `docs/development.md`「持续集成」与 roadmap 中把 GitHub 当标准 CI 的叙述为 GitLab 主路径现状。

## 8. Windows 人工验收（保持既有约束）

即便 SaaS runner 构建成功，正式发布前仍须在真实 Windows 10/11 完成人工验收（清单见 `../v0.5.0-release-plan.md` 第 12 节）：WebView2、无边框窗口、DPI、默认样例 `在LeanCloud上，花了5000元` → `在 LeanCloud 上，花了 5000 元`、规则 / 设置持久化。

## 9. 风险与备注

- SaaS Windows runner 属 beta，可能随试用计划变化；`build:windows` 始终按「全新 VM、自装工具」编写；
- `prepare_release_version.py` 只应在干净发布工作区 / CI 内运行，禁止在待提交开发工作区直接执行；
- GitHub Release token 只允许在同步 job 内使用，MCP 及其他 job 不得接触（见 gitlab-mcp.md 安全规则）。