# PDF/CAJ 真实语料验收准备

本目录只保存验收模板、清单和脱敏后的文本摘要，不保存未经许可的 PDF、CAJ、截图或原始复制内容。真实原件应放在 Windows 本地受控目录（建议 `E:\\CopyPolish-private\\pdf-caj\\`），验收结束后按来源许可和组织保留策略处理。

## 目标

为 `cleanup.cjk-internal-space` 保守试实现建立真实来源证据，并评估未来段内软换行规则。当前不解析 PDF/CAJ 文件本体，也不把合成 fixture 当作真实来源通过。

## 需要准备的样本

至少准备并取得许可或完成脱敏：

1. 单栏 PDF：正文、标题、列表、页眉/页脚；
2. 多栏 PDF：记录人工期望阅读顺序；
3. CAJViewer 复制文本：中文、英文、数字、公式、表格；
4. 每类至少一条应合并、一条应保留、一条无法判断的行边界；
5. CJK 单 ASCII 空格的正例、反例和结构保护样本；
6. LF/CRLF、连字符断词、Unicode 组合字符样本。

## 操作步骤

1. 在 Windows 本地受控目录保存原始文件，记录来源许可、文件 hash、来源类型、是否多栏、提取工具/版本和日期。
2. 使用 PDF 阅读器或 CAJViewer 复制文本；不要让 CopyPolish 读取原始 PDF/CAJ 文件本体。
3. 将复制结果脱敏后保存为本地 `.txt`，为每个样本建立一条 `manifest.yaml` 记录。
4. 人工标注每个候选行边界：`merge`、`keep` 或 `unknown`；标注 CJK 空格应删除/保留及理由。
5. 在 E 盘 Windows checkout 中通过 GUI/TUI/CLI 输入脱敏文本，分别记录规则关闭和开启的输出、差异、幂等性、换行风格和误删/漏删。
6. 只把脱敏后的最小文本片段和标注摘要加入评审包；原件、截图、日志和临时 artifact 不提交仓库。

## 验收门槛

- 每条样本均有来源许可/脱敏说明和 hash；
- `merge`、`keep`、`unknown` 三类边界均有覆盖；
- 保护 Markdown、代码、URL、LaTeX、HTML、表格、公式和多栏顺序；
- 报告 precision/recall、误删率、漏合并率和无法判断比例；
- 输出保持幂等并保留 LF/CRLF；
- 在达到门槛前，不修改 roadmap 中 PDF/CAJ Spike 的未完成状态。

## 文件模板

- `manifest.example.yaml`：样本元数据与许可记录模板；
- `annotation-template.md`：逐条边界和输出差异记录模板。

