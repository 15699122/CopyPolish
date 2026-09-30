# PDF/CAJ 样本标注模板

- Sample ID: `<id>`
- Source type: `pdf-single-column | pdf-multi-column | cajviewer-copy`
- Permission/redaction record: `<reference>`
- Extraction tool/version: `<tool>`
- Input text hash: `<sha256>`

## 行边界标注

| boundary | context summary | label | expected output | reason |
| --- | --- | --- | --- | --- |
| 1 | `<脱敏摘要>` | `merge/keep/unknown` | `<text or unchanged>` | `<reason>` |

## CJK 空格标注

| case | context summary | label | protected structure | reason |
| --- | --- | --- | --- | --- |
| 1 | `<脱敏摘要>` | `remove/keep/protected` | `<none/code/table/...>` | `<reason>` |

## 运行记录

- Rule disabled output: `<local artifact>`
- Rule enabled output: `<local artifact>`
- Idempotence: `pass/fail`
- Newline style preserved: `yes/no`
- False deletions: `<count and IDs>`
- Missed merges/removals: `<count and IDs>`
- Unknown cases changed: `<count and IDs>`
- Reviewer/date: `<reviewer> / YYYY-MM-DD`

