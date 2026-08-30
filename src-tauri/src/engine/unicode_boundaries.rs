// engine/unicode_boundaries.rs
// =============================================================================
// Unicode 文本边界层（roadmap §5）。
//
// 职责：把文本切分为“判定单位”（extended grapheme cluster 或 legacy 单个
// char），并给出保守的类别（Han / Latin / Digit / Other）。插空类规则
// 只依赖这里的单位与类别，不自行做边界判断。
//
// 设计取舍：
// - 边界来自 unicode-segmentation 的 UAX #29 extended grapheme clusters，
//   保证 emoji ZWJ 序列、肤色修饰符、组合附加符不会被切断；
// - 分类是产品所需的保守子集（手写区间表，集中维护），不是完整的
//   Unicode Script 属性——后者仍属于 roadmap §6 ICU4X Spike；
// - Han 范围在既有 Ext-A / 基本区 / 兼容区之上扩展 CJK Extension B；
// - Kana、Hangul、emoji 等一律归入 Other，首期不改变其行为；
// - LegacyChars 策略仅为新旧实现对比保留，生产路径固定使用 Graphemes。
// =============================================================================

use unicode_segmentation::UnicodeSegmentation;

/// 文本边界策略：生产使用 Graphemes；LegacyChars 仅供对比与回归测试。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BoundaryStrategy {
    /// 仅测试/对比使用，生产不构造（见 spacing_rules_grapheme_strategy_matches_legacy）。
    #[allow(dead_code)]
    LegacyChars,
    Graphemes,
}

/// 插空规则所需的保守类别。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScriptClass {
    /// 汉字（含 CJK 扩展区，见 HAN_RANGES）。
    Han,
    /// 拉丁字母（ASCII + Latin-1 Supplement / Latin Extended，
    /// 保证 NFC 组合形式（如 é）与 NFD 分解形式分类一致）。
    Latin,
    /// 半角/全角数字。
    Digit,
    /// 其余（标点、emoji、Kana、Hangul、占位符等）。
    Other,
}

/// 一个文本判定单位：完整 &str 切片 + 字节起点 + 类别。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextUnit<'a> {
    pub text: &'a str,
    pub byte_start: usize,
    pub script: ScriptClass,
}

/// CJK 汉字范围表（集中维护；新增扩展区时只改这里）：
/// Ext-A、基本区、兼容表意文字、CJK Extension B。
const HAN_RANGES: &[(u32, u32)] = &[
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xF900, 0xFAFF),
    (0x20000, 0x2A6DF),
];

fn is_han_char(c: char) -> bool {
    let u = c as u32;
    HAN_RANGES.iter().any(|&(lo, hi)| (lo..=hi).contains(&u))
}

/// 组合附加符 / 变体选择符 / ZWJ：跟随基字符分类，不单独作为分类依据。
fn is_combining(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x200D | 0xFE00..=0xFE0F)
}

fn classify_scalar(c: char) -> ScriptClass {
    if is_han_char(c) {
        ScriptClass::Han
    } else if c.is_ascii_alphabetic() || matches!(c, '\u{00C0}'..='\u{024F}') {
        ScriptClass::Latin
    } else if matches!(c, '0'..='9' | '０'..='９') {
        ScriptClass::Digit
    } else {
        ScriptClass::Other
    }
}

/// grapheme cluster 的类别由首个非组合字符决定：
/// `e` + U+0301 与组合形式的 é 仍是 Latin，汉字 + 组合标记仍是 Han，
/// emoji ZWJ / 肤色修饰序列整体为 Other。
pub(crate) fn script_of_grapheme(grapheme: &str) -> ScriptClass {
    let base = grapheme
        .chars()
        .find(|&c| !is_combining(c))
        .unwrap_or_else(|| grapheme.chars().next().expect("grapheme is non-empty"));
    classify_scalar(base)
}

/// 按策略把文本切成判定单位序列。
pub(crate) fn units(text: &str, strategy: BoundaryStrategy) -> Vec<TextUnit<'_>> {
    match strategy {
        BoundaryStrategy::Graphemes => text
            .grapheme_indices(true)
            .map(|(start, g)| TextUnit {
                text: g,
                byte_start: start,
                script: script_of_grapheme(g),
            })
            .collect(),
        BoundaryStrategy::LegacyChars => text
            .char_indices()
            .map(|(start, ch)| TextUnit {
                byte_start: start,
                text: &text[start..start + ch.len_utf8()],
                script: classify_scalar(ch),
            })
            .collect(),
    }
}

