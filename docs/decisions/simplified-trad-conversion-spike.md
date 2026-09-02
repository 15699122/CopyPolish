# 简繁转换 Spike：opencc-fmmseg 接入

## 状态

已采纳（以 `simplified-trad-conversion` feature 接入 `opencc-fmmseg`）。

## 背景

`FormatRequest.conversion` 承载互斥的 `none` / `t2s` / `s2t` 模式（见
[unified-request-model.md](unified-request-model.md)），但仅 `None` 实际生效。
roadmap §P1「字符转换」要求先做独立 Spike，确认 Rust 依赖、许可证、词汇级语义、
跨平台构建和性能，再落地实现。本文将结论与取舍完整记录。

追加约束：项目一贯的依赖占位面是 permissive（MIT/Apache/BSD/ISC/Unlicense/MPL/LGPL），
**不含 GPL**；大型依赖与跨平台能力须先独立验证。

## 候选对比（桌面实测，release 构建，Linux）

| 维度 | zhconv 0.4.1 | ferrous-opencc 0.4.0 | opencc-fmmseg 0.11.5 |
|---|---|---|---|
| 许可证 | **GPL-2.0-or-later** | Apache-2.0 | **MIT** |
| 形态 | 纯 Rust API + zstd native | 纯 Rust（无 native） | 纯 Rust API + zstd native |
| 词典 | MediaWiki（GPL）/ OpenCC（Apache） | OpenCC | OpenCC 风格 |
| 构建时间 | ~29s（含 zstd 编译） | ~20s | ~33s（含 zstd 编译） |
| 二进制增量 | 3.2 MB | 2.8 MB | **2.1 MB** |
| `t2s("乾燥")` | 漏转为“乾燥” | “干燥”（正确） | “干燥”（正确） |
| `t2s("網路與硬體")` | 网路与硬体 | 网路与硬体 | 网路与硬体 |
| 1 MB `s2t` 性能 | ~3.5ms（≈288 MB/s） | ~78ms（≈13 MB/s） | ~7.7ms（**≈130 MB/s**） |

- `zhconv` 的 crate 许可证为 GPL-2.0-or-later，即使关闭默认 MediaWiki 数据、
  改用 OpenCC 数据也不改变它是 GPL 释放的依赖 —— **与项目 permissive 取向冲突，排除**。
- `ferrous-opencc` 满足许可/纯 Rust，但性能仅 ~13 MB/s，为三者最低。
- `opencc-fmmseg` 综合最优：**MIT、体积最小（2.1 MB）、性能 ~130 MB/s、
  语义（`乾燥→干燥`）正确**。

## 决策

采用 **`opencc-fmmseg` 0.11.5**：

1. 以可选 feature `simplified-trad-conversion` 引入，默认构建不启用，
   避免为所有用户引入 native（`zstd-sys`）编译与体积开销。
2. T2S/S2T 分别以 `OnceLock` 缓存 `OpenCC` 单例（内置压缩词典解压成本高），
   `convert_with_config(&self)` 可安全跨请求共享。
3. 转换只作用于**可编辑区间**：复用 `opaque_ranges` 排除 Markdown 链接、URL、
   行内/fenced 代码、LaTeX 与化学式 span，改写正文。
4. 互斥由 `CharacterConversion` 在类型层保证；`None` 恒为原文。

## 语义边界（首版已知，不在此版本修复）

`opencc-fmmseg` 的词典把某些台湾地区词按字面转为“大陆简体”：

- `t2s("網路") -> 网路`（大陆应为“网络”）
- `t2s("硬體") -> 硬体`（大陆应为“硬件”）

正确转：`t2s("乾燥") -> 干燥`、`s2t("元数据") -> 元數據` 等常用词处理良好。
这类地区词误转符合 OpenCC 词典的有限语义，GUI 侧应提供“转换前预览/可撤销”，
不作为可自动规避的缺陷；fixture 只断言无歧义样本，不在正确断言中固化误转。

## 验证

- Rust：默认占位回归、`--features simplified-trad-conversion` 全量、TUI 组合均通过；
  Clippy、fmt 通过；`cargo audit` 通过（无新增漏洞，较基线多 1 项允许警告：
  `serde_cbor` RUSTSEC-2021-0127，为 opencc-fmmseg 的传递依赖，仅在本地解压内置
  词典、不解析用户输入，风险可控并列入允许清单）。
- fixture：`simplified-trad-conversion.yaml` 覆盖基本 T2S/S2T、结构保护
  （链接/行内代码/fenced 代码不被改写）、只转可编辑区间。
- 许可证：`docs/licenses.md` 以 `generate_licenses.py --features simplified-trad-conversion`
  重新生成，新增 `opencc-fmmseg`(MIT)、`rayon`、`zstd*`、`serde_cbor` 等 12 条，均 permissive。

## 接入点

- `src-tauri/Cargo.toml`：`opencc-fmmseg`（optional）+ `simplified-trad-conversion` feature
- `src-tauri/src/engine/pipeline.rs`：feature 门控的 `apply_character_conversion` /
  `convert_editable`
- `scripts/generate_licenses.py`：新增 `--features` 参数以便许可证清单纳入可选依赖
- 测试：`src-tauri/tests/fixtures/simplified-trad-conversion.yaml` +
  feature/non-feature 单测
- 文档：`architecture.md`、`roadmap.md`、`README.md`、`CHANGELOG.md`

## 备注 / 风险

- 依赖含 `zstd-sys`（vendored C，跨平台常见）与 `rayon`（多线程）；需在
  Windows CI 实测一次生产构建以确认工具链兼容。
- GUI/TUI 尚无转换控件；默认构建不启用该 feature，接入 UI 时再决定是否随安装默认开启。
- 全角半角与 Unicode 等价字符转换属另一独立条目，不在此 Spike 范围。