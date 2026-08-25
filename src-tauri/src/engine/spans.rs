//! 统一文本 span 模型与重叠仲裁。
//!
//! 本模块暂不替换现有占位符管线，只为后续 TextSpan/TextEdit 重构提供稳定的
//! 优先级和重叠规则。仲裁结果按原文位置排序，调用方可以安全地按字节区间消费。

use super::semantic_tokens::scan_math_expressions;
use super::tokenizer::detect_chemical_formulas;
use super::unit_lexicon::scan_measurements;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpanPriority {
    Editable = 0,
    SemanticAtomic = 1,
    OpaqueStructure = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpanKind {
    EditableText,
    ChemicalFormula,
    Measurement,
    Temperature,
    ScientificUnit,
    MathExpression,
    InlineCode,
    MarkdownLink,
    HtmlBlock,
    LatexMath,
}

impl SpanKind {
    pub(crate) fn priority(self) -> SpanPriority {
        match self {
            Self::EditableText => SpanPriority::Editable,
            Self::ChemicalFormula
            | Self::Measurement
            | Self::Temperature
            | Self::ScientificUnit
            | Self::MathExpression => SpanPriority::SemanticAtomic,
            Self::InlineCode | Self::MarkdownLink | Self::HtmlBlock | Self::LatexMath => {
                SpanPriority::OpaqueStructure
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextSpan {
    pub start: usize,
    pub end: usize,
    pub kind: SpanKind,
    pub priority: SpanPriority,
}

impl TextSpan {
    pub(crate) fn new(start: usize, end: usize, kind: SpanKind) -> Option<Self> {
        (start < end).then_some(Self {
            start,
            end,
            priority: kind.priority(),
            kind,
        })
    }

    pub(crate) fn len(self) -> usize {
        self.end - self.start
    }

    fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// 以“优先级高 > 同优先级更长 > 更早出现”为规则选择不重叠 span。
pub(crate) fn arbitrate_spans(mut candidates: Vec<TextSpan>) -> Vec<TextSpan> {
    candidates.sort_by_key(|span| {
        (
            std::cmp::Reverse(span.priority),
            span.start,
            std::cmp::Reverse(span.len()),
        )
    });

    let mut accepted: Vec<TextSpan> = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if accepted
            .iter()
            .copied()
            .all(|span| !span.overlaps(candidate))
        {
            accepted.push(candidate);
        }
    }
    accepted.sort_by_key(|span| (span.start, span.end));
    accepted
}

/// 将现有化学式、单位和数学扫描结果汇总为统一语义 span。
///
/// 结构保护 span 尚未接入这里；后续 Markdown scanner 可将结构 span 与本结果
/// 合并后再次调用 `arbitrate_spans`，从而让结构自动覆盖内部语义 token。
pub(crate) fn scan_semantic_spans(text: &str) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    for (start, end) in detect_chemical_formulas(text) {
        if let Some(span) = TextSpan::new(start, end, SpanKind::ChemicalFormula) {
            spans.push(span);
        }
    }
    for measurement in scan_measurements(text) {
        let kind = if measurement
            .unit_start
            .checked_sub(measurement.number_end)
            .is_some_and(|_| {
                matches!(
                    &text[measurement.unit_start..measurement.end],
                    "℃" | "℉" | "°C" | "°F"
                )
            }) {
            SpanKind::Temperature
        } else if text[measurement.unit_start..measurement.end]
            .chars()
            .any(|ch| matches!(ch, '·' | '⋅' | '⁰'..='⁹' | '₀'..='₉' | '⁺' | '⁻'))
        {
            SpanKind::ScientificUnit
        } else {
            SpanKind::Measurement
        };
        if let Some(span) = TextSpan::new(measurement.start, measurement.end, kind) {
            spans.push(span);
        }
    }
    for (start, end) in scan_math_expressions(text) {
        if let Some(span) = TextSpan::new(start, end, SpanKind::MathExpression) {
            spans.push(span);
        }
    }
    arbitrate_spans(spans)
}

#[cfg(test)]
mod tests {
    use super::{arbitrate_spans, scan_semantic_spans, SpanKind, SpanPriority, TextSpan};

    #[test]
    fn opaque_structure_wins_over_inner_semantic_span() {
        let spans = arbitrate_spans(vec![
            TextSpan::new(2, 7, SpanKind::Measurement).unwrap(),
            TextSpan::new(0, 12, SpanKind::InlineCode).unwrap(),
        ]);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].kind, SpanKind::InlineCode);
        assert_eq!(spans[0].priority, SpanPriority::OpaqueStructure);
    }

    #[test]
    fn longer_same_priority_span_wins() {
        let spans = arbitrate_spans(vec![
            TextSpan::new(0, 5, SpanKind::MathExpression).unwrap(),
            TextSpan::new(0, 8, SpanKind::ScientificUnit).unwrap(),
        ]);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].end, 8);
    }

    #[test]
    fn semantic_scanners_share_one_arbitration_result() {
        let spans = scan_semantic_spans("样品Fe²⁺厚度10μm且计算∂f/∂x");
        let kinds: Vec<SpanKind> = spans.iter().map(|span| span.kind).collect();
        assert_eq!(
            kinds,
            vec![
                SpanKind::ChemicalFormula,
                SpanKind::Measurement,
                SpanKind::MathExpression
            ]
        );
        assert!(spans.windows(2).all(|pair| pair[0].end <= pair[1].start));
    }

    #[test]
    fn invalid_empty_span_is_rejected() {
        assert!(TextSpan::new(4, 4, SpanKind::EditableText).is_none());
        assert!(TextSpan::new(8, 3, SpanKind::EditableText).is_none());
    }
}
