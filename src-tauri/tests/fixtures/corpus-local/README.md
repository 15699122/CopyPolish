# 本地来源文本语料目录

本目录用于存放真实 PDF/CAJ 复制文本的脱敏语料，供 `scripts/evaluate_corpus.py` 本地评测使用。

**所有 `.yaml` 文件已被 `.gitignore` 忽略，不会提交到仓库。**

## 文件格式

```yaml
- id: sample-001
  source_type: pdf-single-column
  input: |
    这是从PDF复 制的中 文文本。
  expected: |
    这是从PDF复制的中文文本。
  annotation: |
    字符定位误差导致的单空格，应删除。
```

## 运行评测

```bash
cargo build --manifest-path src-tauri/Cargo.toml --features tui --bin copypolish-tui

python3 scripts/evaluate_corpus.py \
  --fixtures src-tauri/tests/fixtures/corpus-local/ \
  --rules cleanup.cjk-internal-space
```

## 隐私提醒

- 不得包含真实个人信息、未公开文档、凭据或日志
- 仅提交获得许可或彻底脱敏的纯文本
