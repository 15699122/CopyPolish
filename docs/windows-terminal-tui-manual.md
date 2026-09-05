# Windows Terminal 交互式 TUI 手动验证

## 当前专项状态

| 项目 | 状态 | 说明 |
| --- | --- | --- |
| 125% 真实 DPI GUI | 手动测试完成 | 已由人工在真实 Windows 显示缩放下完成 |
| 150% 真实 DPI GUI | 手动测试完成 | 已由人工在真实 Windows 显示缩放下完成 |
| GitLab Windows 可选 E2E stage | 跳过（不执行） | 项目决定不配置、不运行；不得记为通过 |
| Windows Terminal 交互式 TUI | 手动测试完成（通过） | 用户已确认按下述流程完成并通过；artifact 目录待补充实际路径 |

## 1. 前置条件

1. 使用 Windows Terminal 1.24 或更新版本，并打开 PowerShell 7 配置文件。
2. 从 Windows 原生环境 Windows 原生 checkout 执行，不要在 `<wsl-path>` 路径中构建。
3. 确认工具链：`node --version` 为 24.19.0，`rustc --version` 为 1.98.0 MSVC。
4. 进入项目：

   ```powershell
   Set-Location -LiteralPath 'C:\src\CopyPolish'
   ```

5. 构建 TUI：

   ```powershell
   cargo build --manifest-path .\src-tauri\Cargo.toml --features tui --release --bin copypolish-tui
   ```

6. 新建证据目录：

   ```powershell
   $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
   $evidence = Join-Path (Get-Location) "e2e\artifacts\windows-terminal-tui-manual\$stamp"
   New-Item -ItemType Directory -Path $evidence | Out-Null
   $PSVersionTable | Out-File (Join-Path $evidence 'powershell-version.txt') -Encoding utf8
   wt.exe --version | Out-File (Join-Path $evidence 'windows-terminal-version.txt') -Encoding utf8
   rustc --version --verbose | Out-File (Join-Path $evidence 'rust-version.txt') -Encoding utf8
   ```

## 2. 启动与基础输入

1. 将 Windows Terminal 窗口调整为至少 100×30 字符。
2. 启动程序：

   ```powershell
   .\src-tauri\target\release\copypolish-tui.exe
   ```

3. 确认界面完整、边框未错位、光标可见，且启动后没有残留 ANSI/转义字符。
4. 输入 `在LeanCloud上，花了5000元！`，执行格式化。
5. 期望输出为 `在 LeanCloud 上，花了 5000 元！`，中英文、数字和标点显示宽度正常。

## 3. 光标移动与 Delete 回归

1. 清空输入框，逐字输入 `sajgvwfwe`，确认屏幕显示 9 个字符。
2. 按 `Home`；若终端配置不发送 Home，则连续按左方向键直到光标位于 `s` 前。
3. 按一次 `Delete`，期望 `s` 被删除，内容变为 `ajgvwfwe`。
4. 使用右方向键移动至字符串中间，例如 `v` 与 `w` 之间；按一次 `Delete`，期望仅删除光标右侧字符。
5. 连续按 `End`；若 End 不生效，则连续按右方向键。期望光标能够位于最后一个 `e` 之后。
6. 按 `Backspace`，期望最后一个 `e` 被删除。
7. 再输入 `sajgvwfwe`，将光标移回开头，连续按 `Delete`，直到字符串完全为空。
8. 通过条件：最后一个字符可以删除，且光标可以移动至逻辑末尾。若最后的 `e` 无法删除或光标停在其前方，记录为该回归仍存在。
9. 再分别使用中文、emoji 和组合文本重复边界测试，例如 `中文A😀e`，确认按字符边界移动和删除，不出现半个代理字符或显示残片。

## 4. 编辑键与退出恢复

依次验证：

1. 左/右方向键逐字符移动。
2. `Home`、`End` 到达逻辑首尾。
3. `Backspace` 删除左侧字符，`Delete` 删除当前位置字符。
4. 在开头按 `Backspace`、在末尾按 `Delete` 均不崩溃、不移动到非法位置。
5. 粘贴多行文本后仍可移动、删除和格式化。
6. 使用程序定义的退出键正常退出，再用 `Ctrl+C` 中断一次。
7. 两种退出方式后，PowerShell 提示符、输入回显、方向键和光标均恢复正常；执行 `Write-Output 'terminal-restored'` 不应出现原始模式残留。

## 5. OSC52 剪贴板

1. 在 Windows Terminal 设置中确认允许应用写入剪贴板；记录测试时所用配置。
2. 先将剪贴板设为可辨识的基线文本：

   ```powershell
   Set-Clipboard -Value 'osc52-before'
   ```

