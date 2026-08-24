# CopyPolish 本地构建与手动发布指南

本指南描述在不依赖 GitHub Actions 构建的情况下，于本地编译 Release 产物并手动上传到 GitHub Releases 的完整流程。标准自动发布仍由 `.github/workflows/release.yml` 承担；两种模式共享相同的验证门槛、版本策略、资产命名与人工验收标准。中长期发布相关维护项见 [roadmap.md](roadmap.md)。

## 1. 适用范围与发布模式

| 模式 | 用途 | 说明 |
| --- | --- | --- |
| GitHub Actions 自动构建发布 | 默认路线 | 推送 `v*` tag 或 workflow_dispatch 触发；含 Windows GUI smoke 与自动 Release 创建 |
| 本地构建 + 手动上传 | 备用路线 | Actions 配额/网络受限、需要在真实机器构建验收、或需要更强的发布前人工控制时使用 |

原则：

- 本地发布**不替代** CI：手动上传前必须完成本地等价验证，或确认对应 commit 的 CI 已通过；
- 每个 Release 必须能追溯到一个明确的 Git commit 与 Git tag；
- 未经过 `prepare_release_version.py` 同步版本的二进制不得作为 Release 资产上传；
- Windows 资产必须在 Windows 上构建，Linux 资产必须在 Linux 上构建（本项目未配置交叉编译）。

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

## 7. Linux 安装包构建与资产整理

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

## 8. Windows 真机人工验收

正式发布前，在真实 Windows 10/11 环境运行本地构建的 `CopyPolish.exe` 完成 [v0.5.0-release-plan.md](v0.5.0-release-plan.md) 第 12 节的全部人工验收项，至少包括：

- 启动、WebView2、无边框窗口拖动与最小化/最大化/关闭、最小尺寸 800×600；
- 100%–200% DPI 布局；
- 默认样例：输入 `在LeanCloud上，花了5000元` → 输出 `在 LeanCloud 上，花了 5000 元`；
- 规则全选/恢复默认/自定义/全不选语义；Markdown、URL、LaTeX、代码块、化学式保护；
- 设置保存、重启恢复、不可写目录错误提示、快捷键可用。

## 9. 创建与上传 GitHub Release

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

## 10. 发布后复核与回滚原则

- [ ] tag、Release 标题、应用内"关于"版本三者一致（预发布带 pre 后缀）；
- [ ] 五个资产齐全且命名正确；
- [ ] 正式版标记 latest，预发布标记 prerelease 且不占用 latest；
- [ ] Release Notes 经人工审阅：覆盖本次用户可感知的变化，不重复上一版内容，保留固定说明（便携版命名、设置迁移、已知限制等）；
- [ ] Windows 资产经过实际下载并运行验证；
- [ ] 发布结果同步回对应版本计划文档（如 `v0.5.0-release-plan.md`）。

回滚原则：GitHub Release 可编辑资产列表与 Notes，但**不要删除已发布的 tag**；发现严重问题时优先发预发布修复版，而不是撤回历史 Release。

## 11. 常见失败与排查

| 现象 | 原因 | 处理 |
| --- | --- | --- |
| `check_version.py` 报版本不一致 | 忘记执行 `prepare_release_version.py`，或在错误的工作区执行 | 回到干净发布 worktree 重跑第 4 节两条命令 |
| `.7z` 解压多出一层目录 | 在 staging 目录外压缩 | 删除重打：进入 staging 目录内部再压缩 |
| exe 无法启动 | 缺少旁置 DLL 或 WebView2 Runtime | 对照构建输出目录补齐 DLL；安装 WebView2 Evergreen Runtime |
| 应用内版本与 tag 不符 | 构建发生在版本同步之前 | 重新执行版本同步后重建 |
| AppImage 无法运行 | 构建环境缺 WebKitGTK 系统依赖 | 安装第 2 节列出的 Linux 依赖后重建 |

## 12. 发布记录模板

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
