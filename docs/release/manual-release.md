# CopyPolish 本地构建与手动发布指南

本指南描述当前使用本地构建或 GitLab Build Service 完成构建、资产校验和 GitHub 手动发布的流程。GitHub 负责源码、tag 和公开 Release；构建由本地环境或 GitLab 完成。中长期发布相关维护项见 [roadmap.md](../roadmap.md)。

> 提示：`scripts/build_release_local.sh`（Linux）、`scripts/build_release_local.ps1`（Windows）与 `scripts/verify_release_assets.py`（产物校验）封装了本指南的核心步骤；仍需遵守下文的干净发布工作区与人工验收要求，首次使用前请先通读本指南。

## 1. 适用范围与发布模式

| 模式 | 用途 | 说明 |
| --- | --- | --- |
| GitLab 构建 + 手动整理/发布 | 当前主路线之一 | 手动将合法 `v*` tag 推送到 GitLab；GitLab 构建 Linux/Windows、生成内部资产；维护者下载、校验并手动发布 |
| 本地 Linux/Windows 构建 + 手动发布 | 当前主路线之一 | 在对应原生平台构建全部资产，执行统一校验后手动上传到 GitHub 或 GitLab Release |
| GitHub Actions | 当前不使用 | workflow 已从源码树移除，不得依赖 GitHub runner 自动构建或发布 |

原则：

- 当前 GitHub Actions 暂停期间，本地验证和 GitLab pipeline 是构建门禁；手动上传前必须保留验证日志和资产校验结果；
- 每个 Release 必须能追溯到一个明确的 Git commit 与 Git tag；
- 未经过 `prepare_release_version.py` 同步版本的二进制不得作为 Release 资产上传；
- Windows 资产必须在 Windows 上构建，Linux 资产必须在 Linux 上构建（本项目未配置交叉编译）。

GitLab 手动构建的关键约束：GitLab `.gitlab-ci.yml` 只响应合法 `v*` tag。创建 tag 后，需要将 tag（以及其所需对象）推送到 GitLab；不需要、也不应将 `dev` / `master` 分支日常同步到 GitLab。GitLab 构建完成后，必须由维护者手动下载、校验并完成 Windows smoke 验收。

## 2. 发布前置条件

- Node.js（`.nvmrc` 固定版本）、Rust 工具链（`rust-toolchain.toml` 固定版本）；
- Linux 需要 Tauri 系统依赖：`libwebkit2gtk-4.1-dev`、`libappindicator3-dev`、`librsvg2-dev`、`patchelf`；
- Windows 需要 WebView2 工具链（Tauri CLI 自动处理）；
- `gh` CLI（可选，用于命令行创建 Release；也可用网页手动上传）。

## 3. 使用干净发布工作区

`prepare_release_version.py` 会修改以下文件的版本号：`frontend/package.json`、`frontend/package-lock.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`。**禁止在日常开发工作区直接执行**，避免把临时版本修改混入提交。

推荐使用 Git worktree 从目标 tag/commit 建立隔离发布目录：

```bash
git fetch origin --tags
git worktree add ../copypolish-release <tag-or-commit>
cd ../copypolish-release
```

（也可以单独 clone 一个发布专用目录。）发布完成后删除 worktree 即可丢弃所有版本脚本改动：

```bash
cd - && git worktree remove ../copypolish-release
```

## 4. 版本与 tag 策略

- 正式版 tag：`vX.Y.Z`（如 `v0.5.0`），Release 标记为 latest；
- 预发布 tag：名称含 `-`（如 `v0.5.1-pre1`），Release 必须标记 pre-release 且不占用 latest；
- Release 标题保持与 tag 一致，不加 `CopyPolish` 前缀。

同步 tag 完整版本到构建配置：

```bash
python3 scripts/prepare_release_version.py vX.Y.Z[-suffix]
python3 scripts/check_version.py vX.Y.Z[-suffix]
```

同步后打包产物内 `getVersion()` 显示的版本与 Git tag 一致。

## 5. 发布前统一验证

在发布工作区执行与 CI 对齐的完整验证：

