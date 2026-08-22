# 文案净排（CopyPolish）：开发说明

本文档面向后续维护者，记录当前项目结构、架构边界、开发运行方式、测试命令、打包方式和已知注意事项。普通用户使用说明请参阅 [README.md](README.md)。

> 历史说明：项目早期为 Python + customtkinter 桌面 GUI（入口 `chinese_copywriting_formatter.py`，设置写入 `rules.yaml`）。该路线已于 2026-08 彻底移除，相关文件（`gui/`、`python/formatter_bridge.py`、`run.sh`、`packaging/`、PyInstaller 工作流等）均不再存在，其 `rules.yaml` 设置也不再读取或写入。

## 当前架构

```text
Tauri 2
├── frontend/            React + TypeScript + Tailwind v4 + shadcn/ui
│   └── 无边框标题栏：拖动、最小化、最大化、关闭
│   └── src/lib/tauri.ts 受限 command 封装（前端唯一后端入口）
└── src-tauri/
    ├── src/rust_engine.rs      Rust 原生排版引擎
    ├── src/user_settings.rs    用户设置持久化（exe 同目录 rules.yaml，YAML）
    └── src/commands.rs         Tauri command 层
```

- **应用为纯 Rust 实现**：`format_text` / `get_rules` / `get_enabled_defaults` / 设置读写全部由 Rust 提供，构建与打包不依赖 Python。
- **双引擎一致性**：仓库根目录的 `ccw_engine.py` 是权威 Python 引擎，`test/compare_rust_parity.py` 用 71 条语料 × defaults/all 两模式 = 142 项检查对比 Rust 与 Python 输出，当前 **0 差异**。修改任一引擎后必须重跑（parity 脚本经 `src-tauri/examples/parity_dump.rs` 调用 Rust 引擎）。
- **用户设置**：保存在 exe 相同目录的 `rules.yaml`（YAML；见下文），首次运行自动迁移旧版 `ccw-formatter-settings.json`。

## 当前开发状态

Tauri 2 迁移与 Rust 主引擎已完成，当前关键状态如下：

- `frontend/`：React + Vite + TypeScript + Tailwind v4 + shadcn/ui 界面已落地。
- `src-tauri/`：纯 Rust 实现（`rust_engine.rs` + `user_settings.rs`），无 Python/PyO3 运行时依赖。
- 仓库根目录的 `ccw_engine.py` 仅作为 parity 权威基准保留，不参与应用构建和打包。
- 设置 Dialog 已改为稳定响应式布局：固定 header/footer + 原生 `overflow-y-auto` 内容滚动；不再模拟 Dialog 内部拖动/缩放，主 Tauri 窗口仍可拖动和 resize。
- 主窗口最小尺寸为 `800×600`，用于避免布局过度压缩。
- 输入框已有示例 placeholder；输出框已有真实空状态提示。
- Linux 打包目标：`.deb` / `.rpm` / `.AppImage`。
- Windows 仅提供无边框便携版：`CopyPolish.exe` 与 `CopyPolish-windows-x64.7z`，不提供安装器。

## 编码说明

全链路统一使用 UTF-8：Rust `String` 原生 UTF-8，Tauri command 走 JSON（UTF-8），设置文件 `rules.yaml`（YAML）以 UTF-8 写入/读取；中文输入输出、emoji、CJK 扩展区字符均有自动化回归测试覆盖。

## 目录结构

```text
├── ccw_engine.py                  # 权威 Python 引擎（12 条规则 + 保护层），parity 基准
├── rule_catalog.yaml              # 规则元数据（只读参考，不参与构建与打包）
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

- 文件位置：**exe 相同目录**（`std::env::current_exe()` 的父目录；无法定位时回退 cwd）下的 `rules.yaml`（YAML）：
  ```yaml
  enabled:
    - 中英文之间需要增加空格
  last_input: ...
  theme: system
  ```
- 字段：`enabled`（启用规则）、`last_input`（最近输入）、`theme`（`system` / `light` / `dark`）。旧版设置文件无 `theme` 时默认回落为 `system`。
- 迁移：`rules.yaml` 不存在但同目录存在旧版 `ccw-formatter-settings.json` 时，自动读取旧 JSON 并转换写入新 YAML。
- 保存策略：写临时文件 `rules.yaml.tmp` 后原子 rename 到 `rules.yaml`，避免中途退出产生半截文件；目标是 exe 同目录，目录不存在或无写权限时返回带完整路径的诊断错误（前端在设置弹窗中展示，提示把便携版放到可写目录）。
- 实现：`user_settings.rs` 提供 `load_from/save_to`（可注入路径）、`load_from_dir(dir)`（可注入目录，含迁移逻辑）与 `load/save`（exe 目录）；command 层暴露 `get_user_settings`（文件缺失返回 `null`）、`save_user_settings`（保存前过滤未知规则 key）与 `get_settings_path`（返回设置文件完整路径，供界面显示）。
- 前端行为：启动恢复；规则开关/全选/恢复默认/清空/主题即时保存，输入防抖（160ms）保存；浏览器预览回退 localStorage。
- 测试约定：所有设置读写测试一律使用系统临时目录中的唯一随机文件（PID + 计数器），禁止写仓库内固定路径。
- 该文件已加入 `.gitignore`（根目录 `/rules.yaml`）。

## 开发环境

常用变量：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

依赖：Node.js/npm、Rust 工具链、Linux 下 Tauri/WebKitGTK 系统依赖。应用构建无需 Python；仅运行 `test/` 的 Python 单测与 parity 校验需要本地 Python（当前 `.venv`）。

工具链版本通过仓库文件固定：

- Node.js：`.nvmrc`（当前 22，与 GitHub Actions 一致）；
- Rust：`rust-toolchain.toml`（当前 1.98.0，包含 rustfmt / clippy）；
- 前端依赖：`frontend/package-lock.json` + `npm ci`；
- Rust 依赖：`src-tauri/Cargo.lock`。

建议本地开发前执行：

```bash
nvm use
npm ci --prefix frontend
```

依赖更新不在普通 CI 构建中自动执行；Dependabot 会为 npm、Cargo 和 GitHub Actions 依赖创建独立 PR，合并前必须通过 CI。

## 分支开发流程

默认在 `dev` 分支开发，`master` 只保留已验证的稳定代码：

```bash
git switch dev
git pull --ff-only origin dev

