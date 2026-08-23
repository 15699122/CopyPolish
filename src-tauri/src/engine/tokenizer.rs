use regex::Regex;
use std::sync::OnceLock;

// engine/tokenizer.rs
// =============================================================================
// 字符分类与语义片段识别。
//
// 在旧的“逐字符类别”之上，新增对特殊字符（Unicode 上下标、化学式连接符）
// 与化学式（Fe²⁺、SO₄²⁻、FeCl₂·4H₂O 等）的整体识别。识别结果以字节区间
// 返回，由保护层转成占位符，保证任何规则都不会从公式内部插入空格或改写
// 符号；同时为后续新规则提供可靠的判定单元。
//
// 识别策略是保守的：只有包含上下标、电荷标记或水合物连接符的片段才被
// 判定为化学式；普通英文单词和不含这些特征的简单式子（如 H2O）不参与，
// 避免把普通文本误吞为受保护内容。
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharKind {
    Cjk,
    Latin,
    Digit,
    /// Unicode 上标字符（⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻ 等）。
    Superscript,
    /// Unicode 下标字符（₀₁₂₃₄₅₆₇₈₉ 等）。
    Subscript,
    /// 化学式连接符：· / ⋅。
    Middot,
    FullwidthPunctuation,
    Whitespace,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub ch: char,
    pub kind: CharKind,
}

const SUPERSCRIPT_CHARS: &str = "⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾";
const SUBSCRIPT_CHARS: &str = "₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎";

pub fn is_superscript(ch: char) -> bool {
    SUPERSCRIPT_CHARS.contains(ch)
}

pub fn is_subscript(ch: char) -> bool {
    SUBSCRIPT_CHARS.contains(ch)
}

pub fn is_middot(ch: char) -> bool {
    ch == '·' || ch == '⋅'
}

pub fn classify(ch: char) -> CharKind {
    match ch {
        '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}' => {
            CharKind::Cjk
        }
        'A'..='Z' | 'a'..='z' => CharKind::Latin,
        '0'..='9' | '０'..='９' => CharKind::Digit,
        c if is_superscript(c) => CharKind::Superscript,
        c if is_subscript(c) => CharKind::Subscript,
        c if is_middot(c) => CharKind::Middot,
        '，' | '。' | '；' | '：' | '！' | '？' | '、' | '（' | '）' | '《' | '》' | '【'
        | '】' | '「' | '」' | '『' | '』' | '…' | '—' => CharKind::FullwidthPunctuation,
        c if c.is_whitespace() => CharKind::Whitespace,
        _ => CharKind::Other,
    }
}

pub fn tokenize(text: &str) -> Vec<Token> {
    text.chars()
        .map(|ch| Token {
            ch,
            kind: classify(ch),
        })
        .collect()
}

pub fn contains_cjk(text: &str) -> bool {
    text.chars().any(|ch| classify(ch) == CharKind::Cjk)
}

// ---------------------------------------------------------------------------
// 化学式识别（保守语法）：
//   formula_part := 大写字母 [小写字母] { 数字 | 下标数字 }*
//   charge       := 上标序列 | (+|-)
//   hydrate      := (·|⋅) 数字* formula_part...
//   整体必须含至少一个“特殊”特征（下标/上标/电荷/连接符）才被接受。
// ---------------------------------------------------------------------------

/// 在文本中查找化学式片段，返回字节区间 `(start, end)`（按出现顺序、互不重叠）。
pub fn detect_chemical_formulas(text: &str) -> Vec<(usize, usize)> {
    // 快速预筛：文本中完全没有特征字符时直接返回，避免逐位尝试解析。
    let has_feature = text
        .chars()
        .any(|ch| is_subscript(ch) || is_superscript(ch) || is_middot(ch));
    if !has_feature {
        return Vec::new();
    }

    static FORMULA: OnceLock<Regex> = OnceLock::new();
    let formula = FORMULA.get_or_init(|| {
        Regex::new(
            r"(?:[A-Z][a-z]?(?:[0-9]+|[₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎]+)?)+(?:[⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾]+|[0-9]*[+-])?(?:[·⋅][0-9]*(?:[A-Z][a-z]?(?:[0-9]+|[₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎]+)?)+(?:[⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾]+|[0-9]*[+-])?)*",
        )
        .expect("invalid chemical formula pattern")
    });

    let mut spans = Vec::new();
    for matched in formula.find_iter(text) {
        let start = matched.start();
        let end = matched.end();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        if before.is_some_and(|c| c.is_ascii_alphanumeric())
            || after.is_some_and(|c| c.is_ascii_alphanumeric())
        {
            continue;
        }
        if spans
            .last()
            .is_some_and(|(_, previous_end)| start < *previous_end)
        {
            continue;
        }
        spans.push((start, end));
    }
    spans
}
