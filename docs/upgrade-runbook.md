# Node / Rust / Tauri / React 升级 Runbook

本 Runbook 用于升级开发工具链、运行时和直接依赖。目标是让升级可审阅、可回滚，且不把临时构建工作区或生产凭据带入提交。

## 1. 升级原则

- 一次只处理一个主题：Node、Rust、前端依赖、Rust 依赖或 Tauri/React 联动升级；
- 优先接受 Dependabot 的小范围 PR，不在同一个提交中混入无关格式化或规则行为变化；
- 始终提交 lockfile 变化：`frontend/package-lock.json` 和/或 `src-tauri/Cargo.lock`；
- Tauri 与 React/Vite 相关升级必须额外执行桌面 smoke，不以浏览器预览通过代替桌面验证；
- 升级失败时优先恢复 lockfile 和版本固定文件，不删除审计记录或绕过验证门禁。

## 2. 升级前基线

在仓库根目录执行：

```bash
git status --short --branch
git switch dev
git pull --ff-only origin dev

node --version
npm --version
rustc --version
cargo --version
python3 scripts/verify.py --profile checks
python3 scripts/verify.py --profile audit
```

记录升级前的基线，至少包括：

- `.nvmrc`、`rust-toolchain.toml`、`frontend/package.json`、`src-tauri/Cargo.toml` 的版本状态；
- `frontend/package-lock.json`、`src-tauri/Cargo.lock` 是否干净；
- `python3 scripts/generate_licenses.py` 生成的 `docs/licenses.md` 是否无缺失许可证字段；
- `python3 scripts/verify.py --profile rust` 和 `--profile frontend` 是否通过。

## 3. Node.js 升级

1. 修改 `.nvmrc`，选择当前项目和 CI 都支持的 Node 主版本；
2. 同步检查 `.github/workflows/ci.yml`、`.gitlab-ci.yml` 和 `package.json` 的 engines；
3. 使用新 Node 安装依赖并刷新 lockfile：

   ```bash
   nvm use
   npm ci --prefix frontend
   ```

4. 运行前端测试、构建、依赖审计和许可证清单生成；
5. 若 Node 主版本改变，至少在 Linux 上完成一次 Tauri `--no-bundle --ci` smoke。

不要直接提交 `frontend/node_modules` 或 `frontend/dist`；它们是可再生目录并受 `.gitignore` 忽略。

## 4. Rust 工具链与依赖升级

### 4.1 工具链升级

1. 修改 `rust-toolchain.toml` 的 `channel`，保留 `rustfmt` 和 `clippy` 组件；
2. 使用新工具链运行格式化、Clippy、默认测试和 TUI 测试；
3. 观察编译器 warning、Tauri/Wry/GTK 兼容性和构建时间变化；
4. 不要仅因编译器升级而批量改写无关代码。

### 4.2 Cargo 依赖升级

在 `src-tauri` 目录或仓库根目录使用 Cargo 的显式 manifest：

```bash
cargo update --manifest-path src-tauri/Cargo.toml
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 >/dev/null
```

如果只升级一个 crate，应优先使用精确版本约束和最小 lockfile 变化；升级 Tauri 时同时检查 `tauri-build`、CLI、Wry 和 Tao 的版本协调性。

## 5. Tauri / React / Vite 联动升级

1. 先升级 `@tauri-apps/api` 与 `@tauri-apps/cli`，再评估 Rust `tauri` / `tauri-build`；
2. React、Vite、TypeScript 和 Vitest 应分开提交，除非升级本身要求联动；
3. 检查 `frontend/src/lib/tauri.ts` 的 IPC 封装、Tauri capabilities、`tauri.conf.json` 的 CSP 和 `vite.config.ts` 的 dev server；
4. 从 `frontend` 目录执行：

   ```bash
   npm ci
   npm test -- --run
   npm run build
   npm run tauri -- build --no-bundle --ci
   ```

5. 验证窗口启动、真实 Rust command、规则读取、格式化、设置保存和应用版本显示；
6. CSP 发生变化时，确认生产 `csp` 没有意外加入 localhost、外部 CDN 或宽泛 `*`，开发 `devCsp` 仍允许 Vite/HMR。

## 6. 统一验收门禁

升级提交至少执行：

```bash
python3 scripts/verify.py --profile rust
python3 scripts/verify.py --profile frontend
python3 scripts/verify.py --profile audit
python3 scripts/generate_licenses.py
python3 scripts/verify.py --profile checks
git diff --check
```

若升级涉及 Tauri、Node 主版本或前端构建配置，还必须执行：

```bash
cd frontend
npm run tauri -- build --no-bundle --ci
```

审阅以下差异：

- lockfile 是否只包含预期包和版本变化；
- `cargo audit` / `npm audit` 是否出现新的 high/critical 问题；
- `docs/licenses.md` 是否出现许可证缺失或不允许的许可证；
- 前端测试 warning、Vite 配置 warning 和 Tauri 构建 warning 是否可解释；
- 构建脚本是否仍从正确目录调用 Tauri CLI，且没有把参数错误转交给 Cargo。

## 7. 回滚与失败处理

- 未提交升级：恢复 `.nvmrc`、`rust-toolchain.toml`、package manifest 和两个 lockfile，删除可再生构建目录后重跑基线；
- 已提交但未发布：`git revert` 升级提交，重新运行完整门禁；不要重写共享 `dev` 分支历史；
- 已发布版本出现严重问题：按 `docs/release/manual-release.md` 的回滚原则发布修复版本，不删除历史 tag；
- 审计发现 high/critical 漏洞时停止升级合并，先修复或明确评估，不通过忽略 advisory 绕过门禁；
- 任何升级都不得把 token、age 私钥、明文 SOPS 文件或构建日志加入提交。

## 8. 提交与维护

推荐提交拆分：

1. 工具链版本固定文件；
2. npm/Cargo manifest 与 lockfile；
3. Tauri/React 兼容性修复；
4. 许可证清单、审计结果和文档。

Dependabot PR 仍须遵守本 Runbook：人工审阅变更范围，运行统一验证入口，并确认 `docs/licenses.md` 与锁文件同步。