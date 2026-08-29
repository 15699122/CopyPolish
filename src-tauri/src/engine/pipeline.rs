// engine/pipeline.rs
// =============================================================================
// 格式化主流程（全部规则执行已收敛到 TextEdit 应用层）：
//   1. 归一化换行符（处理后还原）；
//   2. 在可编辑区间通过 TextEdit 应用标点/名词规范化规则；
//   3. 保护层：不透明结构 span（含化学式）转为内部占位符；
//   4. 在受保护文本上通过 TextEdit 应用结构边界/文本边界/清理规则；
//   5. 行内占位符补边界空格 -> 还原全部占位符。
//
// 规则选择由 `RuleSelection` 显式表达；未知 key 安全忽略。
// =============================================================================

use super::edit_plan::{apply_editable_rules, apply_protected_text_rules};
use super::model::FormatRequest;
use super::protection::{
    placeholder, restore, space_around_inline_placeholders, space_around_math_placeholders,
};
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
// 用 scan_all_spans 划定不可编辑区间：可编辑规则在原文上以 TextEdit 应用，
// 其余规则在受保护文本上以 TextEdit 应用。占位符仅用于承载不可编辑 span；
// 全部规则执行已收敛到 edit_plan.rs 的 TextEdit 模型。
// ---------------------------------------------------------------------------

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
    let (text, newline) = normalize_newlines(&req.text);

    // 标点/名词规范化：原文上的可编辑区间 TextEdit。
    let text = apply_editable_rules(&text, &req.selection)?;

    let spans = scan_all_spans(&text);
    let protected_spans = protect_spans(&text, &spans);

    // 结构边界/文本边界/清理规则：受保护文本上的 TextEdit，
    // 顺序与迁移前保护层行循环一致，随后补占位符边缘空格并还原。
    let formatted = apply_protected_text_rules(&protected_spans.text, &req.selection)?;
    let formatted = space_around_inline_placeholders(&formatted, &protected_spans.inline);
    let formatted = space_around_math_placeholders(&formatted, &protected_spans.math);
    let restored = restore(&formatted, &protected_spans.all);
    Ok(restore_newlines(&restored, newline))
}
