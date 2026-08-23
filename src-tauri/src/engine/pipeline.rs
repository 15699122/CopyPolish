// engine/pipeline.rs
// =============================================================================
// 格式化主流程：
//   1. 归一化换行符（处理后还原）；
//   2. 保护层：先化学式，再 Markdown / LaTeX / URL / 邮箱；
//   3. 缩进代码行整行占位；
//   4. 逐行按 registry 注册顺序应用已启用规则；
//   5. 行内占位符补边界空格 -> 还原全部占位符。
//
// `enabled` 为空表示全部启用；未知 key 安全忽略。
// =============================================================================

use std::collections::HashSet;

use super::model::FormatRequest;
use super::protection::{
    is_placeholder_line, protect, protect_byte_spans, protect_markdown_lines, restore,
    space_around_inline_placeholders,
};
use super::registry::rules;
use super::tokenizer::detect_chemical_formulas;

pub fn format_text(req: &FormatRequest) -> Result<String, String> {
    let enabled_all = req.enabled.is_empty();
    let enabled: HashSet<&str> = req.enabled.iter().map(String::as_str).collect();

    let (text, newline) = normalize_newlines(&req.text);

    let mut placeholders: Vec<(String, String)> = Vec::new();

    // 化学式保护必须最先执行：识别基于原始文本的字节区间。
    let chem_spans = detect_chemical_formulas(&text);
    let text = protect_byte_spans(&text, &chem_spans, &mut placeholders);

    let protected = protect(&text, &mut placeholders)?;
    let line_protected = protect_markdown_lines(&protected, &mut placeholders);

    let registered = rules();
    let mut out: Vec<String> = Vec::new();
    for line in line_protected.split('\n') {
        if line.trim().is_empty() {
            // 空白行规范化为空行。
            out.push(String::new());
            continue;
        }
        if is_placeholder_line(line) {
            out.push(line.to_string());
            continue;
        }

        let mut current = line.to_string();
        for rule in registered {
            if enabled_all || enabled.contains(rule.key()) {
                current = (rule.apply)(&current);
            }
        }
        out.push(current);
    }

    let formatted = out.join("\n");
    let formatted = space_around_inline_placeholders(&formatted, &placeholders);
    let restored = restore(&formatted, &placeholders);
    Ok(restore_newlines(&restored, newline))
}

fn normalize_newlines(text: &str) -> (String, &'static str) {
    if text.contains("\r\n") {
        (text.replace("\r\n", "\n"), "\r\n")
    } else if text.contains('\r') {
        (text.replace('\r', "\n"), "\r")
    } else {
        (text.to_string(), "\n")
    }
}

fn restore_newlines(text: &str, newline: &str) -> String {
    if newline == "\n" {
        text.to_string()
    } else {
        text.replace('\n', newline)
    }
}
