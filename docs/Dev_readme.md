# 文案净排（CopyPolish）：开发说明

本文档面向后续维护者，记录当前项目结构、架构边界、开发运行方式、测试命令、打包方式和已知注意事项。普通用户使用说明请参阅 [README.md](README.md)。

## `v0.5.0` 正式发布计划

当前 `v0.5.0` 正式发布的开发、测试、Windows 验收和发布门槛统一记录在 [v0.5.0-release-plan.md](v0.5.0-release-plan.md)。后续开发应按该计划的阶段顺序执行；在黄金样例回归测试体系建立前，不新增格式化规则或大型 UI 功能。

> 历史说明：项目早期为 Python + customtkinter 桌面 GUI（入口 `chinese_copywriting_formatter.py`，曾使用 `rules.yaml` 保存设置）。该路线已于 2026-08 彻底移除，相关文件（`gui/`、`python/formatter_bridge.py`、`run.sh`、`packaging/`、PyInstaller 工作流等）均不再存在。当前 Rust 应用使用新的 `rules.yaml` 设置实现，并通过同目录旧版 `ccw-formatter-settings.json` 进行一次性迁移；不复用旧 Python 的读写逻辑。

## 当前架构

```text
Tauri 2
├── frontend/            React + TypeScript + Tailwind v4 + shadcn/ui
│   └── 无边框标题栏：拖动、最小化、最大化、关闭
│   └── src/lib/tauri.ts 受限 command 封装（前端唯一后端入口）
│   └── src/lib/fonts.ts 统一字体令牌（全 UI 共享 --app-font-family）
└── src-tauri/
    ├── src/engine/            Rust 原生排版引擎
    │   ├── registry.rs        规则注册表（稳定 key / 元数据 / 默认启用 / key 迁移）
    │   ├── pipeline.rs        格式化主流程（保护 → 逐行规则 → 还原）
    │   ├── protection.rs      Markdown / LaTeX / URL / 邮箱保护层
    │   ├── tokenizer.rs       字符分类 + 化学式识别
    │   ├── rule_impls.rs      各条规则的纯函数实现
    │   ├── model.rs           RuleMeta / FormatRequest
    │   └── tests.rs           引擎单元测试
    ├── src/user_settings.rs   用户设置持久化（exe 同目录 rules.yaml，YAML）
    └── src/commands.rs        Tauri command 层
```

- **应用为纯 Rust 实现**：`format_text` / `get_rules` / `get_enabled_defaults` / 设置读写全部由 Rust 提供，构建与打包不依赖 Python。
- **规则注册表驱动**：规则的唯一事实来源是 `src-tauri/src/engine/registry.rs`。每条规则有稳定的机器 key（如 `spacing.cjk-latin`），展示名/分组仅存于元数据；新增规则只需在注册表追加一个 `RuleDef`，command 层、pipeline 与前端均无需改动。历史 12 条规则已全部迁移为独立注册项（效果与默认开关保持不变）。
- **用户设置**：保存在 exe 相同目录的 `rules.yaml`（YAML；见下文），首次运行自动迁移旧版 `ccw-formatter-settings.json`；读取与保存时通过 `normalize_rule_keys` 把旧版中文 key 迁移为稳定 key 并丢弃未知 key。
- **化学式识别**：tokenizer 保守识别含 Unicode 上下标、电荷标记或水合物连接符的片段（`Fe²⁺`、`SO₄²⁻`、`FeCl₂·4H₂O` 等），在规则处理前转为占位符整体保护，为后续新规则提供可靠判定单元。

## 当前开发状态

Tauri 2 迁移与 Rust 主引擎已完成，当前关键状态如下：

- `frontend/`：React + Vite + TypeScript + Tailwind v4 + shadcn/ui 界面已落地。
- `src-tauri/src/engine/`：纯 Rust 可扩展引擎（注册表 + 保护层 + 化学式识别），无 Python/PyO3 运行时依赖，也不受固定规则数量约束。
- `reference/`：历史 Python 引擎与规则目录仅作归档保留，不参与构建、打包与测试门禁。
- 设置 Dialog 使用稳定响应式布局：在视口安全边距内居中并限制最大宽高，固定 header/footer + 原生 `overflow-y-auto` 内容滚动；不再模拟 Dialog 内部拖动/缩放，主 Tauri 窗口仍可拖动和 resize。
- 设置 Dialog 支持主题切换、界面字体预设（含恢复默认）、Footer 显示完整应用版本 / 保存状态 / 设置文件路径（路径截断时悬停或键盘聚焦展示完整值）。
- 主窗口最小尺寸为 `800×600`，用于避免布局过度压缩。
- 输入框 placeholder 固定为「请在这里粘贴或输入文字」（字号更小、颜色更淡）；输出框已有真实空状态提示。
- Linux 打包目标：`.deb` / `.rpm` / `.AppImage`。
- Windows 仅提供无边框便携版：`CopyPolish.exe` 与 `CopyPolish-windows-x64.7z`，不提供安装器。

