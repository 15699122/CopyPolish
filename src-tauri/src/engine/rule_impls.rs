// engine/rule_impls.rs
// =============================================================================
// 规则实现：每条规则是一个 `fn(&str) -> String` 纯函数，由 registry 注册。
// 迁移自旧 rust_engine.rs，行为保持不变（含默认启用集下的输出）。
// =============================================================================

use regex::{Captures, Regex};
use std::sync::OnceLock;

use super::tokenizer::{contains_cjk, tokenize, CharKind, Token};

// ---------------------------------------------------------------------------
// 基础工具
// ---------------------------------------------------------------------------

pub fn normalize_spaces(text: &str) -> String {
    static MULTI: OnceLock<Regex> = OnceLock::new();
    MULTI
        .get_or_init(|| Regex::new(r" {2,}").unwrap())
        .replace_all(text, " ")
        .to_string()
}

fn is_ascii_alnum(ch: Option<char>) -> bool {
    ch.map(|c| c.is_ascii_alphanumeric()).unwrap_or(false)
}

fn is_ascii_alpha(ch: Option<char>) -> bool {
    ch.map(|c| c.is_ascii_alphabetic()).unwrap_or(false)
}

fn is_closing_quote(ch: Option<char>) -> bool {
    matches!(ch, Some('」' | '』' | '》' | '’' | '”' | '）'))
}

/// 折叠同一字符的连续重复。
fn collapse_repeated_runs(text: &str, set: &[char]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev: Option<char> = None;
    for ch in text.chars() {
        if prev == Some(ch) && set.contains(&ch) {
            continue;
        }
        out.push(ch);
        prev = Some(ch);
    }
    out
}

fn insert_space_between<F>(tokens: &[Token], should_insert: F) -> String
where
    F: Fn(&Token, &Token) -> bool,
{
    let mut out = String::new();
    for (idx, token) in tokens.iter().enumerate() {
        if idx > 0 && should_insert(&tokens[idx - 1], token) {
            out.push(' ');
        }
        out.push(token.ch);
    }
    out
}

fn replace_word_case_insensitive(text: &str, wrong: &str, right: &str) -> String {
    let pattern = Regex::new(&format!(
        r"(?i)(^|[^A-Za-z0-9]){}([^A-Za-z0-9]|$)",
        regex::escape(wrong)
    ))
    .unwrap();
    pattern
        .replace_all(text, |caps: &Captures| {
            format!("{}{}{}", &caps[1], right, &caps[2])
        })
        .to_string()
}

