# 真实 Tauri GUI E2E 开发说明

本文记录 CopyPolish 真实 Tauri GUI E2E 的当前开发状态、环境边界、实施步骤、测试方法和验收标准。

本文专门区分“可以在当前 Linux/WSL 环境完成的开发工作”和“必须在 Windows 桌面环境完成的验证工作”。
**构建检查通过不等于真实 GUI E2E 通过**：只有在真实桌面会话中启动 Tauri 二进制并完成 WebView、IPC 和设置链路后，才可以将对应 smoke 标记为通过。

## 1. 当前状态

截至 2026-08-31：

- E2E 工具路线已确定为 WebdriverIO + `@wdio/tauri-service` + embedded provider；
- 当前仓库已提交最小 WebdriverIO E2E 工程和 `e2e` Cargo feature；
- 当前仓库已提交两个 E2E WebDriver plugin 的条件注册与 capability 隔离；
- 当前工作环境是 Linux WSL2，不是 Linux 原生桌面会话，也不是 Windows；
- 当前 Node 实际版本为 `v26.7.0`，超出项目 `.nvmrc` / `package.json` 要求的 `>=24 <25`，不能作为正式 E2E 基线；
- 默认 Tauri 配置仍使用现有 `src-tauri/tauri.conf.json` 和生产 capability；
- 当前 Linux/WSLg 已通过真实 GUI smoke：embedded WebDriver、WebView、真实 `format_text` IPC、全不选恒等和设置路径/保存链路均已验证；本次通过日期为 2026-08-30。
- 参考项目 `Choochmeque/tauri-plugin-webdriver` 的 `0.2.1` 版本已作为并行 provider 完成 Linux/WSLg PoC；标准 WebDriver 连接、真实 WebView、真实 IPC、全不选恒等和设置保存均已通过，本次复核日期为 2026-08-31。
- Windows 原生最小桌面链路及修复后的 GUI/TUI/设置/ACL/双 provider 回归均已完成；TUI-EDIT-DELETE-001 已通过编辑器边界修复、Rust 回归测试和 Windows 定向复验关闭。自动化专用故障注入 spec 和 CI artifact 收集仍未建立。

本环境已完成的前置确认：

1. Cargo 可以解析并编译 `tauri-plugin-wdio` 与 `tauri-plugin-wdio-webdriver` 的 1.3.0 版本；
2. WebdriverIO 9.31.x 与 Tauri service 1.3.0 可安装到独立测试 workspace；
3. 前端已经存在适合 E2E 的稳定选择器，例如：
   - `input-textarea`；
   - `output-text`；
   - `formatting-status`；
   - `open-settings`；
   - `settings-dialog`；
   - `settings-load-notices`；
   - `settings-status`；
   - `settings-path`；
   - `select-none`；
   - `settings-done`；
4. Rust 设置实现的当前主文件是 `rules.yaml`，备份文件是 `rules.yaml.bak`，旧版 JSON 文件只用于迁移；
5. 项目已有 Rust、前端和非交互 TUI 回归覆盖，但不能替代真实 Tauri GUI E2E；当前 Linux/WSLg smoke 已补充完成。
6. 当前 E2E 构建通过 `TAURI_CONFIG` 仅为测试 binary 叠加 `app.withGlobalTauri: true`，正式 `tauri.conf.json` 不改变；这是 `@wdio/tauri-plugin` 访问全局 Tauri API 所需的测试配置。
7. Tauri 前端资源使用 Vite `base: "./"`，避免打包后的 `index.html` 以 `/assets/...` 绝对路径加载资源而在 custom protocol 下白屏。
8. E2E Cargo 构建显式启用 `custom-protocol` feature；直接用 Cargo 构建 embedded binary 时不能依赖 Tauri CLI 自动注入该 feature。
9. `e2e/scripts/run-specs.ts` 会为每个 spec 启动独立 WDIO 进程并创建独立临时设置目录；单个进程内部仍将 `maxInstances`、`maxInstancesPerCapability` 和 capability 的 `wdio:maxInstances` 固定为 `1`，避免同一进程内共享状态。
10. 并行参考插件 provider 使用 `e2e-webdriver` feature、`wdio.webdriver.conf.ts` 和 `run-webdriver-specs.ts`；它直接连接应用内标准 WebDriver server，不使用 `@wdio/tauri-service`、`@wdio/tauri-plugin` 或 E2E capability。

