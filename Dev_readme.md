# 中文文案排版助手：开发说明

本文档面向后续维护者，记录当前项目结构、架构边界、开发运行方式、测试命令、GUI 行为约束、打包方式和已知注意事项。普通用户使用说明请参阅 [README.md](README.md)。

## 当前开发状态

截至 2026-08-21，项目已完成以下主要工作：

- GUI 已从早期 `ttkbootstrap` 方案迁移到 `customtkinter`。
- 界面固定为 Light 外观，不再提供主题切换。
- GUI 已拆分为 `gui/` 包：入口、主窗口、设置窗口和复用控件分离。
- 主窗口为无边框自绘窗口，主体为圆角矩形。
- 主窗口右上角使用 Windows 顺序的圆形控制按钮：最小化、最大化/还原、关闭。
- 设置窗口使用普通 `CTkToplevel`，保留系统标题栏，并通过 transient + grab 避免被主窗口遮挡或抢焦点。
- 规则引擎支持 Markdown、LaTeX、URL、邮箱、行内代码等保护。
- 引擎测试和 GUI 冒烟测试已覆盖主要路径。
- Windows PyInstaller 打包脚本和 GitHub Actions 工作流已存在。

### 正在进行的 Tauri + shadcn/ui + PyO3 重构（进行中）

当前计划使用 **Tauri 2 + React + shadcn/ui** 全面替换 customtkinter GUI，并由 **Rust/PyO3 在进程内嵌入 CPython** 调用现有 `ccw_engine.py`。已完成可落地部分：