```bash
npm ci --prefix frontend
npm test --prefix frontend -- --run
npm run build --prefix frontend

cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml

git diff --check
python3 scripts/check_version.py vX.Y.Z[-suffix]
```

任一失败都不得继续构建与发布。

## 6. Windows 便携版构建与打包

在 Windows 环境、版本同步之后执行：

```powershell
npm ci --prefix frontend
npm run tauri --prefix frontend -- build -- --no-bundle
```

产物位于 `src-tauri/target/release/chinese-copywriting-formatter.exe`。

打包规范（与 `release.yml` 一致）：

1. 将 exe 重命名为 `CopyPolish.exe` 放入临时 staging 目录；
2. 将构建输出同目录存在的旁置 DLL 一并复制进 staging 根目录；
3. 在 **staging 目录内部**压缩为 `CopyPolish-windows-x64.7z`，确保压缩包根目录直接包含 `CopyPolish.exe`，不含 `dist/`、`windows/` 等额外父目录；
4. 最终资产两个文件平级放置：

```text
CopyPolish.exe
CopyPolish-windows-x64.7z
```

不生成任何安装器（WiX MSI 无法处理中文产品名，且产品定位为免安装便携版）。

## 7. WSL + Windows 主机编译器构建 Windows Release

### 7.1 方案边界

本方案适用于开发者主要在 WSL 中工作，但希望使用 Windows 主机上的原生 MSVC 工具链、Node.js、Tauri CLI 和 7-Zip 构建 Windows 便携版的场景。

```text
WSL
├── Git worktree / 版本同步 / Rust 与前端测试
├── 调用 Windows PowerShell 或 .exe 工具
└── 访问 /mnt/c/... 下的 Windows 工作区
        │
        ▼
Windows 主机
├── Windows Node.js/npm
├── Visual Studio Build Tools / MSVC
├── Windows Rust toolchain（stable-x86_64-pc-windows-msvc）
├── Windows Tauri CLI
├── WebView2 Runtime
└── 7-Zip
```

这里的“Windows 主机编译器”指 Windows 环境中的 MSVC Rust target 与其链接器，不是 WSL 中的 Linux `rustc`。WSL 本身不直接生成可发布的 Windows Tauri 二进制；它负责准备工作区、执行检查，并通过 `powershell.exe` / `cmd.exe` 调度 Windows 侧构建。

### 7.2 推荐前置条件

- 已安装并可从 WSL 调用的 WSL 发行版；
- Windows 主机已安装 Node.js/npm，版本与仓库 `.nvmrc` 一致；
- Windows 主机已安装 Visual Studio Build Tools 的 **Desktop development with C++** 工作负载；
- Windows 主机已安装 Rust MSVC toolchain，并确认 `rustup default` 或项目 toolchain 可使用 `stable-x86_64-pc-windows-msvc`；
- Windows 主机已安装 WebView2 Runtime；
- Windows 主机已安装 7-Zip，并可通过 `7z.exe` 调用；
- WSL 中可执行 `powershell.exe` 或 `pwsh.exe`，且 Windows 工具能访问同一份源码目录；
- 源码位于 Windows 文件系统（例如 `C:\src\chinese_copywriting_formatter`，WSL 中对应 `/mnt/c/src/chinese_copywriting_formatter`）。

不建议将发布工作区放在 WSL 的 ext4 文件系统中后再让 Windows 工具通过网络或特殊路径访问。Node、Cargo、Tauri 和 Windows 文件监视器在 `/mnt/c/...` 下的行为更容易与 Windows 原生构建保持一致；如果性能明显不足，可在 Windows 文件系统中准备独立发布 worktree。

### 7.3 从 WSL 准备隔离发布工作区

在 WSL 中执行 Git 操作和版本同步。以下路径仅为示例，请替换为实际 Windows 路径：

