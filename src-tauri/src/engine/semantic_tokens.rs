//! 语义 token 的最小层。
//!
//! 阶段 C 首先落地计量单位；数学表达式只保留类型接口，暂不扩大保护范围。

use super::unit_lexicon::{scan_measurements, MeasurementSpan};
use regex::Regex;
use std::sync::OnceLock;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SemanticTokenKind {
    Measurement,
    Temperature,
    ScientificUnit,
    MathExpression,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SemanticToken {
    pub kind: SemanticTokenKind,
    /// 识别层使用的稳定语义 key；仅用于等价判断，不参与输出规范化。
    pub canonical_unit: &'static str,
    pub start: usize,
    pub end: usize,
    pub number_end: usize,
    pub unit_start: usize,
}

impl SemanticToken {
    fn from_measurement(span: MeasurementSpan, text: &str) -> Self {
        let unit = &text[span.unit_start..span.end];
        let kind = if matches!(unit, "℃" | "℉" | "°C" | "°F") {
            SemanticTokenKind::Temperature
        } else if unit.contains('·')
            || unit.contains('⋅')
            || unit.chars().any(|ch| {
                matches!(
                    ch,
                    '⁰'..='⁹' | '₀'..='₉' | '⁺' | '⁻' | '₊' | '₋'
                )
            })
        {
            SemanticTokenKind::ScientificUnit
        } else {
            SemanticTokenKind::Measurement
        };

        Self {
            kind,
            canonical_unit: canonical_unit(unit),
            start: span.start,
            end: span.end,
            number_end: span.number_end,
            unit_start: span.unit_start,
        }
    }
}

fn canonical_unit(unit: &str) -> &'static str {
    match unit {
        "μm" | "µm" => "um",
        "Å" | "Å" => "angstrom",
        _ => "unit",
    }
}

pub(crate) fn scan_semantic_tokens(text: &str) -> Vec<SemanticToken> {
    scan_measurements(text)
        .into_iter()
        .map(|span| SemanticToken::from_measurement(span, text))
        .collect()
}

/// 扫描明确的数学表达式片段。
///
/// 首期只接受带有明确数学运算符的短表达式，避免把普通英文、URL 或自然语言
/// 中的单个符号误判为数学 token。表达式内部由保护层整体保留，外部边界仍交给
/// 现有占位符边界逻辑处理。
pub(crate) fn scan_math_expressions(text: &str) -> Vec<(usize, usize)> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        [
            r"∂[A-Za-z]+/[∂A-Za-z]+",
            r"[A-Za-z0-9]+(?:≤|≥|≈)[A-Za-z0-9]+",
            r"\d+(?:\.\d+)?(?:±|×)\d+(?:\.\d+)?",
        ]
        .into_iter()
        .map(|pattern| Regex::new(pattern).expect("invalid math expression pattern"))
        .collect()
    });

    let mut spans: Vec<(usize, usize)> = patterns
        .iter()
        .flat_map(|pattern| {
            pattern
                .find_iter(text)
                .map(|matched| (matched.start(), matched.end()))
        })
        .collect();
    spans.sort_unstable_by_key(|(start, _)| *start);
    spans.dedup();

    let mut non_overlapping = Vec::with_capacity(spans.len());
    let mut last_end = 0;
    for (start, end) in spans {
        if start >= last_end {
            non_overlapping.push((start, end));
            last_end = end;
        }
    }
    non_overlapping
}

#[cfg(test)]
mod tests {
    use super::{scan_math_expressions, scan_semantic_tokens, SemanticTokenKind};

    #[test]
    fn classifies_measurement_temperature_and_scientific_units() {
        let tokens = scan_semantic_tokens("10μm 25°C 3mg·mL⁻¹");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, SemanticTokenKind::Measurement);
        assert_eq!(tokens[1].kind, SemanticTokenKind::Temperature);
        assert_eq!(tokens[2].kind, SemanticTokenKind::ScientificUnit);
    }

    #[test]
    fn equivalent_unicode_units_share_canonical_keys_without_normalizing_output() {
        let tokens = scan_semantic_tokens("10μm 10µm 3Å 3Å");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].canonical_unit, tokens[1].canonical_unit);
        assert_eq!(tokens[2].canonical_unit, tokens[3].canonical_unit);
        assert_eq!(tokens[0].canonical_unit, "um");
        assert_eq!(tokens[2].canonical_unit, "angstrom");
    }

    #[test]
    fn scans_only_explicit_math_expression_shapes() {
        let text = "∂f/∂x x≤y a≥1 a≈b 3±0.5 2×3 chapter";
        let spans = scan_math_expressions(text);
        let values: Vec<&str> = spans
            .iter()
            .map(|(start, end)| &text[*start..*end])
            .collect();
        assert_eq!(values, vec!["∂f/∂x", "x≤y", "a≥1", "a≈b", "3±0.5", "2×3"]);
    }
}
