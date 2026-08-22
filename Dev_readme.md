# 中文文案排版助手：开发说明

本文档面向后续维护者，记录当前项目结构、架构边界、开发运行方式、测试命令、打包方式和已知注意事项。普通用户使用说明请参阅 [README.md](README.md)。

> 历史说明：项目早期为 Python + customtkinter 桌面 GUI（入口 `chinese_copywriting_formatter.py`，设置写入 `rules.yaml`）。该路线已于 2026-08 彻底移除，相关文件（`gui/`、`python/formatter_bridge.py`、`run.sh`、`packaging/`、PyInstaller 工作流等）均不再存在，其 `rules.yaml` 设置也不再读取或写入。

## 当前架构

```text
Tauri 2
├── frontend/            React + TypeScript + Tailwind v4 + shadcn/ui
│   └── src/lib/tauri.ts 受限 command 封装（前端唯一后端入口）
└── src-tauri/
    ├── src/rust_engine.rs      Rust 原生排版引擎（主路径）
    ├── src/user_settings.rs    用户设置持久化（cwd JSON 文件）
    ├── src/commands.rs         Tauri command 层
    ├── src/python_runtime.rs   PyO3 运行时（仅兜底路径使用）
    └── src-python/main.py      Python 桥接模块（兜底）
        └── ccw_engine.py       权威 Python 引擎（兜底 + parity 基准）
```

- **格式化主路径是 Rust**：`format_text` / `get_rules` / `get_enabled_defaults` 默认完全由 Rust 实现，**默认构建不含 Python 依赖**。启用 `python-fallback` feature（`cargo build/test --features python-fallback`）时，Rust 出错或元数据缺失会回退 PyO3 → `src-python/main.py` → `ccw_engine.py`。
- **双引擎一致性**：`test/compare_rust_parity.py` 用 71 条语料 × defaults/all 两模式 = 142 项检查对比 Rust 与 Python 输出，当前 **0 差异**。修改任一引擎后必须重跑。
- **用户设置**：保存在当前工作目录的 `ccw-formatter-settings.json`（见下文），与旧版 `rules.yaml` 设置无关。

## 目录结构

```text
├── ccw_engine.py                  # 权威 Python 引擎（13 条规则 + 保护层），parity 基准
├── rules.yaml                     # 规则元数据（只读参考，不参与构建与打包）
├── frontend/                      # React/Vite/TS/Tailwind v4/shadcn-ui 界面
│   └── src/App.tsx                # 主界面：双栏编辑、设置 Dialog、防抖实时排版
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── examples/parity_dump.rs    # parity 语料 dump 示例（供 compare 脚本调用）
│   ├── src/rust_engine.rs         # Rust 排版引擎（规则 + 保护层管线）
│   ├── src/user_settings.rs       # 设置持久化 + 单元测试（临时文件）
│   ├── src/commands.rs            # format_text / get_rules / get_enabled_defaults
│   │                              # / get_user_settings / save_user_settings
│   ├── src/python_runtime.rs      # PyO3 内嵌 CPython 3.14
│   ├── src/lib.rs                 # command 注册 + 资源路径解析
│   └── src-python/main.py         # Python 桥接（开发树与打包 _up_/ 布局兼容）
└── test/
    ├── test_ccw_engine.py         # Python 引擎单元测试（40 项，设置测试用 tempfile）
    └── compare_rust_parity.py     # 双引擎一致性校验
```

## Rust 引擎要点（改动前必读）

1. **规则 key 必须与 Python `_slug()` 输出一致**（如 `遇到完整的英文整句_特殊名词_其内容使用半角标点`），不能用显示名——曾导致部分规则静默失效。
2. 保护层正则依赖 lookbehind/backreference，必须使用 `fancy-regex`（`regex` crate 不支持）。
3. 占位符格式与 Python 完全一致：`\u{E000}CCWPROTECTED{n}\u{E001}`；保护模式共 13 类（fenced code block、LaTeX 环境/display/inline/command、Markdown 图片/链接/autolink、行内代码、URL、邮箱、缩进代码行）。
4. 行内占位符补空格时须过滤跨行值（fenced block 不补空格），与 Python `inline_ph` 过滤一致。
5. 中英文/中文数字/数字单位/全角标点四项基础空格收尾函数始终执行，不受规则开关影响（引擎既定设计）。
6. 保护规则新增或调整后，必须补充 Markdown / LaTeX / URL / 邮箱 / 代码边界测试，并重跑 parity。

## 用户设置持久化

- 文件位置：**当前工作目录**下的 `ccw-formatter-settings.json`：
  ```json
  { "enabled": ["..."], "last_input": "..." }
  ```
- 实现：`user_settings.rs` 提供 `load_from/save_to`（可注入路径）与 `load/save`（cwd）；command 层暴露 `get_user_settings`（文件缺失返回 `null`）与 `save_user_settings`。
- 前端行为：启动恢复；规则开关/全选/恢复默认/清空即时保存，输入防抖（160ms）保存；浏览器预览回退 localStorage。
- 测试约定：所有设置读写测试一律使用系统临时目录中的唯一随机文件（PID + 计数器），禁止写仓库内固定路径。
- 该文件已加入 `.gitignore`。

## 开发环境

常用变量：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export PYO3_PYTHON=/usr/bin/python3.14
export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}
```

依赖：Node.js/npm、Rust 工具链、Python 3.14 开发库（`libpython3.14.so`）、Linux 下 Tauri/WebKitGTK 系统依赖。

## 启动 / 构建

```bash
cd frontend && npm run tauri dev     # 开发运行
cd frontend && npm run tauri build   # Linux deb/rpm/AppImage
# 产物：src-tauri/target/release/bundle/
```

安装态资源注意：Linux bundle 中 `../xxx` 形式的资源落在 `_up_/` 子目录，`src-python/main.py` 的 `_UP_ROOT` 探测逻辑不可删。

## 验证命令

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml          # 默认纯 Rust：11 项
cargo test --manifest-path src-tauri/Cargo.toml --features python-fallback  # 含 PyO3：15 项
npm run build --prefix frontend                           # tsc + vite
.venv/bin/python -m unittest discover -s test             # Python 40 项
.venv/bin/python test/compare_rust_parity.py              # 必须 0 差异
```

## 重要实现约束

1. 前端只能通过 `frontend/src/lib/tauri.ts` 的封装访问后端，不直接 `invoke`。
2. `ccw_engine.py` 不导入任何 GUI/UI 模块；Rust 与 Python 输出保持逐字节一致（parity 保证）。
3. 改动规则定义时，同步更新：`ccw_engine.py` RULES、`rust_engine::default_rules()`、前端 fallback 列表（如有）、parity 语料。
4. 打包资源变更需同步 `tauri.conf.json` 的 `bundle.resources` 并做安装态 smoke。

## 后续计划

1. 真实窗口 smoke 验证（WSL2 图形栈限制暂无法本地执行）：输入 `在LeanCloud上，花了5000元` → 应输出 `在 LeanCloud 上，花了 5000 元`；验证设置弹窗 13 条规则、开关即时重排、设置文件持久化。
2. 替换正式图标集（当前为临时生成的 RGBA PNG）。
3. Windows/macOS 打包与嵌入式 CPython 分发策略验证（若启用 python-fallback feature）。
4. 评估从 bundle 资源移除 `src-python/main.py` 与 `ccw_engine.py`（默认纯 Rust 构建已不使用它们；保留是为 feature 构建与 parity 基准）。
