# 中文文案排版助手

中文文案排版助手（Chinese Copywriting Formatter）是一款本地桌面端中文文案排版工具，用于按照 [chinese-copywriting-guidelines](https://github.com/sparanoid/chinese-copywriting-guidelines) 的简体中文文案规范，自动整理中文、英文、数字、单位和标点之间的格式。

项目当前有两条界面路线：

- **新版主线（进行中）**：Tauri 2 + React + shadcn/ui 前端，Rust/PyO3 在进程内调用现有 Python 规则引擎 `ccw_engine.py`。该路线已经可以构建、运行、通过 Rust/PyO3 桥接测试，并可生成 Linux deb/rpm/AppImage 包。
- **旧版可用界面**：Python + `customtkinter` GUI，仍可按下文方式本地运行。

两条路线共享同一套核心规则引擎：左侧输入原文，右侧实时显示规范化结果；规则可逐条启用或关闭。

## 功能亮点

- **实时排版**：输入或粘贴文本后自动生成格式化结果。
- **左右双栏编辑**：输入区和输出区并排显示，适合对照检查。
- **规则可配置**：设置窗口支持逐条启用/停用规则，也支持全选、全不选和恢复默认。
- **争议规则默认关闭**：例如链接之间增加空格、简体中文使用直角引号，可按个人习惯开启。
- **Markdown / LaTeX 保护**：尽量避免误改代码块、行内代码、链接、图片链接、URL、邮箱和公式内容。
- **低依赖规则引擎**：核心排版逻辑在 `ccw_engine.py`，不依赖 GUI；`rules.yaml` 可由内置轻量 YAML 读写器处理，PyYAML 不是必需依赖。
- **现代桌面壳迁移中**：新版使用 Tauri 2 承载 React/shadcn/ui，前端只通过受限 Tauri commands 调用 Rust/PyO3，避免向 UI 暴露任意 Python 调用。
- **Rust 原生后端迁移中**：新版 Tauri 已新增第一版 Rust 文字处理引擎，参考 `typeset-rs` 的字符分类与渲染管线思路，当前采用 Rust 优先、Python/PyO3 回退的保守策略。
- **浏览器预览回退**：新版前端在非 Tauri 浏览器环境中也可以预览界面和基础交互，便于开发调试。
- **旧版 GUI 仍可运行**：customtkinter 版本固定浅色界面，主窗口为自绘圆角窗口，设置窗口保留系统标题栏。

> 说明：旧版 customtkinter GUI 会把规则启用状态与最近输入写入 `rules.yaml`。新版 Tauri 路线当前优先完成格式化与规则开关链路，后续计划改为把用户设置持久化到系统 AppData，而不是直接修改随包分发的 `rules.yaml`。

## 当前 Tauri 版状态

新版 Tauri 2 迁移已完成以下关键链路：

- `frontend/`：React + Vite + TypeScript + Tailwind v4 + shadcn/ui 界面已落地。
- `src-tauri/`：Tauri 2 应用已接入自定义 PyO3 运行时。
- `src-tauri/src/rust_engine.rs`：新增第一版 Rust 原生排版引擎，已覆盖基础中英文/数字空格、数字单位、标点、专有名词和缩写等核心样例。
- `src-tauri/src-python/main.py`：作为 Python 桥接模块，调用 `ccw_engine.py` 的格式化和规则读取能力。
- Rust 单元测试已覆盖 `PyO3 → main.py → ccw_engine.py`：
  - 读取 13 条规则；
  - 读取 11 条默认规则；
  - 格式化 `在LeanCloud上，花了5000元` 为 `在 LeanCloud 上，花了 5000 元`。
- Linux 打包已能生成：
  - `.deb`
  - `.rpm`
  - `.AppImage`

当前仍建议把 Tauri 版视为迁移中的开发版。正式分发前还需要继续确认安装后资源路径、用户设置持久化、真实图标集和多平台打包策略。

## 当前支持的规则

当前内置 13 条规则：

| 分类 | 规则 |
| --- | --- |
| 空格 | 中英文之间需要增加空格 |
| 空格 | 中文与数字之间需要增加空格 |
| 空格 | 数字与单位之间需要增加空格 |
| 空格 | 全角标点与其他字符之间不加空格 |
| 空格 | 用 `text-spacing` 来挽救 |
| 标点符号 | 不重复使用标点符号 |
| 全角和半角 | 使用全角中文标点 |
| 全角和半角 | 数字使用半角字符 |
| 全角和半角 | 遇到完整的英文整句、特殊名词，其内容使用半角标点 |
| 名词 | 专有名词使用正确的大小写 |
| 名词 | 不要使用不地道的缩写 |
| 争议 | 链接之间增加空格，默认关闭 |
| 争议 | 简体中文使用直角引号，默认关闭 |

> 说明：排版规则尽量贴近规范，但自然语言文本存在上下文差异。建议在重要文案发布前人工复核一次。

## 安装要求

### 旧版 Python GUI

推荐环境：

- Python 3.10 或更高版本；当前开发环境使用 Python 3.14。
- tkinter。
  - Debian / Ubuntu 可安装：
    ```bash
    sudo apt install python3-tk
    ```
- customtkinter。

安装 GUI 依赖：

```bash
python3 -m pip install customtkinter
```

如果使用项目本地虚拟环境，可执行：

```bash
.venv/bin/python -m pip install customtkinter
```

`rules.yaml` 的读写优先使用 PyYAML；如果未安装 PyYAML，程序会回退到项目内置的轻量 YAML 子集读写器。

### 新版 Tauri 开发环境

Tauri 版开发需要：

- Node.js / npm；
- Rust 工具链（`cargo` / `rustc` / `rustfmt`）；
- Python 开发库与动态库（当前开发环境使用 Python 3.14，PyO3 通过 `/usr/bin/python3.14` 与 `libpython3.14.so` 验证）；
- Linux 下还需要 Tauri/WebKitGTK 相关系统依赖。

开发环境常用变量：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export PYO3_PYTHON=/usr/bin/python3.14
export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}
```

## 启动方式

### 新版 Tauri GUI（开发版）

```bash
cd frontend
npm run tauri dev
```

构建 Linux 安装包：

```bash
cd frontend
npm run tauri build
```

构建产物位于：

```text
src-tauri/target/release/bundle/
```

### 旧版 Python GUI

在项目目录中运行：

```bash
./run.sh
```

或直接运行入口文件：

```bash
python chinese_copywriting_formatter.py
```

如果使用项目虚拟环境：

```bash
.venv/bin/python chinese_copywriting_formatter.py
```

## 基本使用

1. 启动应用。
2. 在左侧输入框输入或粘贴中文文案。
3. 右侧输出框会自动显示排版后的结果。
4. 点击 **复制结果** 将输出复制到剪贴板。
5. 点击 **清空输入** 清除当前文本。
6. 点击 **设置** 打开规则窗口：
   - 勾选或取消勾选单条规则；
   - 使用全选、全不选、恢复默认；
   - 点击完成或直接关闭窗口保存并返回主界面。

## 文本保护范围

格式化过程中会优先保护以下内容，降低误改概率：

- Markdown fenced code block；
- Markdown 行内代码；
- Markdown 链接和图片；
- 自动链接形式的 URL / 邮箱；
- 普通 URL 和邮箱；
- LaTeX 行内公式、展示公式、常见环境和命令。

例如 `$E=mc^2$`、代码块中的符号、链接地址通常不会被规则拆开或替换。

## 配置文件

`rules.yaml` 同时保存两类内容：

- `rules`：规则元数据，包括章节、默认启用状态、是否争议规则。
- `settings`：用户本地状态，包括已启用规则和最近输入。

示例：

```yaml
settings:
  enabled:
  last_input: ""
```

如需恢复默认状态，可关闭程序后手动清空 `settings.enabled` 与 `settings.last_input`，或在设置窗口中点击恢复默认。

## 测试

完整 Python 测试：

```bash
.venv/bin/python -m unittest discover -s test
```

规则引擎单元测试：

```bash
.venv/bin/python -m unittest -v test.test_ccw_engine
```

GUI 集成冒烟测试：

```bash
.venv/bin/python -m test.gui_integration_test
```

GUI 测试需要可用图形显示环境；没有 display 时会跳过。

Tauri/Rust 检查：

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run build --prefix frontend
```

## Windows 打包

Windows 10/11 上可在项目根目录运行：

```bat
packaging\build_win.bat
```

脚本会使用 PyInstaller 构建单文件窗口程序，并将结果移动到：

```text
build\windows\chinese_copywriting_formatter.exe
```

项目也包含 GitHub Actions 工作流 `.github/workflows/build-windows.yml`，用于在 Windows runner 上生成 exe 构建产物。

## 开发文档

项目结构、架构说明、验证命令、打包细节和后续计划请参阅：

- [Dev_readme.md](Dev_readme.md)
