# CopyPolish 测试指南

## 1. 测试层次

必须依赖 Windows 原生环境的剩余 E2E 计划、步骤、artifact 结构和 GitLab runner 要求集中见 [windows-e2e-runbook.md](windows-e2e-runbook.md)。本文继续作为测试层次、功能映射和已有验证结果的入口。

| 层次 | 位置 | 目的 |
| --- | --- | --- |
| Rust 单元/集成测试 | `src-tauri/src/**`、`src-tauri/tests/fixtures/` | 验证规则、管线、保护层、Unicode、设置和 TUI 状态 |
| 前端 hook 测试 | `frontend/src/hooks/*.test.ts` | 验证异步状态、竞态、防抖、持久化和窗口交互 |
| 前端组件测试 | `frontend/src/App.test.tsx` | 验证用户操作、设置窗口、快捷键和界面反馈 |
| 性能门禁 | `scripts/check_performance.py`、`src-tauri/examples/unicode_baseline.rs` | 捕获数量级性能回退，不替代 profiling |
| 桌面 smoke/E2E | 当前主要为人工验证 | 验证真实 Tauri IPC、窗口、设置和平台行为 |

## 2. 功能—测试映射

| 功能 | 现有覆盖 | 后续补强 |
| --- | --- | --- |
| 规则注册表 | 稳定 key、默认状态、legacy key、依赖图、alias 唯一性和迁移归一化 | 自动检查 README 与注册表一致性 |
| 格式化管线 | 规则选择、组合、换行、幂等性、未知 key | 属性测试和更大真实语料 |
| Markdown/HTML/LaTeX | span、嵌套结构、未闭合结构、后续文本不吞并、保护 fixture | 继续扩展真实文档样本 |
| Unicode | grapheme、emoji、组合符、CJK Ext-B | Unicode 数据/工具链升级回归 |
| 单位和数学 | 有限词典、复合单位、数学边界 | 按真实语料扩展词典 |
| 设置 | Rust Windows 测试 16/16；Windows 真实 GUI 修复后已手动完成保存、重启恢复、损坏 fixture、ACL 保存失败及视觉/DPI/窄窗口回归；损坏设置、重启恢复和 NTFS ACL 已在两个 provider 自动化通过；统一 artifact、受控失败 probe 和 GUI 主题/窄窗口 artifact 已实现 | embedded/W3C Windows E2E 已复验真实 WebView2、IPC、全不选恒等、临时路径、规则保存、ACL 拒写恢复和失败留证；三档人工 DPI 已完成，GUI DPI 自动验证已跳过（不执行）；三档人工 GUI 验证已完成；GitLab Windows stage 已跳过 |
| 前端状态 | 防抖、竞态、错误、主题、字体、快捷键 | 真实 IPC E2E |
| TUI | CLI、编辑器、规则、OSC 52、共享设置；Linux 非交互 smoke/transcript；Windows release、stdin 及修复后 Windows Terminal 手动回归 | Rust TUI 158/158、Windows release/stdin 和非交互 transcript 已通过；本轮多行显示源码修复已由用户确认在 Windows Terminal 复验通过；TUI-EDIT-DELETE-001 已修复，真实 Terminal raw-mode/OSC 52 交互 artifact 已通过 |
| 发布脚本 | 主要由脚本和人工 Runbook 覆盖 | 参数和失败路径自动化测试 |

### 2.1 Windows 原生验证快照

前端测试 57/57、Rust 设置测试 16/16、Rust/TUI 测试 158/158 均通过；embedded 与标准 W3C provider 的普通 WebView2/Rust IPC 用例各 3/3、设置重启 write/read 各 1/1、三种损坏设置 fixture 各 3/3、NTFS ACL 各 1/1 通过。统一 artifact、受控失败 probe、GUI 主题/窄窗口 artifact 和 TUI 非交互 transcript 已验证；Windows TUI release 构建和 `--stdin --no-config` smoke 通过，TUI-EDIT-DELETE-001 已关闭。

