# 中文文案排版助手：开发说明

本文档面向后续维护者，记录当前项目结构、架构边界、开发运行方式、测试命令、打包方式和已知注意事项。普通用户使用说明请参阅 [README.md](README.md)。

> 历史说明：项目早期为 Python + customtkinter 桌面 GUI（入口 `chinese_copywriting_formatter.py`，设置写入 `rules.yaml`）。该路线已于 2026-08 彻底移除，相关文件（`gui/`、`python/formatter_bridge.py`、`run.sh`、`packaging/`、PyInstaller 工作流等）均不再存在，其 `rules.yaml` 设置也不再读取或写入。

## 当前架构

```text
Tauri 2
├── frontend/            React + TypeScript + Tailwind v4 + shadcn/ui
│   └── src/lib/tauri.ts 受限 command 封装（前端唯一后端入口）
└── src-tauri/
    ├── src/rust_engine.rs      Rust 原生排版引擎
    ├── src/user_settings.rs    用户设置持久化（cwd JSON 文件）
    └── src/commands.rs         Tauri command 层
```

- **应用为纯 Rust 实现**：`format_text` / `get_rules` / `get_enabled_defaults` / 设置读写全部由 Rust 提供，构建与打包不依赖 Python。
- **双引擎一致性**：仓库根目录的 `ccw_engine.py` 是权威 Python 引擎，`test/compare_rust_parity.py` 用 71 条语料 × defaults/all 两模式 = 142 项检查对比 Rust 与 Python 输出，当前 **0 差异**。修改任一引擎后必须重跑（parity 脚本经 `src-tauri/examples/parity_dump.rs` 调用 Rust 引擎）。
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
│   └── src/lib.rs                 # command 注册
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
```

依赖：Node.js/npm、Rust 工具链、Linux 下 Tauri/WebKitGTK 系统依赖。应用构建无需 Python；仅运行 `test/` 的 Python 单测与 parity 校验需要本地 Python（当前 `.venv`）。

## 启动 / 构建

```bash
cd frontend && npm run tauri dev     # 开发运行
cd frontend && npm run tauri build   # Linux deb/rpm/AppImage
# 产物：src-tauri/target/release/bundle/
```

## 验证命令

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml          # 纯 Rust：11 项
npm run build --prefix frontend                           # tsc + vite
npm test --prefix frontend                                # vitest 组件测试（jsdom）
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
2. 关注 `.github/workflows/build.yml` 的 Linux/Windows 构建结果；如需分发产物再启用 upload-artifact。

## 图标

完整桌面图标集（ico/各尺寸 PNG）由 Tauri CLI 从 `icons/icon.png` 生成：
`./frontend/node_modules/.bin/tauri icon src-tauri/icons/icon.png -o src-tauri/icons`（生成后删除 `icon.icns`——项目仅支持 Linux/Windows）。更换设计稿后重跑该命令即可。

## 持续集成

`.github/workflows/build.yml`：

- `test` job（ubuntu）：cargo fmt + 默认纯 Rust cargo test + 前端 vitest 组件测试 + tsc/vite 构建；
- `tauri-build` matrix（ubuntu/windows）：Linux/Windows 双平台 `tauri build`，构建产物通过 upload-artifact 上传（bundle-ubuntu / bundle-windows），失败时上传 build-log-*。项目不支持 macOS，构建矩阵中已移除 macos-latest 与 Apple target。因默认构建不含 Python，Windows 无需任何 Python 工具链。
