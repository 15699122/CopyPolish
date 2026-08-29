# Unicode 基线记录（unicode-segmentation 引入前后）

对应 roadmap §5 改造原则第 5 条：引入 unicode-segmentation 前后，记录编译时间、
二进制体积与文本处理性能基线。测量方法可重复：

```bash
# 处理耗时（10 KB / 100 KB / 1 MB x 五类语料）
cargo run --release --manifest-path src-tauri/Cargo.toml --example unicode_baseline

# 完整 release 构建 + 主程序体积
/usr/bin/time -p cargo build --release --manifest-path src-tauri/Cargo.toml
stat -c '%s bytes' src-tauri/target/release/chinese-copywriting-formatter
```

环境：Linux (WSL2)，Rust 1.98.0，release profile。耗时为 5 轮平均值。
1 MB 的“中英数混排”与“Markdown/LaTeX 密集”样例在引入前后均触发既有的
fancy-regex 回溯上限（`Max limit for backtracking count exceeded`），
属引擎既有限制，与本次改动无关，标记为 ERR。

## 引入前（dev @ b199a4e + 基线示例）

- 完整增量 release 构建：约 1.4 s；示例首次全量编译（含全部依赖）：约 153 s
- release 主程序体积：14,465,672 bytes

| 语料 | 10 KB | 100 KB | 1 MB |
| --- | --- | --- | --- |
| 纯中文 | ~6.8 ms | ~17 ms | ~144 ms |
| 中英数混排 | ~6–11 ms | ~21–33 ms | ERR |
| Markdown/LaTeX 密集 | ~13–20 ms | ~149–169 ms | ERR |
| emoji/组合字符密集 | ~7–15 ms | ~18–20 ms | ~144 ms |
| CJK Ext-B 密集 | ~6–16 ms | ~19–20 ms | ~155 ms |

## 引入后（unicode-boundaries 分支，unicode-segmentation 1.x）

- 依赖增量构建：约 25 s（含编译 unicode-segmentation）；release 主程序体积：14,522,904 bytes（+56.7 KB，约 +0.39%）

| 语料 | 10 KB | 100 KB | 1 MB |
| --- | --- | --- | --- |
| 纯中文 | ~5.6 ms | ~21 ms | ~121 ms |
| 中英数混排 | ~5.4 ms | ~22 ms | ERR |
| Markdown/LaTeX 密集 | ~10.8 ms | ~106 ms | ERR |
| emoji/组合字符密集 | ~8.2 ms | ~16 ms | ~131 ms |
| CJK Ext-B 密集 | ~7.4 ms | ~16 ms | ~142 ms |

对比结论：处理耗时与引入前处于同一噪声区间（±20% 以内波动，
多次采样互有高低），无系统性退化；二进制体积增量 < 0.5%。
ERR 为引擎既有 fancy-regex 回溯上限，与本次改动无关。


## TextEdit 迁移与热点修复后（dev，release profile，Rust 1.98.0）

对应 roadmap §8：TextEdit 应用层落地并修复两处热点/上限问题后重跑。修复内容：

1. `space_around_inline_placeholders` / `space_around_math_placeholders` 改为单一通用
   占位符模式 + 成员集合判断：此前按占位符 escape 后用 `|` 拼接正则，1 MB 文本
   会导致正则编译超过大小上限并 panic（`CompiledTooBig`）。
2. `proper_nouns` / `no_abbr` 的词级替换改为预编译正则缓存：TextEdit 迁移后规则按
   可编辑片段高频调用，每片段每词重新编译正则曾使 10 KB Markdown 语料达 ~1.1 s
   （缓存后 ~2.6 ms，约 400 倍改善）。
3. `apply_editable_rules` span 扫描从每规则一次降为单次，规则在片段内串行执行。

测量环境同上（WSL2，5 轮平均）：

| 语料 | 10 KB | 100 KB | 1 MB |
| --- | --- | --- | --- |
| 纯中文 | ~1.4 ms | ~12.6 ms | ~127 ms |
| 中英数混排 | ~1.4 ms | ~14.8 ms | ~160 ms（此前 ERR） |
| Markdown/LaTeX 密集 | ~2.6 ms | ~70 ms | ~4.9 s（此前 ERR，仍待优化） |
| emoji/组合字符密集 | ~1.6 ms | ~12.8 ms | ~138 ms |
| CJK Ext-B 密集 | ~1.2 ms | ~13.4 ms | ~140 ms |

结论：常规语料 1 MB 内均在 ~160 ms 以内；不再出现 ERR 或 panic。剩余热点是
1 MB 级 Markdown/LaTeX 密集语料（~4.9 s），对应 roadmap §5 的 fancy-regex 替换
（反引号、平衡括号链接、HTML block 状态机化），以及错误路径的优雅降级。