## 编码说明

全链路统一使用 UTF-8：Rust `String` 原生 UTF-8，Tauri command 走 JSON（UTF-8），设置文件 `rules.yaml`（YAML）以 UTF-8 写入/读取；中文输入输出、emoji、CJK 扩展区字符均有自动化回归测试覆盖。

## 目录结构

```text
├── README.md                      # 用户文档
├── docs/
│   └── Dev_readme.md              # 开发者文档（本文件）
├── reference/                     # 历史参考资产（不参与构建、打包与测试门禁）
│   ├── ccw_engine.py              # 历史 Python 实现
│   └── rule_catalog.yaml          # 历史规则元数据
├── frontend/                      # React/Vite/TS/Tailwind v4/shadcn-ui 界面
│   └── src/App.tsx                # 主界面：双栏编辑、设置 Dialog、防抖实时排版
├── scripts/
│   ├── check_version.py           # 版本一致性校验（CI 与本地共用）
│   └── prepare_release_version.py # 发布时把 tag 完整版本写入构建配置
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── src/engine/                # 可扩展 Rust 排版引擎
│   ├── src/user_settings.rs       # 设置持久化 + 单元测试（临时文件）
│   ├── src/commands.rs            # format_text / get_rules / get_enabled_defaults
│   │                              # / get_user_settings / save_user_settings
│   └── src/lib.rs                 # command 注册
```

## Rust 引擎要点（改动前必读）

1. 规则 key 使用稳定的英文点分标识（如 `spacing.cjk-latin`）；中文展示名不参与内部寻址。旧版设置中的中文 key 由 `normalize_rule_keys` 迁移。
2. 保护层正则依赖 lookbehind/backreference，必须使用 `fancy-regex`（`regex` crate 不支持）。
3. 占位符格式为 `\u{E000}CCWPROTECTED{n}\u{E001}`；保护范围包括 fenced code block、LaTeX 环境/display/inline/command、Markdown 图片/链接/autolink、行内代码、URL、邮箱、缩进代码行及化学式。
4. 行内占位符补空格时须过滤跨行值（fenced block 不补空格）。
5. 规则选择由 `FormatRequest.selection` 显式表达：`all` 执行全部规则，`defaults` 执行默认规则，`only` 执行指定 key，`none` 不执行任何规则。用户设置文件继续保存 `enabled: string[]`，其中空数组表示“全不选”，由前端转换为 `none`。
6. 保护规则或 tokenizer 新增/调整后，必须补充 Markdown / LaTeX / URL / 邮箱 / 代码 / 化学式边界测试。

## 用户设置持久化

- 文件位置：**exe 相同目录**（`std::env::current_exe()` 的父目录；无法定位时回退 cwd）下的 `rules.yaml`（YAML）：
  ```yaml
  enabled:
    - spacing.cjk-latin
  last_input: ...
  theme: system
  font: system
  editor_font_size: normal
  ui_scale: normal
  ```
