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
    is_placeholder_line, placeholder, restore, space_around_inline_placeholders,
    space_around_math_placeholders,
};
#[cfg(test)]
use super::protection::{
    protect, protect_byte_spans, protect_byte_spans_with_offset, protect_markdown_lines,
    restore_escaped_markdown_adjacency,
};
use super::registry::{execution_rules, rules};
#[cfg(test)]
use super::semantic_tokens::scan_math_expressions;
use super::spans::{scan_all_spans, TextSpan};
#[cfg(test)]
use super::tokenizer::detect_chemical_formulas;

/// 迁移期保留的旧 placeholder 管线，仅供新旧路径等价性回归测试使用。
#[cfg(test)]
pub(crate) fn format_text_legacy(req: &FormatRequest) -> Result<String, String> {
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

/// 格式化文本的正式入口。
///
/// 生产路径已切换到 span-aware 混合管线；旧 placeholder 管线暂时保留在
/// 本模块内，待发布后完成输出与性能观察后再删除。
pub fn format_text(req: &FormatRequest) -> Result<String, String> {
    format_text_span_aware(req)
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
// 这是「placeholder → TextEdit」重构的 span-aware 混合实现：用 scan_all_spans
// 划定的不可编辑区间替代 protection 的部分占位符，然后对可编辑区间复用现有
// execution_rules 纯函数规则。当前已接入生产 format_text；旧 placeholder 管线
// 暂时保留为迁移期对照实现；
// URL / 邮箱、硬换行、引用式链接、未闭合反引号及数学复合单位等结构已纳入
// span 扫描和对照门禁，后续重点转为旧 placeholder 清理、完整 TextEdit 迁移和性能验证。
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
type Placeholder = (String, String);

struct ProtectedSpans {
    text: String,
    all: Vec<Placeholder>,
    inline: Vec<Placeholder>,
    math: Vec<Placeholder>,
}

fn protect_spans(text: &str, spans: &[TextSpan]) -> ProtectedSpans {
    let mut opaque: Vec<TextSpan> = spans
        .iter()
        .filter(|span| {
            span.priority == super::spans::SpanPriority::OpaqueStructure
                || span.kind == super::spans::SpanKind::ChemicalFormula
        })
        .copied()
        .collect();

    // `2×3cm²` 会被数学扫描识别为 `2×3`、单位扫描识别为 `3cm²`。
    // 生产 pipeline 的占位符边界不会在二者之间插空，因此混合管线需要将
    // 数学表达式后紧邻的单位后缀扩展进同一保护区间。
    for span in spans
        .iter()
        .filter(|span| span.kind == super::spans::SpanKind::MathExpression)
    {
        let mut end = span.end;
        while let Some(ch) = text[end..].chars().next() {
            let next_end = end + ch.len_utf8();
            if ch.is_ascii_alphanumeric()
                || matches!(ch, '²' | '³' | '⁰'..='⁹' | '₀'..='₉' | '⁻' | '⁺')
            {
                end = next_end;
            } else {
                break;
            }
        }
        if end > span.end {
            opaque.push(TextSpan {
                start: span.start,
                end,
                kind: span.kind,
                priority: super::spans::SpanPriority::OpaqueStructure,
            });
        }
    }
    opaque.sort_by_key(|span| (span.start, std::cmp::Reverse(span.end)));
    opaque = super::spans::arbitrate_spans(opaque);
    let mut output = String::with_capacity(text.len());
    let mut placeholders = Vec::with_capacity(opaque.len());
    let mut inline_placeholders = Vec::with_capacity(opaque.len());
    let mut math_placeholders = Vec::with_capacity(opaque.len());
    let mut cursor = 0usize;
    for (index, span) in opaque.iter().enumerate() {
        output.push_str(&text[cursor..span.start]);
        let ph = placeholder(index);
        let value = text[span.start..span.end].to_string();
        placeholders.push((ph.clone(), value.clone()));
        if span.kind == super::spans::SpanKind::LatexMath {
            math_placeholders.push((ph.clone(), value));
        } else {
            inline_placeholders.push((ph.clone(), value));
        }
        output.push_str(&ph);
        cursor = span.end;
    }
    output.push_str(&text[cursor..]);
    ProtectedSpans {
        text: output,
        all: placeholders,
        inline: inline_placeholders,
        math: math_placeholders,
    }
}

/// span 感知混合管线：用 span 划分不可编辑区间 → 可编辑区间复用纯函数规则
/// → 还原。仅供测试/对照，不改变生产路径。
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn format_text_span_aware(req: &FormatRequest) -> Result<String, String> {
    let enabled = enabled_set(req);
    let (text, newline) = normalize_newlines(&req.text);

    let spans = scan_all_spans(&text);
    let protected_spans = protect_spans(&text, &spans);
    let protected = protected_spans.text;

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
    let formatted = space_around_inline_placeholders(&formatted, &protected_spans.inline);
    let formatted = space_around_math_placeholders(&formatted, &protected_spans.math);
    let restored = restore(&formatted, &protected_spans.all);
    Ok(restore_newlines(&restored, newline))
}
