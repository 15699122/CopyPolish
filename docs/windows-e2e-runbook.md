# Windows 原生 E2E 与交互留证 Runbook

本文记录必须在 **Windows 原生桌面环境**执行或重新留证的步骤。当前优先项是默认 embedded GUI 完整回归、`simplified-trad-conversion` feature 的 embedded GUI 双向真实转换、标准 W3C 兼容性 smoke、NTFS ACL 保存失败和按需 GUI 视觉 artifact；Windows Terminal TUI 交互与三档 DPI 人工检查已有结果，GUI DPI 自动矩阵和 GitLab Windows 可选 stage 按项目决定跳过，不得重复记为未完成。

Linux、WSL/WSLg 和普通 Chrome 可以验证部分业务语义，但不能替代本文所需的 Windows WebView2、Windows 显示缩放、NTFS/Windows 进程、Windows Terminal raw-mode 或 OSC 52 行为。

## 1. 范围与当前状态

### 1.1 已完成、无需重复判定为未测的内容

以下 Windows 功能性验证已经完成：

- WebView2 embedded provider 最小启动、session、真实 Rust IPC 和进程清理；标准 W3C provider 已于 2026-09-01 收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`），不再与 embedded provider 并行跑完整回归；
- 设置保存、重启恢复、损坏设置恢复和 NTFS ACL 拒写；
- GUI 主题/窄窗口 artifact 基础入口；
- Windows Terminal + PowerShell 7 下的 raw-mode、规则排序、快捷键、粘贴、OSC 52、保存和重启恢复的人工回归（**单行输入场景**）；
- TUI grapheme 编辑边界修复及 Rust/TUI 回归。

> **2026-09-01 更新**：Windows Terminal 交互复验曾发现三个**多行/自动换行显示缺陷**（详见 [windows-terminal-tui-manual.md](windows-terminal-tui-manual.md)，问题证据截图仅本地留存、测试后已按约定清理，不入库）。这三个问题已在修复版本中完成，并由用户确认通过真实 Windows Terminal 复验；该段保留为历史发现记录。

### 1.2 当前 Windows 执行矩阵

| 工作项 | 目标 | 当前状态 | 完成标志 |
| --- | --- | --- | --- |
| 默认 embedded GUI 完整回归 | 验证 Windows WebView2、真实 Rust IPC、设置保存/恢复和替换输出 | 已有基线；换 binary 或前端后应重新执行 | `npm run test --prefix e2e` 通过，artifact 完整 |
| 简繁 feature embedded GUI | 验证 `s2t`/`t2s` 的真实字符转换，而非默认构建占位 | Windows 当前复验 2/2 通过（s2t、t2s） | feature 构建和 `simplified-trad-conversion.spec.ts` 2/2 通过 |
| 标准 W3C provider | 验证 session、主窗口、一次格式化、一次设置保存和退出清理 | 已收敛为兼容性 smoke | `npm run test:webdriver --prefix e2e` 通过 |
| GUI DPI artifact | 在 100%/125%/150% Windows 缩放下保存可审计截图、page source 和环境信息 | 自动验证跳过；三档人工 GUI 验证已完成 | 不纳入自动化门禁；按需保留人工结果 |
| Terminal 多行显示修复 | 修复自动换行后新增字符绘制位置、光标不可见和 emoji 显示（WT-TUI-001/002/003） | 源码修复已完成，Windows MSVC 166/166 通过；真实 Terminal 复验已由用户确认通过 | Rust/UI 回归通过，Windows 原生复验通过 |
| Terminal TUI artifact | 将真实 Windows Terminal 交互和退出清理固化为输入/输出/截图/日志留证 | 已通过（依据用户确认） | 关键场景、清理和结果均已确认 |
| GitLab Windows stage | 在 Windows runner 上重复运行 E2E 并始终上传 artifact | 跳过（不执行） | 项目决定不配置、不运行；不得记为通过 |

### 1.2.1 PR #24 Windows 验收结果（2026-09-04）

PR #24（`fix/settings-dialog-polish`，代码范围起点 `fa2fc1c`）修改了设置窗口的布局、路径交互和规则展示。2026-09-04 已在包含 PR #24 的 Windows binary 上完成以下复验：

| 待办 | 必须确认的内容 | 完成证据 |
| --- | --- | --- |
| 主题布局 | “跟随系统 / 浅色 / 深色”三个选项等宽，选项间距统一；浅色、深色主题下文字、禁用态和焦点环可读 | 已通过：浅色/深色截图、page source 和环境 metadata |
| 窄窗口 Footer | 正常窗口及至少一个窄窗口下，设置文件区域不挤压操作按钮，底部不溢出 | 已通过：GUI visual artifact 和人工检查 |
| 设置路径交互 | 页面仅显示 `rules.yaml`；悬停/聚焦可获取完整路径；鼠标点击、键盘 Enter/Space 可复制完整路径；成功/失败反馈正常 | 已通过：page source 和 Windows 剪贴板人工确认 |
| 简繁转换布局 | “简繁转换”标签与选择框有明确垂直间距，默认构建的 T2S/S2T 禁用状态保持正确 | 已通过：设置截图和 page source |
| 规则示例提示 | 悬停规则卡片显示 `before → after` 示例；键盘聚焦 Checkbox 时可通过辅助描述获取同一示例 | 已通过：page source 和鼠标/键盘交互记录 |
| 当前 binary 设置回归 | 规则、主题、字体、缩放、替换和转换设置保存/恢复，真实 Rust IPC 输出正常 | 已通过：selection 3/3、默认/feature restart 各 2/2 |

本章节的验收不能由 WSL、普通浏览器或 Linux GUI 替代。本次 PR #24 已取得对应 Windows 原生证据；后续仅在相关代码、工具链或诊断范围变化时复跑。

**状态更新**：用户已确认 WT-TUI-001/002/003 修复后完成 Windows 原生复验，Terminal 交互 artifact 标记为通过。GitLab Windows E2E stage 已决定跳过（不执行）。
### 1.3 2026-09-02 Windows 原生复验记录

本轮在 E 盘 checkout、Windows WebView2 151.0.4129.107、Node 24.19.0、Rust 1.98.0 `x86_64-pc-windows-msvc`、PowerShell 7.6.5 和 Visual Studio Build Tools 17.14.37614.0 上重新执行了可自动化项目。前端测试 69/69、E2E typecheck、embedded/WebDriver/`simplified-trad-conversion`/TUI release 构建均通过；标准 W3C smoke 2/2、设置重启 write/read 2/2、损坏设置三种 fixture 3/3、NTFS ACL 1/1、GUI 视觉 artifact 1/1、设置快捷键控制台 1/1、TUI transcript 4/4 均通过。受控失败 artifact probe 按预期生成失败结果并通过 bundle 自检。随后 Linux/WSL 已完成当前修复 binary 的替换链路 3/3 和 feature 双向转换 2/2；Windows 仍需用当前修复 binary 重新执行对应 GUI spec。

以下内容是历史失败记录，原始 artifact 保留在 `e2e/artifacts/embedded/`；PR #24 的当前状态以本 Runbook §14 的最终闭环记录为准：

- `selection-and-persistence.spec.ts`：旧 binary 的第三个“真实 GUI 保存替换项和简繁转换设置” case 在设置已写入后，输入 `TODO` 未得到 `待办`（本轮 artifact `1788353402936`，早先复现 artifact `1788351053668`）；该问题已在后续 Windows 当前 binary 复验中闭环。
- `simplified-trad-conversion.spec.ts`：首次运行因实际使用了默认 binary 而未产生转换结果；先执行 `build:app:simplified-trad` 后，s2t/t2s 均通过（2/2）。
- PR #24 设置页验收：主题三项等宽间距、`rules.yaml` 路径仅显示文件名并支持完整路径复制、简繁转换间距、规则 hover 示例、键盘辅助描述和窄窗口 Footer 已在 §14 的 Windows 当前 binary 复验中闭环。

上述第一项属于历史失败，已由后续当前 binary 复验闭环；其余专项结果不受影响。GUI DPI 自动矩阵和 GitLab Windows 可选 stage 仍按项目决定跳过，Windows Terminal 交互 artifact 仍按用户确认标记通过。Unix-only 权限测试和 Windows MSVC 结果以 §14 的最终记录为准。

已确认问题清单（详见 [windows-terminal-tui-manual.md](windows-terminal-tui-manual.md)）：

- `WT-TUI-001`：多行输入后新增字符串行绘制/定位错误，重叠到底部状态栏；
- `WT-TUI-002`：多行输入时光标不可见；
- `WT-TUI-003`：Windows Terminal 下 emoji 显示问题（需区分应用宽度/绘制与终端字体回退）。

React 19 `act` warning 和依赖/许可证审计不属于本文的 Windows 必需项；它们应在各自的路线图任务中独立处理。

## 2. 通用 Windows 前置条件

### 2.1 主机要求

- 可交互的 Windows 桌面会话，不能使用仅服务会话或无桌面 runner；
- Windows Terminal 与 PowerShell 7；
- WebView2 Runtime；
- Visual Studio Build Tools 的 Desktop development with C++；
- Rust `x86_64-pc-windows-msvc` toolchain；
- Node 满足仓库约束 `>=24 <25`，优先使用 `.nvmrc` 对应版本；
- Git、7-Zip（构建或 artifact 打包需要时）；
- 具备查看显示设置、剪贴板和进程/端口的权限；
- 项目 checkout 位于短路径，避免长路径和特殊字符掩盖布局问题。

### 2.2 记录基线

在每轮测试开始前记录：

```powershell
git rev-parse HEAD
node --version
npm --version
rustc --version
cargo --version
$PSVersionTable.PSVersion
$host.Version
wt --version
```

同时手工记录 Windows build、显示器/分辨率/缩放、Windows Terminal profile、字体、字号、窗口尺寸、WebView2 Runtime、provider、随机端口、artifact 目录、binary 绝对路径和 commit SHA。命令不可用时记录 `unknown` 和原因，不得猜测。

### 2.3 构建和基础检查

在项目根目录执行：

```powershell
npm ci --prefix frontend
npm ci --prefix e2e
npm run build:app --prefix e2e
npm run build:app:webdriver --prefix e2e
npm run typecheck --prefix e2e
cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui
```

构建失败时不得开始 DPI 或 Terminal 结果判定；先保留构建日志并修复工具链、依赖或路径问题。

### 2.4 简繁转换 feature 的 Windows GUI 验证

该步骤必须使用独立的 feature binary，不能在默认 binary 上仅凭设置选择器或 `rules.yaml` 字段判定实际转换生效。它只验证 embedded provider；标准 W3C provider 仍只运行兼容性 smoke。

```powershell
npm run build:app:simplified-trad --prefix e2e
npm run test --prefix e2e -- --spec specs/simplified-trad-conversion.spec.ts
```

#### 操作与通过条件

1. 确认当前 checkout 的 commit SHA、binary 绝对路径和临时设置目录已记录。
2. 构建 feature binary；构建输出必须包含 `Additional Cargo features: simplified-trad-conversion`。
3. 运行 spec，确认设置窗口能选择 `s2t` 和 `t2s`，并且临时 `rules.yaml` 分别写入 `conversion: s2t`、`conversion: t2s`。
4. 确认真实 GUI 输出分别为：
   - `设计软件与打印` → `設計軟件與打印`；
   - `後設資料與說明` → `后设资料与说明`。
5. 结果必须是 2/2 passing；不能用浏览器 fallback、mock IPC 或仅前端单测替代。
6. 保存 `manifest.json`、`result.json`、WDIO/应用日志和失败时的 `settings-fixture`；成功后按第 6 节清理。

若 feature 构建失败，先记录 Cargo、MSVC、网络/缓存和 `opencc-fmmseg` 错误，不得将默认构建的占位输出记为 feature 验证结果。

## 2.5 2026-09-03 当前必须执行的 Windows 收尾流程

以下流程用于验证当前修复提交，而不是复用旧 binary 或旧 artifact 的结论。当前工作区的 Linux/WSL 定向结果不能替代 Windows WebView2、NTFS ACL、Windows 进程清理和 Windows Terminal 验证。必须在**同一次 Windows 原生 checkout**中按顺序执行，并在结果中记录 commit SHA、binary 绝对路径和每一步的退出状态。

### 2.5.1 初始化本轮运行目录

在 PowerShell 7 中打开项目根目录。建议使用短路径 checkout，例如 `E:\CopyPolish`；不要在 WSL、Git Bash 或无桌面服务会话中执行 GUI/TUI 项目。

```powershell
$ErrorActionPreference = 'Stop'
$root = (Get-Location).Path
$runId = Get-Date -Format 'yyyyMMdd-HHmmss'
$artifactRoot = Join-Path $env:TEMP ("copypolish-e2e-" + $runId)
$settingsRoot = Join-Path $env:TEMP ("copypolish-settings-" + $runId)
New-Item -ItemType Directory -Path $artifactRoot, $settingsRoot | Out-Null