3. 启动 TUI，输入并格式化一段唯一文本，例如 `OSC52测试20260901`。
4. 执行 TUI 的复制操作。
5. 退出后读取剪贴板：

   ```powershell
   Get-Clipboard | Tee-Object -FilePath (Join-Path $evidence 'clipboard-after.txt')
   ```

6. 通过条件：剪贴板内容与 TUI 显示的预期复制文本完全一致，屏幕上没有显示 OSC52 转义序列。如果 Windows Terminal 禁止 OSC52，应记为“环境策略阻止”，不能记为产品通过。

## 6. 证据与结果记录

1. 对启动界面、Delete 中间状态、字符串完全删除、格式化结果和退出后提示符分别截图，保存至 `$evidence`。
2. 创建 `$evidence\result.md`，记录：日期、Windows Terminal/PowerShell/Rust 版本、终端配置、每个步骤的通过/失败、实际与期望结果。
3. 对失败项目记录完整按键序列、输入文本、光标位置、是否可稳定复现以及截图文件名。
4. 最终状态只能是：
   - 全部步骤通过：`手动测试完成（通过）`；
   - 任一步骤失败：`手动测试完成（发现问题）`；
   - 因终端策略无法执行 OSC52：整体结果中单列 `OSC52 环境阻止`，不要推定通过。

## 7. 已确认问题（2026-09-01）

本轮 Windows Terminal 手动测试结果为 `手动测试完成（发现问题）`。问题证据截图仅本地留存（测试后已按约定清理，不入库、不上传远程），截图当时显示输入区内容已经换行，底部状态栏和输入法候选栏同时可见。记录以下三个独立问题：

### WT-TUI-001：多行输入后的新增字符绘制位置错误

- 前置条件：输入内容长度超过输入框单行可显示宽度并发生换行。
- 操作：继续输入字符。
- 实际结果：新输入字符未显示在输入框内的正确续行位置，而是绘制到窗口底部状态栏区域，与快捷键/状态文字重叠。
- 期望结果：新增字符应显示在输入框的下一视觉行，并保持在输入框裁剪区域内；底部状态栏不应被覆盖。
- 影响：用户无法可靠判断实际输入内容和插入位置，属于阻断多行编辑的布局缺陷。

### WT-TUI-002：多行输入时光标不可见

- 前置条件：输入内容超过一行。
- 操作：在第二行继续输入，或使用左右、Home、End 移动光标。
- 实际结果：输入光标不再显示。
- 期望结果：光标应始终显示在当前逻辑插入位置；换行、水平/垂直滚动和状态栏布局变化后均应重新计算并绘制。
- 影响：无法确定插入点，光标移动、Delete 和 Backspace 操作不可可靠验证。

### WT-TUI-003：emoji 无法显示

- 前置条件：Windows Terminal 使用支持 emoji 的字体回退设置。
- 操作：在输入区输入 emoji，例如 `😀`，并执行格式化及光标移动。
- 实际结果：emoji 未正确显示。
- 期望结果：emoji 应通过 Windows Terminal 的字体回退正常显示；宽度计算、光标移动和删除应以完整 grapheme cluster 为边界，不产生占位错位或残片。
- 影响：包含 emoji 的文案无法预览，同时可能连带破坏列宽和光标位置计算。

### 复验要求

修复后必须在同一 Windows Terminal 环境重新执行：长文本跨行输入、第二行继续输入、跨行 Home/End/左右移动、Delete/Backspace、中文与 emoji 混排，并保存修复前后截图。三个问题应分别关闭，不能以单行输入测试或非交互 transcript 代替。

## 8. Windows Terminal 交互 artifact 自动记录流程

以下流程只在真实 Windows Terminal + PowerShell 7 窗口执行；不要从普通 IDE 终端、服务会话或 WSL 直接执行完整入口。

### 8.1 准备与环境留证

1. 在 Windows Terminal 打开 PowerShell 7，确认 `$env:WT_SESSION` 非空，并记录：

   ```powershell
   Set-Location -LiteralPath 'C:\src\CopyPolish'
   $env:WT_SESSION
   wt.exe --version
   $PSVersionTable.PSVersion
   [Console]::WindowWidth
   [Console]::WindowHeight
   ```

2. 确认终端窗口至少 100×30 字符，记录 profile、字体、字号、编码和窗口尺寸。
3. 构建当前 TUI binary，并检查 E2E 脚本：

   ```powershell
   cargo build --manifest-path .\src-tauri\Cargo.toml --features tui --release --bin copypolish-tui
   npm ci --prefix e2e
   npm run typecheck --prefix e2e
   ```

4. 先生成 artifact 目录和手动清单：

   ```powershell
   npm run test:tui-terminal-artifact:prepare --prefix e2e
   ```

   记录命令输出的 `e2e\artifacts\windows-terminal-tui\<timestamp>` 路径，确认其中存在 `manifest.json`、`result.json` 和 `manual-checklist.json`。