- 字段：`enabled`（启用规则）、`last_input`（最近输入）、`theme`（`system` / `light` / `dark`）、`font`（字体预设）、`editor_font_size`（`small` / `normal` / `large` / `x-large`）、`ui_scale`（`compact` / `small` / `normal` / `large` / `x-large`）。旧版设置文件缺少新增字段时分别回落为 `normal`。
- 迁移：`rules.yaml` 不存在但同目录存在旧版 `ccw-formatter-settings.json` 时，自动读取旧 JSON 并转换写入新 YAML。
- 保存策略：写临时文件 `rules.yaml.tmp` 并 `sync_all` 后原子 rename 到 `rules.yaml`；替换前将上一份有效文件轮换为 `rules.yaml.bak`，避免中途退出产生半截文件，并为主文件损坏提供恢复来源。目标是 exe 同目录，目录不存在或无写权限时返回带完整路径的诊断错误（前端在设置弹窗中展示，提示把便携版放到可写目录）。
- 实现：`user_settings.rs` 提供 `load_from/save_to`（可注入路径）、`load_from_dir_with_status`（区分旧版本迁移、主设置损坏、备份损坏和恢复状态）、`load_from_dir`（兼容返回设置内容）与 `load_with_status`（exe 目录）；command 层暴露 `get_user_settings`（返回 `notices` 提醒列表）、`save_user_settings`（保存前过滤未知规则 key并保存字号/缩放）与 `get_settings_path`（返回设置文件完整路径，供界面显示）。
- 前端行为：启动恢复；规则开关/全选/恢复默认/清空/主题/字体即时保存，输入防抖（160ms）保存；浏览器预览回退 localStorage。字体使用固定跨平台预设与 CSS fallback 栈，不尝试枚举系统已安装字体。
- 长文本排版：普通文本使用 160ms 防抖，达到 50,000 字符使用 450ms，达到 200,000 字符使用 900ms；界面显示排版中、长文本和最近一次耗时提示，并通过序列号丢弃过期结果。
- 设置 Dialog 的版本号通过 `getAppVersion()` 读取：打包环境使用 Tauri `getVersion()`，浏览器预览使用 Vite 从 `frontend/package.json` 注入的 `__APP_VERSION__` 回退值，避免重复维护版本常量。
- 设置弹窗 Footer 保持与主界面一致的 `py-4` 纵向内边距；桌面端单行显示版本/保存状态/设置路径与操作按钮，小屏或较长保存错误时采用 flex-wrap 响应式换行。
- 预发布版本一致性：release 工作流在测试校验后调用 `scripts/prepare_release_version.py <tag>`，把 tag 的完整版本（如 `v0.5.0-pre1` → `0.5.0-pre1`）写入 package.json / package-lock.json / tauri.conf.json / Cargo.toml / Cargo.lock，仅作用于 CI 工作区，不回写源码提交；`check_version.py` 同时接受源码基础版本与同步后的完整版本两种状态。
- 主界面输入框与输出框共享 `--app-font-family`、`--editor-font-size` 和 `--editor-line-height`；主界面内容通过 `--app-ui-scale` 缩放。设置文件路径在 Footer 中单行截断，保留 `title` 和 `aria-label`，不再显示自定义悬停浮层，改用灰色点状下划线。输入框 placeholder 固定为「请在这里粘贴或输入文字」。
- 设置窗口的“字体”部分支持字号预设（13/14/16/18px），“主题”部分支持主界面缩放预设（80/90/100/110/125%）。旧 `rules.yaml` 缺少新字段时回退为标准字号与 100% 缩放。
- 设置加载提醒包括旧版本设置迁移、旧设置损坏、主设置损坏后的备份恢复、主/备份均损坏以及备份损坏但主设置可用等状态；提醒会显示在主界面提示条和设置文件区域。
- 测试约定：所有设置读写测试一律使用系统临时目录中的唯一随机文件（PID + 计数器），禁止写仓库内固定路径。
- 该文件已加入 `.gitignore`（根目录 `/rules.yaml`）。

## 开发环境

常用变量：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

依赖：Node.js/npm、Rust 工具链、Linux 下 Tauri/WebKitGTK 系统依赖。应用构建与测试无需 Python。

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

使用 Cline Act Mode 修改本地文件时，完成实现与必要验证后应自动同步本次修改结果至 Markdown 文档，并同步到 GitHub 远程仓库的相应分支：

1. 先检查 `git status --short --branch`，确认当前分支、远程跟踪分支与待提交文件；
2. 查看 diff，确保提交范围只包含本次任务相关更改；
3. 审阅仓库 Markdown 文档并按本次修改的影响范围同步：用户可见的功能、用法、限制、下载/运行方式或设置行为有变化时更新 `README.md`；架构、开发流程、实现约束、测试/构建命令、CI 或发布流程有变化时更新 `Dev_readme.md`；
4. 若本次修改不改变已有文档描述，不做无意义改写，但必须在最终结果中明确说明“已审阅 Markdown 文档，确认无需更新”；文档更新应与代码修改一起纳入本次 diff、验证、提交与推送；
5. 运行与本次修改相关的必要验证；
6. 使用清晰的 commit message 提交；
7. 推送到当前分支对应的远程分支（通常为 `origin/<当前分支>`，例如 `dev -> origin/dev`）；
8. 推送后再次检查 `git status --short --branch`，确认本地分支与远程分支已同步。

```bash
git switch dev
git pull --ff-only origin dev

# 开发、测试、提交
git push origin dev

# 功能确认后创建 PR：dev → master
gh pr create --base master --head dev
```

稳定发布仅从 `master` 创建 `v*` tag；后端引擎或其他重大功能变更应先从 `dev` 推送预发布 tag（如 `v0.5.0-pre2`，tag 可指向 `dev` 提交），Release 会自动标记为 pre-release、不占用 latest。Release 名称应直接使用 tag 名称（例如 `v0.5.0-pre2`），不要额外添加 `CopyPolish` 前缀；Release Notes 由 GitHub 自动生成后再人工审阅：

```bash
git switch dev
git pull --ff-only origin dev
git tag v0.5.0-pre2
git push origin v0.5.0-pre2
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
cargo test --manifest-path src-tauri/Cargo.toml          # Rust 引擎与设置测试
npm test --prefix frontend                                # vitest 组件测试（jsdom，10 项）
npm run build --prefix frontend                           # tsc + vite
```

