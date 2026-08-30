# CopyPolish 测试指南

## 1. 测试层次

| 层次 | 位置 | 目的 |
| --- | --- | --- |
| Rust 单元/集成测试 | `src-tauri/src/**`、`src-tauri/tests/fixtures/` | 验证规则、管线、保护层、Unicode、设置和 TUI 状态 |
| 前端 hook 测试 | `frontend/src/hooks/*.test.ts` | 验证异步状态、竞态、防抖、持久化和窗口交互 |
| 前端组件测试 | `frontend/src/App.test.tsx` | 验证用户操作、设置窗口、快捷键和界面反馈 |
| 性能门禁 | `scripts/check_performance.py`、`src-tauri/examples/unicode_baseline.rs` | 捕获数量级性能回退，不替代 profiling |
| 桌面 smoke/E2E | 当前主要为人工验证 | 验证真实 Tauri IPC、窗口、设置和平台行为 |

## 2. 功能—测试映射

| 功能 | 现有覆盖 | 后续补强 |
| --- | --- | --- |
| 规则注册表 | 稳定 key、默认状态、legacy key、依赖图 | 自动检查 README 与注册表一致性 |
| 格式化管线 | 规则选择、组合、换行、幂等性、未知 key | 属性测试和更大真实语料 |
| Markdown/HTML/LaTeX | span、嵌套结构、未闭合结构、保护 fixture | 继续扩展真实文档样本 |
| Unicode | grapheme、emoji、组合符、CJK Ext-B | Unicode 数据/工具链升级回归 |
| 单位和数学 | 有限词典、复合单位、数学边界 | 按真实语料扩展词典 |
| 设置 | 缺失、损坏、备份、旧 JSON 迁移 | 真实桌面重启和不可写目录 |
| 前端状态 | 防抖、竞态、错误、主题、字体、快捷键 | 真实 IPC E2E |
| TUI | CLI、编辑器、规则、OSC 52、共享设置 | Windows Terminal、Linux、SSH smoke |
| 发布脚本 | 主要由脚本和人工 Runbook 覆盖 | 参数和失败路径自动化测试 |

## 3. 常用命令

```bash
python3 scripts/verify.py --profile checks
python3 scripts/verify.py --profile frontend
python3 scripts/verify.py --profile rust
python3 scripts/verify.py --profile audit
```

直接运行前端测试：

```bash
npm test --prefix frontend -- --run
```

## 4. 新规则测试要求

每条规则至少应包含：

1. 单规则输入/输出；
2. 与相关规则的组合输出；
3. Markdown、LaTeX、URL、代码或化学式等保护场景；
4. 重复执行后的幂等性断言；
5. 默认开关、规则选择和稳定 key 验证；
6. 设置迁移、GUI 动态元数据和 TUI 兼容性检查（适用时）。

争议性规则必须明确默认关闭或开启的理由，并在 README 和 CHANGELOG 中说明用户可见影响。

## 5. Fixture 规范

- 一个 fixture 文件聚焦一个领域；
- 输入、规则选择和期望输出应清晰可读；
- 修复 bug 先添加最小回归用例；
- 不通过批量改写 fixture 隐藏行为变化；
- 同时关注 LF/CRLF、Unicode 边界和重复格式化；
- 设置读写测试使用系统临时目录，禁止写仓库内固定路径。

## 6. 桌面验证缺口

当前 mock 测试不能完全替代真实桌面验证。后续 E2E 应覆盖启动、真实 Rust command、规则选择、快捷键、设置保存/恢复、损坏设置和不可写目录，并使用临时设置目录。Linux 与 Windows 至少各保留一条稳定链路。

## 7. 测试完成标准

- 没有新增未解释的 warning；
- `git diff --check` 通过；
- Markdown 链接检查通过；
- 密钥扫描通过；
- 涉及规则、设置、Tauri 或发布时已完成相应额外验证。