```bash
export WIN_REPO='C:\src\chinese_copywriting_formatter'
export WSL_REPO='/mnt/c/src/chinese_copywriting_formatter'

cd "$WSL_REPO"
git fetch origin --tags
git worktree add "$WSL_REPO-release" <tag-or-commit>
cd "$WSL_REPO-release"

python3 scripts/prepare_release_version.py vX.Y.Z[-suffix]
python3 scripts/check_version.py vX.Y.Z[-suffix]
```

`prepare_release_version.py` 会修改版本文件，因此仍然必须在隔离 worktree 中执行。WSL 与 Windows PowerShell 后续必须使用同一个发布 worktree，不能一个使用 `/mnt/c/...`、另一个使用另一个 clone。

### 7.4 WSL 中执行纯 Rust 与前端验证

WSL 可以执行不依赖 Windows GUI 的验证：

```bash
cd "$WSL_REPO-release"

npm ci --prefix frontend
npm test --prefix frontend -- --run
npm run build --prefix frontend

cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml

git diff --check
python3 scripts/check_version.py vX.Y.Z[-suffix]
```

这些命令验证前端、纯 Rust 引擎、设置测试和版本一致性；它们**不等于** Windows Tauri Release 构建。最终 Windows 产物仍必须由 Windows 主机工具链生成并在 Windows 上启动验收。

### 7.5 从 WSL 调用 Windows 主机构建

推荐让 Windows PowerShell 完成 `npm ci`、Tauri build 和资产打包。先在 WSL 中把 Linux 路径转换为 Windows 路径：

```bash
WIN_RELEASE_REPO="$(wslpath -w "$WSL_REPO-release")"
echo "$WIN_RELEASE_REPO"

powershell.exe -NoProfile -ExecutionPolicy Bypass -Command \
  "Set-Location -LiteralPath '$WIN_RELEASE_REPO'; \
   npm ci --prefix frontend; \
   npm run tauri --prefix frontend -- build -- --no-bundle"
```

如果主机使用 PowerShell 7，也可以将 `powershell.exe` 替换为 `pwsh.exe`。路径中包含空格时，优先使用 `-LiteralPath`；复杂路径或复杂参数建议写入一个 Windows `.ps1` 脚本后由 WSL 调用，避免 Bash、PowerShell 和 JSON 字符串多重转义。

构建完成后，Windows 输出仍应位于发布 worktree 的：

```text
src-tauri\target\release\chinese-copywriting-formatter.exe
```

### 7.6 Windows 侧打包 `.exe` 与 `.7z`

可以继续从 WSL 调度 Windows PowerShell 完成与 GitHub Actions 等价的 staging 和压缩流程：

```bash
powershell.exe -NoProfile -ExecutionPolicy Bypass -Command \
  "Set-Location -LiteralPath '$WIN_RELEASE_REPO'; \
   \$exe = 'src-tauri/target/release/chinese-copywriting-formatter.exe'; \
   if (-not (Test-Path -LiteralPath \$exe)) { throw 'Executable not found' }; \
   \$staging = Join-Path \$PWD 'windows-portable-staging'; \
   New-Item -ItemType Directory -Force -Path \$staging | Out-Null; \
   Copy-Item -LiteralPath \$exe -Destination (Join-Path \$staging 'CopyPolish.exe'); \
   Get-ChildItem (Split-Path \$exe) -Filter '*.dll' -File -ErrorAction SilentlyContinue | Copy-Item -Destination \$staging; \
   \$sevenZip = (Get-Command 7z -ErrorAction SilentlyContinue).Source; \
   if (-not \$sevenZip) { \$sevenZip = 'C:\\Program Files\\7-Zip\\7z.exe' }; \
   \$dist = Join-Path \$PWD 'dist/windows'; \
   New-Item -ItemType Directory -Force -Path \$dist | Out-Null; \
   Push-Location \$staging; \
   try { & \$sevenZip a -t7z -mx=9 (Join-Path \$dist 'CopyPolish-windows-x64.7z') (Get-ChildItem -File | ForEach-Object { \$_.Name }) } finally { Pop-Location }; \
   if (\$LASTEXITCODE -ne 0) { throw '7z failed' }; \
   Copy-Item -LiteralPath (Join-Path \$staging 'CopyPolish.exe') -Destination (Join-Path \$dist 'CopyPolish.exe'); \
   Remove-Item -Recurse -Force -Path \$staging"
```