/// 仅转换直引号（全角中文标点规则内部的行为）。
fn straight_corner_quotes(text: &str) -> String {
    static DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SINGLE: OnceLock<Regex> = OnceLock::new();
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let text = DOUBLE
        .get_or_init(|| Regex::new(&format!(r#""([^"\n]*?{cjk}[^"\n]*?)""#)).unwrap())
        .replace_all(text, "「$1」");
    SINGLE
        .get_or_init(|| Regex::new(&format!(r"'([^'\n]*?{cjk}[^'\n]*?)'")).unwrap())
        .replace_all(&text, "『$1』")
        .to_string()
}

// ---------------------------------------------------------------------------
// 规则实现
// ---------------------------------------------------------------------------

/// spacing.cjk-latin：中英文之间增加空格。
pub fn cn_en_space(text: &str) -> String {
    insert_space_between(&tokenize(text), |a, b| {
        matches!(
            (a.kind, b.kind),
            (CharKind::Cjk, CharKind::Latin) | (CharKind::Latin, CharKind::Cjk)
        )
    })
}

/// spacing.cjk-number：中文与数字之间增加空格。
pub fn cn_digit_space(text: &str) -> String {
    insert_space_between(&tokenize(text), |a, b| {
        matches!(
            (a.kind, b.kind),
            (CharKind::Cjk, CharKind::Digit) | (CharKind::Digit, CharKind::Cjk)
        )
    })
}

/// spacing.number-unit：数字与单位之间增加空格。
pub fn digit_unit_space(text: &str) -> String {
    static DEG_PERCENT: OnceLock<Regex> = OnceLock::new();
    static UNIT: OnceLock<Regex> = OnceLock::new();
    let text = DEG_PERCENT
        .get_or_init(|| Regex::new(r"(\d)\s+([°%％])").unwrap())
        .replace_all(text, "$1$2");
    UNIT.get_or_init(|| Regex::new(r"(\d)([A-Za-z]{1,4})([^A-Za-z0-9]|$)").unwrap())
        .replace_all(&text, "$1 $2$3")
        .to_string()
}

/// spacing.no-space-around-fw-punct：全角标点与其他字符之间不加空格。
pub fn fw_punct_no_space(text: &str) -> String {
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    static AFTER: OnceLock<Regex> = OnceLock::new();
    let text = BEFORE
        .get_or_init(|| Regex::new(r"\s+([，。；：！？、（）《》【】「」『』…—])").unwrap())
        .replace_all(text, "$1");
    AFTER
        .get_or_init(|| Regex::new(r"([，。；：！？、（）《》【】「」『』…—])\s+").unwrap())
        .replace_all(&text, "$1")
        .to_string()
}

/// punctuation.no-repetition：不重复使用标点符号。
pub fn no_repeat_punct(text: &str) -> String {
    static MIXED: OnceLock<Regex> = OnceLock::new();
    // 三步：1) [！？!?~～] 同字符折叠；2) [。，；：、] 同字符折叠；3) 混合叹问号 -> ？！
    let collapsed = collapse_repeated_runs(
        &collapse_repeated_runs(text, &['！', '？', '!', '?', '~', '～']),
        &['。', '，', '；', '：', '、'],
    );
    MIXED
        .get_or_init(|| Regex::new(r"[！？!?][！？!?]+").unwrap())
        .replace_all(&collapsed, "？！")
        .to_string()
}

/// punctuation.fullwidth-cjk：使用全角中文标点（仅对含中文的行生效）。
pub fn fullwidth_chinese_punct(text: &str) -> String {
    if !contains_cjk(text) {
        return text.to_string();
    }
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (idx, ch) in chars.iter().enumerate() {
        let prev = idx.checked_sub(1).and_then(|i| chars.get(i)).copied();
        let next = chars.get(idx + 1).copied();
        let converted = match *ch {
            ',' if !is_ascii_alnum(prev) && !is_ascii_alnum(next) => '，',
            ';' if !is_ascii_alnum(prev) && !is_ascii_alnum(next) => '；',
            ':' if !is_ascii_alnum(prev) && !is_ascii_alnum(next) => '：',
            '!' if !is_ascii_alnum(next) => '！',
            '?' if !is_ascii_alnum(next) => '？',
            '(' if !is_ascii_alnum(prev) => '（',
            ')' if !is_ascii_alnum(next) => '）',
            '.' if !is_ascii_alnum(prev) && !is_ascii_alnum(next) => '。',
            _ => *ch,
        };
        out.push(converted);
    }
    straight_corner_quotes(&out)
}

/// text.halfwidth-digits：数字使用半角字符。
pub fn fullwidth_digits(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '０'..='９' => char::from_u32(ch as u32 - 0xfee0).unwrap_or(ch),
            _ => ch,
        })
        .collect()
}

/// text.ascii-punct-in-latin：英文整句/特殊名词中使用半角标点。
pub fn halfwidth_in_english(text: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (idx, ch) in chars.iter().enumerate() {
        let prev = idx.checked_sub(1).and_then(|i| chars.get(i)).copied();
        let next = chars.get(idx + 1).copied();
        let replacement = match *ch {
            '，' if is_ascii_alpha(prev) && is_ascii_alnum(next) => Some(", "),
            '：' if is_ascii_alpha(prev) && is_ascii_alnum(next) => Some(": "),
            '；' if is_ascii_alpha(prev) && is_ascii_alnum(next) => Some("; "),
            '＆' if is_ascii_alpha(prev) && is_ascii_alnum(next) => Some(" & "),
            '。' if is_ascii_alnum(prev) && (is_ascii_alnum(next) || is_closing_quote(next)) => {
                Some(".")
            }
            '！' if is_ascii_alnum(prev) && (is_ascii_alnum(next) || is_closing_quote(next)) => {
                Some("!")
            }
            '？' if is_ascii_alnum(prev) && (is_ascii_alnum(next) || is_closing_quote(next)) => {
                Some("?")
            }
            '（' if is_ascii_alnum(prev) && is_ascii_alnum(next) => Some("("),
            '）' if is_ascii_alnum(prev) && is_ascii_alnum(next) => Some(")"),
            _ => None,
        };
        if let Some(s) = replacement {
            out.push_str(s);
        } else {
            out.push(*ch);
        }
    }
    normalize_spaces(&out)
}

/// punctuation.corner-quotes：简体中文使用直角引号（争议规则）。
pub fn corner_quotes(text: &str) -> String {
    static SMART_DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SMART_SINGLE: OnceLock<Regex> = OnceLock::new();
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let text = straight_corner_quotes(text);
    let text = SMART_DOUBLE
        .get_or_init(|| Regex::new(&format!(r"“([^”\n]*?{cjk}[^”\n]*?)”")).unwrap())
        .replace_all(&text, "「$1」");
    SMART_SINGLE
        .get_or_init(|| Regex::new(&format!(r"‘([^’\n]*?{cjk}[^’\n]*?)’")).unwrap())
        .replace_all(&text, "『$1』")
        .to_string()
}

// ---------------------------------------------------------------------------
// 词典型规则数据
// ---------------------------------------------------------------------------

const PROPER_NOUNS: &[(&str, &str)] = &[
    ("github", "GitHub"),
    ("foursquare", "Foursquare"),
    ("microsoft", "Microsoft"),
    ("google", "Google"),
    ("facebook", "Facebook"),
    ("twitter", "Twitter"),
    ("youtube", "YouTube"),
    ("linkedin", "LinkedIn"),
    ("instagram", "Instagram"),
    ("wikipedia", "Wikipedia"),
    ("wechat", "WeChat"),
    ("javascript", "JavaScript"),
    ("typescript", "TypeScript"),
    ("html5", "HTML5"),
    ("css3", "CSS3"),
    ("json", "JSON"),
    ("http", "HTTP"),
    ("https", "HTTPS"),
    ("api", "API"),
    ("sql", "SQL"),
    ("php", "PHP"),
    ("ios", "iOS"),
    ("ipados", "iPadOS"),
    ("android", "Android"),
    ("iphone", "iPhone"),
    ("ipad", "iPad"),
    ("imac", "iMac"),
    ("mac", "Mac"),
    ("macos", "macOS"),
    ("windows", "Windows"),
    ("linux", "Linux"),
    ("bluetooth", "Bluetooth"),
    ("wifi", "Wi-Fi"),
    ("wi-fi", "Wi-Fi"),
    ("nextjs", "Next.js"),
    ("npm", "npm"),
    ("react", "React"),
    ("vue", "Vue"),
    ("mongodb", "MongoDB"),
    ("corporation", "Corporation"),
    ("inc", "Inc"),
];

const ABBR_MAP: &[(&str, &str)] = &[
    ("ts", "TypeScript"),
    ("h5", "HTML5"),
    ("rjs", "React"),
    ("nextjs", "Next.js"),
    ("fed", "前端开发者"),
];

/// naming.proper-nouns：专有名词使用正确的大小写。
pub fn proper_nouns(text: &str) -> String {
    let mut out = text.to_string();
    for (wrong, right) in PROPER_NOUNS {
        out = replace_word_case_insensitive(&out, wrong, right);
    }
    out
}

/// naming.expand-abbreviations：不要使用不地道的缩写。
pub fn no_abbr(text: &str) -> String {
    let mut out = text.to_string();
    for (wrong, right) in ABBR_MAP {
        out = replace_word_case_insensitive(&out, wrong, right);
        if contains_cjk(right) {
            let pattern = Regex::new(&format!(r"\s+{}", regex::escape(right))).unwrap();
            out = pattern.replace_all(&out, *right).to_string();
        }
    }
    out
}

/// spacing.around-links：链接之间增加空格（争议规则）。
/// 实现位于 protection::space_around_links（需要保护层配合），此处转发。
pub fn around_links(text: &str) -> String {
    super::protection::space_around_links(text)
}