## 2. 采用的技术路线

WebdriverIO 官方当前将 `@wdio/tauri-service` 作为 Tauri 桌面测试入口。embedded provider 将 WebDriver server 放在应用进程内运行，不需要单独的 `tauri-driver`；配置核心是 `appBinaryPath` 和 `driverProvider: "embedded"`。

当前官方路线涉及两个 Tauri plugin：

| Plugin | 用途 | embedded provider |
| --- | --- | --- |
| `tauri-plugin-wdio` | `browser.tauri.execute()`、IPC mock、前后端日志转发 | 需要，用于高级测试能力 |
| `tauri-plugin-wdio-webdriver` | 在应用内启动 embedded WebDriver server | 必需 |

官方参考：

- Tauri WebDriver：<https://v2.tauri.app/develop/tests/webdriver/>
- WebdriverIO Tauri quick start：<https://webdriver.io/docs/desktop-testing/tauri/quick-start>
- WebdriverIO Tauri plugin setup：<https://webdriver.io/docs/desktop-testing/tauri/plugin-setup>
- WebdriverIO Tauri configuration：<https://webdriver.io/docs/desktop-testing/tauri/configuration>

版本说明：官方文档和 npm crate 的版本发布节奏可能不同。实施前必须以当前 lockfile 和实际安装包的类型定义为准，不能只复制历史示例中的版本号。

## 3. 当前 Linux/WSL 环境可以完成的工作

### 3.1 可以完成：依赖和代码设计

可以在 Linux 环境完成以下工作：

1. 创建独立 `e2e/` Node workspace；
2. 安装并锁定 WebdriverIO、Mocha、Tauri service 和测试类型依赖；
3. 创建 WebdriverIO 配置文件；
4. 创建 E2E 测试目录、共享 helper 和报告目录；
5. 增加 Tauri `e2e` feature；
6. 将两个 Tauri plugin 设计为可选依赖；
7. 让 plugin 只在 E2E 构建中注册；
8. 为 E2E 设置目录设计隔离机制；
9. 增加稳定 `data-testid` / `data-rule-key` 选择器；
10. 编写测试脚本、文档和本地验证命令；
11. 运行 TypeScript 类型检查、Rust 编译检查和配置 JSON 校验。

### 3.2 可以完成：不启动窗口的构建检查

在有正确 Rust、Node 和 Tauri 系统依赖的 Linux 环境中，可以执行：

```bash
nvm use
npm ci --prefix frontend
npm ci --prefix e2e

npm run build --prefix frontend
cargo check --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml --features e2e
npm run typecheck --prefix e2e
```

这些命令可以验证：

- 依赖是否可以解析；
- 默认构建是否仍然不需要 E2E plugin；
- E2E feature 是否可以编译；
- E2E 配置是否可以通过 TypeScript 检查；
- 前端普通构建是否仍然成功。

它们不能验证窗口是否显示、WebView 是否加载、WebDriver 是否连接、快捷键是否工作或真实设置是否写入正确目录。

### 3.3 当前 WSL2 的限制

当前环境是 WSL2。除非 WSL 配置了可用的 WSLg/X11/Wayland 图形会话，否则不能启动并验证真实 GUI 窗口。

执行真实 Linux GUI smoke 前，先检查：

```bash
echo "DISPLAY=${DISPLAY:-}"
echo "WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-}"
```

如果两个变量都为空，或 Tauri/WebKitGTK 无法连接图形显示服务，则只能完成配置和构建检查，不能报告 Linux GUI smoke 通过。

## 4. 推荐的代码集成方式

### 4.1 Cargo feature

目标是生产构建不带测试 plugin：

```toml
[features]
default = []
e2e = ["dep:tauri-plugin-wdio", "dep:tauri-plugin-wdio-webdriver"]
```

两个依赖应声明为 optional：

```toml
[dependencies]
tauri-plugin-wdio = { version = "1", optional = true }
tauri-plugin-wdio-webdriver = { version = "1", optional = true }
```

### 4.2 Rust plugin 注册

在 Tauri Builder 中使用条件编译：