- `ccw_engine.py`：**已移除「模块导入即写 rules.yaml」的副作用**，改为显式调用 `initialize(config_path=None)`；原先依赖该副作用的 `gui/app.py` 已改为在 `FormatterApp.__init__` 中显式调用。
- 新增 `python/formatter_bridge.py`：面向 PyO3 的稳定受限入口（`format_document` / `list_rules` / `enabled_defaults`），不含持久化职责。
- 新增 `test/test_formatter_bridge.py`：桥接层单元测试。
- `frontend/`：**已从脚手架升级为可构建、可运行的真实 shadcn/ui 界面**——左输入/右输出（移动尺寸上下堆叠）、操作栏（设置/清除/复制，带一键状态反馈）、规则设置 Dialog（分组展示、单条开关、全选/全不选/恢复默认）、160ms 防抖实时排版、浏览器预览回退等。已验证 `npm run build`（tsc + vite）通过，`npm run dev` 在 1420 端口返回 200。
- `src-tauri/`：已从 `tauri-plugin-python` 占位路线切换为**自定义 PyO3 适配层（路线 B）**，包含 `commands.rs` / `python_runtime.rs`；前端通过受限 Tauri commands 调用嵌入式 CPython。
- `src-tauri/src/python_runtime.rs`：新增 Rust 单元测试，直接验证 `PyO3 → src-python/main.py → ccw_engine.py` 的三条核心业务链路：读取 13 条规则、读取 11 条默认规则、格式化 `在LeanCloud上，花了5000元`。
- Rust/PyO3 链路：已安装并验证 `rustup` / `cargo` / `rustc`（1.98.0）与 `rustfmt`，系统已有 `Python.h` 与 `libpython3.14.so`；独立冒烟工程已验证 `PyO3 → src-tauri/src-python/main.py → ccw_engine.py`，结果包括 `get_rules=13`、`format_text("在LeanCloud上，花了5000元") = "在 LeanCloud 上，花了 5000 元"`、默认规则 11 条。
- `frontend/package.json`：`tauri` 脚本已改为先进入兄弟目录 `../src-tauri` 再调用本地 Tauri CLI，避免在 `frontend/` 目录下运行 `npm run tauri dev` 时 CLI 找不到 `tauri.conf.json`。
- `src-tauri/tauri.conf.json`：`bundle.resources` 已改为显式文件路径 `src-python/main.py`、`../ccw_engine.py` 与 `../rules.yaml`，避免 Tauri build script 对 glob `src-python/**` 报未匹配，同时确保安装包不依赖源码树中的 `ccw_engine.py`；`beforeDevCommand` / `beforeBuildCommand` 已改为兼容从仓库根目录或 `src-tauri/` 执行的路径判断。
- `src-tauri/src/lib.rs`：Tauri setup 阶段已改为优先通过 `BaseDirectory::Resource` 解析打包资源中的 `src-python/main.py`，再把其所在目录传给 PyO3；失败时才回退到开发源码目录 `src-tauri/src-python`，避免 release/install 场景依赖当前工作目录。
- `src-tauri/src-python/main.py`：已改为同时支持开发树和 Tauri Linux 资源布局。Linux bundle 中 `../ccw_engine.py` 与 `../rules.yaml` 会落在资源根的 `_up_/` 下，因此桥接模块会把资源根、`_up_`、项目根候选路径加入 `sys.path`，并优先选择已存在的打包 `rules.yaml`。
- `frontend/tsconfig.app.json`：移除了 TypeScript 5.9 不接受的 `"ignoreDeprecations": "6.0"` 配置；`npm run build --prefix frontend` 已恢复通过。
- Git：项目已初始化为 Git 仓库，并提交基线 `chore: baseline before rust typesetting engine migration`，便于后续 Rust 后端引擎迁移回滚与对比。
- `src-tauri/src/rust_engine.rs`：新增第一版 Rust 原生文字处理引擎。该模块参考 `typeset-rs` 的字符分类 / token 化 / 渲染管线思路，但不复制其源码；当前实现基础中英文/数字空格、数字单位、全角标点空格、重复标点、全角中文标点、半角数字、英文语境半角标点、专有名词与缩写等核心规则。
- 保护层已迁移到 Rust（第二版）：`rust_engine.rs` 现在完整复刻 `ccw_engine.py` 的 `_protect` / `_protect_markdown_lines` / `_space_around_inline_placeholders` / `_restore` 管线，覆盖 fenced code block、LaTeX 环境/display/inline/command、Markdown 图片/链接/autolink、行内代码、URL、邮箱与缩进代码行共 13 类保护模式，占位符格式与 Python 完全一致（`\u{E000}CCWPROTECTED{n}\u{E001}`）。保护层正则需要 lookbehind/backreference，因此引入 `fancy-regex` 依赖。`format_text` 的 `should_fallback_to_python` 拦截已移除——普通含保护内容的输入不再回退 Python。
- 规则 key 已对齐：Rust 引擎改用与 Python `_slug()` 完全一致的规则 key（如 `遇到完整的英文整句_特殊名词_其内容使用半角标点`、`用_text_spacing_来挽救`），修复了此前前端传真实 key 时部分规则不生效的问题；规则执行顺序也与 Python RULES 注册表一致。
- `get_rules` 已切换为 Rust 端内置元数据（`rust_engine::default_rules()`，13 条规则的 key/section/name/disputed/default 与 Python `_EMBEDDED_RULES` 完全一致），仅在异常时回退 Python/rules.yaml。至此 `format_text` / `get_rules` / `get_enabled_defaults` 三个 command 均以 Rust 为主路径，PyO3/Python 降级为兜底。
- parity 语料已从 50 条扩充到 71 条（新增半角标点中文语境、纯英文行、多链接、嵌套引号+链接、相邻公式、句尾 URL 等边界用例），defaults/all 两种模式下仍为 0 差异。
- **旧 GUI 已彻底移除**：删除 `gui/`、`chinese_copywriting_formatter.py`、`python/formatter_bridge.py`、`test/test_formatter_bridge.py`、`packaging/build_win.bat`、`run.sh` 与 PyInstaller 的 GitHub Actions 工作流。项目不再保留 customtkinter 界面与其 `rules.yaml` 设置兼容。
- **用户设置持久化（新方案）**：新增 `src-tauri/src/user_settings.rs` 与 `get_user_settings` / `save_user_settings` command。设置保存在**当前工作目录**的 `ccw-formatter-settings.json`（`enabled` + `last_input`），文件缺失/损坏时回落默认规则集。前端 `App.tsx` 启动时恢复设置，规则开关与清空操作即时持久化、输入内容防抖持久化；浏览器预览环境回退 localStorage。Rust 单元测试全部使用系统临时目录中的随机文件，避免写覆盖仓库内文件；设置文件已加入 `.gitignore`。