5. 为本轮使用隔离设置目录：

   ```powershell
   $settingsDir = Join-Path $env:TEMP ('copypolish-tui-' + [guid]::NewGuid())
   New-Item -ItemType Directory -Path $settingsDir | Out-Null
   $env:COPYPOLISH_E2E_SETTINGS_DIR = $settingsDir
   ```

### 8.2 启动完整 artifact 记录

1. 仍在同一个 Windows Terminal 窗口执行：

   ```powershell
   npm run test:tui-terminal-artifact --prefix e2e
   ```

2. 入口会检查 `WT_SESSION`，记录 Windows Terminal/PowerShell/窗口信息、binary 路径和剪贴板前后 SHA-256，然后以继承的标准输入输出启动 `copypolish-tui.exe`。
3. 测试期间不要通过脚本注入键盘或伪造终端变量；所有按键、截图和实际结果由操作者记录。

### 8.3 按顺序执行手动用例

1. **raw-mode**：启动后确认界面、边框和光标正常；退出一次，确认 PowerShell 提示符和回显恢复。
2. **普通输入/快捷键**：输入裸 `r`、`q`、`?`，确认只进入正文；再验证 `Ctrl+R`、`Ctrl+?`、`Ctrl+Q` 的规则、帮助和退出行为。
3. **多行显示缺陷复验**：输入超过一行的连续字符并继续输入；分别检查 WT-TUI-001（字符是否落到底栏）、WT-TUI-002（光标是否消失）和 WT-TUI-003（emoji 是否显示）。每项保存截图和按键序列。
4. **grapheme 编辑**：输入 `sajgvwfwe`，执行 Home/Delete、中间 Delete、End、Backspace 和连续 Delete；确认最后的 `e` 可删除、光标可到末尾，不出现半个 emoji 或组合字符。
5. **bracketed paste**：在另一个 PowerShell 窗口执行：

   ```powershell
   Set-Clipboard -Value "第一行 r q ?`n第二行 中文🙂é"
   ```

   回到 TUI 粘贴，确认换行、中文、emoji 和组合字符完整插入，可继续编辑并格式化。
6. **规则面板**：确认默认启用规则排在关闭规则前，同组顺序稳定；使用 Space、`a`、`d`、`n` 切换并以 Esc 返回。
7. **OSC 52/复制**：复制唯一文本，退出后使用 `Get-Clipboard` 检查内容；记录成功或“环境策略阻止”，屏幕不得出现 OSC 52 转义序列。
8. **保存与重启**：使用 `Ctrl+S` 保存规则和最近输入，退出后用相同 `$settingsDir` 重启，确认规则和输入恢复。
9. **退出清理**：正常退出和 `Ctrl+C` 各执行一次，确认 raw-mode、光标、回显和提示符恢复。

### 8.4 artifact 完成与清理

1. 在 artifact 目录补充 `input-sequence.txt`、`terminal-transcript.txt`、`stdout.txt`、`stderr.txt`、截图和 `result.md`；剪贴板只保存 SHA-256 或非敏感摘要。
2. 将 `manual-checklist.json` 中每个 case 从 `pending` 更新为 `passed`、`failed` 或 `blocked`，并在 `result.md` 写明实际值、预期值和截图文件名。
3. 检查 `result.json`：完整交互运行应保留退出码、信号、剪贴板 hash 和 `manualConfirmationRequired=true`。
4. 清除临时设置目录并执行：

   ```powershell
   Get-Process | Where-Object { $_.ProcessName -match 'copypolish|wdio|node' }
   Get-NetTCPConnection -State Listen | Where-Object { $_.LocalPort -ge 44000 }
   Test-Path -LiteralPath $settingsDir
   Write-Output 'terminal-restored'
   ```

   预期无残留进程/监听端口，`Test-Path` 为 `False`，提示符和命令回显正常。
5. 只有所有用例均有截图、按键序列、环境记录和清理结果，并在同一环境连续 5 轮无未解释 flaky，才能将交互 artifact 标记为通过；任一 WT-TUI 问题失败时，整体标记为“手动测试完成（发现问题）”。

## 9. 通过状态确认（2026-09-01）

用户已确认 Windows Terminal TUI artifact 全流程通过，包括 raw-mode、跨行输入与光标、emoji、grapheme Delete/Backspace、规则面板、快捷键、bracketed paste、OSC 52、保存/重启和终端清理。WT-TUI-001/002/003 按修复版本完成复验并关闭。若需要审计留证，请补充实际 `e2e/artifacts/windows-terminal-tui/<timestamp>` 路径、截图和 `result.md`。
