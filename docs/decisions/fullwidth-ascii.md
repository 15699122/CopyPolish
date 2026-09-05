# 全角 ASCII 转半角决策

状态：Accepted  
日期：2026-09-03

## 决策

新增默认关闭的 `text.halfwidth-ascii` 规则，但不引入全文 Unicode NFKC。

规则只对可编辑文本中的 U+FF01–U+FF5E 做有限映射，并跳过 U+FF10–U+FF19：

- 全角 ASCII 字母和标点映射到对应的 ASCII 半角字符；
- 全角数字继续由既有 `text.halfwidth-digits` 负责；
- 不处理全角空格 U+3000；
- 不改写 Å、㎏、数学字母、兼容汉字或其他 NFKC 目标；
- 链接、URL、邮箱、Markdown、HTML、LaTeX、代码和化学式沿用现有 span 保护边界。

## 原因

U+FF01–U+FF5E 与 ASCII 存在明确的一对一偏移映射，适合做窄范围字符转换；但全文 NFKC 还会涉及语义、单位、数学字符和兼容汉字，不能作为无上下文的清洗行为。全角数字已经有独立稳定规则，继续单独保留可以避免用户启用 ASCII 转换时意外改变数字策略，也保持现有 key 和组合行为清晰。

## 验收

- 注册表拥有稳定 key、默认关闭状态、说明、风险和执行顺序；
- fixture 覆盖字母、标点、全角数字、全角空格及非 ASCII NFKC 反例；
- 结构保护回归沿用现有 Rust pipeline；
- 规则执行幂等，README 规则表与注册表一致；
- 不新增 PDF/CAJ 解析依赖，也不改变 PDF/CAJ 软换行 Spike 的未完成状态。

## 后续限制

该决策只覆盖字符映射，不证明所有来源文本中的全角/半角使用都应被自动纠正。真实 PDF/CAJ 语料和人工标注仍需按 `docs/decisions/pdf-soft-wrap-spike.md` 的前置条件收集。