- 争议规则「链接之间增加空格」已在 Rust 端实现（含独立的链接保护模式子集）。
- **双引擎 parity 校验通过**：新增 `test/compare_rust_parity.py` 与 `src-tauri/examples/parity_dump.rs`，对 50 条语料在 defaults / all 两种规则模式下逐字对比 ccw_engine.py 与 Rust 引擎输出，结果 0 差异。对比中发现并修复了三处行为偏差：混合重复标点折叠、空白行规范化、弯引号误转换。

- `src-tauri/src/commands.rs`：`format_text` 已改为 **Rust engine 优先，PyO3/Python fallback**。`get_rules` 仍由 Python/rules.yaml 提供，避免迁移早期破坏规则元数据；`get_enabled_defaults` 已可从 Rust 返回，并保留 Python fallback 分支。

`typeset-rs` 调研结论：该仓库适合作为 Rust 原生排版核心的架构参考（字符分类、token、上下文渲染、全半角和中英文空格处理），许可证为 MIT；但其 README/TODO 显示 URL/文件名保护、专有名词大小写、复杂语义引号等仍非完整覆盖。本项目因此采用“本项目内 Rust engine 渐进重写 + Python fallback”的路线，不直接把上游作为即插即用依赖。

工具链说明：本机已安装 Node（v26）/npm、Rust stable（1.98.0）、`rustfmt` 与 Tauri Linux WebKitGTK/GTK 系统依赖，且 `libpython3.14-dev` 实际已满足。`cargo check --manifest-path src-tauri/Cargo.toml`、`cargo test --manifest-path src-tauri/Cargo.toml`、Python 单元测试、前端构建均已通过；`npm run tauri dev` 已能启动 Vite、完成 Rust dev 编译并运行 `target/debug/chinese-copywriting-formatter`；`npm run tauri build` 已成功产出 deb/rpm/AppImage 三种 Linux bundle。当前 dev 日志中仍可能出现 WSL 图形栈相关的 `libEGL` / `MESA` / `ZINK` 警告，这属于 WSLg/GPU 渲染环境问题，不是 Tauri/Rust/PyO3 编译错误。

最近一次完整验证（2026-08-22）：

- 隔离安装态资源 smoke：构造仅包含 `src-python/main.py`、`_up_/ccw_engine.py`、`_up_/rules.yaml` 的临时目录，用 `/usr/bin/python3.14` 直接 import `main.py`，结果为 `rules=13`、`defaults=11`、`formatted=在 LeanCloud 上，花了 5000 元`，且 `_RULES_PATH` 指向临时目录中的 `_up_/rules.yaml`。
- Python 测试：`.venv/bin/python -m unittest discover -s test`，49 passed。
- Rust 格式检查：`cargo fmt --manifest-path src-tauri/Cargo.toml --check`，通过。
- Rust/PyO3 测试：`cargo test --manifest-path src-tauri/Cargo.toml`，4 passed。
- 前端构建：`npm run build --prefix frontend`，通过。
- Tauri 打包：`npm run tauri build`，通过并重新产出 deb/rpm/AppImage。

Rust engine 第一版验证（2026-08-22）：

- Git 基线提交已完成。
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过，无 Rust warning。
- `cargo test --manifest-path src-tauri/Cargo.toml`：10 passed（包含 6 个 Rust engine 新测试 + 既有 PyO3 测试）。
- `.venv/bin/python -m unittest discover -s test`：49 passed。
- `npm run build --prefix frontend`：通过。
- `npm run tauri build`：通过并重新产出 deb/rpm/AppImage。

