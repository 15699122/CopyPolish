# Unicode 基线记录（unicode-segmentation 引入前后）

对应 roadmap §5 改造原则第 5 条：引入 unicode-segmentation 前后，记录编译时间、
二进制体积与文本处理性能基线。测量方法可重复：

```bash
# 处理耗时（10 KB / 100 KB / 1 MB x 五类语料）
cargo run --release --manifest-path src-tauri/Cargo.toml --example unicode_baseline

# 完整 release 构建 + 主程序体积
/usr/bin/time -p cargo build --release --manifest-path src-tauri/Cargo.toml
stat -c '%s bytes' src-tauri/target/release/chinese-copywriting-formatter

# release 基准进程峰值 RSS（需要已构建的示例）
/usr/bin/time -v src-tauri/target/release/examples/unicode_baseline
```

环境：Linux (WSL2)，Rust 1.98.0，release profile。耗时为 5 轮平均值。
1 MB 的“中英数混排”与“Markdown/LaTeX 密集”样例在引入前后均触发既有的
旧版 `fancy-regex` 保护路径曾触发回溯上限（`Max limit for backtracking count exceeded`），
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
ERR 为历史保护路径的 `fancy-regex` 回溯上限；该依赖已在当前生产管线中移除。


## TextEdit 迁移与热点修复后（dev，release profile，Rust 1.98.0）

对应 roadmap §8：TextEdit 应用层落地、词级规则与占位符正则热点修复后重跑。修复内容：

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
| Markdown/LaTeX 密集 | ~3.2 ms | ~37.6 ms | ~1.59 s（此前约 4.9 s） |
| emoji/组合字符密集 | ~1.6 ms | ~12.8 ms | ~138 ms |
| CJK Ext-B 密集 | ~1.2 ms | ~13.4 ms | ~140 ms |

结论：常规语料 1 MB 内均在 ~160 ms 以内；不再出现 ERR 或 panic。Markdown/LaTeX
密集语料已由此前约 4.9 s 降至约 1.59 s，但仍是主要热点。保护阶段 release profiling
约为 `scan_all_spans 591 ms`、`protect_spans 109 ms`、行内占位符间距 `19 ms`、
还原 `9 ms`；后续应优先减少结构扫描重复遍历，并继续推进 roadmap §5 的状态机化。

### 峰值内存测量（2026-08-29）

在同一 WSL2 环境中，使用已构建的 release 基准示例执行：

```bash
/usr/bin/time -v src-tauri/target/release/examples/unicode_baseline
```

结果：

- 最大常驻集（Maximum resident set size）：`64,492 KB`，约 `63.0 MiB`；
- 运行时间：`14.12 s`（包含五类语料的 5 轮平均基准）；
- 无交换分区使用，进程正常退出。

该 RSS 是整个基准进程在完整语料循环期间的峰值，包含运行时、正则缓存、
保护占位符和输出分配，不能直接等同于单次 `format_text` 调用的精确内存成本。
后续若需要 UI 级内存门禁，应增加独立的单次请求测量和更细粒度分配 profiling。

## 共享结构行范围后（dev，2026-08-29）

本次优化让 `scan_structure_spans` 在一次入口调用中构造共享的行范围表，
front matter、fenced code、引用定义、缩进代码、表格分隔行、HTML block 和硬换行
扫描器复用同一结果，避免各自重复执行 `split('\n')` 和创建临时行集合。该改动不改变
扫描顺序、span 仲裁或边界策略。

同一 release 基准在三次连续运行中的 1 MB 结果：

| 语料 | Run 1 | Run 2 | Run 3 |
| --- | ---: | ---: | ---: |
| 纯中文 | 121.0 ms | 126.4 ms | 113.6 ms |
| 中英数混排 | 150.4 ms | 199.8 ms | 140.2 ms |
| Markdown/LaTeX 密集 | 1350.2 ms | 1377.0 ms | 1323.2 ms |
| emoji/组合字符密集 | 121.4 ms | 114.6 ms | 115.0 ms |
| CJK Ext-B 密集 | 131.0 ms | 123.6 ms | 120.6 ms |

结果与此前约 1312 ms 的单次观测处于同一抖动范围内，暂不宣称端到端稳定加速；
确定性收益是减少结构扫描阶段的重复行表构造。Markdown/LaTeX 仍是主要热点，后续
继续评估重复字符串分配和嵌套结构扫描。