Set-Location $root
git status --short
git rev-parse HEAD
node --version
npm --version
rustc --version
cargo --version
$PSVersionTable.PSVersion
wt --version

$env:COPYPOLISH_E2E_ARTIFACT_DIR = $artifactRoot
$env:COPYPOLISH_E2E_SETTINGS_DIR = $settingsRoot
```

同时记录 Windows build、WebView2 Runtime、Windows Terminal profile、字体、字号、显示器分辨率/缩放、窗口尺寸和当前 provider。若某个版本命令不可用，记录 `unknown` 及原因，不得用其他平台版本代替。

### 2.5.2 关闭旧进程并安装依赖

开始构建前关闭旧的 CopyPolish、WDIO、Node 和 TUI 进程。不要强制结束不属于本轮测试的用户进程；如无法区分，先记录进程列表并停止测试。

```powershell
Get-Process | Where-Object {
  $_.ProcessName -match 'chinese-copywriting-formatter|copypolish|wdio|node|copypolish-tui'
} | Select-Object Id, ProcessName, Path

npm ci --prefix frontend
npm ci --prefix e2e
npm run typecheck --prefix e2e
```

`npm ci`、typecheck 或后续构建失败时，应先保留 PowerShell 输出，不得继续用不完整的 binary 执行 GUI 结论。

### 2.5.3 构建并运行默认 embedded GUI 完整回归

默认 embedded 是当前 GUI 完整回归主路线。必须先构建当前 checkout 的 binary，再串行执行 spec；不要直接运行 `target` 中上一次构建留下的 binary。

```powershell
npm run build:app --prefix e2e
npm run test --prefix e2e -- --spec specs/selection-and-persistence.spec.ts
```

通过条件：

1. `selection-and-persistence.spec.ts` 全部 **3/3 passing**；
2. 第三个真实 GUI case 中，自定义替换设置保存后，输入 `TODO` 输出 `待办`；
3. 默认构建同一 case 确认 capability=false、T2S/S2T 禁用并归一化为 `conversion: none`；真实简繁输出只在第 2.5.4 节的 feature binary 中验收；
4. 设置状态、格式化请求和结果没有错误；
5. 若失败，artifact 中必须能看到 `lastFormatRequest`、`lastFormatResult`、`lastSettingsSave`、`inputValue`、`outputText` 和 replacement/conversion 字段。

若最后一次请求中的 `replacements` 为空，优先判断为 GUI 输入事件、设置保存时序或 binary/bundle 不匹配问题；不得直接把失败归因于 Rust replacement 管线。若 artifact 的 `finished` 为 `0`，即使 `exitCode` 为 `0`，也只能记录为 runner 未实际完成，不能记为通过。

### 2.5.4 构建并运行简繁转换 feature GUI

该步骤必须在默认 embedded 完整回归之后执行。构建顺序不可调换：先生成带 feature 的 binary，再运行 spec。

```powershell
npm run build:app:simplified-trad --prefix e2e
npm run test --prefix e2e -- --spec specs/simplified-trad-conversion.spec.ts
```

通过条件为 **2/2 passing**，且真实 GUI 输出满足：

```text
设计软件与打印  ->  設計軟件與打印   # s2t
後設資料與說明  ->  后设资料与说明   # t2s
```

必须在构建日志中确认 `Additional Cargo features: simplified-trad-conversion`。如果没有该标记，或运行的是默认 binary，不得记录为 feature 验证结果。

### 2.5.5 运行标准 W3C provider 兼容性 smoke

标准 W3C provider 只运行兼容性 smoke，不运行 replacement/简繁 feature 的完整回归。

```powershell
npm run build:app:webdriver --prefix e2e
npm run test:webdriver --prefix e2e
```

通过条件：`specs/w3c/smoke.spec.ts` 完成 session 创建、主窗口发现、一次真实格式化、一次设置保存、正常退出和清理。应记录 WebDriver 端口；端口默认可能为 `4445`，以本轮 `wdio.webdriver.conf.ts` 和 artifact 为准。

### 2.5.6 重新确认 Windows 专属 Rust/TUI 编译与测试

这一步必须在 Windows 原生 MSVC toolchain 上执行，用于确认新增 `#[cfg(unix)]` 后 Windows 测试目标可以编译。Linux/WSL 的 Rust 结果不能替代它。

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --features tui
```

通过条件：命令退出码为 `0`，没有编译错误、平台条件错误或未使用导入 warning。记录完整测试计数、Rust toolchain、MSVC host 和 commit SHA；旧的 `158/158` 结果只能作为历史记录，不能直接充当当前修复提交的新鲜 Windows 证据。

如需重新确认 release binary 可启动，再执行：

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui
```