复现验证命令：

```bash
cd "/home///chinese_copywriting_formatter"
export PATH="$HOME/.cargo/bin:$PATH"
export PYO3_PYTHON=/usr/bin/python3.14
export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml

.venv/bin/python -m unittest discover -s test
npm run build --prefix frontend

cd frontend
npm run tauri dev
npm run tauri build
```

最近一次 Linux bundle 输出：

```text
src-tauri/target/release/bundle/deb/中文文案排版助手_0.1.0_amd64.deb
src-tauri/target/release/bundle/rpm/中文文案排版助手-0.1.0-1.x86_64.rpm
src-tauri/target/release/bundle/appimage/中文文案排版助手_0.1.0_amd64.AppImage
```

最近一次已确认的 bundle 资源布局（Linux）：

```text
usr/lib/中文文案排版助手/src-python/main.py
usr/lib/中文文案排版助手/_up_/ccw_engine.py
usr/lib/中文文案排版助手/_up_/rules.yaml
```

注意：`_up_` 是 Tauri 对 `../...` 资源路径的打包布局结果。`main.py` 中的路径探测必须保留 `_UP_ROOT`，否则安装后可能无法 `import ccw_engine`。

### 后续计划 / 下一步

1. **真实窗口交互验证**：在 Tauri 窗口中输入 `在LeanCloud上，花了5000元`，确认输出为 `在 LeanCloud 上，花了 5000 元`；打开设置弹窗，确认 13 条规则 / 11 条默认规则，切换规则后输出更新。
2. **扩大 Rust engine 覆盖面**：保护层与全部 13 条规则均已迁移到 Rust，双引擎 parity（50 语料 × defaults/all 模式）为 0 差异；后续可把 fallback 降级为仅在 Rust engine 出错时兜底，并考虑从 bundle 中移除 Python/PyO3。
3. **release 二进制 / AppImage 短时启动验证**：隔离资源 import smoke 已通过；仍建议在目标图形环境中运行 release 二进制或 AppImage，确认 WebView、Rust engine 与 PyO3 fallback 在安装态同时可用。WSL 下如出现 `libEGL` / `MESA` / `ZINK` 警告，优先按图形栈问题排查。
4. **处理用户设置持久化**：正式版不要修改随包分发的 `rules.yaml`。建议将用户选择写入 Tauri `AppData` / `AppConfig`，`rules.yaml` 仅作为只读默认元数据。
5. **替换正式图标集**：当前 `src-tauri/icons/icon.png` 是临时有效 PNG，只用于解除构建阻塞。发布前应生成完整 Linux/Windows/macOS 图标集合。
6. **完善分发策略**：当前 Linux deb/rpm/AppImage 已能生成；Windows/macOS 的 Tauri 打包、嵌入式 Python 分发与 PyO3 动态库策略仍需单独验证。
7. **补充命令级集成测试**：若条件允许，增加不启动 WebView 的 Tauri command smoke，覆盖 `format_text` / `get_rules` / `get_enabled_defaults`，并验证 Rust engine 与 Python fallback 两条路径。

## 项目结构

