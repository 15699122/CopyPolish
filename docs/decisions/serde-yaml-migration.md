# serde_yaml 迁移 Spike（决策记录）

> **状态**：结论——**暂不迁移，保持观察**（2026-09-01）。重新评估条件见文末。

## 1. 背景

`serde_yaml` 0.9.34+deprecated 上游已停止维护（仓库归档，无后续 release）。项目内使用范围极小：

- `src-tauri/src/user_settings.rs`：`rules.yaml` 的读/写（各 1 处）；
- `src-tauri/src/engine/tests.rs`：fixture 解析；
- 经 `Cargo.lock` 进入依赖树，`cargo audit` 无漏洞告警（仅 unmaintained/deprecated 提示）。

风险本质：**格式解析器停止接收修复**，而非当前存在已知漏洞。YAML 解析输入是用户本地的 `rules.yaml`，攻击面有限。

## 2. 候选替代

| 选项 | 说明 | 评估 |
| --- | --- | --- |
| `serde_yml` | serde_yaml 的活跃 fork（x3vision/serde-yaml-ng 同源分支的另一步进版本） | 社区对维护质量与额外行为变更（如标签处理）存在争议；引入需重跑全部设置/fixture 回归 |
| `serde-yaml-ng` | 直接延续 0.9 行为的维护 fork | API 与 `serde_yaml` 基本一致，迁移成本最低（改 crate 名）；维护单一维护者，长期活跃度待观察 |
| 改用 JSON/TOML | 设置换格式 | 破坏现有用户 `rules.yaml` 兼容，需要迁移与双读；收益不抵成本 |
| 自维护 vendored fork | 复制 0.9.34 源码 | 维护负担转嫁自身，仅在出现实际漏洞且无可用替代时考虑 |

## 3. 结论

1. **暂不迁移**：无漏洞告警、行为风险低；在无安全压力下切换解析器只会引入输出格式差异风险（YAML 序列化细节，如字符串引号风格），影响用户 diff 与测试快照。
2. 在 roadmap「依赖与安全维护」中保留跟踪项；每次依赖审计（`verify.py --profile audit`）复核 RUSTSEC 是否升级为有漏洞告警。
3. 若需要迁移，首选 `serde-yaml-ng`（API 兼容、改动最小），迁移时必须：
   - 用全部设置 round-trip 测试与 YAML fixture 做输出 diff 对照；
   - 验证旧版 `rules.yaml`（含无 `shortcuts` 字段的旧文件）解析不变；
   - CHANGELOG 记录依赖变化。

## 4. 重新评估条件

- RUSTSEC 为 YAML 解析链发布漏洞告警；
- `serde-yaml-ng` 出现多个维护者/成为事实标准；
- 需要升级到不兼容 YAML 子集的新功能。
