//! 语义 token 的最小层。
//!
//! 阶段 C 首先落地计量单位；数学表达式只保留类型接口，暂不扩大保护范围。

use super::unit_lexicon::{scan_measurements, MeasurementSpan};

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
            start: span.start,
            end: span.end,
            number_end: span.number_end,
            unit_start: span.unit_start,
        }
    }
}

pub(crate) fn scan_semantic_tokens(text: &str) -> Vec<SemanticToken> {
    scan_measurements(text)
        .into_iter()
        .map(|span| SemanticToken::from_measurement(span, text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{scan_semantic_tokens, SemanticTokenKind};

    #[test]
    fn classifies_measurement_temperature_and_scientific_units() {
        let tokens = scan_semantic_tokens("10μm 25°C 3mg·mL⁻¹");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, SemanticTokenKind::Measurement);
        assert_eq!(tokens[1].kind, SemanticTokenKind::Temperature);
        assert_eq!(tokens[2].kind, SemanticTokenKind::ScientificUnit);
    }
}