/// 流式遍历相邻判定单位，供生产规则避免构造完整 `Vec<TextUnit>`。
///
/// 单位切分和 `units` 完全一致；回调只在当前调用期间使用当前/上一单位借用，
/// 不需要把单位保存到回调外部。
pub(crate) fn for_each_adjacent_unit<F>(text: &str, strategy: BoundaryStrategy, mut visit: F)
where
    F: FnMut(Option<(ScriptClass, &str)>, ScriptClass, &str),
{
    let mut previous: Option<(ScriptClass, &str)> = None;
    match strategy {
        BoundaryStrategy::Graphemes => {
            for (_, grapheme) in text.grapheme_indices(true) {
                let script = script_of_grapheme(grapheme);
                visit(previous, script, grapheme);
                previous = Some((script, grapheme));
            }
        }
        BoundaryStrategy::LegacyChars => {
            for (start, ch) in text.char_indices() {
                let script = classify_scalar(ch);
                let unit_text = &text[start..start + ch.len_utf8()];
                visit(previous, script, unit_text);
                previous = Some((script, unit_text));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scripts(text: &str) -> Vec<ScriptClass> {
        units(text, BoundaryStrategy::Graphemes)
            .iter()
            .map(|u| u.script)
            .collect()
    }

    fn texts(text: &str) -> Vec<&str> {
        units(text, BoundaryStrategy::Graphemes)
            .iter()
            .map(|u| u.text)
            .collect()
    }

    #[test]
    fn zwj_family_emoji_is_single_unit() {
        let units = texts("中文👨‍👩‍👧‍👦GitHub");
        assert_eq!(units, vec!["中", "文", "👨‍👩‍👧‍👦", "G", "i", "t", "H", "u", "b"]);
        assert_eq!(scripts("👨‍👩‍👧‍👦"), vec![ScriptClass::Other]);
    }

    #[test]
    fn skin_tone_modifier_stays_with_emoji() {
        assert_eq!(texts("👍🏽").len(), 1);
        assert_eq!(scripts("👍🏽"), vec![ScriptClass::Other]);
    }

    #[test]
    fn combining_mark_follows_base_char() {
        // 分解形式的 é：一个 Latin cluster。
        assert_eq!(texts("Café"), vec!["C", "a", "f", "é"]);
        assert_eq!(scripts("é"), vec![ScriptClass::Latin]);
        // 汉字 + 组合标记仍是 Han。
        assert_eq!(scripts("好\u{0301}"), vec![ScriptClass::Han]);
    }

    #[test]
    fn cjk_extension_b_is_han() {
        for ch in ['\u{20000}', '\u{2A6DF}', '𠀀', '𠮷'] {
            assert_eq!(
                script_of_grapheme(&ch.to_string()),
                ScriptClass::Han,
                "{ch}"
            );
        }
        assert_eq!(scripts("𠀀LeanCloud")[0], ScriptClass::Han);
    }

    #[test]
    fn kana_hangul_and_punct_are_other() {
        assert_eq!(scripts("カタカナ"), vec![ScriptClass::Other; 4]);
        assert_eq!(scripts("한글"), vec![ScriptClass::Other; 2]);
        assert_eq!(scripts("，。"), vec![ScriptClass::Other; 2]);
    }

    #[test]
    fn units_reconstruct_input_byte_for_byte() {
        let sample = "中a1👍🏽👨‍👩‍👧‍👦e\u{0301}𠀀， ";
        for strategy in [BoundaryStrategy::Graphemes, BoundaryStrategy::LegacyChars] {
            let rebuilt: String = units(sample, strategy).iter().map(|u| u.text).collect();
            assert_eq!(rebuilt, sample);
        }
    }

    #[test]
    fn streaming_adjacent_units_match_materialized_units() {
        let sample = "中a1👍🏽👨‍👩‍👧‍👦e\u{0301}𠀀， ";
        for strategy in [BoundaryStrategy::Graphemes, BoundaryStrategy::LegacyChars] {
            let expected: Vec<(String, ScriptClass)> = units(sample, strategy)
                .iter()
                .map(|unit| (unit.text.to_string(), unit.script))
                .collect();
            let mut actual = Vec::new();
            for_each_adjacent_unit(sample, strategy, |_, script, text| {
                actual.push((text.to_string(), script));
            });
            assert_eq!(actual, expected);
        }
    }
}