也可以直接打开 Windows PowerShell，在同一发布 worktree 中执行本指南第 6 节的打包步骤。无论从 WSL 调度还是在 Windows 终端直接执行，最终必须确认：

- `dist/windows/CopyPolish.exe` 存在；
- `dist/windows/CopyPolish-windows-x64.7z` 存在；
- `.7z` 根目录直接包含 `CopyPolish.exe`，没有额外父目录；
- 不把 WSL 构建出的 Linux ELF 文件误命名为 Windows `.exe`；
- 用 Windows 终端或资源管理器启动最终 exe，而不是只检查文件存在。

### 7.7 WSL + Windows 构建的限制

- WSL 中的 Linux `cargo test` 与 Windows MSVC Release 构建使用的 target 不同；必须分别验证；
- 不要在 WSL 中运行 `npm run tauri build` 后把结果当作 Windows 资产，除非命令明确由 Windows `npm`/Tauri CLI 执行；
- Windows 主机构建依赖 MSVC、Windows SDK、WebView2 和 Windows Node.js，WSL 安装的 Linux 依赖不能替代它们；
- Windows 与 WSL 共同访问 `/mnt/c` 时可能较慢，发布构建建议使用独立、干净的 Windows 文件系统 worktree；
- 构建过程中不要同时从 WSL 和 Windows 运行两个 npm/Cargo 进程，避免 `node_modules`、`target` 或 staging 文件锁冲突；
- 发布前仍需在真实 Windows 10/11 环境完成 GUI、DPI、窗口控制、设置持久化和格式化人工验收。

## 8. Linux 安装包构建与资产整理

在满足系统依赖的 Linux 环境、版本同步之后执行：

```bash
npm ci --prefix frontend
npm run tauri --prefix frontend -- build
```

从 `src-tauri/target/release/bundle/` 收集并统一命名为：

```text
CopyPolish_linux_amd64.deb
CopyPolish-linux-x86_64.rpm
CopyPolish_linux_amd64.AppImage
```

### 8.1 发布资产来源

当前标准流程由 GitLab tag pipeline 构建并汇总全部五项平台资产及 `SHA256SUMS`。维护者下载后执行完整校验，再使用 GitHub CLI 创建公开 Release；不再维护本地分阶段上传 Linux 资产的备用脚本。

若从 GitLab Package Registry 下载 AppImage 遇到网络错误，应先解决认证或网络问题；在五项资产齐全前不要创建 GitHub Release。下载前可确认以下 URL 对应文件返回 HTTP 200：

```text
https://gitlab.com/api/v4/projects/85804438/packages/generic/copypolish/vX.Y.Z[-suffix]/CopyPolish_linux_amd64.AppImage
```

手动发布前必须下载并校验全部五项资产，缺少 AppImage 时不得继续发布，这是预期的安全门禁。

> 不要把 `GITLAB_TOKEN` 写入 remote URL、脚本、仓库文件或提交历史。PAT 仅用于本次上传和 GitLab API 操作；GitLab MCP Server 仍使用 OAuth，不接受 PAT。

## 9. Windows 真机人工验收

正式发布前，在真实 Windows 10/11 环境运行本地构建的 `CopyPolish.exe` 完成 [v0.5.0-release-plan.md](../v0.5.0-release-plan.md) 第 12 节的全部人工验收项，至少包括：

- 启动、WebView2、无边框窗口拖动与最小化/最大化/关闭、最小尺寸 800×600；
- 100%–200% DPI 布局；
- 默认样例：输入 `在LeanCloud上，花了5000元` → 输出 `在 LeanCloud 上，花了 5000 元`；
- 规则全选/恢复默认/自定义/全不选语义；Markdown、URL、LaTeX、代码块、化学式保护；
- 设置保存、重启恢复、不可写目录错误提示、快捷键可用。

## 10. 创建与上传 GitHub Release

确认 tag 已存在（或先推送 tag）：

