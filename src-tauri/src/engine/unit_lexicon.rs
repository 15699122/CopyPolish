//! 有限单位词典与计量片段扫描。
//!
//! 这里故意不使用 `\p{L}+` 之类的宽泛模式。单位识别必须来自有限词典，
//! 否则 `10chapter`、`2beta` 等普通文本会被误判为计量单位。

use regex::Regex;
use std::sync::OnceLock;

/// 由数字、可选空白和有限单位组成的计量片段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeasurementSpan {
    pub start: usize,
    pub number_end: usize,
    pub unit_start: usize,
    pub end: usize,
}

fn measurement_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // 复合单位放在前面，避免先匹配其中的简单 ASCII 片段。
        // 单位前后的 ASCII 边界在 scan_measurements 中用字节区间检查，
        // 因为 regex crate 不支持 look-around。
        let unit = concat!(
            r"(?:",
            r"(?:mg|kg|mol)\s*(?:[·⋅])\s*[A-Za-z]+[⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎]+",
            r"|",
            r"(?:mg|kg|mol|mL|m|g|s|L|Pa|Hz|N|J|W|V|A|rad|rpm|px|eV|mmHg)\s*[/／]\s*(?:mg|kg|mol|mL|m|g|s|L|Pa|Hz|N|J|W|V|A|rad|rpm|px|eV|mmHg)(?:[⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎]+)?",
            r"|",
            r"(?:mmHg|hPa|dB|rpm|Hz|Pa|mol|rad|px|eV|TB|GB|Gbps|Mbps|kΩ|MΩ|GΩ|Ω|cm|cL)",
            r"|",
            r"(?:k|M|G|T|m|μ|µ|n|p)?(?:m|g|s|L|K|Pa|Hz|N|J|W|V|A|B)",
            r"|",
            r"(?:Å|Å|℃|℉|°C|°F|‰|%)",
            r")"
        );
        Regex::new(&format!(
            r"(?P<number>\d+(?:\.\d+)?)(?P<space>\s*)(?P<unit>{unit})"
        ))
        .expect("invalid measurement lexicon pattern")
    })
}

/// 扫描文本中的有限词典计量片段。
pub(crate) fn scan_measurements(text: &str) -> Vec<MeasurementSpan> {
    measurement_re()
        .captures_iter(text)
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            let number = captures.name("number")?;
            let unit = captures.name("unit")?;

            let before = text[..whole.start()].chars().next_back();
            let after = text[whole.end()..].chars().next();
            if before.is_some_and(|ch| ch.is_ascii_alphanumeric())
                || after.is_some_and(|ch| ch.is_ascii_alphanumeric())
            {
                return None;
            }

            Some(MeasurementSpan {
                start: whole.start(),
                number_end: number.end(),
                unit_start: unit.start(),
                end: whole.end(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::scan_measurements;

    #[test]
    fn recognizes_finite_unicode_and_compound_units() {
        let spans = scan_measurements(
            "10μm 10µm 10Å 10Å 20kΩ 3mg·mL⁻¹ 2kg·m⁻³ 3mg/mL 2kg/m³ 4mol/L 25°C 10cm 20cL 1013hPa 5km 2kHz 4kPa 8kW",
        );
        assert_eq!(spans.len(), 18);
    }

    #[test]
    fn rejects_ordinary_words_and_variables() {
        let spans = scan_measurements("10chapter 2beta version2alpha DA-PEG-DA 10context 20class");
        assert!(spans.is_empty());
    }
}
