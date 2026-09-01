# E2E 传递依赖修复记录

> **状态**：部分 Accepted（2026-09-01）。`deepmerge-ts` 已通过 override 修复；
> `@puppeteer/browsers` 暂不覆盖，保留后续评估。

## 1. 当前依赖链

WebdriverIO 9.31.5 的工具链包含以下存在 npm high advisory 的传递依赖：

- `deepmerge-ts@7.1.6`（修复前版本；当前已通过 override 固定为 `8.0.2`），由 `@wdio/config`、`@wdio/utils`、`webdriver` 等使用；
- `@puppeteer/browsers@2.13.2`，由 `@wdio/utils` 使用，并继续引入 `extract-zip@2.0.1`。

## 2. `deepmerge-ts` 修复

在 `e2e/package.json` 增加：

```json
{
  "overrides": {
    "deepmerge-ts": "8.0.2"
  }
}
```

验证结果：

- `npm install --package-lock-only --ignore-scripts` 成功；
- `npm ci --ignore-scripts --no-audit --no-fund` 成功；
- `deepmerge`、`deepmergeCustom` 等 WebdriverIO 使用的导出仍可用；
- `@wdio/utils`、`@wdio/config`、`webdriver`、`webdriverio` 均可动态导入；
- E2E TypeScript 类型检查通过；
- 审计从 14 项 high 降为 13 项 high，`deepmerge-ts` 不再出现在审计结果中。

## 3. `@puppeteer/browsers` 暂不覆盖

`@puppeteer/browsers@3.2.1` 的隔离实验可以安装并动态导入，但暂不写入生产 lockfile，原因如下：

1. 当前 `@wdio/utils@9.31.5` 和 `@wdio/utils@9.30.0` 都声明 `^2.2.0`，override 会跨 major 改变 WebdriverIO 预期的浏览器工具实现；
2. 3.2.1 引入 `modern-tar`、新的 yargs 依赖和 peer 约束，不能只根据安装成功判断行为兼容；
3. `@wdio/tauri-service@1.3.0` 还嵌套使用 WebdriverIO 9.30.0，必须在 embedded 和 W3C provider 上执行完整回归；
4. 该包最新版本要求 Node `>=22.12.0`，虽然当前开发环境满足，但仓库正式基线是 Node `>=24 <25`，仍需确认 Windows 基线和 provider 行为。

因此保留 `@puppeteer/browsers@2.13.2` 与 `extract-zip@2.0.1`，后续随 WebdriverIO/浏览器工具升级窗口处理。

## 4. 当前审计结论

截至 2026-09-01，E2E 审计为 13 项 high、0 moderate、0 critical。剩余告警涉及
WebdriverIO 9 工具链、`@puppeteer/browsers`、`extract-zip`、`expect-webdriverio`
等传递依赖，不能通过当前已验证的局部 override 全部安全消除。

后续 E2E 依赖变更必须重新运行 `npm ci`、`npm run typecheck`、embedded/W3C
provider 回归和 `npm audit`。