与 CI 对齐的快速验证命令：

```bash
npm ci --prefix frontend
npm test --prefix frontend -- --run
npm run build --prefix frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

## 重要实现约束

1. 前端只能通过 `frontend/src/lib/tauri.ts` 的封装访问后端，不直接 `invoke`。
2. `reference/` 仅作历史参考，不定义 Rust 引擎行为。
3. 改动规则定义时，只需更新 `src-tauri/src/engine/registry.rs`、对应规则实现及 Rust 回归测试；前端通过 command 动态读取 metadata。
4. 打包资源变更需同步 `tauri.conf.json` 的 `bundle.resources` 并做安装态 smoke。
5. Tailwind 优先使用间距刻度类（如 `min-h-130` = 130 × 0.25rem），仅当数值不在刻度上时才用 `[...]` 任意值写法，避免 lint 警告。

## 后续计划

1. 真实 Windows 机器人工验收：下载 CI 的 `windows-portable` artifact，运行 `.exe`，输入 `在LeanCloud上，花了5000元` → 应输出 `在 LeanCloud 上，花了 5000 元`；验证设置弹窗 12 条规则、开关即时重排、设置文件持久化、高 DPI 显示。
2. 正式发布 `v0.5.0`：人工复核 Release 资产、Release Notes、版本号和 latest 标记。
3. 后续维护：评估格式化 hook 拆分、10 KB / 100 KB / 1 MB 性能基准、真实 Tauri E2E 测试和更多规则边界案例。

## 图标

完整桌面图标集（ico/各尺寸 PNG）由 Tauri CLI 从 `icons/icon.png` 生成：
`./frontend/node_modules/.bin/tauri icon src-tauri/icons/icon.png -o src-tauri/icons`（生成后删除 `icon.icns`——项目仅支持 Linux/Windows）。更换设计稿后重跑该命令即可。

## 持续集成

`.github/workflows/ci.yml` 与 `.github/workflows/release.yml`：

- `ci.yml` 在 `dev`、`master` 与 PR 上运行快速验证：cargo fmt + cargo clippy（`-D warnings`）+ 纯 Rust cargo test（当前包含黄金样例、设置备份恢复和 UTF-8 多字节回归）+ `git diff --check` + 前端 vitest 组件测试 + tsc/vite 构建；
- `release.yml` 在推送 `v*` tag 时运行完整发布流水线，也支持 workflow_dispatch 手动指定既有 tag 并可选标记 pre-release（tag 名含 `-` 时自动识别为预发布）；
- `tauri-build` matrix（ubuntu/windows）：Linux 构建 deb/rpm/AppImage 并上传 `bundle-ubuntu-latest`；Windows 以 `--no-bundle` 构建无边框便携 `.exe`，以临时根目录打包为只含 exe/DLL 的 `.7z` 后上传 `windows-portable`（不生成任何安装器——WiX MSI 无法处理中文产品名，且产品定位为免安装便携版）。项目不支持 macOS。
- `windows-smoke` job（windows-latest）：真实启动 GUI 冒烟测试——构建 exe → 启动进程 → 轮询等待主窗口句柄出现（最长 60 秒）→ 保持 10 秒验证稳定性 → 强制结束进程。已通过。

CI/Release 会打印 Node/npm/Rust/Cargo/系统版本，便于核对本地与 Runner 环境差异。

发布产物命名约定：

| 平台 | Artifact | Release 资产 |
| --- | --- | --- |
| Linux | `bundle-ubuntu-latest` | `CopyPolish_linux_amd64.deb` / `CopyPolish-linux-x86_64.rpm` / `CopyPolish_linux_amd64.AppImage` |
| Windows | `windows-portable` | `CopyPolish.exe` / `CopyPolish-windows-x64.7z` |

Release Notes 由 GitHub 自动生成后，必须在正式发布前重新审查并按需编辑；Release 标题保持与 tag 一致，不添加产品名前缀：

- 确认自动生成内容覆盖本次更新的大致范围（功能、修复、规则调整、构建/发布流程变化等），不要只保留 commit/PR 标题而遗漏用户可感知的变化；
- 对比上一版 Release Notes，删除或改写与既有版本重复的描述，避免把旧版本已发布的内容再次列为“本次更新”；
- 保留并按需更新固定说明（Windows 便携版命名、设置迁移、规则调整、已知限制等）；
- 最终发布前再检查资产列表与 Release Notes 是否一致，特别是 Linux `.deb` / `.rpm` / `.AppImage` 与 Windows `CopyPolish.exe` / `.7z` 是否均已说明清楚。