```text
chinese_copywriting_formatter/
├── README.md
├── Dev_readme.md
├── .gitignore
├── ccw_engine.py                         # 纯 Python 规则引擎，无 GUI 依赖
├── chinese_copywriting_formatter.py      # (旧) GUI 启动入口，导出 FormatterApp/main
├── gui/                                  # (旧) customtkinter GUI
│   ├── __init__.py
│   ├── app.py                            # customtkinter 主窗口、输入输出、窗口控制
│   ├── settings_window.py                # 规则设置窗口
│   └── widgets.py                        # GUI 常量、字体、复用 card 等
├── python/
│   └── formatter_bridge.py               # 面向 Rust/PyO3 的稳定受限桥接入口
├── frontend/                             # (新) React + Vite + TS + Tailwind v4 + shadcn（可构建）
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig*.json
│   ├── components.json                   # shadcn 配置
│   └── src/
│       ├── main.tsx / App.tsx / index.css
│       ├── lib/ (tauri.ts / utils.ts)
│       └── components/ui/                # shadcn: button/card/textarea/checkbox/
│           │                             #        dialog/label/scroll-area
│           └── ...
├── src-tauri/                            # (新) Tauri 2 壳 + 自定义 PyO3 适配层
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/icon.png                    # 临时占位图标；正式打包前应替换为完整图标集
│   ├── src/
│   │   ├── main.rs                       # Tauri 入口
│   │   ├── lib.rs                        # 注册 commands 并初始化 Python
│   │   ├── commands.rs                   # 前端 invoke 的受限命令入口
│   │   └── python_runtime.rs             # PyO3 内嵌 CPython 运行时适配层
│   └── src-python/main.py                # Python 桥接模块，调用 ccw_engine
├── rules.yaml                            # 规则元数据和用户本地 settings
├── run.sh                                # Linux/macOS 本地启动脚本，默认调用 .venv
├── test/
│   ├── __init__.py
│   ├── test_ccw_engine.py                # 引擎单元测试（40 项）
│   ├── test_formatter_bridge.py          # (新) 桥接层单元测试（9 项）
│   └── gui_integration_test.py           # GUI 集成冒烟测试
├── packaging/
│   └── build_win.bat                     # Windows PyInstaller 打包脚本
└── .github/
    └── workflows/
        └── build-windows.yml             # Windows exe 构建工作流
```

本地开发产物包括但不限于：

- `.venv/`
- `.pylib/`
- `__pycache__/`
- `build/`
- `dist/`
- PyInstaller `.spec` 文件

这些目录/文件不应作为运行逻辑的一部分依赖。当前工作区不是 Git 仓库；如后续初始化 Git，应补充 `.gitignore`。

## 架构说明

### 规则引擎：`ccw_engine.py`

职责：

- 加载规则元数据；
- 执行文本格式化；
- 保护 Markdown / LaTeX / URL / 邮箱 / 代码片段；
- 读写 `rules.yaml` 中的本地设置；
- 提供纯函数式入口供测试和 GUI 调用。

重要约束：

- 引擎不应依赖 tkinter、customtkinter 或任何 GUI 模块。
- 内置规则实现保留在 Python 中；`rules.yaml` 只存规则元数据和用户设置。
- `rules.yaml` 中无内置实现的多余规则 key 会被忽略。
- 缺失或解析失败时，引擎应回退到内嵌规则表。
- YAML 读写优先使用 PyYAML；不可用时使用内置轻量 YAML 子集实现。

### GUI 入口：`chinese_copywriting_formatter.py`

职责：

- 作为应用启动入口；
- 从 `gui.app` 导出 `FormatterApp`、`_import_gui`、`main`；
- 保持入口文件简短，避免把 GUI 细节堆在顶层脚本中。

### 主窗口：`gui/app.py`

职责：

- 初始化 `customtkinter`；
- Windows 下尽量启用 DPI 感知；
- 构建主窗口、输入区、输出区和操作按钮；
- 处理实时格式化、防抖、复制、清空、持久化；
- 实现无边框窗口的拖动、最小化、最大化/还原、关闭。

当前关键 GUI 决策：

- `ctk.set_appearance_mode("Light")`：固定浅色模式。
- `ctk.set_widget_scaling(1.0)` 与 `ctk.set_window_scaling(1.0)`：减少 DPI/缩放导致的布局不稳定。
- `root.overrideredirect(True)`：主窗口无系统标题栏。
- `self.main = ctk.CTkFrame(..., corner_radius=18)`：主界面圆角主体。
- 自绘标题栏支持拖动，双击标题栏可最大化/还原。
- 最小化时临时取消 `overrideredirect`，恢复映射时再设置回来。