```rust
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(feature = "e2e")]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());

    builder
        .invoke_handler(tauri::generate_handler![/* existing commands */])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 4.3 Capability 隔离

E2E plugin 权限不能直接加入生产 capability。否则默认构建在未启用 plugin 时会报类似以下错误：

```text
Permission wdio:default not found
```

此外，Tauri 2 会处理 `src-tauri/capabilities/` 目录中的 capability 文件；把 E2E 专用 capability 文件直接放进该目录可能导致默认构建也扫描到 E2E 权限。

因此实现时必须使用以下方案之一，并在默认构建上验证：

1. 使用测试专用 Tauri 配置，将 E2E capability 对象内联到配置中；或
2. 使用隔离的测试配置目录，并确保生产配置不会扫描该目录；或
3. 使用项目当前 Tauri 版本明确支持的配置覆盖机制。

不要在没有验证的情况下把 `wdio:default` 或 `wdio-webdriver:default` 放进生产 `default.json`。

### 4.4 前端 plugin 初始化

需要高级 Tauri E2E 能力时，前端测试构建应加载：

```ts
import "@wdio/tauri-plugin";
```

如果使用动态导入，必须保证：

- 普通浏览器预览仍可构建；
- 只有测试构建初始化 plugin；
- 动态导入不会改变正式生产 bundle 行为；
- 构建时 bundler 能解析该 package。

应在当前项目 Node 版本和 Vite 版本下进行实际构建确认。

## 5. WebdriverIO 配置目标

建议使用独立配置：

```text
e2e/wdio.conf.ts
```

配置至少应包含：

```ts
export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./specs/**/*.spec.ts"],
  maxInstances: 1,
  framework: "mocha",
  services: [
    [
      "@wdio/tauri-service",
      {
        appBinaryPath: "/absolute/path/to/e2e-binary",
        driverProvider: "embedded",
        embeddedPort: 4445,
        captureBackendLogs: true,
        captureFrontendLogs: true,
      },
    ],
  ],
  capabilities: [
    {
      browserName: "tauri",
      "tauri:options": {
        application: "/absolute/path/to/e2e-binary",
      },
    },
  ],
};
```

实际配置必须以安装版本的 `@wdio/tauri-service` 类型定义为准。当前 v9 类型检查表明 `capabilities` 应使用 `WebdriverIO.Config`，而非裸的 `Options.Testrunner`。

建议启用：

- `maxInstances: 1`：第一阶段避免多个 Tauri 实例共享设置状态；
- 前端日志捕获；
- 后端日志捕获；
- 单独的 `artifacts/logs/`；
- 单独的 `artifacts/screenshots/`；
- 失败后截图；
- 30–60 秒启动和命令超时；
- 可通过 `TAURI_WEBDRIVER_PORT` 覆盖 embedded port。

## 6. 测试设置隔离

项目当前桌面设置文件为：

```text
rules.yaml
rules.yaml.bak
```

旧版：

```text
ccw-formatter-settings.json
```

旧版 JSON 只用于迁移测试，不应作为当前主设置文件名。

### 6.1 推荐隔离方案

E2E 构建提供只在测试 feature 中生效的环境变量，例如：

```text
COPYPOLISH_E2E_SETTINGS_DIR=/tmp/copypolish-e2e-<unique-id>
```

设置目录解析优先级：

```text
COPYPOLISH_E2E_SETTINGS_DIR
    ↓
正式程序目录
```

测试生命周期：

```text
创建唯一临时目录
    ↓
写入初始 rules.yaml / rules.yaml.bak
    ↓
设置环境变量
    ↓
启动 E2E binary
    ↓
操作真实 GUI
    ↓
读取临时目录并断言
    ↓
关闭应用
    ↓
检查仓库根目录没有 rules.yaml
    ↓
