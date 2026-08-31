# CopyPolish 测试指南

## 1. 测试层次

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
| 设置 | Rust Windows 测试 16/16；Windows 真实 GUI 修复后已手动完成保存、重启恢复、损坏 fixture、ACL 保存失败及视觉/DPI/窄窗口回归；损坏设置和重启恢复已在两个 provider 自动化通过；ACL 自动化 spec 已实现 | embedded/W3C Windows E2E 已复验真实 WebView2、IPC、全不选恒等、临时路径和规则保存；ACL spec 需在 Windows 原生执行，当前 Linux/WSL 仅显式跳过 |
| 前端状态 | 防抖、竞态、错误、主题、字体、快捷键 | 真实 IPC E2E |
| TUI | CLI、编辑器、规则、OSC 52、共享设置；Linux 非交互 smoke；Windows release、stdin 及修复后 Windows Terminal 手动回归 | Rust TUI 148/148、Windows release/stdin 和 Windows Terminal 修复后手动回归已通过；TUI-EDIT-DELETE-001 已修复，仍需将故障场景固化为自动化 artifact |
| 发布脚本 | 主要由脚本和人工 Runbook 覆盖 | 参数和失败路径自动化测试 |

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

当前 mock 测试不能完全替代真实桌面验证。Linux/WSLg 与 Windows WebView2 最小链路、修复后 Windows GUI/TUI/设置/ACL 手动回归及双 provider 稳定性验证均已完成；TUI-EDIT-DELETE-001 已通过编辑器边界修复和回归测试关闭。损坏设置三种 fixture 和重启恢复已在 embedded/W3C provider 中自动化通过；NTFS ACL 自动化和 Terminal artifact 固化仍是后续工程工作。

TUI 非交互链路已在 Linux 上完成自动化 smoke：验证 `--help`、stdin 格式化、文件输入/输出、
`--rules none` 恒等、未知规则 key 警告、缺失文件返回码 1，以及约 1.29 MB 输入的恒等处理。
这些检查不替代真实 raw-mode 终端、Windows Terminal 交互、GUI 故障注入、ACL 保存失败链路或 Tauri 窗口行为 E2E；本次修复后的 TUI/GUI 回归和双 provider 连续稳定性已由人工完成，但自动化故障注入与 Terminal artifact 固化仍待补齐。

## 7. Windows 原生回归清单（已完成）

以下项目必须在 Windows 原生、可交互的 Windows Terminal + PowerShell 7 会话中执行。WSL、Linux GUI、普通浏览器预览和旧 binary 不能替代本清单。

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
- [ ] NTFS ACL 拒绝写入后保存失败提示正确（Windows 原生 `test:acl-settings` / `test:acl-settings:webdriver`）；
- [x] ACL harness 在 `finally` 中恢复，临时目录可删除；
- [ ] Windows ACL 失败时保留 stdout/stderr、WDIO log、manifest、exit status、截图、page source 和设置 fixture；
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

当前 Linux/WSLg 验证结果：embedded provider 3/3、标准 W3C provider 3/3 通过。该入口覆盖跨平台文件损坏语义，但不替代 Windows NTFS ACL 自动化。

### 7.8 设置重启恢复自动化入口

使用以下入口在同一临时 `rules.yaml` 目录中连续启动两次应用：第一次保存“全不选”和最近输入，第二次验证规则、输入和真实 Rust IPC 输出恢复。

```bash
npm run test:restart-settings --prefix e2e
npm run test:restart-settings:webdriver --prefix e2e
```

当前 Linux/WSLg 验证结果：embedded provider 的 write/read 阶段各 1/1 通过，标准 W3C provider 的 write/read 阶段各 1/1 通过。该入口验证跨平台设置恢复语义，仍需在 Windows 原生环境复验，并不覆盖 NTFS ACL 拒写。

### 7.9 NTFS ACL 保存失败自动化入口

该入口仅允许在 Windows 原生环境运行，使用 `icacls.exe` 为当前用户添加目录级 `(OI)(CI)(W)` deny ACE；不使用 Linux `chmod`、WSL 权限映射或只读属性模拟 NTFS ACL。runner 会先写入有效 `rules.yaml`，再启动 provider，最后在 `finally` 中移除 deny、恢复继承并删除临时目录。

```powershell
npm run test:acl-settings --prefix e2e
npm run test:acl-settings:webdriver --prefix e2e
```

spec 验证设置窗口显示保存失败、错误文本包含 `rules.yaml`，以及保存失败后应用仍能通过真实 Rust IPC 工作。非 Windows 环境会明确输出跳过并返回成功；这不代表 ACL 场景已通过。失败时 runner 会在恢复权限前复制设置 fixture 到 provider artifact 目录。

### 7.10 双 provider 稳定性统计（已完成）

在同一 commit、同一环境下，两个 provider 各连续运行至少 5 次，记录：

- 每次运行的 spec、通过/失败状态和耗时；
- binary 启动、WebView/session 创建和首个 IPC 的耗时；
- 随机端口、端口冲突和重试情况；
- CopyPolish、WDIO、Node 残留进程；
- artifact 是否完整；
- flaky 失败的复现次数和诊断结论。

当前版本两个 provider 的连续稳定性统计已完成并记录；损坏设置 fixture 和重启恢复自动化也已完成。NTFS ACL 自动化 spec 已实现但尚未在 Windows 原生执行，仍需完成 ACL 真实验证、artifact 收集和 GitLab stage 固化后，才可作为阻塞式合并门禁。

## 8. 测试完成标准

- 没有新增未解释的 warning；
- `git diff --check` 通过；
- Markdown 链接检查通过；
- 密钥扫描通过；
- 涉及规则、设置、Tauri 或发布时已完成相应额外验证。