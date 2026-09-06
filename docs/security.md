# CopyPolish 安全模型

> 本文档描述 CopyPolish 的安全边界、资产、威胁、门禁和响应约定。
> 当前基线：`0.6.2-dev.1`，最后更新：2026-09-06。

## 0. 版本范围与功能冻结

v0.6.2 是隐私、安全、依赖和供应链维护版本。该版本不增加用户功能、不新增排版规则，也不改变现有规则默认行为。v0.6.2 完成安全维护并发布前，v0.7.0 的新功能开发冻结。

在维护者明确指定前，不创建 v0.6.2 tag，不执行 v0.6.2 Pre-Release 或正式版构建/发布。完整阶段计划和发布门槛见 [roadmap.md](roadmap.md)。

## 1. 安全目标

- 用户正文默认不落盘；只有显式开启 `restore_last_input` 后才保存。
- 桌面生产构建只处理本地输入，不依赖远程服务，不包含测试诊断能力。
- 前端只能通过受限的 Tauri command 使用 Rust 能力。
- 格式化遵循“宁可漏格式化，不破坏 Markdown、代码、URL、公式等结构”。
- 安全审计必须区分生产依赖、开发依赖和测试依赖，不把测试链风险伪装为生产零风险。

## 2. 资产清单

| 资产 | 所在位置 | 保护要求 |
| --- | --- | --- |
| 用户输入正文 | GUI/TUI/CLI 内存 | 默认不持久化；不进入日志、artifact 或网络请求 |
| 格式化输出 | GUI/TUI/CLI 内存 | 仅在用户显式复制时写入系统剪贴板 |
| 用户设置 | 本地 `rules.yaml` | 原子写入、备份恢复、隐私字段归一化、Unix 私有权限 |
| 自定义替换 | `rules.yaml` | 按字面量处理；限制条目和字段大小 |
| 剪贴板内容 | 操作系统剪贴板 | 只在用户显式复制时访问；复制并清空需复制成功后执行 |
| Release 资产 | GitHub Release | tag/版本一致，使用 `SHA256SUMS` 校验 |
| CI/E2E artifact | CI 临时目录 | 不提交；不包含真实用户正文或凭据 |
| 发布凭据 | 本地 SOPS/age 文件或 CI secret | 不进入 Git、命令参数或日志 |

## 3. 信任边界

```text
用户输入
   ↓
React/WebView 前端
   ↓ 受限 Tauri IPC
Rust command 层
   ↓
Rust 格式化引擎 / 设置存储
   ↓
本地文件系统、系统剪贴板
```

浏览器演示模式是独立边界：它使用浏览器 `localStorage`，不提供桌面 Rust 引擎的安全或行为等价性。

## 4. 输入与资源边界

所有来自前端、TUI 或 CLI 的输入都在共享模型边界校验，当前限制为：

- 输入正文最多 10 MiB，避免异常输入造成内存或 CPU 耗尽；
- 规则选择最多 500 个 key；
- 自定义替换最多 200 项，每个 `from`/`to` 字段最多 16 KiB；
- 快捷键绑定最多 128 字节；
- 设置文件在解析 YAML 前限制为 2 MiB；
- unknown rule key 必须安全丢弃或返回稳定错误，不得触发 panic；
- command 错误不得泄露内部绝对路径、凭据或完整用户正文。

新增限制时必须为正常值、边界值和超限值增加测试，并说明 GUI、TUI、CLI 的差异。

## 5. 设置与隐私

- `restore_last_input` 缺失时默认为 `false`。
- 开关关闭时，保存前必须将 `last_input` 归一化为空字符串。
- 设置写入使用进程内串行锁、唯一临时文件、`0600` 私有权限（Unix）、目录同步和原子替换，并保留可恢复备份；设置、备份和临时目标为 symlink 时拒绝写入。Windows reparse point/junction 与跨进程并发行为已由用户于 2026-09-06 手动确认，完整复现仍需保留 Windows 环境和 artifact 证据。
- 用户可在设置 → 隐私中清除已保存正文；详细用户说明见 [privacy.md](privacy.md)。
- 设置文件不得作为诊断日志、测试 artifact 或 Release 资产上传。

## 6. Tauri 与 WebView 能力

- 生产 capabilities 只授予应用实际需要的窗口、文件和剪贴板能力。
- 测试 capability 与生产 capability 分离；E2E 诊断对象不能进入生产构建。
- CSP 禁止不必要的远程脚本、远程字体和业务网络连接。
- 前端不得直接调用 `invoke`；统一经 `frontend/src/lib/tauri.ts` 访问 command。

## 7. 依赖与供应链门禁

验证入口：

```bash
python3 scripts/verify.py --profile audit
```

审计范围包括：

1. `src-tauri/Cargo.lock` 的 RustSec 审计；
2. `frontend/package-lock.json` 的生产前端依赖审计；
3. `e2e/package-lock.json` 的测试依赖审计；
4. 明文凭据、SOPS 元数据和许可证清单检查。

E2E 当前存在已登记的 `GHSA-jmr9-qjv8-65gv` high 风险，来自 `extract-zip` 的 WebdriverIO 传递依赖链。它仅存在于开发/测试工具链，不进入生产 Release；登记详情、缓解措施和复核日期见 [e2e-audit-policy.json](decisions/e2e-audit-policy.json) 与 [wdio-transitive-dependencies.md](decisions/wdio-transitive-dependencies.md)。

门禁规则：

- Cargo 或 frontend 出现 high/critical 直接失败；
- E2E 出现未登记的 high/critical 直接失败；
- 审计命令网络失败、输出不是合法 JSON 或缺少 policy 时直接失败；
- 已接受 advisory 仍必须定期复核，不得无限期豁免；
- 不使用 `npm audit fix --force` 作为未经验证的自动修复。

## 8. CI、Release 与 artifact

- CI job 权限保持最小化；
- Release 只能从 `master` 的明确 tag 触发；
- 发布前必须检查版本、tag、构建资产和 checksum；
- Release 资产不得包含 `node_modules`、E2E 工具、设置文件、日志或测试 fixture；
- Release 附带生产级 CycloneDX SBOM（`sbom.json`，Rust 生产依赖 + frontend 生产/develop 依赖），并在 `SHA256SUMS` 中纳入校验；
- CI artifact 只保留调试所需的最小内容，并在结束后清理。

## 9. 安全响应

报告安全或隐私问题时，请避免在公开 issue 中粘贴真实正文、凭据、私钥或未脱敏日志。报告应包含：

- 受影响版本和平台；
- 最小复现步骤；
- 影响范围；
- 是否包含真实个人数据；
- 推荐的私密联系方式（若问题包含敏感信息）。

维护者收到报告后应先确认暴露范围，再创建脱敏回归测试；修复完成后更新 CHANGELOG、相关决策文档和版本说明。

## 10. v0.6.2 完成检查

进入 v0.7.0 前必须完成：

- 设置权限、symlink/reparse point、并发写入和备份恢复验证（Unix 自动化与 Windows 用户确认已完成；如需审计级证据仍需补齐原生 artifact）；
- IPC 稳定错误 code 与资源边界测试；
- E2E advisory 修复或限期风险接受复核；
- GitHub Actions SHA 固定、权限审查和 workflow 输入校验；
- 生产 SBOM、provenance/attestation、checksum、许可证和 Release 资产检查；
- 隐私 artifact 扫描和跨平台 smoke；
- 一次完整、可追溯的 v0.6.2 安全维护审计。