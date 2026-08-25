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