> **2026-09-01**：Windows Terminal 交互复验发现三个多行显示缺陷（WT-TUI-001 额外行绘制到状态栏、WT-TUI-002 光标不可见、WT-TUI-003 emoji 显示），证据见 [windows-terminal-tui-manual.md](windows-terminal-tui-manual.md)。已按与 ratatui 渲染等价的视觉换行重算光标与滚动（`src-tauri/src/tui/wrap.rs`），并新增 10 项 Rust/UI 回归；本轮 Windows MSVC 上 158/158 通过，真实 Windows Terminal 复验已由用户确认通过。

平台专用自动化与留证已按项目决策达到当前范围：GUI DPI 自动矩阵决定不执行、GitLab Windows stage 决定跳过（不执行）；Windows Terminal 交互 artifact 已由用户确认通过。详细执行步骤见 [Windows 原生 E2E 与交互留证 Runbook](windows-e2e-runbook.md)：

- [x] GUI 100%/125%/150% DPI 和窄窗口人工验证已完成；GUI DPI 自动截图/矩阵验证按项目决定跳过（不执行、不纳入门禁）；
- [x] Windows Terminal TUI raw-mode、规则面板、粘贴、OSC 52、保存/退出/重启恢复的真实交互 artifact 已通过（用户确认）；非交互 transcript 仍作为补充证据；
- [x] embedded/W3C 受控失败时完整诊断包自检：stdout/stderr、WDIO log、manifest、exit status、截图、page source 和设置 fixture 均已验证；统一 artifact 基础设施已完成；
- [ ] 通过真实 Tauri 设置控制台 runner 检查 React 19 `act` warning；两个 provider 各 1/1、warning=0，但 EdgeDriver 使用 UI 回退，硬件快捷键仍需人工确认；
- [x] GitLab Windows 可选 E2E stage：跳过（不执行）；项目决定不配置、不运行，不计作测试通过；

非阻断告警：E2E 依赖审计报告 16 个已知漏洞（1 个中危、15 个高危）；Cargo 在 E 盘调试构建中曾报告增量缓存目录 `os error 5`，但相关编译、测试和 release 构建均以退出码 0 完成。

## 3. 常用命令

```bash
python3 scripts/verify.py --profile checks
python3 scripts/verify.py --profile frontend
python3 scripts/verify.py --profile rust
python3 scripts/verify.py --profile audit
```

直接运行前端测试：

```bash
npm test --prefix frontend -- --run
```

## 4. 新规则测试要求

每条规则至少应包含：

1. 单规则输入/输出；
2. 与相关规则的组合输出；
3. Markdown、LaTeX、URL、代码或化学式等保护场景；
4. 重复执行后的幂等性断言；
5. 默认开关、规则选择和稳定 key 验证；
6. 设置迁移、GUI 动态元数据和 TUI 兼容性检查（适用时）。

争议性规则必须明确默认关闭或开启的理由，并在 README 和 CHANGELOG 中说明用户可见影响。

## 5. Fixture 规范

- 一个 fixture 文件聚焦一个领域；
- 输入、规则选择和期望输出应清晰可读；
- 修复 bug 先添加最小回归用例；
- 不通过批量改写 fixture 隐藏行为变化；
- 同时关注 LF/CRLF、Unicode 边界和重复格式化；
- 设置读写测试使用系统临时目录，禁止写仓库内固定路径。

## 6. 桌面验证缺口

当前 mock 测试不能完全替代真实桌面验证。Linux/WSLg 与 Windows WebView2 最小链路、修复后 Windows GUI/TUI/设置/ACL 手动回归及双 provider 稳定性验证均已完成；TUI-EDIT-DELETE-001 已通过编辑器边界修复和回归测试关闭。损坏设置三种 fixture、重启恢复和 NTFS ACL 已在 embedded/W3C provider 中自动化通过；统一 artifact、受控失败诊断、GUI 主题/窄窗口截图和 TUI 非交互 transcript 已完成，GUI DPI 自动验证已按项目决定跳过；硬件级快捷键兼容性仍保留诊断说明；GitLab stage 已跳过。

