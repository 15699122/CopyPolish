# 中文文案排版助手

中文文案排版助手（Chinese Copywriting Formatter）是一款本地桌面端中文文案排版工具，用于按照 [chinese-copywriting-guidelines](https://github.com/sparanoid/chinese-copywriting-guidelines) 的简体中文文案规范，自动整理中文、英文、数字、单位和标点之间的格式。

项目界面为 Tauri 2 + React + shadcn/ui 前端，后端为纯 Rust 排版引擎。左侧输入原文，右侧实时显示规范化结果；规则可逐条启用或关闭。

## 功能亮点

- **实时排版**：输入或粘贴文本后自动生成格式化结果。
- **左右双栏编辑**：输入区和输出区并排显示，适合对照检查。
- **规则可配置**：设置窗口支持逐条启用/停用规则，也支持全选、全不选和恢复默认。
- **争议规则默认关闭**：例如链接之间增加空格、简体中文使用直角引号，可按个人习惯开启。
- **Markdown / LaTeX 保护**：尽量避免误改代码块、行内代码、链接、图片链接、URL、邮箱和公式内容。
- **低依赖规则引擎**：Rust 原生引擎为主路径；`ccw_engine.py` 仅作为兜底保留。
- **现代桌面壳**：使用 Tauri 2 承载 React/shadcn/ui，前端只通过受限 Tauri commands 访问后端。
- **用户设置持久化**：规则开关与最近输入保存在当前工作目录的 `ccw-formatter-settings.json`，启动时自动恢复。
- **浏览器预览回退**：前端在非 Tauri 浏览器环境中也可以预览界面和基础交互（设置回退到 localStorage），便于开发调试。

## 当前 Tauri 版状态

新版 Tauri 2 迁移已完成以下关键链路：

- `frontend/`：React + Vite + TypeScript + Tailwind v4 + shadcn/ui 界面已落地。
- `src-tauri/`：**纯 Rust 实现**（`rust_engine.rs` + `user_settings.rs`），无 Python/PyO3 依赖；仓库根目录的 `ccw_engine.py` 仅作为 `test/compare_rust_parity.py` 的权威基准保留。
- Rust 单元测试覆盖引擎核心样例、设置持久化与 UTF-8 多字节回归（emoji / CJK 扩展区 / 全角字符）。
- Linux 打包已能生成：
  - `.deb`
  - `.rpm`
  - `.AppImage`
- Windows 仅提供**便携版**（无安装器）：
  - `chinese-copywriting-formatter.exe`（单文件，直接运行）
  - `chinese-copywriting-formatter-windows-x64.7z`（压缩包）

Windows 便携版依赖系统的 WebView2 Evergreen Runtime（Windows 10/11 一般已内置）；如缺失，请从微软官网安装。CI 已包含 Windows hosted runner 真实启动冒烟测试（进程存活 + 主窗口出现 + 10 秒稳定性校验）。

### 版本发布

推送 `v*` tag 即可自动发布：

```bash
git tag v0.1.1 && git push origin v0.1.1
```

CI 会构建双平台产物并自动创建 GitHub Release（资产统一使用 ASCII 文件名：`.exe`、`.7z`、`_linux_amd64.deb`、`-linux-x86_64.rpm`、`_linux_amd64.AppImage`），Release Notes 由 GitHub 自动生成。

### 编码说明

全链路统一使用 UTF-8：Rust `String` 原生 UTF-8，Tauri command 走 JSON（UTF-8），设置文件 `ccw-formatter-settings.json` 以 UTF-8 写入/读取；中文输入输出、emoji、CJK 扩展区字符均有自动化回归测试覆盖。

当前仍建议把 Tauri 版视为迁移中的开发版。

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

### Tauri 开发环境

Tauri 版开发需要：

- Node.js / npm；
- Rust 工具链（`cargo` / `rustc` / `rustfmt`）；
- Linux 下还需要 Tauri/WebKitGTK 相关系统依赖。

（运行 Python 侧 parity 校验时才需要 Python 环境；应用构建本身无需 Python。）

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

构建 Windows 便携版（不生成安装器）：

```bash
cd frontend
npm run tauri build -- --no-bundle
# 产物：src-tauri/target/release/chinese-copywriting-formatter.exe
```

## 下载与发布

每次 push 到 master 后，CI（GitHub Actions `build` workflow）自动构建并上传 artifact：

| 平台 | Artifact | 内容 |
| --- | --- | --- |
| Linux | `bundle-ubuntu-latest` | `.deb` / `.rpm` / `.AppImage` |
| Windows | `windows-portable` | `chinese-copywriting-formatter.exe` + `chinese-copywriting-formatter-windows-x64.7z` |

Windows 仅提供便携版 `.exe` 与 `.7z` 压缩包两种格式，不提供安装器；运行需系统已安装 WebView2 Runtime。

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

## 用户设置

用户设置（已启用规则 + 最近输入）保存在**当前工作目录**下的：

```text
ccw-formatter-settings.json
```

示例：

```json
{
  "enabled": ["中英文之间需要增加空格", "数字使用半角字符"],
  "last_input": "在LeanCloud上，花了5000元"
}
```

- 启动时自动恢复；文件缺失或损坏时使用内置默认规则集。
- 该文件已加入 `.gitignore`，不会进入版本库。
- 旧版 customtkinter GUI 的 `rules.yaml` 设置已废弃，不再读取或写入。

## 测试

完整 Python 测试（规则引擎兜底实现）：

```bash
.venv/bin/python -m unittest discover -s test
```

双引擎 parity 对比（Python vs Rust，71 条语料 × 两种模式）：

```bash
.venv/bin/python test/compare_rust_parity.py
```

Tauri/Rust 检查：

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run build --prefix frontend
```

## 开发文档

项目结构、架构说明、验证命令、打包细节和后续计划请参阅：

- [Dev_readme.md](Dev_readme.md)