### 设置窗口：`gui/settings_window.py`

职责：

- 展示所有规则；
- 支持单条规则切换；
- 支持全选、全不选、恢复默认；
- 保持设置窗口在主窗口前可用。

当前关键 GUI 决策：

```python
win = ctk.CTkToplevel(self.app.root)
win.transient(self.app.root)
win.attributes("-topmost", False)
win.grab_set()
win.focus_force()
```

含义：

- 不使用 `overrideredirect`，因此保留系统标题栏。
- `transient(root)` 表示它属于主窗口的临时子窗口。
- 不设置永久 topmost，避免遮挡其他应用。
- `grab_set()` 让设置窗口打开期间主窗口不能抢焦点或覆盖它。
- 关闭时必须安全调用 `grab_release()`。

不要轻易改成全局置顶窗口；用户问题是主窗口遮挡/干扰设置窗口，而不是要求设置窗口覆盖所有应用。

### 复用控件：`gui/widgets.py`

职责：

- 应用名称；
- 字体常量；
- 常用 card/frame 创建函数；
- 颜色与布局常量的集中维护。

## 规则和配置文件

`rules.yaml` 结构：

```yaml
rules:
  中英文之间需要增加空格:
    section: 空格
    disputed: False
    default: True
settings:
  enabled:
  last_input: ""
```

开发注意事项：

- `rules` 是规则元数据，不包含 Python 函数。
- `settings` 是用户状态，测试和 GUI 运行都会改写它。
- 提交或交付前建议恢复为：

```yaml
settings:
  enabled:
  last_input: ""
```

- GUI 集成测试末尾会尝试恢复默认设置，但如果测试中断，仍可能留下本地状态，需要人工检查。

## 开发运行方式

进入项目目录：

```bash
cd "/home///chinese_copywriting_formatter"
```

使用项目虚拟环境启动：

```bash
./run.sh
```

或：

```bash
.venv/bin/python chinese_copywriting_formatter.py
```

如果缺少 tkinter：

```bash
sudo apt install python3-tk
```

如果缺少 customtkinter：

```bash
.venv/bin/python -m pip install customtkinter
```

## 测试与验证

### 编译检查

```bash
.venv/bin/python -m py_compile \
  ccw_engine.py \
  chinese_copywriting_formatter.py \
  gui/app.py \
  gui/settings_window.py \
  gui/widgets.py \
  test/test_ccw_engine.py \
  test/gui_integration_test.py
```

### 引擎单元测试

```bash
.venv/bin/python -m unittest -v test.test_ccw_engine
```

当前测试覆盖：

- YAML 加载和设置持久化；
- 空格规则；
- 标点规则；
- 全角/半角规则；
- 名词大小写和缩写规则；
- 争议规则默认关闭及开启行为；
- Markdown 保护；
- LaTeX 保护；
- URL / 邮箱 / 行内代码保护；
- 空行和换行风格；
- 幂等性。

截至当前开发进度，引擎测试共 40 个测试方法并已通过。

### GUI 集成测试

```bash
.venv/bin/python -m test.gui_integration_test
```

GUI 冒烟测试覆盖：

- 应用可初始化；
- 输入文本后自动格式化；
- Markdown / LaTeX 片段在 GUI 路径中保持；
- 复制输出到剪贴板；
- 大小窗口下关键控件仍可见；
- 主窗口保持无边框；
- 右上角控制按钮顺序为最小化、最大化/还原、关闭；
- 设置窗口存在、规则数量正确；
- 设置窗口不是无边框窗口；
- 设置窗口未设置全局 topmost；
- 设置改动可持久化。

无可用图形显示环境时，GUI 测试会打印 SKIP 并返回成功，避免把环境限制误判为代码错误。

### 提交/交付前推荐命令

