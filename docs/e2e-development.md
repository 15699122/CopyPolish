# 真实 Tauri GUI E2E 开发说明

本文记录 CopyPolish 真实 Tauri GUI E2E 的当前开发状态、环境边界、实施步骤、测试方法和验收标准。

本文专门区分“可以在当前 Linux/WSL 环境完成的开发工作”和“必须在 Windows 桌面环境完成的验证工作”。
**构建检查通过不等于真实 GUI E2E 通过**：只有在真实桌面会话中启动 Tauri 二进制并完成 WebView、IPC 和设置链路后，才可以将对应 smoke 标记为通过。

## 1. 当前状态

截至 2026-08-30：

- E2E 工具路线已确定为 WebdriverIO + `@wdio/tauri-service` + embedded provider；
- 当前仓库尚未提交 WebdriverIO E2E 工程；
- 当前仓库尚未提交 `e2e` Cargo feature 或 WebDriver plugin；
- 当前工作环境是 Linux WSL2，不是 Linux 原生桌面会话，也不是 Windows；
- 当前 Node 实际版本为 `v26.7.0`，超出项目 `.nvmrc` / `package.json` 要求的 `>=24 <25`，不能作为正式 E2E 基线；
- 默认 Tauri 配置仍使用现有 `src-tauri/tauri.conf.json` 和生产 capability；
- 当前真实 GUI E2E 仍属于 roadmap P0.2 未完成事项。

本环境已完成的前置确认：

1. Cargo 可以解析并编译 `tauri-plugin-wdio` 与 `tauri-plugin-wdio-webdriver` 的 1.x 版本；
2. WebdriverIO npm 包可安装到独立测试 workspace；
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
5. 项目已有 Rust、前端和非交互 TUI 回归覆盖，但不能替代真实 Tauri GUI E2E。

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

必须在 Windows 上重新执行：

1. E2E binary 构建；
2. WebdriverIO embedded provider 启动；
3. WebView2 加载；
4. 真实 `invoke` IPC；
5. 无边框窗口创建和关闭；
6. 快捷键；
7. `rules.yaml` 保存和恢复；
8. 损坏设置恢复；
9. Windows ACL 不可写目录；
10. 进程退出和临时目录清理。

### 9.3 Windows TUI smoke

这不属于 GUI E2E，但仍是 roadmap 的独立未完成项。必须在 Windows Terminal + PowerShell 7 中执行 raw-mode 交互 smoke，并记录：

- Windows 版本；
- Windows Terminal 版本；
- PowerShell 版本；
- TUI binary commit；
- OSC 52 复制结果；
- 键盘输入、规则面板、保存和退出结果；
- 是否存在终端特有显示问题。

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

P0.2 只有在以下条件全部满足后才能标记完成：

- [ ] Linux 原生桌面至少一条真实 GUI 链路通过；
- [ ] Windows 桌面至少一条真实 GUI 链路通过；
- [ ] embedded provider 能启动测试 binary；
- [ ] 默认示例通过真实 WebView 和真实 Rust IPC；
- [ ] 全不选保持恒等；
- [ ] 规则切换有效；
- [ ] 快捷键有效；
- [ ] 设置保存和重启恢复有效；
- [ ] 损坏主文件和备份恢复有效；
- [ ] 不可写目录错误提示有效；
- [ ] 测试不会污染仓库或用户设置；
- [ ] 失败时可获取截图、前端日志和后端日志；
- [ ] Windows 和 Linux 的临时目录清理均已确认；
- [ ] React 19 `act` warning 已通过真实用户流确认是否仅为 jsdom 环境问题；
- [ ] GitLab 可选 E2E stage 可以重复执行。

## 13. 当前结论

本环境适合完成 E2E 工程设计、依赖锁定、Rust/TypeScript 配置检查、测试 helper、文档和 Linux 构建验证。

本环境不能替代：

- Windows WebView2 GUI 验证；
- Windows ACL 不可写目录验证；
- Windows Terminal + PowerShell 7 TUI smoke；
- 没有图形会话时的真实 Linux GUI 窗口验证。

因此，后续工作应先在原生 Linux 桌面完成最小启动 Spike，再在 Windows 桌面完成同一条最小链路，最后才扩展设置故障注入和 GitLab 可选门禁。