```bash
git tag vX.Y.Z <commit> && git push origin vX.Y.Z
```

使用 GitHub CLI 上传（正式版）：

```bash
gh release create vX.Y.Z \
  --title vX.Y.Z \
  --generate-notes \
  --latest \
  CopyPolish.exe \
  CopyPolish-windows-x64.7z \
  CopyPolish_linux_amd64.deb \
  CopyPolish-linux-x86_64.rpm \
  CopyPolish_linux_amd64.AppImage
```

预发布必须显式改为：

```bash
gh release create vX.Y.Z-preN \
  --title vX.Y.Z-preN \
  --generate-notes \
  --prerelease \
  <assets...>
```

也可在 GitHub Releases 页面手动 "Draft a new release"，选择已有 tag 后逐一上传资产。

## 11. 发布后复核与回滚原则

- [ ] tag、Release 标题、应用内"关于"版本三者一致（预发布带 pre 后缀）；
- [ ] 五个资产齐全且命名正确；
- [ ] 正式版标记 latest，预发布标记 prerelease 且不占用 latest；
- [ ] Release Notes 经人工审阅：覆盖本次用户可感知的变化，不重复上一版内容，保留固定说明（便携版命名、设置迁移、已知限制等）；
- [ ] Windows 资产经过实际下载并运行验证；
- [ ] 发布结果同步回对应版本计划文档（如 `v0.5.0-release-plan.md`）。

回滚原则：GitHub Release 可编辑资产列表与 Notes，但**不要删除已发布的 tag**；发现严重问题时优先发预发布修复版，而不是撤回历史 Release。

## 12. 常见失败与排查

| 现象 | 原因 | 处理 |
| --- | --- | --- |
| `check_version.py` 报版本不一致 | 忘记执行 `prepare_release_version.py`，或在错误的工作区执行 | 回到干净发布 worktree 重跑第 4 节两条命令 |
| `.7z` 解压多出一层目录 | 在 staging 目录外压缩 | 删除重打：进入 staging 目录内部再压缩 |
| exe 无法启动 | 缺少旁置 DLL 或 WebView2 Runtime | 对照构建输出目录补齐 DLL；安装 WebView2 Evergreen Runtime |
| 应用内版本与 tag 不符 | 构建发生在版本同步之前 | 重新执行版本同步后重建 |
| AppImage 无法运行 | 构建环境缺 WebKitGTK 系统依赖 | 安装第 2 节列出的 Linux 依赖后重建 |
| WSL 中产物不是 Windows exe | 实际调用了 WSL 的 Linux `npm`/Tauri/Cargo，而不是 Windows 主机工具链 | 检查 PowerShell 中的 `node --version`、`rustc -vV` 的 host/target，并从 Windows PowerShell 重新构建 |
| `powershell.exe` 找不到仓库路径 | WSL 路径没有通过 `wslpath -w` 转换，或路径指向另一个 clone | 使用同一个 Windows 文件系统 worktree，并用 `wslpath -w` 生成 `-LiteralPath` |
| Windows 构建被文件占用 | WSL 与 Windows 同时运行 npm/Cargo，或旧 exe 仍在运行 | 关闭应用和构建进程，清理 staging 后只保留一个构建进程 |
| 找不到 MSVC linker / Windows SDK | Windows 主机缺少 C++ Build Tools 或 MSVC Rust target | 在 Windows 主机安装/修复 **Desktop development with C++**，并检查 `rustup target list --installed` |

## 13. 发布记录模板

每次手动发布后在对应版本计划文档（或 PR 描述）追加：

```markdown
## 发布记录 vX.Y.Z（YYYY-MM-DD）
- commit：<sha>
- 构建方式：本地构建（Windows <工具链版本> / Linux <发行版>）+ 手动上传
- 验证：CI run <链接> / 本地全量命令通过
- Windows 真机验收：通过（记录人、机型、系统版本、DPI）
- Release URL：<链接>
- 资产核对：exe / 7z / deb / rpm / AppImage 均已上传且命名正确
- latest / prerelease 标记：正确
```