真实 Windows Terminal raw-mode、粘贴、OSC 52、保存/重启和 WT-TUI-001/002/003 已有用户确认结果，不属于本轮默认必重跑项；只有代码、Terminal、字体或 profile 发生变化时才按第 4 节重新留证。

### 2.5.7 可选专项诊断

以下入口不是替换链路通过的前置条件，但在 Windows 复验失败、变更涉及对应模块或需要刷新证据时执行：

```powershell
# 设置重启恢复
npm run test:restart-settings --prefix e2e

# 三种损坏设置 fixture
npm run test:corrupt-settings --prefix e2e

# NTFS ACL 拒写（必须在 Windows 原生 NTFS 上执行）
npm run test:acl-settings --prefix e2e

# 失败 artifact 完整性 probe
npm run test:artifact-probe --prefix e2e

# GUI 主题/窄窗口 artifact；不是 DPI 自动矩阵
npm run test:gui-visual-artifacts --prefix e2e

# 设置控制台与 React act warning
npm run test:settings-shortcut-console --prefix e2e

# 非交互 TUI transcript
npm run test:tui-transcript --prefix e2e
```

ACL 步骤只能使用 `icacls.exe` 的 NTFS deny ACE。禁止使用 Linux `chmod`、WSL 权限映射或文件只读属性模拟。无论测试成功或失败，都必须确认 deny ACE 已移除、继承已恢复并且临时目录可以删除。