删除临时目录
```

如果当前 Tauri service 的进程环境传递机制不能保证该变量在应用启动前生效，应改为复制测试 binary 到临时 staging 目录，或在测试专用启动脚本中设置环境变量后再启动 WebdriverIO。

### 6.2 不允许的做法

- 不得写入仓库根目录；
- 不得写入开发者真实便携版目录；
- 不得复用上一次测试的设置目录；
- 不得依赖测试执行顺序；
- 不得并行共享同一个 `rules.yaml`；
- 不得用浏览器 `localStorage` 测试代替真实桌面文件持久化。

## 7. 测试用例和方法

### 7.1 启动与默认排版

输入：

```text
在LeanCloud上，花了5000元
```

期望输出：

```text
在 LeanCloud 上，花了 5000 元
```

步骤：

1. 启动真实 Tauri binary；
2. 等待 `[data-testid="input-textarea"]` 显示；
3. 输入示例；
4. 等待 `[data-testid="formatting-status"]` 消失或输出稳定；
5. 读取 `[data-testid="output-text"]`；
6. 断言输出；
7. 断言没有错误提示；
8. 关闭应用并确认进程退出。

这条用例必须走真实 WebView 和真实 `format_text` IPC，不能 mock `format_text`。

### 7.2 全不选恒等

步骤：

1. 打开 `[data-testid="open-settings"]`；
2. 点击 `[data-testid="select-none"]`；
3. 等待设置保存状态稳定；
4. 点击 `[data-testid="settings-done"]`；
5. 输入包含可排版内容的文本；
6. 断言输出与输入完全相同；
7. 关闭并重启应用；
8. 再次确认空启用集仍表示全不选。

### 7.3 规则切换

优先使用稳定 key 选择器，例如：

```text
[data-testid="rule-spacing.cjk-latin"]
```

或者新增：

```text
[data-rule-key="spacing.cjk-latin"]
```

不要依赖中文展示名作为机器定位方式。测试应验证单条规则切换确实影响真实 Rust 输出。

### 7.4 快捷键

至少验证：

- `CtrlOrCmd+Enter`：立即排版；
- `CtrlOrCmd+Shift+KeyC`：复制结果；
- `CtrlOrCmd+Comma`：打开设置；
- 关闭快捷键总开关后，上述快捷键不再执行对应动作。

该用例还用于观察 React 19 + Radix Dialog 的 `act` warning 是否只存在于 jsdom 测试环境。

### 7.5 设置保存与重启恢复

验证：

- 规则选择保存到临时 `rules.yaml`；
- 主题、字体、字号和 UI 缩放保存；
- 最近输入保存；
- 关闭应用后再次启动可以恢复；
- 设置窗口显示的路径指向临时目录；
- 仓库根目录未生成设置文件。

### 7.6 损坏设置

至少准备两个场景：

#### 有效备份

```text
rules.yaml       非法 YAML
rules.yaml.bak   有效 YAML
```

期望：

- 应用不崩溃；
- 从备份恢复；
- 显示 `primary_settings_corrupt_recovered_from_backup` 提示；
- GUI 仍可排版。

#### 无有效备份

```text
rules.yaml       非法 YAML
rules.yaml.bak   缺失或非法
```

期望：

- 应用不崩溃；
- 使用默认设置；
- 显示 `primary_settings_corrupt_no_usable_backup` 提示；
- GUI 仍可排版。

### 7.7 不可写目录

Linux 可以使用临时目录和权限控制；Windows 必须使用 ACL。两种平台都不能依赖系统固定目录或管理员权限。

期望：

- 保存失败时显示 `[data-testid="settings-status"]`；
- 错误包含目标设置路径；
- 应用不崩溃；
- 文本排版仍可用；
- 测试恢复权限后可以清理临时目录。

## 8. Linux 执行步骤

### 8.1 原生 Linux 桌面

先准备：

```bash
nvm use
npm ci --prefix frontend
npm ci --prefix e2e
```

确认 Tauri Linux 系统依赖和图形会话可用，然后：

```bash
# 构建前端测试资源
VITE_COPYPOLISH_E2E=true npm run build --prefix frontend

# 使用 E2E feature 构建测试 binary
export TAURI_CONFIG="$(cat src-tauri/tauri.e2e.conf.json)"
cargo build --manifest-path src-tauri/Cargo.toml --features e2e