## 分阶段剖析基线（dev，2026-08-30）

为 placeholder 重构（决策 2）提供归因依据。工具：
`cargo run --release --features profile-stages --example profile_stages`
（`pipeline.rs::format_text_stage_timings`，`RuleSelection::All`，1 MB 语料，5 轮平均）：

| 阶段 | Markdown/LaTeX 密集 | 占比 | 纯中文（对照） | 占比 |
| --- | ---: | ---: | ---: | ---: |
| normalize | 0.15 ms | 0.0% | 0.12 ms | 0.1% |
| editable_rules | 710.02 ms | 47.7% | 97.80 ms | 66.3% |
| scan_spans | 598.28 ms | 40.2% | 10.30 ms | 7.0% |
| protect | 97.87 ms | 6.6% | 0.08 ms | 0.1% |
| protected_rules | 68.96 ms | 4.6% | 38.93 ms | 26.4% |
| placeholder_spacing | 4.66 ms | 0.3% | 0.06 ms | 0.0% |
| restore | 8.11 ms | 0.5% | 0.13 ms | 0.1% |
| **TOTAL** | **1489.25 ms** | | **147.44 ms** | |

结论：

1. Markdown/LaTeX 语料的热点是 `editable_rules`（约 48%）与 `scan_spans`（约 40%），
   合计近 88%；两者都与占位符机制无关；
2. 占位符全链路（protect + placeholder_spacing + restore）合计约 110 ms（约 7.4%），
   **placeholder 重构的性能收益有限**，其价值主要在语义清晰度与边界健壮性，
   优先级应低于 `editable_rules` 与 `scan_spans` 的优化；
3. `editable_rules` 在两种语料下都是单一最大阶段（纯中文下占 66%），是首选优化目标。

## 二级归因（dev，2026-08-30）

新增 `per_rule_timings` 与 `scan_split_timings`（同样在 `profile-stages` feature 下），
对 1 MB Markdown/LaTeX 密集语料细分热点：

### 扫描拆分（1 轮）

| 阶段 | 耗时 |
| --- | ---: |
| scan_semantic（化学式/单位/数学） | 12.93 ms |
| scan_structure（Markdown/HTML/LaTeX/URL 等 20+ 扫描器） | 507.93 ms |

### 逐规则计时（整篇应用，1 轮，降序）

| 规则 key | 耗时 |
| --- | ---: |
| naming.proper-nouns | 56.25 ms |
| spacing.cjk-latin | 25.64 ms |
| spacing.cjk-number | 24.28 ms |
| spacing.around-links | 10.33 ms |
| naming.expand-abbreviations | 8.15 ms |
| 其余 9 条规则合计 | ~11.9 ms |

### 关键发现

1. **`editable_rules` 的 710 ms 中，约 575 ms 是其内部的一次全文
   `scan_all_spans`**（用于可编辑行区间判定），实际规则执行合计仅约 136 ms。
   即管线对全文共执行**两次** span 扫描（第二次在规则改写后的文本上，输入不同、
   无法直接复用），Markdown 密集语料下结构扫描是共同热点；
2. `scan_spans` 的 598 ms 中 **scan_structure 占约 508 ms**，语义扫描仅 13 ms：
   优化靶点明确在结构扫描器（多次独立全文扫描、`find_ascii_case_insensitive`
   的 O(n·m) 窗口匹配等）；
3. 规则侧前三热点为 `naming.proper-nouns`（56 ms）、`spacing.cjk-latin`（26 ms）、
   `spacing.cjk-number`（24 ms），合计约占规则耗时的 78%。

### 优化方向（按预期收益排序）

1. **结构扫描器整合**：减少独立全文遍历次数（如单遍字符扫描合并相邻扫描器、
   替换 `find_ascii_case_insensitive` 的朴素窗口匹配），预期显著降低约 508 ms；
2. **重复全文扫描削减**：评估可编辑行区间判定所需的扫描粒度（无需完整 span
   仲裁，仅判定行是否含不透明覆盖），把第一次全文扫描降级为轻量预检；
3. **前三热点规则的正则优化**（naming.proper-nouns 的逐词替换、cjk-latin/
   cjk-number 的字符级扫描可考虑 memchr/查找表）。