### 2.5.8 结果确认与清理

每个必需项目完成后，先复制或压缩失败 artifact 到 Windows 本地审计位置，再执行清理。成功结果只记录摘要；原始 artifact 不提交仓库。

```powershell
# 检查进程、端口、artifact 和仓库污染
Get-Process | Where-Object {
  $_.ProcessName -match 'chinese-copywriting-formatter|copypolish|wdio|node|copypolish-tui'
} | Select-Object Id, ProcessName, Path
Get-NetTCPConnection -State Listen | Where-Object {
  $_.LocalPort -eq 4445 -or $_.LocalPort -ge 44000
} | Select-Object LocalAddress, LocalPort, OwningProcess
Get-ChildItem $artifactRoot -Recurse -ErrorAction SilentlyContinue
Get-ChildItem $settingsRoot -Recurse -ErrorAction SilentlyContinue
Get-ChildItem . -Force -Filter 'rules.yaml*'
git status --short

# 完成本轮结果记录后清理本地生成物；--deep 会同时删除 node_modules
python scripts/clean.py --deep
```

成功清理的判定：没有 CopyPolish/WDIO/测试 Node 残留进程，没有测试端口残留，ACL deny 已移除，临时设置目录和仓库根目录下的 `rules.yaml*` 已清除，`git status --short` 不显示由测试生成的文件。失败时保留至少一个完整 artifact 副本后再清理其他生成目录。

### 2.5.9 本轮结果记录模板

```text
日期：2026-09-03
Commit：
Windows / build：
Node / npm：
Rust / MSVC：
WebView2：
Windows Terminal / PowerShell：
默认 embedded build：通过 / 失败
selection-and-persistence：3/3 / 失败 / 未执行
simplified-trad build：通过 / 失败
simplified-trad-conversion：2/2 / 失败 / 未执行
W3C smoke：通过 / 失败 / 未执行
Windows cargo test --features tui：通过 / 失败 / 未执行
可选专项：
Artifact 根目录：
失败 diagnostics JSON：
首个 IPC / 总耗时：
进程、端口、ACL 和临时目录清理：通过 / 失败
维护者结论：
```

## 3. GUI 三档 DPI artifact（自动验证跳过）

### 3.1 测试矩阵（历史参考，不执行）

每个缩放档至少执行以下组合：

| 维度 | 要求 |
| --- | --- |
| Windows 缩放 | 100%、125%、150% |
| 主题 | 浅色、深色；跟随系统可作为补充 |
| 窗口 | 默认/正常窗口、至少一个窄窗口 |
| provider | embedded（标准 W3C provider 已于 2026-09-01 收敛为兼容性 smoke，不再并行跑完整回归） |
| 设置目录 | 每次使用全新的临时目录 |
| 应用状态 | 冷启动，不能沿用上一次进程或旧窗口状态 |

三档 DPI 必须分别重启应用。不能只拖动窗口或修改浏览器缩放后声称完成 Windows DPI 验证。

### 3.2 执行步骤（已跳过，仅供未来重新启用时参考）

1. Checkout 待验证 commit，确认工作区和 binary 来源正确。
2. 创建本轮独立 artifact 根目录：

   ```powershell
   $artifactRoot = Join-Path $env:TEMP ("copypolish-dpi-" + (Get-Date -Format 'yyyyMMdd-HHmmss'))
   New-Item -ItemType Directory -Path $artifactRoot | Out-Null
   ```

3. 在 Windows 设置中选择 100% 缩放。
4. 关闭已有 CopyPolish、WDIO 和 Node 测试进程。
5. 为该组合设置新的 `COPYPOLISH_E2E_ARTIFACT_DIR` 和 `COPYPOLISH_E2E_SETTINGS_DIR`。
6. 执行 GUI 视觉 artifact 入口：

   ```powershell
    npm run test:gui-visual-artifacts --prefix e2e
   ```

7. 对浅色、深色和窄窗口分别保存 screenshot、page source、metadata 和 provider 日志。
8. 完成本档后关闭应用，确认进程和临时端口清理，再切换到 125%。
9. 重复步骤 3–8，依次完成 125% 和 150%。
10. 将三档结果汇总到人工审计表，不覆盖原始 artifact。

如果入口还不能把 DPI、窗口矩形或主题写入 metadata，应补充 runner/脚本并重新执行该档，不得用人工笔记替代缺失的原始证据。

### 3.3 每档检查与预期成果（历史人工结果；PR #24 需重新确认）

历史人工结果检查主窗口无白屏/黑屏；输入输出框、设置标题、说明、checkbox、按钮和底部操作区无重叠；窄窗口可访问全部设置；设置文件默认显示 `rules.yaml`，`title`/`aria-label` 保留完整路径；主题、字体、字号、缩放、规则状态保存后保持一致；至少完成一次真实格式化；关闭后无残留进程或监听端口。PR #24 的主题三项间距、路径悬停/复制、简繁转换间距、规则示例提示和键盘交互必须按 §1.2.1 使用当前 binary 重新确认。

每个 `<scale>-<theme>-<window>-<provider>` 目录至少包含：