# 运行 WebdriverIO
cd e2e
npm run test
```

### 8.2 WSL2

WSL2 只能在具备 WSLg 或可用 X11/Wayland 转发时尝试真实 GUI。没有图形会话时执行构建和类型检查即可；运行脚本应返回明确的环境错误，而不是把跳过当成通过。

建议记录：

```bash
uname -a
node --version
npm --version
rustc --version
cargo --version
echo "DISPLAY=${DISPLAY:-}"
echo "WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-}"
```

## 9. 必须在 Windows 完成的工作

以下项目在当前 Linux/WSL 环境中不能代替完成，必须在 Windows 桌面会话中执行。

### 9.1 Windows 环境准备

需要：

- Windows 10/11；
- Windows Node.js，版本符合 `.nvmrc` 和 `package.json`；
- Rust MSVC toolchain；
- Visual Studio Build Tools 的 C++ 工作负载；
- WebView2 Runtime；
- PowerShell 7；
- Windows Terminal；
- 可交互桌面会话。

检查：

```powershell
node --version
npm --version
rustc --version
cargo --version
pwsh --version
```

### 9.2 Windows GUI E2E

以下步骤必须在 Windows 原生交互式桌面执行，不能用 WSL、Linux CI 或普通浏览器预览代替。

#### 9.2.1 环境和版本记录

在仓库根目录的 PowerShell 7 中记录：

```powershell
git rev-parse HEAD
Get-ComputerInfo | Select-Object WindowsProductName, WindowsVersion, OsBuildNumber
node --version
npm --version
rustc --version
rustc -vV | Select-String '^host:'
cargo --version
pwsh --version
wt --version
where.exe msbuild
where.exe cl.exe
where.exe WebView2Loader.dll
```

验收要求：

- Node 必须为项目要求的 `v24.19.0`，或满足 `.nvmrc` / `package.json` 约束；
- Rust host 必须为 `x86_64-pc-windows-msvc`；
- Visual Studio C++ 工具链和 Windows SDK 可用；
- WebView2 Runtime 已安装并可被当前用户访问；
- 测试在可交互的 Windows Terminal 会话执行，不通过无桌面服务会话冒充 GUI 验证。

#### 9.2.2 依赖安装和前端构建

```powershell
npm ci --prefix frontend
npm ci --prefix e2e
npm run typecheck --prefix e2e
npm run build --prefix frontend
```

前端构建只验证资源生成；不能把它当作 WebView2 或 Rust IPC 已通过的证据。

#### 9.2.3 两种 provider 的最小 smoke

先验证原有 embedded provider：

```powershell
npm run build:app --prefix e2e
npm run test --prefix e2e
```

再验证标准 W3C provider：

```powershell
npm run build:app:webdriver --prefix e2e
npm run test:webdriver --prefix e2e
```

单独运行启动用例：

```powershell
npm run test --prefix e2e -- --spec specs/startup-formatting.spec.ts
npm run test:webdriver --prefix e2e -- --spec specs/startup-formatting.spec.ts
```

两种 provider 都必须确认：

1. 应用进程可以启动并正常退出；
2. WebView2 加载实际打包前端资源；
3. embedded provider 或标准 W3C WebDriver 可以创建 session；
4. 主窗口可以被发现并操作；
5. 输入 `在LeanCloud上，花了5000元` 后通过真实 `format_text` IPC 得到预期输出；
6. 没有依赖浏览器 `localStorage` 或 mock `format_text`；
7. 标准 provider 的 `/status`、session、动态端口和退出状态有 artifact 记录；
8. 未产生仓库根目录的 `rules.yaml` 或 `rules.yaml.bak`。

#### 9.2.4 设置保存和重启恢复

每个 provider 至少执行一次以下流程，并为每次流程使用新的临时目录：

1. 启动 binary；
2. 打开设置窗口；
3. 修改规则选择、主题、字体或 UI 缩放中的至少一项；
4. 等待 `[data-testid="settings-status"]` 显示保存成功；
5. 读取 `[data-testid="settings-path"]`，确认目标是临时目录中的 `rules.yaml`；
6. 关闭应用并确认进程退出；
7. 使用同一个临时设置目录重新启动 binary；
8. 打开设置并确认修改后的状态恢复；
9. 执行真实格式化，确认恢复后的规则影响 Rust 输出；
10. 测试结束后删除临时目录，失败时保留一份诊断副本。

#### 9.2.5 损坏设置和备份恢复

分别准备以下 fixture，不要覆盖开发者真实设置：

```text
Case A: rules.yaml 非法，rules.yaml.bak 有效
Case B: rules.yaml 非法，rules.yaml.bak 缺失
Case C: rules.yaml 非法，rules.yaml.bak 也非法
```

每个 case 都要确认：

- 应用和 WebView2 可以启动；
- 主界面仍能通过真实 IPC 排版；
- `settings-load-notices` 显示正确的恢复或降级提醒；
- Case A 使用有效备份；
- Case B/C 按实现使用默认设置，并且不会崩溃；
- 后续保存不会把无效 YAML 或错误的旧设置继续传播；
- 退出状态、stdout/stderr、页面源码和截图均可追溯。

#### 9.2.6 Windows ACL 不可写目录

Windows 必须使用 NTFS ACL，不要用 Linux `chmod` 或仅设置文件只读属性代替。示例流程：

```powershell
$settingsDir = Join-Path $env:TEMP ("copypolish-e2e-acl-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $settingsDir | Out-Null
$user = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name

# 先写入有效 fixture，再拒绝当前用户写权限。
Set-Content -Path (Join-Path $settingsDir 'rules.yaml') -Value "enabled: []`n" -Encoding utf8
icacls $settingsDir /inheritance:r
icacls $settingsDir /deny "${user}:(OI)(CI)(W)"

# 在此处运行任一 provider 的设置保存用例。

# 无论测试成功或失败，都必须恢复权限并删除目录。
icacls $settingsDir /remove:d $user
icacls $settingsDir /inheritance:e
Remove-Item -Recurse -Force $settingsDir
```

验收要求：

- 保存失败显示 `[data-testid="settings-status"]`；
- 错误信息包含 `rules.yaml` 目标路径；
- 应用、WebView2 和 WebDriver session 不崩溃；
- 文本排版仍可执行；
- ACL 恢复后临时目录可以删除；
- 测试不能在权限未恢复时结束，避免污染后续 Windows runner。

建议使用 `try { ... } finally { ... }` 包裹 ACL fixture 的全部操作；如果测试中途异常，`finally` 仍必须执行 `/remove:d`、恢复继承并删除临时目录。

#### 9.2.7 进程、端口和 artifact 清理

每次 provider 运行后记录并检查：

```powershell
Get-Process | Where-Object { $_.ProcessName -match 'chinese-copywriting-formatter|wdio|node' }
Get-NetTCPConnection -State Listen | Where-Object { $_.LocalPort -eq 4445 -or $_.LocalPort -ge 44000 }
Get-ChildItem .\e2e\artifacts -Recurse
Get-ChildItem . -Force -Filter 'rules.yaml*'
```

成功运行后不得残留 CopyPolish、WDIO 或测试专用 WebDriver 进程。失败运行应保留一次 artifact 副本，至少包括：

- WebdriverIO log；
- 应用 stdout/stderr；
- manifest 和退出状态；
- 失败截图；
- page source；
- 临时 `rules.yaml` / `rules.yaml.bak`；
- Windows、Node、Rust、WebView2、provider 和 commit 信息。

### 9.3 Windows TUI smoke

这不属于 GUI E2E；Windows Terminal + PowerShell 7 的修复后 raw-mode、规则排序、快捷键、粘贴、保存和重启恢复手动验收已完成。自动化 artifact 收集仍是后续工作；本次发现的 Delete 光标边界缺陷记录在第 9.6 节。手动验收应记录：

- Windows 版本；
- Windows Terminal 版本；
- PowerShell 版本；
- TUI binary commit；
- OSC 52 复制结果；
- 键盘输入、规则面板、保存和退出结果；
- 是否存在终端特有显示问题。

推荐命令：

```powershell
cargo build --manifest-path src-tauri/Cargo.toml --features tui --release --bin copypolish-tui
& .\src-tauri\target\release\copypolish-tui.exe
```

在 Windows Terminal 中手工验证：

1. raw-mode 启动和退出；
2. 输入文本并执行排版；
3. 打开规则面板并切换规则；
4. 保存设置并确认 `rules.yaml` 路径；
5. 使用复制动作检查 OSC 52 或 Windows Terminal 剪贴板行为；
6. 重启 TUI 确认规则选择恢复；
7. 记录窗口大小、字体、编码、PowerShell 和 Windows Terminal 版本。

### 9.4 本次 Windows 执行记录（2026-08-31）

执行位置：`E:\Shiraishi\VSCode Workspace\chinese_copywriting_formatter`，commit `f6cc3f6bf76e9c5f2037bc44063d706ed8755ffe`（含未提交 E2E 修正）。

环境：Windows Node `v24.19.0`、npm `11.17.0`、Rust `1.98.0`，host `x86_64-pc-windows-msvc`，Visual Studio Build Tools `17.14.39`，WebView2 Runtime `151.0.4129.107`，Windows Terminal `1.24.11911.0`，PowerShell 7。

结果：

- embedded provider：两个 spec、3 个用例通过；覆盖真实 WebView2 启动、Rust IPC 默认格式化、全不选恒等、临时 `rules.yaml` 路径和规则保存。
- 标准 W3C WebDriver provider：两个 spec、3 个用例通过；随机端口 `55755` / `53010`，session、主窗口发现和退出清理正常。
- `cargo test user_settings::tests`：16/16 通过，覆盖备份恢复、主备份损坏降级、迁移、UTF-8 和缺失目录诊断。
- TUI：Windows release 构建通过；`--stdin --no-config` 输出 `在 LeanCloud 上，花了 5000 元！`。
- NTFS ACL：当前用户写入被拒绝，权限恢复和临时目录删除均成功。
- 清理：无 CopyPolish、WDIO 残留进程/监听端口，仓库根目录无 `rules.yaml*`。

自动化缺口：尚无专用 GUI spec 自动验证“第二次启动恢复”和“ACL 下保存失败提示”，也未建立 Windows Terminal raw-mode/OSC 52 的自动 artifact 收集；三种损坏设置 fixture 已通过专用 runner 在两个 provider 中自动化验证，上述项目的修复后人工回归已完成，TUI-EDIT-DELETE-001 也已关闭。

本次修复后复验（2026-08-31）：embedded provider 与标准 W3C provider 各 3 个最小 smoke 用例均通过（设置链路 2/2、默认格式化 1/1）；三种损坏设置 fixture 在两个 provider 中各 3/3 通过；`cargo test user_settings::tests` 16/16 通过；`cargo test --features tui` 148/148 通过；Windows TUI release 构建和 `--stdin --no-config` smoke 通过；NTFS ACL 拒写、恢复和 fixture 删除通过。重启恢复和 ACL 保存失败尚无专用自动化 spec，Windows Terminal raw-mode/OSC 52 及 GUI 视觉/DPI 的自动化 artifact 仍待补齐，但对应 Windows 人工回归已完成。

### 9.5 历史修复前手动基线（2026-08-31，不作为当前结论）

用户确认已在 Windows 原生桌面完成以下修复前基线项目：

- GUI 设置修改、保存、关闭后重启恢复；
- `rules.yaml` 主文件/备份损坏的三种 fixture（备份恢复、默认降级、双文件损坏）；
- NTFS ACL 不可写目录下的 GUI 保存失败提示、路径显示、格式化继续可用及权限恢复；
- 规则切换、应用快捷键、无边框窗口拖动、最小化/最大化/关闭和多 DPI 布局；
- Windows Terminal + PowerShell 7 的 TUI raw-mode、规则面板、保存/退出、重启恢复和 OSC 52/降级提示。

本次清理已删除成功验收后的 `e2e/artifacts/` 历史日志和截图；修复后 GUI/TUI 回归现已完成，TUI-EDIT-DELETE-001 已关闭，编辑器回归已纳入 Rust/TUI 自动测试。

### 9.6 修复后手动核验记录（已完成）

本次自动化已确认双 provider 最小 WebView2/IPC/设置保存链路、Rust 设置 16/16、TUI 148/148、TUI stdin、release 构建和 NTFS ACL 基础；以下项目的修复后 Windows 原生交互式桌面核验已完成，TUI-EDIT-DELETE-001 已关闭。

- GUI 浅色/深色主题、100%/125%/150% DPI、窄窗口、输入/输出框和设置布局视觉；
- 设置修改后的重启恢复；`rules.yaml`/`.bak` 三种损坏 fixture 的提示、降级和再次保存；
- NTFS ACL 拒写时 GUI 保存失败提示、格式化继续可用、权限恢复和清理；
- Windows Terminal raw-mode、裸字符与 Ctrl 快捷键、规则排序、完整剪贴板/bracketed paste、OSC 52、保存和重启恢复；
- 两种 provider 各连续 5 次稳定性、端口/进程清理和失败 artifact。

完整的前置条件、命令、fixture、验收点和清理步骤见 [testing.md 第 7.6 节](testing.md#76-修复后手动核验记录已完成)。
## 10. 验证命令分层

### 10.1 静态和构建验证

```bash
python3 scripts/verify.py --profile checks
python3 scripts/verify.py --profile frontend
python3 scripts/verify.py --profile rust
python3 scripts/verify.py --profile ci
```

如果新增独立 E2E workspace，还应执行：

```bash
npm ci --prefix e2e
npm run typecheck --prefix e2e
```

### 10.2 真实 Linux GUI 验证

```bash
cd e2e
npm run test
npm run test:debug
```

单独运行启动用例：

```bash
npx wdio run wdio.conf.ts --spec specs/startup-formatting.spec.ts
```

参考插件 provider：

```bash
npm run build:app:webdriver --prefix e2e
npm run test:webdriver --prefix e2e
npm run test:webdriver --prefix e2e -- --spec specs/startup-formatting.spec.ts
```

### 10.3 失败诊断

失败时保留：

```text
e2e/artifacts/screenshots/
e2e/artifacts/logs/
临时 rules.yaml
临时 rules.yaml.bak
版本和环境信息
```

检查：

```bash
git status --short
find . -maxdepth 1 -name 'rules.yaml*' -print
```

成功运行后删除临时目录；失败运行应保留一次诊断副本，并在日志中记录路径。

## 11. GitLab 接入策略

E2E 初期不应直接成为 `dev` 合并阻塞门禁。

推荐阶段：

1. 本地 Linux 桌面手动执行；
2. 本地 Windows 桌面手动执行；
3. GitLab 增加 Linux/Windows 可选 E2E stage；
4. 使用 `allow_failure: true` 收集稳定性数据；
5. 连续多次稳定后，再考虑纳入 tag 发布流程；
6. 最后再评估是否成为 `dev` 合并门禁。

E2E job 必须上传：

- WebdriverIO 报告；
- 截图；
- 前端和后端日志；
- 失败时的临时设置文件；
- Node、Rust、Tauri、WebView/WebView2 版本；
- 被测 commit SHA。

GitHub Actions 当前因账户计费问题维持禁用，E2E 不应绕过现有 GitLab/本地替代门禁策略。

## 12. 完成标准

P0.2 只有在以下条件全部满足后才能标记完成。括号中的状态用于区分已通过的 Windows 最小链路和仍未完成的扩展验证：

- [x] Linux/WSLg 至少一条真实 GUI 链路通过；
- [x] Windows WebView2 至少一条真实 GUI 链路通过；
- [x] embedded provider 能启动测试 binary；
- [x] 默认示例通过真实 WebView 和真实 Rust IPC；
- [x] 全不选保持恒等；
- [x] 规则切换有效（Windows GUI 手动验收通过）；
- [x] 快捷键有效（Windows GUI 手动验收通过）；
- [x] 设置保存和重启恢复有效（Windows GUI 手动验收通过）；
- [x] 损坏主文件和备份恢复有效（Windows GUI 三种 fixture 手动验收通过）；
- [x] 不可写目录错误提示有效（Windows NTFS ACL 手动验收通过）；
- [x] 测试不会污染仓库或用户设置；
- [x] 失败时至少可获取 provider 日志、应用 stdout/stderr、manifest/退出状态；完整故障注入 artifact 仍待补齐；
- [x] Windows 和 Linux 的临时目录清理均已确认；
- [ ] React 19 `act` warning 已通过真实用户流确认是否仅为 jsdom 环境问题；
- [ ] GitLab 可选 E2E stage 可以重复执行。

### 13. 当前结论

本项目已完成 E2E 工程设计、依赖锁定、Rust/TypeScript 配置检查、Linux/WSLg 真实 GUI smoke，以及 Windows WebView2 双 provider 最小 smoke、修复后人工回归和 TUI-EDIT-DELETE-001 编辑器边界修复。

自动化仍未覆盖：

- Windows 专用 GUI 重启恢复自动化 spec；
- Windows ACL 下真实 GUI 保存失败提示自动化 spec；
- Windows Terminal + PowerShell 7 raw-mode、规则面板和 OSC 52 交互的自动化 artifact 收集。

三种损坏设置 fixture 已由 `test:corrupt-settings` 和
`test:corrupt-settings:webdriver` 入口覆盖，不再属于未完成项。

因此，后续工作应将已完成的手动步骤继续固化为重启恢复/NTFS ACL GUI 故障注入 spec 和 Terminal artifact 收集，再评估 GitLab 可选 E2E stage 是否纳入更严格门禁。当前未完成项不再包括 Windows 手动回归、损坏设置 fixture 或 TUI 编辑器 Delete 边界。
