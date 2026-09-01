# Windows 原生 E2E 与交互留证 Runbook

本文记录必须在 **Windows 原生桌面环境**执行的剩余 E2E 工作：Windows Terminal TUI 交互 artifact（GUI DPI 自动验证已跳过）。GitLab Windows 可选 E2E stage 已决定跳过（不执行），仅保留历史设计参考。

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

### 1.2 仍需完成的 Windows 专项

| 工作项 | 目标 | 当前状态 | 完成标志 |
| --- | --- | --- | --- |
| GUI DPI artifact | 在 100%/125%/150% Windows 缩放下保存可审计截图、page source 和环境信息 | 跳过（不执行自动验证）；三档人工 GUI 验证已完成 | 不纳入自动化门禁；保留人工结果 |
| Terminal 多行显示修复 | 修复自动换行后新增字符绘制位置、光标不可见和 emoji 显示（WT-TUI-001/002/003） | 源码修复已完成，Windows MSVC 158/158 通过；真实 Terminal 复验已由用户确认通过 | Rust/UI 回归通过，Windows 原生复验通过 |
| Terminal TUI artifact | 将真实 Windows Terminal 交互和退出清理固化为输入/输出/截图/日志留证 | 已通过（依据用户确认） | 关键场景、清理和结果均已确认 |
| GitLab Windows stage | 在 Windows runner 上重复运行 E2E 并始终上传 artifact | 跳过（不执行） | 项目决定不配置、不运行；不得记为通过 |

**状态更新**：用户已确认 WT-TUI-001/002/003 修复后完成 Windows 原生复验，Terminal 交互 artifact 标记为通过。GitLab Windows E2E stage 已决定跳过（不执行）。

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
   npm run test:gui-visual-artifacts:webdriver --prefix e2e
   ```

7. 对浅色、深色和窄窗口分别保存 screenshot、page source、metadata 和 provider 日志。
8. 完成本档后关闭应用，确认进程和临时端口清理，再切换到 125%。
9. 重复步骤 3–8，依次完成 125% 和 150%。
10. 将三档结果汇总到人工审计表，不覆盖原始 artifact。

如果入口还不能把 DPI、窗口矩形或主题写入 metadata，应补充 runner/脚本并重新执行该档，不得用人工笔记替代缺失的原始证据。

### 3.3 每档检查与预期成果（人工结果已完成，自动验证不执行）

检查主窗口无白屏/黑屏；输入输出框、设置标题、说明、checkbox、按钮和底部操作区无重叠；窄窗口可访问全部设置；长 Windows 路径中间省略但保留 `rules.yaml`；`title`/`aria-label` 保留完整路径；主题、字体、字号、缩放、规则状态保存后保持一致；至少完成一次真实格式化；关闭后无残留进程或监听端口。

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

1. 在自动换行的第二行继续输入，或使用 Left/Right<project-root> 在跨行文本上移动。
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

本轮在 Windows native environment Windows 原生 checkout 执行了新增自动化入口：

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