```bash
.venv/bin/python -m py_compile ccw_engine.py chinese_copywriting_formatter.py gui/app.py gui/settings_window.py gui/widgets.py test/test_ccw_engine.py test/gui_integration_test.py
.venv/bin/python -m unittest -v test.test_ccw_engine
.venv/bin/python -m test.gui_integration_test
```

GUI 测试后检查并恢复 `rules.yaml`：

```yaml
settings:
  enabled:
  last_input: ""
```

## Windows 打包

### 本地打包

在 Windows 10/11 项目根目录运行：

```bat
packaging\build_win.bat
```

脚本流程：

1. 安装或更新 PyInstaller；
2. 执行单文件窗口模式构建；
3. 收集 Tcl/Tk 数据；
4. 收集 customtkinter 数据；
5. 将 `rules.yaml` 打包到程序目录；
6. 输出到 `build\windows\chinese_copywriting_formatter.exe`。

核心 PyInstaller 参数：

```bat
python -m PyInstaller ^
  --onefile ^
  --windowed ^
  --name chinese_copywriting_formatter ^
  --collect-tcl-data ^
  --collect-data customtkinter ^
  --add-data "rules.yaml;." ^
  chinese_copywriting_formatter.py
```

### GitHub Actions

`.github/workflows/build-windows.yml` 使用：

- `windows-latest`
- Python 3.12
- PyInstaller
- customtkinter

触发条件：

- push 修改入口、引擎、规则、packaging 或 workflow 文件；
- 手动 `workflow_dispatch`。

产物名称：

```text
chinese_copywriting_formatter-exe
```

## 重要实现约束

后续修改时请尽量保持以下约束：

1. 引擎和 GUI 解耦。
2. `ccw_engine.py` 不导入 GUI 模块。
3. GUI 依赖使用惰性导入，避免引擎测试要求 customtkinter。
4. 主窗口继续使用自绘标题栏时，需要保持最小化/恢复逻辑对 `overrideredirect` 的处理。
5. 设置窗口不要改成无边框窗口；必须保留系统标题栏。
6. 设置窗口不要默认全局置顶；优先使用 transient + grab + focus 解决主窗口干扰。
7. 文本保护规则新增或调整后，必须补充 Markdown / LaTeX / URL / 邮箱 / 代码边界测试。
8. 改动 `rules.yaml` 后，确认内置规则 key、YAML key 和 GUI 显示一致。
9. GUI 测试可能修改 `rules.yaml`，测试结束后要检查本地状态。

## 已验证事项

截至 2026-08-21：

- Python 源文件编译检查通过。
- 引擎单元测试：40 个测试方法通过。
- GUI 集成冒烟测试：有 display 时输出 `GUI_INTEGRATION_OK`。
- 无 display 时 GUI 测试可跳过。
- 主窗口圆角主体存在。
- 设置窗口保留系统标题栏，未设置全局 topmost，并通过 grab 避免主窗口抢焦点。
- 项目在无 PyYAML 时可通过内置 YAML 子集读写 `rules.yaml`。

## 后续计划

建议优先级：

1. 初始化 Git 仓库并补充 `.gitignore`。
2. 在 Windows 10/11 实机验证 PyInstaller 产物，重点检查 tkinter/customtkinter 资源、中文显示、DPI 缩放和 `rules.yaml` 持久化。
3. 为 GUI 测试配置 xvfb 或其他无头显示环境，并纳入 CI。
4. 增加更多真实 Markdown/LaTeX 混排样例测试。
5. 考虑将用户 settings 与规则元数据拆分，避免运行时修改仓库内 `rules.yaml`。
6. 评估更接近 OS 级圆角窗口的跨平台实现；当前主窗口圆角主要由 `CTkFrame` 绘制，真实透明圆角受窗口管理器限制。

## 迁移到 Tauri + PyO3（进行中）

### 总体目标

```text
Tauri 2
├── React + TypeScript + shadcn/ui（前端）
└── Rust 命令层 + PyO3
    └── 嵌入式 CPython
        └── ccw_engine.py
```