TUI 非交互链路已在 Linux 上完成自动化 smoke：验证 `--help`、stdin 格式化、文件输入/输出、
`--rules none` 恒等、未知规则 key 警告、缺失文件返回码 1，以及约 1.29 MB 输入的恒等处理。
这些检查不替代真实 raw-mode 终端、Windows Terminal 交互或 Tauri 窗口行为 E2E；本次修复后的 TUI/GUI 回归和双 provider 连续稳定性已由人工完成，设置与 ACL 故障注入、非交互 transcript 和基础 GUI artifact 已自动化，Terminal 交互 artifact 已通过。

## 7. Windows 原生回归清单（已完成）

以下项目必须在 Windows 原生、可交互的 Windows Terminal + PowerShell 7 会话中执行。WSL、Linux GUI、普通浏览器预览和旧 binary 不能替代本清单。

### 7.0 Windows 原生专用步骤总览

以下顺序适用于一次完整的 Windows 验证。除特别标注外，命令均在项目根目录的 **PowerShell 7** 中执行；建议使用短路径 checkout，避免长路径影响 WebView2、artifact 和 ACL 排查。

#### 1. 确认环境并记录版本

至少记录以下信息：

```powershell
git rev-parse HEAD
$PSVersionTable.PSVersion
node --version
npm --version
rustc --version
rustup show active-toolchain
Get-ComputerInfo | Select-Object WindowsProductName, WindowsVersion, OsBuildNumber
Get-AppxPackage Microsoft.WindowsTerminal | Select-Object Name, Version
Get-Command msedgewebview2.exe -ErrorAction SilentlyContinue
```

正式 Windows 基线要求 Node 满足项目约束 `>=24 <25`，Rust host 为 `x86_64-pc-windows-msvc`，并安装 Visual Studio Build Tools、WebView2 Runtime、Windows Terminal 和 PowerShell 7。版本信息必须和测试结果一起保存。

#### 2. 安装依赖并构建测试 binary

```powershell
npm ci --prefix frontend
npm ci --prefix e2e
npm run build:app --prefix e2e
npm run build:app:webdriver --prefix e2e
npm run typecheck --prefix e2e
cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui
```

两个 GUI provider 都必须使用当前 commit 构建的 binary；不能复用旧 binary 或旧 `frontend/dist`。标准 W3C provider 运行前还必须完成 `build:app:webdriver`。

#### 3. 依次执行 GUI provider 测试

先执行 embedded provider，再执行标准 W3C provider：

```powershell
npm run test --prefix e2e
npm run test:webdriver --prefix e2e
```

每个 provider 都必须确认 WebView2/session 创建、主窗口发现、真实 Rust IPC、全不选恒等、临时 `rules.yaml`、设置保存、进程退出和目录清理成功。

#### 4. 执行设置恢复和损坏 fixture

```powershell
npm run test:restart-settings --prefix e2e
npm run test:restart-settings:webdriver --prefix e2e
npm run test:corrupt-settings --prefix e2e
npm run test:corrupt-settings:webdriver --prefix e2e
```

重启恢复入口会使用同一临时设置目录启动两次应用，验证规则选择、最近输入和真实 Rust IPC 输出恢复。损坏 fixture 入口覆盖主文件损坏且备份有效、无备份，以及主备份均损坏三种情况。

#### 5. 执行 NTFS ACL 保存失败测试

该步骤只能在 Windows 原生执行，不能使用 WSL `chmod`、Linux 权限映射或只读文件属性替代：

```powershell
npm run test:acl-settings --prefix e2e
npm run test:acl-settings:webdriver --prefix e2e
```

runner 会使用 `icacls.exe` 为当前用户添加目录级写入 deny ACE，验证设置保存失败提示、`rules.yaml` 目标路径、应用未崩溃以及真实 Rust IPC 仍可用。测试结束时必须自动移除 deny ACE、恢复继承并删除临时目录；失败时先保留设置 fixture 到 artifact。

#### 6. 执行 Windows Terminal/TUI 回归

