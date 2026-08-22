# 中文文案排版助手

中文文案排版助手（Chinese Copywriting Formatter）是一款本地桌面端中文文案排版工具，用于按照 [chinese-copywriting-guidelines](https://github.com/sparanoid/chinese-copywriting-guidelines) 的简体中文文案规范，自动整理中文、英文、数字、单位和标点之间的格式。

项目界面为 Tauri 2 + React + shadcn/ui 前端，后端以 Rust 原生排版引擎为主路径（`ccw_engine.py` 经 PyO3 作为兜底保留）。左侧输入原文，右侧实时显示规范化结果；规则可逐条启用或关闭。

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

### Tauri 开发环境

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