核心原则：

- 复用现有 Python 规则引擎，不急于重写为 Rust。
- 前端只调用受控的 Tauri commands，不向任意 Python 函数开放调用。
- 用户设置从「仓库内 `rules.yaml`」迁移到系统 AppData（`settings.json`），`rules.yaml` 只作只读规则元数据。
- 移除 Python 导入时的文件写副作用，配置初始化改为显式 `initialize()`。

### 两条 Python 桥接路线（待选一）

**路线 A：tauri-plugin-python（PyO3 后端，快速验证）**

- `src-tauri/src-python/main.py` 中列 `_tauri_plugin_functions`，前端经插件 API 调用。
- 已在仓库留下参考脚手架（`src-tauri/` 相关文件）。
- 生产需按插件 README：把 `src-python/` 纳入 `bundle.resources`，随应用携带 `libpython`（Windows 用 embeddable Python / python-build-standalone），venv 放在 `src-python/.venv`，并注意 `_tauri_plugin_functions` 与 Rust 注册的最小准入。
- 版本号以 crates.io 实际发布版为准（当前 `Cargo.toml` 中已注释占位）。

**路线 B：自定义 PyO3 封装（长期架构更可控，代码更多）**

- 用 `pyo3` crate 在 Rust 里 `Python::attach` -> `import ccw_engine` -> 调用。
- 优点是命令签名、错误边界、生命周期与并发完全由 Rust 掌控，未来可无痛替换成 Rust 实现。
- 注意 GIL：`Bound<'py, _>` 不能跨线程存长期状态；应序列化 Python 调用（单 worker）。

建议先走路线 A 验证整链路，稳定后再评估是否收敛到路线 B。

### 已完成的落地项

- `ccw_engine.py`：移除「模块导入即 `ensure_defaults()`」副作用，新增 `initialize(config_path=None)`。
- `gui/app.py`：改为在 `FormatterApp.__init__` 显式调用 `initialize()`，保持旧 GUI 行为。
- 新增 `python/formatter_bridge.py`、`test/test_formatter_bridge.py`。
- 新增 `frontend/`（Vite + React + TS + Tailwind v4 + shadcn 就绪）与 `src-tauri/`（Tauri 2 参考）骨架、`.gitignore`。

### 工具链要求（当前机器尚未满足）

- Node ≥ 20（本机 v26 ✅）。
- Rust 工具链：`cargo` / `rustc`（**本机未安装**）。
- Linux 侧还需：`pkg-config`、`libpython3-dev`（PyO3 动态嵌入需要 `libpython`）——**均未安装**。
- 嵌入式 CPython 运行时（推荐 `python-build-standalone` 或 Windows embeddable Python）。

### 下一步（激活脚手架的待办）

```bash
# 1) 装前端依赖（已经是 shadcn 就绪配置）
cd frontend && npm install
npm run dev          # 仅验证 Vite 前端可启动

# 2) 安装 Rust 工具链与系统库（示例，发行版差异自理）
sudo apt install rustup pkg-config libpython3-dev  # 或 rustup-init

# 3) 取用嵌入式 CPython，设置 PYO3_PYTHON / PYTHONHOME，并让 Python 能 import ccw_engine

# 4) 接入 Python 桥接（二选一）
#    路线 A：取消 src-tauri/Cargo.toml 中 tauri-plugin-python 依赖注释，按插件 README 集成
#    路线 B：在 src-tauri 增加 commands.rs/python_runtime.rs，注册 format_text 等命令

# 5) 启动
cd frontend && npm run tauri dev
```

注意：`src-tauri/src/lib.rs` 与 `src-python/main.py` 是给路线 A 的参考；若走路线 B，请按其调整 `lib.rs` 并注册命令，前端 `src/lib/tauri.ts` 中 `invoke("format_text")` 等函数名须与 Rust 命令一一对应。