```powershell
& .\src-tauri\target\release\copypolish-tui.exe
```

在 Windows Terminal + PowerShell 7 中验证 raw-mode、规则顺序、裸字符和 Ctrl 快捷键、粘贴/bracketed paste、格式化、复制/OSC 52、保存、退出、重启恢复以及终端状态清理。记录 Windows Terminal、PowerShell、窗口尺寸、字体、编码和 OSC 52 结果。

#### 7. 失败留证与清理

失败时保留对应 provider 的 `e2e/artifacts/`，至少包括 WDIO 日志、应用 stdout/stderr、manifest、退出状态、失败截图、page source、版本信息和临时设置 fixture。完成后确认：

```powershell
Get-Process | Where-Object { $_.ProcessName -match 'chinese-copywriting-formatter|wdio|node' }
Get-NetTCPConnection -State Listen | Where-Object { $_.LocalPort -eq 4445 -or $_.LocalPort -ge 44000 }
Get-ChildItem .\e2e\artifacts -Recurse
Get-ChildItem . -Force -Filter 'rules.yaml*'
git status --short
```

成功运行不得残留 CopyPolish、WDIO、Node 测试进程或监听端口，仓库根目录不得出现 `rules.yaml` / `rules.yaml.bak`。ACL 测试必须确认权限已恢复后才能结束。

#### Windows-only 验收边界

| 项目 | Windows 原生要求 | 当前 Linux/WSL 可做的工作 |
| --- | --- | --- |
| WebView2 双 provider | 真实 Windows WebView2/session 和进程清理 | 可验证 Linux/WSLg provider smoke |
| 设置重启恢复 | 使用 Windows binary 实际启动两次 | 可验证跨平台设置恢复语义 |
| 损坏设置 fixture | 可复验 Windows WebView2 提示 | 可验证文件损坏/备份恢复语义 |
| NTFS ACL | 必须使用 `icacls.exe` 真实 deny ACE | 只能 typecheck 或显式跳过，不能宣称通过 |
| Windows Terminal/TUI | 必须使用可交互 Windows Terminal + PowerShell 7 | 可运行 Linux 非交互 TUI smoke，不能替代 raw-mode/OSC 52 |
| DPI、窄窗口、剪贴板 | 必须在 Windows 桌面实际观察并留证 | 不能由普通浏览器或 Linux GUI 替代 |

### 7.1 环境与构建

