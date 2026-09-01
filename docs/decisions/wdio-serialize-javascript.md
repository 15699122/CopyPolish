# E2E serialize-javascript 修复记录

> **状态**：Accepted（2026-09-01 已落地最小修复）。

## 1. 背景

E2E 测试链的 `@wdio/mocha-framework@9.31.5` 依赖 `mocha@10.8.2`，后者默认解析
到 `serialize-javascript@6.0.2`。npm 审计将该版本报告为 high，直接采用 npm 建议
的修复会把 WebdriverIO/@wdio 降到 major 7/8，风险和变更范围过大。

## 2. Spike 结果

在隔离目录中复制当前 `e2e/package.json` 和 `e2e/package-lock.json`，增加：

```json
{
  "overrides": {
    "serialize-javascript": "7.1.1"
  }
}
```

验证结果：

- `npm install --package-lock-only --ignore-scripts` 成功；
- `npm ci --ignore-scripts --no-audit --no-fund` 成功；
- `mocha` 仍为 `10.8.2`，`@wdio/mocha-framework` 仍为 `9.31.5`；
- 实际解析版本为 `serialize-javascript@7.1.1`；
- E2E TypeScript 类型检查通过；
- npm 审计从 16 项（1 moderate、15 high）降为 14 项（0 moderate、14 high）；
- `serialize-javascript` 不再出现在审计结果中。

## 3. 决策

采用 npm `overrides` 作为最小修复，不进行 WebdriverIO major 升级或降级。该修复只
影响 E2E 测试依赖，不改变生产构建和应用运行时依赖。

剩余审计告警继续作为独立依赖维护项跟踪，后续在 WebdriverIO 或浏览器工具升级时
重新评估。每次 E2E 依赖变更都必须重新运行 `npm ci`、`npm run typecheck`、E2E
provider 回归和 `npm audit`。

## 4. 关联文件

- 依赖约束：`e2e/package.json`；
- 锁定解析：`e2e/package-lock.json`；
- 审计入口：`scripts/verify.py --profile audit`；
- E2E 类型检查：`npm run typecheck --prefix e2e`；
- 后续依赖升级流程：`docs/upgrade-runbook.md`。