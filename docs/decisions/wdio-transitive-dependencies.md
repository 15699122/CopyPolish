# E2E 传递依赖修复记录

> **状态**：部分 Accepted（2026-09-30 复核，见 §7）。`deepmerge-ts`、`serialize-javascript`、`undici`、`brace-expansion`、`ip-address` 已通过 override 修复；
> `@puppeteer/browsers`/`extract-zip` 仍受上游依赖链阻塞，保留后续评估。

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
截至 2026-09-03，E2E 审计仍为 13 项 high、0 moderate、0 critical。当前 lockfile
仍为 WebdriverIO 9.31.5、`@wdio/tauri-service` 1.3.0、
`@puppeteer/browsers` 2.13.2 和 `extract-zip` 2.0.1。`@wdio/cli`、`webdriverio`
和 `@wdio/tauri-service` 均已是 npm 当前 latest；`npm outdated` 没有发现可用的
WebdriverIO/Tauri 直接依赖升级，只有 `expect` 的 patch 更新和 `@types/node` 的
跨主版本更新。

`npm audit fix --package-lock-only --dry-run --ignore-scripts` 仍只能建议
`--force` 降级到 `@wdio/local-runner@8.14.6`，属于 breaking change，不能作为
当前 WebdriverIO 9/Tauri provider 的安全修复。`@puppeteer/browsers@3.2.1`
虽已发布，但 `@wdio/utils@9.31.5` 仍声明 `^2.2.0`，不能仅通过 override
跨 major 替换；
`extract-zip` 2.0.1 的 GHSA-jmr9-qjv8-65gv advisory
截至本日期没有 patched version。

剩余告警涉及 WebdriverIO 9 工具链、`@puppeteer/browsers`、`extract-zip`、
`expect-webdriverio` 等传递依赖，不能通过当前已验证的局部 override 全部安全消除。

后续 E2E 依赖变更必须重新运行 `npm ci`、`npm run typecheck`、embedded/W3C
provider 回归和 `npm audit`。

## 5. 2026-09-03 维护复核

本次复核执行了 `npm outdated --prefix e2e`、依赖树解释、npm registry 版本/engine
查询以及 `npm audit fix --package-lock-only --dry-run --ignore-scripts`。由于没有
可接受的 WebdriverIO 9 升级或 `extract-zip` 修复版本，未修改 `e2e/package.json`
或 `e2e/package-lock.json`，也未将路线图中的持续维护项标记为完成。

## 7. 2026-09-30 维护复核（v0.7.0-pre1 发布准备）

本次复核在 `v0.7.0-pre1` 发布准备阶段重新执行审计，取得以下**新鲜结果**（不沿用 2026-09-03/09-04 的历史输出）：

- 前端：`npm audit --prefix frontend` 报告 **0 vulnerabilities**；此前的 `undici` 告警已随 jsdom 升级到 `undici@8.11.2` 实际消除。
- E2E：审计一度报告 16 个 advisory，其中 **14 个是新暴露且有兼容修复版本**的问题，根因是传递依赖长期未跟进：

| 根包 | 受影响范围 | 处置 |
| --- | --- | --- |
| `undici` | `6.28.0`（经 `webdriver`）、`7.29.0`（经 `cheerio`） | 按版本区间 override 到同 major 修复版 `6.28.1` / `7.29.1` |
| `brace-expansion` | `1.1.18`、`2.1.4` | 按版本区间 override 到 `1.1.21` / `2.1.7` |
| `ip-address` | `10.7.0`（经 `@puppeteer/browsers → proxy-agent → socks`） | override 到 `10.7.2` |
| `extract-zip` | `2.0.1` | **无修复版本**，登记限期风险接受 |

`undici` 与 `brace-expansion` 各有两条依赖线且 major 不同，因此使用
`undici@>=6.25.0 <6.28.1` 这类带版本区间的 override 键，避免跨 major 强制提升。
这些 override 全部落在同一 major 内，不改变 WebdriverIO 预期实现。

override 后验证结果：

- `npm install --package-lock-only --ignore-scripts` 与 `npm ci --ignore-scripts` 成功；
- `npm run typecheck`（`tsc --noEmit`）通过；
- `@wdio/cli`、`@wdio/utils`、`@wdio/config`、`webdriverio`、`@puppeteer/browsers`、
  `cheerio`、`socks`、`@wdio/tauri-service` 均可正常动态导入；
- 审计从 16 个 advisory 降为 **2 个**，仅剩 `extract-zip` 的
  GHSA-jmr9-qjv8-65gv 与 GHSA-7pqw-9j4j-h8q3。

`extract-zip@2.0.1` 已是 npm 当前 latest（`npm view extract-zip version`），
两个 advisory 的 `range` 均为 `<=2.0.1`，**没有 patched version**；npm 唯一给出的
自动修复仍是降级 `@wdio/local-runner@8.14.6`（breaking）。因此新增
GHSA-7pqw-9j4j-h8q3 到 [e2e-audit-policy.json](e2e-audit-policy.json)，
与既有登记同理由、同复核期限（2026-10-06），并补充「只处理工具链下载的归档、
不接受用户提供归档」这一可利用性限定。

`@puppeteer/browsers@3.2.1` 的跨 major override 仍然不采用，理由与第 3 节一致。

下一次复核仍以「WebdriverIO 发布兼容的 `@puppeteer/browsers` 3.x 约束」或
「`extract-zip` 发布修复版本」为落地条件。

## 6. 2026-09-04 维护复核

本次复核使用当前锁文件和工具链重新检查：

- `npm outdated --prefix e2e` 仅报告 `expect` 的 `30.5.1` patch 更新和
  `@types/node` 的 26.x 跨主版本更新；当前离线缓存没有 `expect@30.5.1`，因此未修改
  `e2e/package.json` 或 `e2e/package-lock.json`；
- `npm explain --prefix e2e @puppeteer/browsers extract-zip` 确认当前链路仍为
  `@wdio/utils@9.31.5` → `@puppeteer/browsers@2.13.2` → `extract-zip@2.0.1`，
  另有 `@wdio/tauri-service@1.3.0` 使用 `webdriverio@9.30.0`；
- `npm audit --prefix e2e` 因 2026-09-04 连接 npm registry 时发生 TLS/socket 断开而未取得 advisory 结果，
  不将本次网络失败记为审计通过，也未执行 `npm audit fix --force`；
- `cargo audit --file src-tauri/Cargo.lock --json` 返回 0 个漏洞、21 个允许的 warning；
- `python3 scripts/generate_licenses.py --features simplified-trad-conversion` 已重新生成
-  `docs/licenses.md`：Rust 431 条、npm 294 条、许可证字段缺失 0 条。

同时修正 `scripts/verify.py --profile audit`，使 Cargo 审计显式读取
`src-tauri/Cargo.lock`，避免从仓库根目录执行时误报缺少锁文件。由于 E2E advisory
服务不可用且剩余依赖链没有兼容的安全升级，本次仍不将路线图中的持续维护项标记为完成。