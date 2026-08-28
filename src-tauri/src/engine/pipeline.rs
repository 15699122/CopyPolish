// engine/pipeline.rs
// =============================================================================
// 格式化主流程：
//   1. 归一化换行符（处理后还原）；
//   2. 保护层：先化学式，再 Markdown / LaTeX / URL / 邮箱；
//   3. 缩进代码行整行占位；
//   4. 在可编辑区间通过 TextEdit 应用标点/名词规则；
//   5. 对受保护结构外的文本应用剩余规则；
//   6. 行内占位符补边界空格 -> 还原全部占位符。
//
// 规则选择由 `RuleSelection` 显式表达；未知 key 安全忽略。
// =============================================================================

use std::collections::HashSet;

use super::edit_plan::apply_editable_rules;
use super::model::{FormatRequest, RuleSelection};
use super::protection::{
    is_placeholder_line, placeholder, restore, space_around_inline_placeholders,
    space_around_math_placeholders,
};
use super::registry::{execution_rules, rules};
use super::spans::{scan_all_spans, TextSpan};

/// 格式化文本的正式入口。
pub fn format_text(req: &FormatRequest) -> Result<String, String> {
    format_text_impl(req)
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
// span 感知格式化管线。
//
// 用 scan_all_spans 划定不可编辑区间，再对可编辑区间复用现有规则函数。
// 当前保护层仍使用内部占位符承载不可编辑 span；后续将继续把非边界规则
// 迁移到 edit_plan.rs 的 TextEdit 模型。
// ---------------------------------------------------------------------------

fn enabled_set(req: &FormatRequest) -> HashSet<String> {
    match &req.selection {
        RuleSelection::All => rules().iter().map(|rule| rule.key().to_string()).collect(),
        RuleSelection::Defaults => super::registry::enabled_defaults().into_iter().collect(),
        RuleSelection::Only { keys } => keys.iter().cloned().collect(),
        RuleSelection::None => HashSet::new(),
    }
}

/// 把“不透明结构”span 替代入内部保护占位符，返回受控文本与占位符表。
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

/// 用 span 划分不可编辑区间，再格式化可编辑内容。
fn format_text_impl(req: &FormatRequest) -> Result<String, String> {
    let enabled = enabled_set(req);
    let (text, newline) = normalize_newlines(&req.text);
    let text = apply_editable_rules(&text, &req.selection)?;

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
            if enabled.contains(rule.key())
                && !matches!(
                    rule.phase,
                    super::registry::RulePhase::PunctuationNormalization
                        | super::registry::RulePhase::NamingNormalization
                )
            {
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