```text
manifest.json
result.json
logs/
screenshots/
wdio/
settings-fixture/       # 失败时必须保留
<state>.html             # page source
<state>.json             # 窗口、主题、DPI 和断言 metadata
```

三档人工 GUI 验证已完成；GUI DPI 自动 artifact 按项目决定跳过，不执行，也不作为阻塞门禁。

## 4. Windows Terminal TUI 交互 artifact

### 4.1 目标和边界

该任务留证的是**真实 Windows Terminal 交互**，不是把已有 transcript 重命名为交互测试。非交互 transcript 继续覆盖确定性的 stdin/stdout/error 场景；本文补充终端输入模式、剪贴板协议和退出清理。

推荐“自动化记录 + 人工确认”分层：键序列、文本输入、退出码、stdout/stderr 和设置变化尽量自动保存；Windows Terminal 外观、raw-mode 是否真实生效、OSC 52 是否被策略拦截保留人工确认和截图；不把依赖焦点、剪贴板策略或终端版本的 flaky 行为直接纳入阻断式门禁。

### 4.2 前置准备

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui
$settingsDir = Join-Path $env:TEMP ("copypolish-tui-" + [guid]::NewGuid())
$artifactDir = Join-Path $env:TEMP ("copypolish-tui-artifact-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $settingsDir, $artifactDir | Out-Null
$env:COPYPOLISH_E2E_SETTINGS_DIR = $settingsDir
& .\src-tauri\target\release\copypolish-tui.exe
```

测试必须在 Windows Terminal + PowerShell 7 中进行，并记录 profile、字体、字号、窗口行列数和版本信息。

### 4.3 Windows Terminal 多行显示修复复验（WT-TUI-001/002/003）

本节只记录**必须在 Windows 原生桌面环境执行**的多行显示复验。与之对应的 Rust/UI 回归（`src-tauri/src/tui/wrap.rs`、`cargo test --features tui`）可在任意平台先行通过，但只有按本节的证据在 Windows Terminal 复验通过后，才能关闭任一 WT-TUI 项。

#### 复验前置

1. 使用 Windows Terminal + PowerShell 7，窗口至少 100×30 字符；记录 profile、字体、字号、窗口行列数。
2. 构建本次修复后的 binary：

   ```powershell
   cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui
   ```

3. 对每个 WT-TUI 项分别记录修复前后的截图和按键序列；先复现旧问题（可在旧 commit 上各保存一张问题截图），再在修复 binary 上复验。

#### WT-TUI-001：额外行绘制/状态栏重叠

1. 在输入区输入超过输入框单行可显示宽度的一段连续字符（无空格），使其自动换行。
2. 换行后继续输入若干字符。
3. 预期：新增字符应显示在输入框的下一视觉行，且始终位于输入框裁剪区域内；底部状态栏不被任何输入字符覆盖；状态栏文字与输入法候选栏不重叠。
4. 通过条件：视口高度不足时输入区会垂直滚动而不是溢入状态栏；截图与 region 检查均符合。

#### WT-TUI-002：光标不可见

1. 在自动换行的第二行继续输入，或使用 Left/Right/Home/End 在跨行文本上移动。
2. 预期：光标始终显示在当前逻辑插入位置对应的**视觉坐标**；跨行、自动换行后的滚动重算后光标仍在输入框内部可见。
3. 通过条件：对 ASCII、中文、混杂文本的每一段跨行移动，光标列都与实际渲染位置对齐且可见。

#### WT-TUI-003：emoji 显示（需区分应用与环境）

先做环境基线对照，不能混为一谈：

1. 在同一 Windows Terminal + PowerShell 7 中，分别输入/输出以下内容并观察：
   - `Write-Output '😀 中文 a 👨‍👩‍👧‍👦'`（PowerShell 直接输出）；
   - 一个最小 Rust 程序直接输出同一文本；
   - CopyPolish TUI 输入并格式化同一文本；
   - 分别测试单个 emoji、emoji 与 ASCII 混排、emoji 与中文混排、ZWJ 家庭 emoji；
   - 打开与关闭输入法候选窗口两种状态。
2. 判定分支：
   - 若 PowerShell 与最小 Rust 程序也不显示 → 属于 Windows Terminal 字体回退/profile 配置或终端版本限制，不作为 CopyPolish 缺陷；记录为环境限制并保存证据。
   - 若 PowerShell 能显示而 TUI 不能显示 → 继续检查应用侧：`unicode-width` 宽度判定、Ratatui buffer 写入、Crossterm Windows 事件解码、宽字符 continuation cell、IME/粘贴产生的字符。属于产品问题时再进入代码修复。
3. 通过条件：TUI 内 emoji 完整显示、按完整 grapheme 移动/删除、不产生半个代理字符、占位错位或残片；若为环境限制，则明确记录为“环境策略阻止”，不能记为产品通过。

#### 复验证据与关闭条件

- 每个 WT-TUI 项保存修复目标、修复前后截图（仅本地 artifact，测试后清理，不入库）、按键序列、环境信息和操作记录。
- WT-TUI-001/002/003 已由用户确认在同一 Windows Terminal 环境中复验通过，交互 artifact 已完成；GitLab Windows stage 已跳过，不参与后续门禁。

### 4.4 交互步骤与预期

#### A. raw-mode 和普通输入

1. 启动 TUI，确认终端进入 raw-mode，界面完成绘制。
2. 在输入区逐个输入裸 `r`、`q`、`?`，确认字符进入正文而不是误触规则、退出或帮助。
3. 使用方向键、Home、End、Delete 在 ASCII、中文、emoji、组合字符和多行文本上移动和编辑，特别确认最后一个 grapheme 的 Right/Delete 边界。

#### B. 规则面板和快捷键

1. 使用 `Ctrl+R` 打开规则面板，确认默认启用规则在默认关闭规则之前且同组顺序稳定。
2. 使用 Space、`a`、`d`、`n` 切换规则，使用 Esc 返回正文。
3. 使用 `Ctrl+?` 打开帮助，确认说明与实际行为一致；使用 `Ctrl+Q` 退出并确认终端恢复普通输入。

#### C. 粘贴和 Unicode

```powershell
Set-Clipboard -Value "第一行 r q ?`n第二行 中文🙂é"
```

在 TUI 中粘贴后确认文本完整插入，没有丢失字符、截断换行或误触快捷键；继续编辑和格式化，记录 bracketed paste 处理结果。

#### D. 复制、OSC 52 和降级

测试输出区/规则区导航及复制，用 `Get-Clipboard` 检查结果；记录 OSC 52 成功、被禁用或不可用时的状态栏提示。若不可用，确认提示指导用户使用 `--stdin/--output`。

#### E. 保存、重启和清理

使用 `Ctrl+S` 保存规则和最近输入，记录 `rules.yaml`、备份文件和 metadata；正常退出后用同一临时目录重启，确认规则和最近输入恢复；最后检查无残留进程、监听端口、未恢复终端模式或未预期控制序列。

### 4.5 交互 artifact 与完成判定

每次运行至少保存：

```text
manifest.json
result.json
environment.json
input-sequence.txt
stdout.txt
stderr.txt
terminal-transcript.txt
screenshots/
settings-before/
settings-after/
cleanup.json
```

`input-sequence.txt` 记录键名和阶段；敏感剪贴板内容使用摘要，不默认写入原文。`cleanup.json` 记录进程、端口、设置目录和终端恢复结果。失败时保留全部原始日志和 fixture。

用户已确认 raw-mode、裸字符/Ctrl 快捷键、规则面板、Unicode grapheme、bracketed paste、复制/OSC 52、保存/重启和终端清理均通过，WT-TUI-001/002/003 也已完成复验；本项现标记为通过。后续如需重新审计，仍应按本节要求保留连续运行记录。

## 5. GitLab Windows 可选 E2E stage（跳过，不执行）

> 状态：项目决定不配置、不运行此可选 stage；本节仅保留历史设计参考，不计作测试通过或后续待办。

### 5.1 目标与 runner 条件

GitLab 当前 Windows job 仅负责 release build。此前曾计划增加可选 E2E stage；项目现已决定跳过（不执行），以下内容只保留为历史设计参考。

Runner 必须具备交互式桌面、WebView2、Node、Rust MSVC、Visual Studio Build Tools、PowerShell 7、Windows Terminal、可写工作目录、`icacls.exe` 和隔离临时目录能力；并发不能共享固定端口或设置目录。

### 5.2 推荐流程

1. 仅在手动、夜间或指定变量触发时运行。
2. 检查工具链版本并安装依赖。
3. 构建 embedded 和标准 W3C provider binary。
4. 执行 typecheck、GUI smoke、设置恢复、损坏 fixture、ACL 和 GUI artifact。
5. 在有 Terminal 条件时执行 TUI artifact；没有交互桌面则明确记录 skipped。
6. 汇总 manifest、result、日志、截图、page source、fixture 和稳定性摘要。
7. 无论测试退出码如何都上传 artifact。
8. 在 `after_script` 或等价步骤清理进程、端口、ACL 和临时目录。
9. 统计最近运行的通过率、失败类别、耗时和 artifact 完整率。

### 5.3 配置要求示意

以下只是设计要求，不代表当前 `.gitlab-ci.yml` 已接入：

```yaml
e2e:windows:
  stage: e2e-windows
  tags:
    - saas-windows-medium-amd64
  allow_failure: true
  script:
    - powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\ci\run_windows_e2e.ps1
  after_script:
    - powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\ci\cleanup_windows_e2e.ps1
  artifacts:
    when: always
    expire_in: 7 days
    paths:
      - e2e/artifacts/windows/
```

接入前必须确认 Windows shell、artifact path、取消时 `after_script` 和交互式桌面权限的实际行为；不能凭 YAML 静态检查宣称 job 已可运行。

### 5.4 稳定性统计与升级门槛

每轮记录 commit、runner、Windows/WebView2/Terminal 版本、provider、spec 通过/失败/跳过数、启动/session/首个 IPC/总耗时、重试、端口冲突、进程残留、ACL 恢复、artifact 上传和失败分类。

建议连续观察至少 10 次 Windows E2E 运行：初期保持 `allow_failure: true`；artifact 完整率稳定且没有未解释的环境型 flaky 后，再评估调整门禁。功能回归失败仍需阻止对应发布判断，不能用“允许失败”隐藏。

## 6. 失败诊断、清理和结果模板

失败时必须保留 `manifest.json`、`result.json`、WDIO/应用/PowerShell 日志、截图、page source、终端 transcript、设置前后快照、版本信息、输入摘要、退出码、进程和端口信息。

```powershell
Get-Process | Where-Object { $_.ProcessName -match 'chinese-copywriting-formatter|copypolish|wdio|node' }
Get-NetTCPConnection -State Listen | Where-Object { $_.LocalPort -eq 4445 -or $_.LocalPort -ge 44000 }
Get-ChildItem .\e2e\artifacts -Recurse
Get-ChildItem . -Force -Filter 'rules.yaml*'
git status --short
```

ACL 测试必须确认 deny ACE 已移除、继承已恢复、临时目录可删除；Terminal 测试必须确认 raw-mode 已退出、终端可正常输入命令、没有残留控制序列。

结果记录模板：

```text
日期：
Commit：
Windows / build：
Node / npm：
Rust / MSVC：
WebView2：
Windows Terminal / PowerShell：
Provider：
测试矩阵或交互阶段：
通过 / 失败 / 跳过：
Artifact 路径：
首个 IPC / 总耗时：
重试、端口和残留进程：
失败分类与复现次数：
清理结果：
维护者结论：
```

## 7. 相关文档

- [e2e-development.md](e2e-development.md)：整体 E2E 架构、provider 和跨平台边界；
- [testing.md](testing.md)：测试层次、已有结果和 Windows 验收索引；
- [roadmap.md](roadmap.md)：尚未完成的 Windows 计划和完成门槛；
- [archive/decisions/e2e-provider-selection.md](archive/decisions/e2e-provider-selection.md)：embedded 与标准 W3C provider 选型（已归档决策）；
- [release/manual-release.md](release/manual-release.md)：Windows 构建、打包和发布验收。

## 8. 2026-09-01 自动化执行快照

本轮在 E 盘 Windows 原生 checkout 执行了新增自动化入口：

- `npm run typecheck --prefix e2e`：通过；
- `npm run test:settings-shortcut-console --prefix e2e`：embedded 1/1 通过；标准 W3C provider 已收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`），不再单独跑设置控制台 spec。两个 provider 均无 React `act` warning。当前 EdgeDriver 将 `Ctrl+,` 的原生事件报告为 `code=","`，因此 artifact 明确记录 `webdriverCodeFallback=true`、`uiButtonFallback=true`；设置窗口和控制台告警链路已自动核验，但这两个结果不等价于硬件级快捷键注入已独立通过。
- GUI DPI 自动验证：项目决定跳过（不执行）；此前采集到的 200% 环境 artifact 仅作诊断记录，三档人工 GUI 验证保留为完成结果。
- GUI DPI 自动矩阵脚本（`test:gui-dpi` / `test:gui-dpi-matrix`）已随项目决策移除：DPI 采用发布前人工检查；`test:gui-visual-artifacts` 保留并继续记录 `dpi-environment.json`。
- `npm run test:tui-terminal-artifact:prepare --prefix e2e`：通过，生成环境 manifest、手动清单和 `manualConfirmationRequired=true`；早期普通命令会话缺 `WT_SESSION` 时完整入口按设计拒绝。用户后续在真实 Windows Terminal 交互窗口完成 raw-mode/Terminal 外观/OSC 52 等按第 4 节的确认并通过。

GUI DPI 和 GitLab Windows stage 已跳过；Windows Terminal TUI 交互 artifact 已由用户在真实 Windows Terminal 中确认通过，审计原始 artifact 保留在 Windows 原生测试机（`e2e/artifacts/` 被 `.gitignore` 忽略），可作为后续审计补充，但不再视为功能或复验未完成。

## 9. 2026-09-01 复验结果

本轮复验结论：ACL 失败 fixture 留证路径已修复并在 embedded provider 中通过；GUI DPI 自动验证按项目决定跳过；此前无 `WT_SESSION` 的命令会话尝试已被用户后续真实 Windows Terminal 通过结果取代，完整 TUI artifact 和 WT-TUI-001/002/003 现标记通过。标准 W3C provider 已于 2026-09-01 收敛为兼容性 smoke（`specs/w3c/smoke.spec.ts`），不再与 embedded provider 并行跑完整回归。

## 10. Windows Terminal TUI artifact 通过记录（2026-09-01）

依据用户确认，完整 Windows Terminal TUI artifact 流程已完成并通过：raw-mode、裸字符与 Ctrl 快捷键、规则面板、跨行编辑、Unicode/emoji grapheme、bracketed paste、复制/OSC 52、保存与重启恢复、正常退出及终端清理均已核验。WT-TUI-001/002/003 已按修复版本复验通过。该结论为用户确认状态；如需审计，请将实际 artifact 目录和截图链接补入本节。

## 11. 2026-09-03 当前 Windows 复验结果

本轮在 E 盘 Windows 原生 checkout 串行执行，WebView2 为 152.0.4191.53。结果如下：

- `npm run typecheck --prefix e2e`：通过；前端单测 70/70 通过；
- 默认 embedded `selection-and-persistence.spec.ts`：3/3 通过；日志 `C:\Users\Shiraishi\AppData\Local\Temp\copypolish-selection-current-20260903.log`；
- 历史 `test:restart-settings` 曾记录 write/read 2/2，但该结果不覆盖 2026-09-04 的 capability 复验；当前 spec 已修正，新的 write/read 结果待重新执行；
- `test:corrupt-settings`：三个 fixture 3/3 通过（串行重跑）；
- `test:acl-settings`：1/1 通过（串行重跑）；
- `test:gui-visual-artifacts`：1/1 通过，生成 DPI 200% 诊断 artifact；
- `test:settings-shortcut-console`：1/1 通过；
- `test:tui-transcript`：4/4 通过，artifact 位于 `e2e/artifacts/tui-transcript/1788400328493`；
- Windows MSVC `cargo test --manifest-path src-tauri/Cargo.toml --features tui`：166 passed，0 failed；
- `simplified-trad-conversion.spec.ts`：2/2 通过（s2t、t2s）；本轮日志 `C:\Users\Shiraishi\AppData\Local\Temp\copypolish-feature-r2-20260903.log`；
- `test:webdriver`：2/2 通过；本轮标准 provider 在端口 63433 建立 session、完成格式化和设置保存。

GUI DPI 自动矩阵仍按项目决定跳过；125%/150% 人工 GUI 验证保持已完成；GitLab Windows 可选 E2E stage 跳过（不执行）；Windows Terminal 交互 artifact 保持用户确认通过。所有 runner 均按串行方式执行，未将 `exitCode=0` 且 `finished=0` 的 artifact 计入结果。除默认构建的 `test:restart-settings` 新 spec 复验外，本节列出的 Windows 收尾自动化项目均已有通过证据；restart 的历史 2/2 结果不作为当前 capability 修复的通过依据。

历史记录：修复曾先在 Linux/WSL 定向回归；本轮 Windows 已使用当前 binary 完成 embedded selection 3/3、简繁 feature 2/2 和 W3C smoke 2/2，详见本节结果。

## 12. 2026-09-03 提交 6687c13 capability 刷新结果

本轮使用隔离的 Windows 原生 checkout `E:\CopyPolish-6687`，对应提交 `6687c1390c633385cfd02135cf3072f4d18f94a9`；原有 `E:\Shiraishi\VSCode Workspace\chinese_copywriting_formatter` 的旧 `dev` checkout 未修改。Node 为 24.19.0，Rust 为 1.98.0，host 为 `x86_64-pc-windows-msvc`。

- `npm ci --prefix frontend`：退出码 0；
- `npm ci --prefix e2e`：退出码 0；
- `npm run typecheck --prefix e2e`：退出码 0；
- 默认 embedded：`selection-and-persistence.spec.ts` **3/3 passing**，验证默认构建 `buildCapabilities.simplifiedTradConversion=false`、T2S/S2T 禁用、保存归一化为 `conversion: none`；binary 为 `E:\CopyPolish-6687\src-tauri\target\debug\chinese-copywriting-formatter.exe`；artifact `result.json` 为 `exitCode=0`、`finished=1`、`passed=1`、`failed=0`；
- 简繁 feature embedded：先执行 `npm run build:app:simplified-trad --prefix e2e`，日志确认 `Additional Cargo features: simplified-trad-conversion`，随后 `simplified-trad-conversion.spec.ts` **2/2 passing**；capability=true、s2t/t2s 保存与真实 Rust IPC 输出均通过；
- Windows MSVC `cargo test --manifest-path src-tauri/Cargo.toml --features tui`：**167 passed，0 failed**，退出码 0；
- W3C smoke：先执行 `npm run build:app:webdriver --prefix e2e`，随后 `npm run test:webdriver --prefix e2e`；随机端口 **51737**，标准 provider **2/2 passing**，`result.json` 为 `exitCode=0`、`finished=1`、`passed=1`、`failed=0`；artifact 位于隔离 checkout 的 `e2e/artifacts/webdriver/1788419572347-smoke/`。WDIO session 报告 Edge 152.0.0.0；PowerShell 查询到的 Edge 安装版本为 153.0.4234.13，版本差异保留在本轮环境事实中，不影响 smoke 结果。

本轮未将 npm deprecated/allow-scripts 警告计为失败；所有 runner 均有明确完成数，未出现 `exitCode=0` 且 `finished=0`。隔离 checkout、artifact、临时设置目录和构建生成物已在验证后清理。当前提交的 Windows 默认 embedded、简繁 feature、MSVC TUI 和 W3C smoke 收尾证据均已刷新。
## 13. 2026-09-04 当前 checkout 复验结果

本轮先将 WSL checkout 的源文件与文档同步到 `E:\Shiraishi\VSCode Workspace\chinese_copywriting_formatter`，再在该 Windows 原生 checkout 以 PowerShell 7 串行复验。所有 runner 均检查实际完成数，不把空 artifact 计为通过。

- `npm run typecheck --prefix e2e`：通过（退出码 0）；
- `npm test --prefix frontend`：101/101 通过；
- 默认 embedded 构建与 `selection-and-persistence.spec.ts`：3/3 通过；
- `npm run build:app:simplified-trad --prefix e2e` 与 `simplified-trad-conversion.spec.ts`：2/2 通过（s2t、t2s）；
- `npm run build:app:webdriver --prefix e2e` 与 `test:webdriver`：2/2 通过；
- Windows MSVC `cargo test --manifest-path src-tauri/Cargo.toml --features tui`：182 passed、0 failed；
- `test:corrupt-settings`：三个 fixture 3/3 通过（`primary-corrupt-backup-valid`、`primary-corrupt-no-backup`、`primary-and-backup-corrupt`）；
- `test:acl-settings`：1/1 通过；`test:gui-visual-artifacts`：1/1 通过；`test:settings-shortcut-console`：1/1 通过；
- 构建 `copypolish-tui.exe` 后运行 `test:tui-transcript`：4/4 通过，artifact `e2e/artifacts/tui-transcript/1788520976024`；
- GUI DPI 自动矩阵继续按项目决定跳过；125%/150% 人工 GUI 验证和 Windows Terminal 交互 artifact 保持已完成；GitLab Windows 可选 E2E stage 跳过（不执行）。

本轮 `test:restart-settings` 未通过：默认构建的 `buildCapabilities.simplifiedTradConversion=false` 会禁用 T2S/S2T 并将设置归一化为 `conversion: none`，旧版 `e2e/specs/restart-settings.spec.ts` 却强制注入并等待 `t2s`，在 `restart-settings.spec.ts:53` 报“第一次启动的简繁转换选择未更新”。spec 已按 capability 分支修正，默认构建将验证禁用/归一化，feature binary 将验证 `t2s` 恢复；需重新执行后才能更新通过结论。

本轮日志保存在 `C:\Users\Shiraishi\AppData\Local\Temp\copypolish-*-20260904.log`；WSL 与 E 盘的非生成文件已再次核对为 241/241、无独有文件、无 hash 差异。
## 14. 2026-09-04 修正 spec 后最终复验

WSL→E 盘同步后，针对重启 spec 的 selector 修正重新执行 Windows 原生验证。修正内容仅限测试选择器：排除 `rule-card-*` 容器，避免把无 `data-state` 的卡片误当 checkbox。

- 前端单测：101/101；E2E typecheck：通过；
- 默认 embedded selection：3/3；默认 capability=false 的重启 write/read：2/2，通过 T2S/S2T 禁用、`conversion: none` 归一化与替换/最近输入恢复；
- 简繁 feature 的重启 write/read：2/2，通过 `t2s` 保存/恢复；feature 转换 spec：2/2（s2t、t2s）；
- 损坏设置：3/3；NTFS ACL：1/1；GUI 视觉 artifact：1/1；设置快捷键控制台：1/1；
- Windows MSVC `cargo test --manifest-path src-tauri/Cargo.toml --features tui`：主库 182 passed、properties 5 passed、readme_registry 3 passed，0 failed；
- 构建 TUI release binary 后 `test:tui-transcript`：4/4，artifact `e2e/artifacts/tui-transcript/1788528444537`；
- W3C smoke：首次串行执行出现默认格式化 case 的一次瞬时失败，随后单独重跑端口 61227 为 2/2 通过，最终以重跑结果为准；
- GUI DPI 自动矩阵和 GitLab Windows 可选 stage 继续跳过；125%/150% 人工 GUI 与 Windows Terminal 交互 artifact 保持已完成。

本轮失败已全部闭环：重启失败来自旧 selector，W3C 首次失败未能复现。WSL 与 E 盘非生成文件清单保持 241/241，当前测试 runner 已退出。