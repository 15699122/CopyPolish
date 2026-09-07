# 来源文本清洗语料规范

> 本文档定义真实 PDF/CAJ 复制文本的收集、脱敏、标注和评测规范。
> 适用于 `cleanup.cjk-internal-space` 的正式验收与未来段内软换行规则的基线建设。

## 1. 目标

建立可复现的误改率基线，验证来源文本清洗规则在真实语料上的行为：

- **True Positive (TP)**：正确清理的异常
- **False Positive (FP)**：误删或误合并（结构破坏、语义改变）
- **False Negative (FN)**：应处理但漏处理
- **Unknown**：人工无法判断

高风险规则的首要指标是 **误改率 (FP / (TP + FP))**，而不是召回率。

## 2. 语料收集范围

每条样本应至少覆盖以下来源类型：

1. 单栏 PDF 正文（含段落、标题、列表、页眉页脚）
2. 多栏 PDF（记录预期阅读顺序）
3. CAJViewer 复制文本（中英文、数字、公式、表格）
4. Zotero 复制文本
5. 网页复制文本（作为对照）

## 3. 样本格式

```yaml
id: sample-001
source_type: pdf-single-column  # pdf-single-column | pdf-multi-column | caj | zotero | web
layout: single                   # single | multi | unknown
locale: zh-CN
input: |
  这是从PDF复 制的中 文文本。
expected: |
  这是从PDF复制的中文文本。
allowed_changes:
  - cleanup.cjk-internal-space
must_preserve:
  - 连续空格
  - 汉字与拉丁/数字边界
  - 代码块与链接内容
annotation: |
  字符定位误差导致的单空格，应删除。
confidence: high                 # high | medium | low
failure_category: null           # null | fp | fn | unknown
```

## 4. 隐私与合规

- **原始 PDF、用户文档、截图不入库**
- 仅提交获得许可或彻底脱敏的纯文本
- 不得包含真实个人信息、未公开文档、凭据或日志
- 不可提交仓库的语料放在 `src-tauri/tests/fixtures/corpus-local/`（已加入 `.gitignore`）

## 5. 评测脚本

`scripts/evaluate_corpus.py` 执行本地评测：

```bash
python3 scripts/evaluate_corpus.py \
  --fixtures src-tauri/tests/fixtures/corpus-local/ \
  --rules cleanup.cjk-internal-space
```

输出每个样本的匹配情况与汇总误改率。

## 6. Go/No-Go 门槛

进入生产规则调整前，应满足：

- 至少 20 条真实样本，覆盖单栏/多栏/CAJ/Zotero
- 误改率 < 2%（按字符数加权）
- 每个 FP 案例都有明确原因与修复方案
- Markdown/HTML/LaTeX/URL/邮箱/代码/化学式零结构破坏
- 换行风格保持
