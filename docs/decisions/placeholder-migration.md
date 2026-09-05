# Placeholder 重构 Spike（决策记录）

> **状态**：结论——**暂不进行大规模重构，保持当前混合管线**（2026-09-01）。
> 本文记录方案比较、实测基线和重新评估条件；不代表已经替换生产实现。

## 1. 背景

当前引擎采用混合管线：可编辑文本通过 `edit_plan.rs` 生成并应用 UTF-8 安全的
`TextEdit`；Markdown、HTML、LaTeX、URL、邮箱、代码和化学式等不可编辑结构仍由
保护层转换为内部 placeholder，完成边界规则后再恢复原文。

现状的主要不便是保护结构与可编辑文本使用两种中间表示，增加了理解和调试成本。
但 placeholder 已经经过多轮边界、幂等性、Unicode 和大文本回归，不能仅因为架构上
“看起来可以统一”就直接替换。

## 2. 候选方案

| 方案 | 说明 | 兼容性风险 | 预期收益 | 结论 |
| --- | --- | --- | --- | --- |
| A. 保持受控 placeholder | 保留不可编辑结构的 token 替换、边界处理和最终恢复；继续把新规则迁移到 TextEdit | 低，当前行为不变 | 低到中；边界逻辑已有稳定实现 | **采用** |
| B. 全程 span/TextEdit | 所有结构保护和边界操作都在原文 span 上规划编辑，不再生成 placeholder 文本 | 高；需要重写保护边界、嵌套结构和恢复逻辑 | 中；中间表示更少，但编辑冲突更复杂 | 暂不实施 |
| C. 分段 rope/token stream | 将文本拆为结构 token 和可编辑片段，在 token 流或 rope 上执行规则 | 高；需要新的数据模型、索引、序列化和调试工具 | 潜在较高，但只有大文本或增量编辑需求明确时才值得 | 暂不实施 |

## 3. 性能基线

使用现有本地工具：

```bash
cargo run --release --manifest-path src-tauri/Cargo.toml \
  --features profile-stages --example profile_stages
```

2026-09-01 本地观测，Rust release profile，1 MB 语料，5 轮平均：

| 语料 | 总耗时 | protect | placeholder_spacing | restore | placeholder 合计 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Markdown/LaTeX 密集 | 131.72 ms | 3.72 ms | 4.55 ms | 7.69 ms | 15.96 ms（约 12.1%） |
| 纯中文对照 | 58.41 ms | 0.04 ms | 0.05 ms | 0.13 ms | 0.22 ms（约 0.4%） |

该观测受机器、编译器和运行时抖动影响，只用于阶段归因，不作为跨机器绝对性能
承诺。当前主要热点是 `protected_rules`、`editable_rules`、`scan_spans` 以及
URL/邮箱等结构扫描；placeholder 相关阶段不是首要热点。

## 4. 决策

1. **保留方案 A**：继续使用受控 placeholder，避免在没有明确产品收益时重写已经稳定
   的保护和恢复路径。
2. 新的规则和边界行为继续优先通过 `edit_plan.rs` 的 TextEdit 接入；placeholder
   只负责承载不可编辑结构，不再扩展为第二套规则执行管线。
3. 当前不引入 rope/token stream，也不改变 `TextSpan`、`TextEdit` 和现有保护 token
   的公共行为。
4. 真实语料、属性测试和大文本基准继续作为重构前门禁；任何未来替换都必须逐例比较
   现有 fixture、真实 corpus、幂等性、保护结构和换行结果。

## 5. 重新评估条件

满足以下任一条件时，可以重新启动 Spike 或进入实现阶段：

- 出现只能通过 placeholder 改造解决、且无法局部修复的结构边界缺陷；
- placeholder 相关阶段在代表性生产语料中成为主要性能瓶颈；
- 需要增量编辑、局部重排或大文本交互，当前整段字符串管线无法满足；
- 新增保护结构持续显著增加 token 边界和恢复逻辑的维护成本；
- 能够提供完整的输出兼容、内存、性能和失败诊断对照结果。

## 6. 关联文件

- 当前管线：`src-tauri/src/engine/pipeline.rs`；
- 保护实现：`src-tauri/src/engine/protection.rs`；
- span 与冲突仲裁：`src-tauri/src/engine/spans.rs`、`edit_plan.rs`；
- 性能工具：`src-tauri/examples/profile_stages.rs`；
- 性能历史：`docs/benchmarks/unicode-baseline.md`；
- 语料回归：`src-tauri/tests/fixtures/real-world-corpus.yaml`。