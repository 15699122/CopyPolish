// engine/pipeline.rs
// =============================================================================
// 格式化主流程：
//   1. 归一化换行符（处理后还原）；
//   2. 保护层：先化学式，再 Markdown / LaTeX / URL / 邮箱；
//   3. 缩进代码行整行占位；
//   4. 逐行按 registry 注册顺序应用已启用规则；
//   5. 行内占位符补边界空格 -> 还原全部占位符。
//
// 规则选择由 `RuleSelection` 显式表达；未知 key 安全忽略。
// =============================================================================

use std::collections::HashSet;

use super::model::{FormatRequest, RuleSelection};
use super::protection::{
    is_placeholder_line, placeholder, protect, protect_byte_spans, protect_byte_spans_with_offset,
    protect_markdown_lines, restore, restore_escaped_markdown_adjacency,
    space_around_inline_placeholders, space_around_math_placeholders,
};
use super::registry::{execution_rules, rules};
use super::semantic_tokens::scan_math_expressions;
use super::spans::{scan_all_spans, TextSpan};
use super::tokenizer::detect_chemical_formulas;

pub fn format_text(req: &FormatRequest) -> Result<String, String> {
    let enabled: HashSet<String> = match &req.selection {
        RuleSelection::All => rules().iter().map(|rule| rule.key().to_string()).collect(),
        RuleSelection::Defaults => super::registry::enabled_defaults().into_iter().collect(),
        RuleSelection::Only { keys } => keys.iter().cloned().collect(),
        RuleSelection::None => HashSet::new(),
    };

    let (text, newline) = normalize_newlines(&req.text);

    let mut placeholders: Vec<(String, String)> = Vec::new();

    // 化学式保护必须最先执行：识别基于原始文本的字节区间。
    let chem_spans = detect_chemical_formulas(&text);
    let text = protect_byte_spans(&text, &chem_spans, &mut placeholders);

    // 数学 token 单独保存：表达式内部需要保护，但不应参与 Markdown/链接占位符
    // 的通用边界补空格，否则会在全角逗号后产生非预期空格。
    let math_spans = scan_math_expressions(&text);
    let mut math_placeholders: Vec<(String, String)> = Vec::new();
    let text =
        protect_byte_spans_with_offset(&text, &math_spans, &mut math_placeholders, 1_000_000);

    let protected = protect(&text, &mut placeholders)?;
    let line_protected = protect_markdown_lines(&protected, &mut placeholders);

    let registered = execution_rules();
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
        for rule in &registered {
            if enabled.contains(rule.key()) {
                current = (rule.apply)(&current);
            }
        }
        out.push(current);
    }

    let formatted = out.join("\n");
    let formatted = space_around_inline_placeholders(&formatted, &placeholders);
    let formatted = space_around_math_placeholders(&formatted, &math_placeholders);
    let restored = restore(&formatted, &placeholders);
    let restored = restore(&restored, &math_placeholders);
    let restored = restore_escaped_markdown_adjacency(&restored, &placeholders);
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

// ---------------------------------------------------------------------------
// span 感知混合管线（roadmap §5.7 R3 收尾的探索入口）。
//
// 这是「placeholder → TextEdit」重构的对照骨架：用 scan_all_spans 划定的
// 不可编辑区间替代 protection 的部分占位符，然后对可编辑区间复用现有
// execution_rules 纯函数规则。当前仅作测试探针，不替换生产 format_text。
// polymorphic 差异（URL / LaTeX / 邮箱等 spans 仍未覆盖的结构）会在对照中
// 暴露出来，作为后续补齐扫描器的依据。
// ---------------------------------------------------------------------------

fn enabled_set(req: &FormatRequest) -> HashSet<String> {
    match &req.selection {
        RuleSelection::All => rules().iter().map(|rule| rule.key().to_string()).collect(),
        RuleSelection::Defaults => super::registry::enabled_defaults().into_iter().collect(),
        RuleSelection::Only { keys } => keys.iter().cloned().collect(),
        RuleSelection::None => HashSet::new(),
    }
}

/// 把「不透明结构」span 替代入主占位符，返回受控文本与占位符表。
/// 语义原子（测量/温度/科学单位/数学）不占位——它们应作为普通文本参与
/// 逐行规则（如 `spacing.number-unit`、`temperature-cjk`），与生产一致。
fn protect_spans(text: &str, spans: &[TextSpan]) -> (String, Vec<(String, String)>) {
    let opaque: Vec<&TextSpan> = spans
        .iter()
        .filter(|span| span.priority == super::spans::SpanPriority::OpaqueStructure)
        .collect();
    let mut output = String::with_capacity(text.len());
    let mut placeholders = Vec::with_capacity(opaque.len());
    let mut cursor = 0usize;
    for (index, span) in opaque.iter().enumerate() {
        output.push_str(&text[cursor..span.start]);
        let ph = placeholder(index);
        placeholders.push((ph.clone(), text[span.start..span.end].to_string()));
        output.push_str(&ph);
        cursor = span.end;
    }
    output.push_str(&text[cursor..]);
    (output, placeholders)
}

/// span 感知混合管线：用 span 划分不可编辑区间 → 可编辑区间复用纯函数规则
/// → 还原。仅供测试/对照，不改变生产路径。
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn format_text_span_aware(req: &FormatRequest) -> Result<String, String> {
    let enabled = enabled_set(req);
    let (text, newline) = normalize_newlines(&req.text);

    let spans = scan_all_spans(&text);
    let (protected, placeholders) = protect_spans(&text, &spans);

    let registered = execution_rules();
    let mut out: Vec<String> = Vec::new();
    for line in protected.split('\n') {
        if line.trim().is_empty() {
            out.push(String::new());
            continue;
        }
        if is_placeholder_line(line) {
            out.push(line.to_string());
            continue;
        }
        let mut current = line.to_string();
        for rule in &registered {
            if enabled.contains(rule.key()) {
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
