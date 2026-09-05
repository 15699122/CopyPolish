# ICU4X Script / General Category Spike

本记录对应 roadmap §6，评估纯 Rust ICU4X 是否适合替代当前手写 Unicode 分类。
Spike 在仓库外的 `/tmp/copypolish-icu-spike` 隔离目录执行，没有修改项目
`Cargo.toml`、`Cargo.lock` 或生产代码。

## 测试范围

使用 ICU4X `icu_properties` 2.3.0 的 `compiled_data` feature，调用：

```rust
CodePointMapData::<Script>::new().get(ch)
CodePointMapData::<GeneralCategory>::new().get(ch)
```

比较对象是当前 `unicode_boundaries.rs` 的手写 Han/Latin/Digit/Other 分类。测试字符串包含：

- 基本汉字和 CJK Extension B；
- ASCII/Latin 字母和数字；
- 中文、英文、数字混排；
- emoji ZWJ 序列。

两种实现均扫描相同字符串 100 次 × 100 次，并输出分类汇总值和耗时。

## 结果

| 实现 | release 二进制 | 冷构建时间 | 扫描耗时 |
| --- | ---: | ---: | ---: |
| 手写分类 | 467,920 bytes | 1.00 s | 64.7 ms |
| ICU4X Script + General Category | 512,328 bytes | 7.56 s | 91.6 ms |

ICU4X 版本相对手写实现：

- 二进制增加 44,408 bytes，约 9.5%；
- 冷构建增加约 6.5 s，约 6.6 倍；
- 扫描耗时增加约 41.5%。

两个 Spike 程序对测试字符串得到相同的分类汇总值，说明该样本上没有观察到输出分类差异。
这不是完整 Unicode 一致性证明；正式产品仍须依赖现有黄金 fixture 和行为测试。

## 结论

当前不把 ICU4X 引入正式依赖。现有手写分类已经覆盖产品实际需要的保守子集，并且性能、构建时间和二进制体积均更低；ICU4X 的主要价值是完整 Script/General Category 数据，但当前没有解决已确认的产品问题。

后续只有在以下条件同时满足时才重新评估：

1. 产品需要 Kana、Hangul 或更完整 Script 属性参与排版决策；
2. 能够接受增加的构建和二进制成本；
3. 新实现先在 feature flag 或独立分支中与黄金样例做完整差异审阅；
4. 重新测量 Linux/Windows 构建和 10 KB/100 KB/1 MB 性能。