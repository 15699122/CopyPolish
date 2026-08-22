// rust_engine.rs
// =============================================================================
// Rust 原生文字处理引擎（迁移中的第一版）。
//
// 架构参考 typeset-rs 的“字符分类 / token 化 / 渲染管线”思路，但不复制
// 其源码。当前优先复刻本项目 ccw_engine.py 中已稳定测试的核心中文文案
// 排版规则；复杂 Markdown/LaTeX 保护仍在后续阶段迁移。
// =============================================================================

use regex::{Captures, Regex};
use std::collections::BTreeSet;
use std::sync::OnceLock;

pub struct FormatRequest {
    pub text: String,
    pub enabled: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharKind {
    Cjk,
    Latin,
    Digit,
    FullwidthPunctuation,
    Whitespace,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    ch: char,
    kind: CharKind,
}

fn classify(ch: char) -> CharKind {
    match ch {
        '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}' => {
            CharKind::Cjk
        }
        'A'..='Z' | 'a'..='z' => CharKind::Latin,
        '0'..='9' | '０'..='９' => CharKind::Digit,
        '，' | '。' | '；' | '：' | '！' | '？' | '、' | '（' | '）' | '《' | '》' | '【'
        | '】' | '「' | '」' | '『' | '』' | '…' | '—' => CharKind::FullwidthPunctuation,
        c if c.is_whitespace() => CharKind::Whitespace,
        _ => CharKind::Other,
    }
}

fn tokenize(text: &str) -> Vec<Token> {
    text.chars()
        .map(|ch| Token {
            ch,
            kind: classify(ch),
        })
        .collect()
}

pub fn enabled_defaults() -> Vec<String> {
    vec![
        "中英文之间需要增加空格",
        "中文与数字之间需要增加空格",
        "数字与单位之间需要增加空格",
        "全角标点与其他字符之间不加空格",
        "用 `text-spacing` 来挽救？",
        "不重复使用标点符号",
        "使用全角中文标点",
        "数字使用半角字符",
        "遇到完整的英文整句、特殊名词，其内容使用半角标点",
        "专有名词使用正确的大小写",
        "不要使用不地道的缩写",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub fn format_text(req: &FormatRequest) -> Result<String, String> {
    if req.text.is_empty() {
        return Ok(req.text.clone());
    }

    // 第一版 Rust engine 暂不处理复杂保护场景，检测到高风险标记时回退 Python。
    if should_fallback_to_python(&req.text) {
        return Err("contains protected markdown/latex/url pattern not migrated yet".to_string());
    }

    let enabled: BTreeSet<&str> = req.enabled.iter().map(String::as_str).collect();
    let enabled_all = req.enabled.is_empty();
    let (normalized, newline) = normalize_newlines(&req.text);
    let mut out = Vec::new();

    for line in normalized.split('\n') {
        if line.trim().is_empty() {
            out.push(String::new());
            continue;
        }

        let mut current = line.to_string();
        if enabled_all || enabled.contains("数字使用半角字符") {
            current = fullwidth_digits(&current);
        }
        if enabled_all || enabled.contains("使用全角中文标点") {
            current = fullwidth_chinese_punct(&current);
        }
        if enabled_all || enabled.contains("遇到完整的英文整句、特殊名词，其内容使用半角标点")
        {
            current = halfwidth_in_english(&current);
        }
        if enabled_all || enabled.contains("专有名词使用正确的大小写") {
            current = proper_nouns(&current);
        }
        if enabled_all || enabled.contains("不要使用不地道的缩写") {
            current = no_abbr(&current);
        }
        if enabled_all || enabled.contains("不重复使用标点符号") {
            current = no_repeat_punct(&current);
        }
        if enabled_all || enabled.contains("简体中文使用直角引号") {
            current = corner_quotes(&current);
        }

        // 与 Python 保持一致：基础空格收尾规则始终执行。
        current = cn_en_space(&current);
        current = cn_digit_space(&current);
        current = digit_unit_space(&current);
        current = fw_punct_no_space(&current);
        current = normalize_spaces(&current);
        out.push(current);
    }

    Ok(restore_newlines(&out.join("\n"), newline))
}

fn should_fallback_to_python(text: &str) -> bool {
    text.contains("```")
        || text.contains('`')
        || text.contains("http://")
        || text.contains("https://")
        || text.contains("\\(")
        || text.contains("\\[")
        || text.contains("\\begin{")
        || text.contains("$$")
        || text.contains("$E=")
        || text.contains("![")
        || text.contains("](")
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

fn cn_en_space(text: &str) -> String {
    insert_space_between(&tokenize(text), |a, b| {
        matches!(
            (a.kind, b.kind),
            (CharKind::Cjk, CharKind::Latin) | (CharKind::Latin, CharKind::Cjk)
        )
    })
}

fn cn_digit_space(text: &str) -> String {
    insert_space_between(&tokenize(text), |a, b| {
        matches!(
            (a.kind, b.kind),
            (CharKind::Cjk, CharKind::Digit) | (CharKind::Digit, CharKind::Cjk)
        )
    })
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

fn digit_unit_space(text: &str) -> String {
    static DEG_PERCENT: OnceLock<Regex> = OnceLock::new();
    static UNIT: OnceLock<Regex> = OnceLock::new();
    let text = DEG_PERCENT
        .get_or_init(|| Regex::new(r"(\d)\s+([°%％])").unwrap())
        .replace_all(text, "$1$2");
    UNIT.get_or_init(|| Regex::new(r"(\d)([A-Za-z]{1,4})([^A-Za-z0-9]|$)").unwrap())
        .replace_all(&text, "$1 $2$3")
        .to_string()
}

fn fw_punct_no_space(text: &str) -> String {
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

fn no_repeat_punct(text: &str) -> String {
    static SAME: OnceLock<Regex> = OnceLock::new();
    static MIXED: OnceLock<Regex> = OnceLock::new();
    let text = SAME
        .get_or_init(|| Regex::new(r"([！？!?~～])+").unwrap())
        .replace_all(text, |caps: &Captures| {
            caps[0].chars().next().unwrap().to_string()
        });
    MIXED
        .get_or_init(|| Regex::new(r"[？?！!]{2,}").unwrap())
        .replace_all(&text, "？！")
        .to_string()
}

fn fullwidth_chinese_punct(text: &str) -> String {
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
    corner_quotes(&out)
}

fn fullwidth_digits(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '０'..='９' => char::from_u32(ch as u32 - 0xfee0).unwrap_or(ch),
            _ => ch,
        })
        .collect()
}

fn halfwidth_in_english(text: &str) -> String {
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

fn proper_nouns(text: &str) -> String {
    let mut out = text.to_string();
    for (wrong, right) in PROPER_NOUNS {
        out = replace_word_case_insensitive(&out, wrong, right);
    }
    out
}

fn no_abbr(text: &str) -> String {
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

fn corner_quotes(text: &str) -> String {
    static DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SMART_DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SINGLE: OnceLock<Regex> = OnceLock::new();
    static SMART_SINGLE: OnceLock<Regex> = OnceLock::new();
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let text = DOUBLE
        .get_or_init(|| Regex::new(&format!(r#""([^"\n]*?{}[^"\n]*?)""#, cjk)).unwrap())
        .replace_all(text, "「$1」");
    let text = SMART_DOUBLE
        .get_or_init(|| Regex::new(&format!(r"“([^”\n]*?{}[^”\n]*?)”", cjk)).unwrap())
        .replace_all(&text, "「$1」");
    let text = SINGLE
        .get_or_init(|| Regex::new(&format!(r"'([^'\n]*?{}[^'\n]*?)'", cjk)).unwrap())
        .replace_all(&text, "『$1』");
    SMART_SINGLE
        .get_or_init(|| Regex::new(&format!(r"‘([^’\n]*?{}[^’\n]*?)’", cjk)).unwrap())
        .replace_all(&text, "『$1』")
        .to_string()
}

fn normalize_spaces(text: &str) -> String {
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

fn contains_cjk(text: &str) -> bool {
    text.chars().any(|ch| classify(ch) == CharKind::Cjk)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn req(text: &str) -> FormatRequest {
        FormatRequest {
            text: text.to_string(),
            enabled: enabled_defaults(),
        }
    }

    #[test]
    fn exposes_eleven_defaults() {
        assert_eq!(enabled_defaults().len(), 11);
    }

    #[test]
    fn formats_basic_copywriting_sample() {
        assert_eq!(
            format_text(&req("在LeanCloud上，花了5000元")).unwrap(),
            "在 LeanCloud 上，花了 5000 元"
        );
    }

    #[test]
    fn formats_digit_units_and_percentages() {
        assert_eq!(
            format_text(&req("宽带有 10Gbps")).unwrap(),
            "宽带有 10 Gbps"
        );
        assert_eq!(
            format_text(&req("SSD 一共有 20TB")).unwrap(),
            "SSD 一共有 20 TB"
        );
        assert_eq!(
            format_text(&req("角度为 90 ° 的角")).unwrap(),
            "角度为 90° 的角"
        );
        assert_eq!(
            format_text(&req("有 15 % 的 CPU")).unwrap(),
            "有 15% 的 CPU"
        );
    }

    #[test]
    fn formats_punctuation_and_proper_nouns() {
        assert_eq!(
            format_text(&req("德国队竟然战胜了巴西队！！")).unwrap(),
            "德国队竟然战胜了巴西队！"
        );
        assert_eq!(
            format_text(&req("使用 github 登录")).unwrap(),
            "使用 GitHub 登录"
        );
        assert_eq!(
            format_text(&req("只卖 １０００ 元")).unwrap(),
            "只卖 1000 元"
        );
    }

    #[test]
    fn falls_back_for_protected_content() {
        assert!(format_text(&req("公式$E=mc^2$很重要")).is_err());
        assert!(format_text(&req("请看[链接](https://example.com)")).is_err());
    }

    #[test]
    fn preserves_newline_style() {
        assert_eq!(
            format_text(&req("在LeanCloud上\r\n\r\n花了5000元")).unwrap(),
            "在 LeanCloud 上\r\n\r\n花了 5000 元"
        );
    }
}
