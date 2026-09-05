# 康熙部首修复规则决策

> **状态**：Accepted（2026-09-03）。规则默认关闭，仅使用 Unicode 官方数据中的明确兼容分解映射。

## 1. 范围

新增 `cleanup.kangxi-radicals`，只处理连续码位范围 U+2F00–U+2FD5 的康熙部首字符，将其映射到 UnicodeData 中对应的兼容分解目标字符。该范围共 214 个码位，首项 U+2F00 映射到 U+4E00，末项 U+2FD5 映射到 U+9FA0。

规则默认关闭，用户显式启用后才执行；不执行全文 NFKC，不改写范围外字符，也不处理形近字或语义推断。

## 2. 数据来源与核验

映射来源为 Unicode Consortium Unicode Character Database 的 `UnicodeData.txt`，版本 17.0.0，发布日期对应本项目 2026-09-03 复核。U+2F00–U+2FD5 每个记录均包含 `<compat>` 分解且目标为单个 BMP 字符；实现以连续码位对应的静态字符串保存 214 项目标字符。

本次核验检查了：

1. 范围首尾和条目数量；
2. 每项分解标签均为 `<compat>` 且目标数量为 1；
3. 映射字符串长度为 214 个 Unicode code point；
4. 非康熙部首字符保持不变；
5. Markdown、URL、代码和公式通过现有结构保护层跳过。

## 3. 不采用的方案

- 不引入 Unicode normalization crate；
- 不使用全文 NFKC，避免同时改写其他兼容字符；
- 不根据字形相似度猜测 CJK 字符映射；
- 不默认开启该清洗规则。

后续若升级 Unicode 数据版本，必须重新核验 214 项映射、更新本文件日期并运行完整 Rust/TUI/GUI 回归。