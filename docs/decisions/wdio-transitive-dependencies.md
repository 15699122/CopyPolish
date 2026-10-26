# E2E 传递依赖修复记录

> **状态**：部分 Accepted（2026-10-12 复核，见 §8）。`deepmerge-ts`、`serialize-javascript`、`smol-toml`、`undici`、`brace-expansion`、`ip-address` 已通过 override 修复；
> `@puppeteer/browsers`/`extract-zip`、`basic-ftp`、`braces` 仍受上游依赖链阻塞，保留限期风险接受与后续评估。

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

## 8. 2026-10-12 维护复核（开放 PR 处置轮）

本次复核在评估全部开放 PR 时重新执行审计，发现 2026-09-30 之后新发布/修订的 advisory
已使 `--profile audit` 重新变红，因此按「先恢复门禁、再合并依赖 PR」的顺序处理。

复核前（`dev` = `eff7f68`）的实际结果：

- 前端：1 个 high —— `source-map-js@1.2.1`（GHSA-68fv-2mgg-jv7q，影响范围 `>=1.0.0 <1.2.2`）；
  另有 moderate `smol-toml@1.8.0`（GHSA-r4xh-jqrq-34v2，`<=1.8.0`）。
- E2E：20 个 high/critical 条目，根 advisory 为 `extract-zip`（已登记）、
  `basic-ftp@5.3.1`（GHSA-c475-qrg2-pj4r，`<=6.2.0`）、`braces@3.0.3`
  （GHSA-vfj7-8cjw-p6xm，`<=3.0.3`）与 `serialize-javascript@7.1.1`
  （GHSA-gfhx-hw2g-v5hg，`>=7.1.1 <7.1.2`），以及 `smol-toml@1.8.0`。

本次处置：

| 根包 | 受影响范围 | 处置 |
| --- | --- | --- |
| `source-map-js` | `1.2.1` | 刷新 `frontend/package-lock.json` 到同 major 修复版 `1.2.2`（父约束 `^1.2.1` 允许，无 manifest 变更） |
| `smol-toml` | `1.8.0`（前端与 E2E） | 前端刷新到 `1.9.1`；E2E 新增 override `"smol-toml": "1.9.1"`（同 major，父约束 `^1.6.0`/`^1.7.1` 允许） |
| `serialize-javascript` | `7.1.1` | 既有 override 由 `7.1.1` 抬到 `7.1.2`（同 major） |
| `basic-ftp` | `5.3.1`（经 `get-uri`） | **无同 major 修复版**：修复版 `6.2.3` 属跨 major，而 `get-uri`（含最新 `8.0.1`）仍约束 `^5.3.1`；按 §8.1 原则不采用跨 major override，登记限期风险接受（`GHSA-c475-qrg2-pj4r`，复核期限 2026-11-30） |
| `braces` | `3.0.3`（经 `mocha → chokidar`） | **上游无 patched version**（`3.0.3` 即 latest）；修复需迁移到 chokidar 4+/mocha 12 线路，超出当前 WebdriverIO 9 工具链，登记限期风险接受（`GHSA-vfj7-8cjw-p6xm`，复核期限 2026-11-30） |
| `extract-zip` | `2.0.1` | 保持风险接受，复核期限由 2026-10-06 续期到 2026-11-30；本次确认 `2.0.1` 仍为 npm latest |

处置后验证：

- `Frontend dependency audit`：无 high/critical；
- `E2E dependency audit`：20 个 high/critical 条目全部由 4 个已登记 advisory
  （`GHSA-7pqw-9j4j-h8q3`、`GHSA-c475-qrg2-pj4r`、`GHSA-jmr9-qjv8-65gv`、`GHSA-vfj7-8cjw-p6xm`）覆盖；
- `python3 scripts/verify.py --profile frontend`（`npm ci`、Vitest、Vite build）通过；
- `npm run typecheck --prefix e2e` 通过；
- `python3 scripts/generate_licenses.py --check` 与 `python3 scripts/generate_sbom.py --check` 通过
  （Rust 431 条、npm 285 条、SBOM 716 组件）。

下一次复核条件：`get-uri`（或 `proxy-agent` 链路）发布允许 `basic-ftp` 6.x 的版本，
或上游发布 `braces` 修复版 / `extract-zip` 修复版。届时重新执行
`npm ci --prefix e2e`、`npm run typecheck --prefix e2e`、embedded/W3C provider 回归与
`python3 scripts/verify.py --profile audit`，并更新本文件与 `e2e-audit-policy.json`。

## 9. 2026-10-26 维护复核（安全门禁恢复）

本次复核在执行开放 PR 评估时发现 `--profile audit` 在 `dev` 上再次变红，因此先恢复门禁、再合并依赖 PR。

复核前（`dev` = `2fb6e0a`）的实际结果：

- 前端 2 个 high：`undici@8.11.2`（`GHSA-xjj6-f33q-929c` high，加 `GHSA-4qv8-2p9j-2685`、`GHSA-f72v-6w45-q472`，影响 `>=8.0.0 <8.12.2`；经 `jsdom` 的 `^8.10.2` 引入）与 `vite@8.3.1`（`>=8.3.0 <=8.3.2` 的 6 个 advisory，含 `launch-editor` 的 Windows 相关项）；
- E2E 5 个未登记 advisory：`undici` 的 6.x/7.x 线（修复版 `6.29.1` / `7.30.1`，此前 override 固定的 `6.28.1` / `7.29.1` 已落在影响区间内）、`serialize-javascript`（`GHSA-m5mm-p8h7-m29v`，`<7.1.3`）、`ip-address`（`GHSA-rxp7-6hxx-x8q5`，`<=10.7.3`）。

本次处置（全部为同 major 修复）：

| 根包 | 处置 |
| --- | --- |
| `undici`（前端） | 锁文件刷新到 `8.12.3`（父约束 `^8.10.2` 允许，无 manifest 变更） |
| `undici`（E2E 6.x/7.x） | override 区间与目标同步抬高：`undici@>=6.25.0 <6.29.1` → `6.29.1`、`undici@>=7.0.0 <7.30.1` → `7.30.1` |
| `serialize-javascript` | override `7.1.2` → `7.1.4` |
| `ip-address` | override `10.7.2` → `10.7.5` |
| `braces` | `GHSA-vfj7-8cjw-p6xm` **已被上游撤回**（`withdrawn`），不再出现在审计报告中，因此从 `e2e-audit-policy.json` 移除该登记；`braces@3.0.3` 仍随 `mocha → chokidar 3` 安装，但无有效 advisory |

处置后验证（2026-10-26）：

- `Frontend dependency audit`：无 high/critical；
- `E2E dependency audit`：17 个 high/critical 条目全部由 3 个已登记 advisory（`GHSA-jmr9-qjv8-65gv`、`GHSA-7pqw-9j4j-h8q3`、`GHSA-c475-qrg2-pj4r`）覆盖；
- `python3 scripts/verify.py --profile frontend`（`npm ci`、Vitest、Vite build）通过；
- `python3 scripts/generate_licenses.py --check` 与 `python3 scripts/generate_sbom.py --check` 通过（419 Rust / 291 npm；SBOM 组件数随锁文件更新）。

剩余接受项与复核期限不变：`extract-zip` 两项与 `basic-ftp` 仍为 2026-11-30 前复核；上游发布修复版后按同 major 优先修复的原则处理。