- [x] 记录 commit、Windows 版本、Node/npm、Rust host、Visual Studio Build Tools、WebView2 Runtime、Windows Terminal 和 PowerShell 版本；
- [x] 确认 Node 满足项目约束，Rust host 为 `x86_64-pc-windows-msvc`；
- [x] 执行 `npm ci --prefix frontend`、`npm ci --prefix e2e`、前端构建和 E2E typecheck；
- [x] 构建 embedded provider 和标准 W3C provider 的测试 binary；
- [x] 构建 TUI release binary：

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui
```

### 7.2 双 provider 最小真实 smoke

对 embedded provider 和标准 W3C WebDriver provider 各执行一次，使用新的临时设置目录：

- [x] WebView2 加载实际打包前端；
- [x] session 创建和主窗口发现成功；
- [x] `在LeanCloud上，花了5000元` 通过真实 Rust IPC 得到预期结果；
- [x] 全不选时输出保持输入不变；
- [x] 设置路径指向临时 `rules.yaml`；
- [x] 保存成功、进程退出、临时目录清理成功；
- [x] 不产生仓库根目录的 `rules.yaml` 或 `rules.yaml.bak`。

### 7.3 GUI 样式和布局回归

在浅色/深色主题、100%/125%/150% Windows 缩放和正常/窄窗口尺寸下检查：

- [x] 输入框和输出框的边框、圆角、阴影一致；
- [x] “字体”和“快捷键”的标题与说明间距一致；
- [x] “恢复默认字体”和“恢复默认快捷键”视觉样式一致；
- [x] 长 Windows 路径使用中间省略，并保留 `rules.yaml` 文件名；
- [x] 路径 `title` 和 `aria-label` 仍包含完整路径；
- [x] 主题“跟随系统”、快捷键总开关和规则 checkbox 使用统一黑白样式；
- [x] 设置滚动区、底部操作区和长路径不会相互挤压或溢出；
- [x] 键盘焦点、Space 切换和 disabled 状态正常；
- [x] 保存、重启恢复和真实格式化仍正常。

每个主题和至少一个窄窗口尺寸保存截图、page source 和版本信息。

### 7.4 修复后的 Windows Terminal TUI 回归

在 Windows Terminal + PowerShell 7 中启动：

```powershell
& .\src-tauri\target\release\copypolish-tui.exe
```

必须重新验证：

- [x] raw mode 启动和退出后终端状态恢复；
- [x] 规则面板中默认启用规则全部位于默认关闭规则之前；
- [x] 同一默认状态内顺序稳定；
- [x] 输入区输入裸 `r`、`q`、`?` 时，字符进入正文，不打开规则、帮助或退出；
- [x] 输入区使用 `Ctrl+R`、`Ctrl+?`、`Ctrl+Q` 时，分别打开规则、帮助和退出；
- [x] 粘贴 `Get-Clipboard` 不截断，包含多个 `r`、`q`、`?`、中文和换行的文本保持完整；
- [x] bracketed paste 插入后可继续编辑并正常格式化；
- [x] 输出区、规则区的导航和复制行为仍正常；
- [x] `Ctrl+S` 保存规则和最近输入；
- [x] 重启后规则选择和最近输入恢复；
- [x] OSC 52 可用时复制成功，不可用时显示降级提示；
- [x] TUI 退出后无残留进程、终端控制序列或错误状态。

### 7.5 设置故障和清理回归

每个 provider 至少覆盖一次：

- [x] `rules.yaml` 损坏、`.bak` 有效（embedded/W3C 自动化通过）；
- [x] `rules.yaml` 损坏、`.bak` 缺失（embedded/W3C 自动化通过）；
- [x] `rules.yaml` 和 `.bak` 均损坏（embedded/W3C 自动化通过）；
- [x] NTFS ACL 拒绝写入后保存失败提示正确（Windows 原生 embedded/W3C 各 1/1 通过）；
- [x] ACL harness 在 `finally` 中恢复，临时目录可删除；
- [x] Windows ACL 失败时保留 stdout/stderr、WDIO log、manifest、exit status、截图、page source 和设置 fixture；
- [x] 成功后无 CopyPolish、WDIO、Node 测试残留进程和监听端口。

### 7.6 修复后手动核验记录（已完成）

以下项目不能由已有 mock、stdin smoke 或最小 WebView2 E2E 代替。本次已在 E 盘 checkout、Windows Terminal + PowerShell 7、可交互桌面和修复后的 binary 上完成；以下步骤同时作为复现和审计记录。

#### A. GUI 视觉、窗口和 DPI

1. 在 E 盘项目根目录执行 `npm ci --prefix frontend`、`npm ci --prefix e2e`、`npm run build:app --prefix e2e`。
2. 新建临时设置目录并启动 `src-tauri\\target\\debug\\chinese-copywriting-formatter.exe`：

   ```powershell
   $settingsDir = Join-Path $env:TEMP ("copypolish-manual-" + [guid]::NewGuid())
   New-Item -ItemType Directory -Path $settingsDir | Out-Null
   $env:COPYPOLISH_E2E_SETTINGS_DIR = $settingsDir
   & .\\src-tauri\\target\\debug\\chinese-copywriting-formatter.exe
   ```

3. 在 Windows 显示设置依次使用 100%、125%、150% 缩放；每个缩放值都重新启动应用并保存一张截图。
4. 在正常窗口和窄窗口检查：输入/输出框边框、圆角、阴影；设置标题和说明间距；“恢复默认字体/快捷键”按钮；主题、快捷键总开关和规则 checkbox 的黑白样式；设置滚动区与底部按钮是否溢出。
5. 在设置窗口确认长 Windows 路径使用中间省略但保留 `rules.yaml`，并通过路径的 `title`/`aria-label` 读取完整路径。
6. 切换浅色/深色主题，修改字体、字号、缩放和至少一条规则；确认保存状态为“设置已保存”，关闭并重新启动后状态和格式化结果仍正确。
7. 关闭应用，删除临时目录；若失败，保留截图、page source、stdout/stderr 和临时设置目录。

#### B. 设置恢复、损坏 fixture 和 NTFS ACL

1. 每个 case 使用新的临时目录，并设置 `$env:COPYPOLISH_E2E_SETTINGS_DIR`；不要操作用户真实设置。
2. 分别准备：
   - Case A：`rules.yaml` 非法，`rules.yaml.bak` 有效；
   - Case B：`rules.yaml` 非法，`.bak` 缺失；
   - Case C：`rules.yaml` 和 `.bak` 均非法。
3. 启动应用并确认 WebView2、主界面和真实格式化均可用；检查 `settings-load-notices`：A 显示从备份恢复，B/C 显示使用默认设置；保存后确认不会继续传播非法 YAML。
4. 关闭并再次启动同一临时目录，确认有效备份或默认规则仍能恢复；记录设置文件内容、提示文本和退出状态。
5. ACL 流程必须用 `try/finally`：

   ```powershell
   $settingsDir = Join-Path $env:TEMP ("copypolish-acl-" + [guid]::NewGuid())
   New-Item -ItemType Directory -Path $settingsDir | Out-Null
   $user = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
   try {
     Set-Content (Join-Path $settingsDir 'rules.yaml') "enabled: []`n" -Encoding utf8
     icacls $settingsDir /inheritance:r
     icacls $settingsDir /deny "${user}:(OI)(CI)(W)"
     $env:COPYPOLISH_E2E_SETTINGS_DIR = $settingsDir
     & .\\src-tauri\\target\\debug\\chinese-copywriting-formatter.exe
     # 在设置窗口修改规则并保存。
     # 必须看到 settings-status 的“设置保存失败”及 rules.yaml 路径。
   } finally {
     icacls $settingsDir /remove:d $user /T /C
     icacls $settingsDir /inheritance:e /T /C
     Remove-Item -LiteralPath $settingsDir -Recurse -Force
   }
   ```

6. ACL 拒写期间确认应用不崩溃、格式化仍可用；`finally` 后确认目录可删除、没有 CopyPolish/WDIO/Node 残留进程。

#### C. Windows Terminal TUI raw-mode、快捷键、粘贴和恢复

1. 在项目根目录执行 `cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui`，使用同一临时设置目录启动：

   ```powershell
   $settingsDir = Join-Path $env:TEMP ("copypolish-tui-" + [guid]::NewGuid())
   New-Item -ItemType Directory -Path $settingsDir | Out-Null
   $env:COPYPOLISH_E2E_SETTINGS_DIR = $settingsDir
   & .\\src-tauri\\target\\release\\copypolish-tui.exe
   ```

2. 确认 raw mode 启动；在输入区逐个输入裸 `r`、`q`、`?`，确认字符进入正文而不是打开面板或退出。
3. 使用 `Ctrl+R` 打开规则面板，确认默认启用规则排在默认关闭规则之前且同组顺序稳定；用 Space、`a`、`d`、`n` 切换后以 Esc 关闭。
4. 使用 `Ctrl+?` 打开帮助，确认快捷键说明；使用 `Ctrl+Q` 退出，确认终端回到正常输入状态。
5. 用 PowerShell 准备含中文、换行以及多个 `r`/`q`/`?` 的剪贴板文本：`Set-Clipboard -Value "第一行 r q ?`n第二行 中文"`，在 TUI 中粘贴；确认完整插入、可继续编辑且格式化不截断。
6. 在输出区测试导航和复制；用 `Get-Clipboard` 验证复制结果，记录 OSC 52 成功或降级提示。
7. 用 `Ctrl+S` 保存规则和最近输入，退出后重新启动同一目录，确认规则选择和最近输入恢复。
8. 退出后检查没有残留进程、控制序列或损坏终端；保存终端截图、版本和操作记录。

TUI-EDIT-DELETE-001（2026-08-31）已修复：`TextEditor` 现在将最后一个 grapheme 的下一个边界正确处理为 `text.len()`，因此 Right 可以移动到文本末尾，Delete 可以删除最后一个 grapheme。Rust 编辑器和 TUI 事件级回归已覆盖 ASCII、中文、emoji、组合字符和多行边界。

#### D. 双 provider 连续稳定性和清理记录

1. 在同一 commit、同一环境下，embedded 与 W3C provider 各运行 5 次：分别执行 `npm run test --prefix e2e` 和 `npm run test:webdriver --prefix e2e`。
2. 每次记录 spec 通过数、总耗时、随机端口、首个 IPC 时间、失败重试和 artifact 路径。
3. 每轮结束执行 `Get-Process`、`Get-NetTCPConnection -State Listen`，确认没有 CopyPolish、WDIO、Node 残留；成功轮次可删除 artifact，失败轮次必须保留日志、截图、page source、manifest、退出码和 fixture。
4. 只有 5 次均可复现通过且无未解释 flaky，才可把 provider 标记为稳定；否则记录失败轮次和诊断，不纳入阻塞式门禁。
### 7.7 损坏设置自动化入口

普通 `npm run test` 和 `npm run test:webdriver` 不会自动执行需要外部 fixture 环境变量的损坏设置 spec；使用以下入口会依次执行三种 fixture：

```bash
npm run test:corrupt-settings --prefix e2e
npm run test:corrupt-settings:webdriver --prefix e2e
```

每个 fixture 都会在 provider 启动前写入独立临时目录，并验证恢复/降级提醒和真实 Rust IPC 格式化。未提供 `COPYPOLISH_E2E_SETTINGS_FIXTURE` 时，`corrupt-settings.spec.ts` 会显式跳过，避免污染普通 smoke。

当前 Linux/WSLg 与 Windows 原生验证结果：embedded provider 3/3、标准 W3C provider 3/3 通过。该入口覆盖跨平台文件损坏语义；NTFS ACL 由独立入口验证。

### 7.8 设置重启恢复自动化入口

使用以下入口在同一临时 `rules.yaml` 目录中连续启动两次应用：第一次保存“全不选”和最近输入，第二次验证规则、输入和真实 Rust IPC 输出恢复。

```bash
npm run test:restart-settings --prefix e2e
npm run test:restart-settings:webdriver --prefix e2e
```

当前 Linux/WSLg 与 Windows 原生验证结果：embedded provider 的 write/read 阶段各 1/1 通过，标准 W3C provider 的 write/read 阶段各 1/1 通过。该入口验证跨平台设置恢复语义；NTFS ACL 拒写由独立入口验证。

### 7.9 NTFS ACL 保存失败自动化入口

该入口仅允许在 Windows 原生环境运行，使用 `icacls.exe` 为当前用户添加目录级 `(OI)(CI)(W)` deny ACE；不使用 Linux `chmod`、WSL 权限映射或只读属性模拟 NTFS ACL。runner 会先写入有效 `rules.yaml`，再启动 provider，最后在 `finally` 中移除 deny、恢复继承并删除临时目录。

```powershell
npm run test:acl-settings --prefix e2e
npm run test:acl-settings:webdriver --prefix e2e
```

spec 验证设置窗口显示保存失败、错误文本包含 `rules.yaml`，以及保存失败后应用仍能通过真实 Rust IPC 工作。非 Windows 环境会明确输出跳过并返回成功；这不代表 ACL 场景已通过。失败时 runner 会在恢复权限前复制设置 fixture 到 provider artifact 目录。

2026-08-31 Windows 原生结果：embedded provider 1/1、标准 W3C provider 1/1 通过；两个 runner 均完成 deny ACE 注入、保存失败与真实 IPC 验证，并在 `finally` 中恢复 ACL、删除 fixture。

### 7.10 双 provider 稳定性统计（已完成）

在同一 commit、同一环境下，两个 provider 各连续运行至少 5 次，记录：

- 每次运行的 spec、通过/失败状态和耗时；
- binary 启动、WebView/session 创建和首个 IPC 的耗时；
- 随机端口、端口冲突和重试情况；
- CopyPolish、WDIO、Node 残留进程；
- artifact 是否完整；
- flaky 失败的复现次数和诊断结论。

当前版本两个 provider 的连续稳定性统计已完成并记录；损坏设置 fixture、重启恢复、NTFS ACL 自动化、统一 artifact 基础设施、受控失败 artifact 自检、主题/窄窗口 GUI artifact 和 TUI 非交互 transcript 已完成。React 19 warning 闭环和硬件级快捷键兼容性仍保留说明；Terminal 交互 artifact 已通过；GUI DPI 自动验证和 GitLab Windows stage 均已跳过。

### 7.11 TUI 非交互 transcript artifact

使用以下入口采集 TUI 非交互模式的输入、stdout、stderr、退出码、命令参数、白名单环境摘要和结果汇总：

```bash
npm run test:tui-transcript --prefix e2e
```

当前 Linux/WSL 验证结果为 4/4 通过，覆盖默认格式化、`--rules none` 恒等、未知规则 warning 和缺失输入文件错误。该入口可以在 Windows 原生复用，但不替代 Windows Terminal raw-mode、规则面板、剪贴板或 OSC 52 交互 artifact。

### 7.12 Windows 剩余计划入口

Windows 100%/125%/150% DPI 人工 GUI 验证已完成；GUI DPI 自动验证已跳过（不执行），不纳入自动矩阵；Windows Terminal 交互 artifact 已通过（用户确认）。GitLab Windows 可选 E2E stage 已跳过（不执行），不计作通过。请按 [windows-e2e-runbook.md](windows-e2e-runbook.md) 执行，并分别记录 DPI 三档的缩放/主题/窗口矩阵和原始 artifact、Terminal raw-mode/规则面板/快捷键/Unicode 粘贴/OSC 52/保存重启/终端清理。

当前 `.gitlab-ci.yml` 只有 Windows release build；可选 E2E stage 已决定跳过（不执行），Runbook 中的 YAML 仅保留为历史设计参考。

## 8. 测试完成标准

- 没有新增未解释的 warning；
- `git diff --check` 通过；
- Markdown 链接检查通过；
- 密钥扫描通过；
- 涉及规则、设置、Tauri 或发布时已完成相应额外验证。

### 7.13 2026-09-01 自动化补充结果

- E2E TypeScript 类型检查通过。
- 设置快捷键控制台 runner：embedded 与标准 WebDriver 各 1/1 通过，`actWarningCount=0`。由于当前 EdgeDriver 将逗号键上报为 `code=","`，两个 artifact 均记录原生键事件诊断并通过 UI “打开设置”回退完成界面/控制台验证；不得把它表述为硬件级 `Ctrl+,` 注入已独立通过。
- GUI DPI 自动验证已按项目决定跳过（不执行）；既有 200% artifact 仅作历史诊断记录，三档人工 GUI 验证保持完成。
- Windows Terminal TUI artifact 已由用户确认完整交互通过；`--prepare-only` 仍可用于生成 `manifest.json`、`result.json` 和 `manual-checklist.json`，实际交互结果以用户确认的 artifact 为准。

### 7.14 2026-09-01 复验记录

- ACL fixture 保留路径已修复：先解除 deny ACE，再复制 `settings-fixture`，最后删除临时目录；embedded/WebDriver 测试均通过，保留目录包含 `rules.yaml`，权限恢复和清理完成。
- GUI DPI 自动验证已决定跳过，不再切换 Windows 显示设置或重新执行目标矩阵。
- 早期普通命令会话运行完整 Windows Terminal TUI artifact 时因缺 `WT_SESSION` 按设计退出；`--prepare-only` 可生成手动清单。用户随后在真实 Windows Terminal 完成 WT-TUI-001/002/003 真实终端复验和完整交互 artifact，并确认通过。