# 开发、测试、提交
git push origin dev

# 功能确认后创建 PR：dev → master
gh pr create --base master --head dev
```

发布仅从 `master` 创建 `v*` tag：

```bash
git switch master
git pull --ff-only origin master
git tag v0.3.x
git push origin v0.3.x
```

## 启动 / 构建

```bash
cd frontend && npm run tauri dev       # 开发运行
cd frontend && npm run tauri build     # Linux deb/rpm/AppImage
cd frontend && npm run tauri build -- --no-bundle  # Windows 便携 exe（不生成安装器）
```

构建产物位置：

- Linux bundle：`src-tauri/target/release/bundle/`
- Windows 便携 exe：`src-tauri/target/release/chinese-copywriting-formatter.exe`，发布流水线会重命名为 `CopyPolish.exe`

Windows `.7z` 压缩包约定：根目录直接包含 `CopyPolish.exe` 及构建目录中存在的旁置 DLL 依赖（如有），不得包含 `dist`、`windows` 或其他上级目录。

## 验证命令

开发提交前建议至少运行：

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml          # 纯 Rust：16 项
npm test --prefix frontend                                # vitest 组件测试（jsdom，9 项）
npm run build --prefix frontend                           # tsc + vite
.venv/bin/python -m unittest discover -s test             # Python 40 项
.venv/bin/python test/compare_rust_parity.py              # 必须 0 差异
```

与 CI 对齐的快速验证命令：

```bash
npm ci --prefix frontend
npm test --prefix frontend -- --run
npm run build --prefix frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

## 重要实现约束

1. 前端只能通过 `frontend/src/lib/tauri.ts` 的封装访问后端，不直接 `invoke`。
2. `ccw_engine.py` 不导入任何 GUI/UI 模块；Rust 与 Python 输出保持逐字节一致（parity 保证）。
3. 改动规则定义时，同步更新：`ccw_engine.py` RULES、`rust_engine::default_rules()`、前端 fallback 列表（如有）、parity 语料。
4. 打包资源变更需同步 `tauri.conf.json` 的 `bundle.resources` 并做安装态 smoke。

## 后续计划

1. 真实 Windows 机器人工验收：下载 CI 的 `windows-portable` artifact，运行 `.exe`，输入 `在LeanCloud上，花了5000元` → 应输出 `在 LeanCloud 上，花了 5000 元`；验证设置弹窗 12 条规则、开关即时重排、设置文件持久化、高 DPI 显示。
2. GUI 功能迭代：继续完善错误态、长文本性能提示、无障碍焦点顺序与键盘操作体验。

## 图标

完整桌面图标集（ico/各尺寸 PNG）由 Tauri CLI 从 `icons/icon.png` 生成：
`./frontend/node_modules/.bin/tauri icon src-tauri/icons/icon.png -o src-tauri/icons`（生成后删除 `icon.icns`——项目仅支持 Linux/Windows）。更换设计稿后重跑该命令即可。

## 持续集成

`.github/workflows/ci.yml` 与 `.github/workflows/release.yml`：

- `ci.yml` 在 `dev`、`master` 与 PR 上运行快速验证：cargo fmt + 纯 Rust cargo test（16 项，含 UTF-8 多字节回归）+ 前端 vitest 组件测试 + tsc/vite 构建；
- `release.yml` 仅在 `v*` tag 或手动指定既有 `v*` tag 时运行完整发布流水线；
- `tauri-build` matrix（ubuntu/windows）：Linux 构建 deb/rpm/AppImage 并上传 `bundle-ubuntu-latest`；Windows 以 `--no-bundle` 构建无边框便携 `.exe`，以临时根目录打包为只含 exe/DLL 的 `.7z` 后上传 `windows-portable`（不生成任何安装器——WiX MSI 无法处理中文产品名，且产品定位为免安装便携版）。项目不支持 macOS。
- `windows-smoke` job（windows-latest）：真实启动 GUI 冒烟测试——构建 exe → 启动进程 → 轮询等待主窗口句柄出现（最长 60 秒）→ 保持 10 秒验证稳定性 → 强制结束进程。已通过。

CI/Release 会打印 Node/npm/Rust/Cargo/系统版本，便于核对本地与 Runner 环境差异。

发布产物命名约定：

| 平台 | Artifact | Release 资产 |
| --- | --- | --- |
| Linux | `bundle-ubuntu-latest` | `CopyPolish_linux_amd64.deb` / `CopyPolish-linux-x86_64.rpm` / `CopyPolish_linux_amd64.AppImage` |
| Windows | `windows-portable` | `CopyPolish.exe` / `CopyPolish-windows-x64.7z` |

Release Notes 由 GitHub 自动生成后追加固定说明（Windows 便携版命名、设置迁移、规则调整等）。
