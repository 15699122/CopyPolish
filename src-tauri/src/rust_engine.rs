// rust_engine.rs
// =============================================================================
// Rust 原生文字处理引擎（迁移中的第一版）。
//
// 架构参考 typeset-rs 的“字符分类 / token 化 / 渲染管线”思路，但不复制
// 其源码。当前优先复刻本项目 ccw_engine.py 中已稳定测试的核心中文文案
// 排版规则；复杂 Markdown/LaTeX 保护仍在后续阶段迁移。
// =============================================================================

use fancy_regex::Regex as FancyRegex;
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

// ---------------------------------------------------------------------------
// 保护层：Markdown / LaTeX / URL / 邮箱 / 代码片段 -> 私有区占位符
// 与 ccw_engine.py 的 _PROTECT_PATTERNS 一一对应（顺序也一致）。
// ---------------------------------------------------------------------------
const PH_START: char = '\u{E000}';

fn placeholder(idx: usize) -> String {
    format!("{PH_START}CCWPROTECTED{idx}\u{E001}")
}

fn protect_patterns() -> &'static Vec<FancyRegex> {
    static PATTERNS: OnceLock<Vec<FancyRegex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            // fenced code block（``` 或 ~~~，支持缩进闭合）
            r"(?s)(^|\n)([ \t]*)(`{3,}|~{3,})[^\n]*\n.*?\n\2\3[ \t]*(?=\n|$)",
            // LaTeX environment
            r"(?s)\\begin\{(equation\*?|align\*?|gather\*?|multline\*?|matrix|pmatrix|bmatrix|cases)\}.*?\\end\{\1\}",
            // LaTeX display \[...\]
            r"(?s)\\\[.*?\\\]",
            // LaTeX inline \(...\)
            r"(?s)\\\(.*?\\\)",
            // LaTeX display $$...$$（排除转义 \$）
            r"(?s)(?<!\\)\$\$(?!\$).*?(?<!\\)\$\$",
            // LaTeX inline $...$（排除转义与空白起始）
            r"(?<!\\)\$(?!\s|\$)(?:\\.|[^$\n\\]){1,300}?(?<!\\)\$(?!\$)",
            // Markdown image
            r"!\[[^\]\n]*\]\([^\n)]*\)",
            // Markdown link
            r"\[[^\]\n]+\]\([^\n)]*\)",
            // autolink <https://...> / <mail@...>
            r"(?i)<(?:(?:https?://[^>\s]+)|(?:[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}))>",
            // inline code
            r"`[^`\n]*`",
            // LaTeX command（\frac{a}{b} 等）
            r"\\[A-Za-z]+\*?(?:\[[^\]\n]*\])?(?:\{[^{}\n]*(?:\{[^{}\n]*\}[^{}\n]*)*\})+",
            // URL（内含双引号，使用 r#""# 形式）
            r#"(?i)https?://[^\s，。；：！？、（）《》【】「」“”‘’…—<>'"]+"#,
            // Email
            r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}",
        ]
        .iter()
        .map(|p| FancyRegex::new(p).expect("invalid protect pattern"))
        .collect()
    })
}

/// 把受保护片段替换为占位符；placeholders 按创建顺序保存 (占位符, 原文)。
fn protect(text: &str, placeholders: &mut Vec<(String, String)>) -> Result<String, String> {
    protect_with(protect_patterns(), text, placeholders)
}

fn protect_with(
    patterns: &[FancyRegex],
    text: &str,
    placeholders: &mut Vec<(String, String)>,
) -> Result<String, String> {
    let mut current = text.to_string();
    for pat in patterns {
        let mut out = String::new();
        let mut last = 0;
        for m in pat.find_iter(&current) {
            let m = m.map_err(|e| format!("protect regex error: {e}"))?;
            let (s, e) = (m.start(), m.end());
            let ph = placeholder(placeholders.len());
            placeholders.push((ph.clone(), current[s..e].to_string()));
            out.push_str(&current[last..s]);
            out.push_str(&ph);
            last = e;
        }
        out.push_str(&current[last..]);
        current = out;
    }
    Ok(current)
}

/// “链接之间增加空格”使用的保护模式子集（与 Python _r_space_around_links 一致）。
fn link_patterns() -> &'static Vec<FancyRegex> {
    static PATTERNS: OnceLock<Vec<FancyRegex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            r"!\[[^\]\n]*\]\([^\n)]*\)",
            r"\[[^\]\n]+\]\([^\n)]*\)",
            r"(?i)<(?:(?:https?://[^>\s]+)|(?:[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}))>",
            r#"(?i)https?://[^\s，。；：！？、（）《》【】「」“”‘’…—<>'"]+"#,
            r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}",
        ]
        .iter()
        .map(|p| FancyRegex::new(p).expect("invalid link protect pattern"))
        .collect()
    })
}

/// 12. 链接之间增加空格（争议规则，默认关闭）。
fn space_around_links(text: &str) -> String {
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    static AFTER: OnceLock<Regex> = OnceLock::new();
    let mut phs: Vec<(String, String)> = Vec::new();
    let protected = match protect_with(link_patterns(), text, &mut phs) {
        Ok(p) => p,
        Err(_) => return text.to_string(),
    };
    let before = BEFORE
        .get_or_init(|| Regex::new(&format!(r"(\S)({PH_START}CCWPROTECTED\d+\u{{E001}})")).unwrap())
        .replace_all(&protected, "$1 $2");
    let after = AFTER
        .get_or_init(|| Regex::new(&format!(r"({PH_START}CCWPROTECTED\d+\u{{E001}})(\S)")).unwrap())
        .replace_all(&before, "$1 $2");
    restore(&after, &phs)
}

/// 保护缩进代码行（整行占位）；普通 Markdown 标记行继续参与排版。
fn protect_markdown_lines(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    let mut lines = Vec::new();
    for line in text.split('\n') {
        if line.starts_with("    ") || line.starts_with('\t') {
            let ph = placeholder(placeholders.len());
            placeholders.push((ph.clone(), line.to_string()));
            lines.push(ph);
        } else {
            lines.push(line.to_string());
        }
    }
    lines.join("\n")
}

fn is_placeholder_line(line: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(&format!(r"^{PH_START}CCWPROTECTED\d+\u{{E001}}$")).unwrap())
        .is_match(line.trim())
}

/// 为行内保护片段补边界空格；整行/跨行保护块保持原样。
fn space_around_inline_placeholders(text: &str, placeholders: &[(String, String)]) -> String {
    static BEFORE_CACHE: OnceLock<Regex> = OnceLock::new();
    static AFTER_CACHE: OnceLock<Regex> = OnceLock::new();
    let inline: Vec<String> = placeholders
        .iter()
        .filter(|(_, val)| !val.contains('\n'))
        .map(|(ph, _)| regex::escape(ph))
        .collect();
    if inline.is_empty() {
        return text.to_string();
    }
    let alt = inline.join("|");
    let before = BEFORE_CACHE
        .get_or_init(|| Regex::new(&format!(r"(\S)({alt})")).unwrap())
        .replace_all(text, "$1 $2");
    AFTER_CACHE
        .get_or_init(|| Regex::new(&format!(r"({alt})([^\s，。；：！？、）】》」』])")).unwrap())
        .replace_all(&before, "$1 $2")
        .to_string()
}

/// 为行内保护片段补边界空格；整行保护块保持原样。
/// 按创建顺序的逆序还原占位符，保证嵌套内容（如链接中的 URL）正确还原。
fn restore(text: &str, placeholders: &[(String, String)]) -> String {
    let mut current = text.to_string();
    for (ph, val) in placeholders.iter().rev() {
        current = current.replace(ph.as_str(), val);
    }
    current
}

// 规则 key 与 ccw_engine.py 的 _slug() 输出完全一致（前端传递的就是这些 key）。
pub const K_CN_EN_SPACE: &str = "中英文之间需要增加空格";
pub const K_CN_DIGIT_SPACE: &str = "中文与数字之间需要增加空格";
pub const K_DIGIT_UNIT_SPACE: &str = "数字与单位之间需要增加空格";
pub const K_FW_PUNCT_NO_SPACE: &str = "全角标点与其他字符之间不加空格";
pub const K_CSS_TEXT_SPACING: &str = "用_text_spacing_来挽救";
pub const K_NO_REPEAT_PUNCT: &str = "不重复使用标点符号";
pub const K_FW_CHINESE_PUNCT: &str = "使用全角中文标点";
pub const K_HALFWIDTH_DIGITS: &str = "数字使用半角字符";
pub const K_HALFWIDTH_IN_ENGLISH: &str = "遇到完整的英文整句_特殊名词_其内容使用半角标点";
pub const K_PROPER_NOUNS: &str = "专有名词使用正确的大小写";
pub const K_NO_ABBR: &str = "不要使用不地道的缩写";
pub const K_SPACE_AROUND_LINKS: &str = "链接之间增加空格";
pub const K_CORNER_QUOTES: &str = "简体中文使用直角引号";

pub fn enabled_defaults() -> Vec<String> {
    vec![
        K_CN_EN_SPACE,
        K_CN_DIGIT_SPACE,
        K_DIGIT_UNIT_SPACE,
        K_FW_PUNCT_NO_SPACE,
        K_CSS_TEXT_SPACING,
        K_NO_REPEAT_PUNCT,
        K_FW_CHINESE_PUNCT,
        K_HALFWIDTH_DIGITS,
        K_HALFWIDTH_IN_ENGLISH,
        K_PROPER_NOUNS,
        K_NO_ABBR,
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub fn format_text(req: &FormatRequest) -> Result<String, String> {
    if req.text.is_empty() {
        return Ok(req.text.clone());
    }

    let enabled: BTreeSet<&str> = req.enabled.iter().map(String::as_str).collect();
    let enabled_all = req.enabled.is_empty();
    let (normalized, newline) = normalize_newlines(&req.text);

    // 保护层：Markdown / LaTeX / URL / 邮箱 / 代码片段先替换为占位符。
    let mut placeholders: Vec<(String, String)> = Vec::new();
    let protected = protect(&normalized, &mut placeholders)?;
    let protected = protect_markdown_lines(&protected, &mut placeholders);

    let mut out = Vec::new();
    for line in protected.split('\n') {
        if is_placeholder_line(line) {
            out.push(line.to_string());
            continue;
        }
        if line.trim().is_empty() {
            // Python 版会把空白行规范化为空行（result.append("")）。
            out.push(String::new());
            continue;
        }

        let mut current = line.to_string();
        // 执行顺序与 ccw_engine.py 的 RULES 注册表一致（规则 6 -> 13）。
        if enabled_all || enabled.contains(K_NO_REPEAT_PUNCT) {
            current = no_repeat_punct(&current);
        }
        if enabled_all || enabled.contains(K_FW_CHINESE_PUNCT) {
            current = fullwidth_chinese_punct(&current);
        }
        if enabled_all || enabled.contains(K_HALFWIDTH_DIGITS) {
            current = fullwidth_digits(&current);
        }
        if enabled_all || enabled.contains(K_HALFWIDTH_IN_ENGLISH) {
            current = halfwidth_in_english(&current);
        }
        if enabled_all || enabled.contains(K_PROPER_NOUNS) {
            current = proper_nouns(&current);
        }
        if enabled_all || enabled.contains(K_NO_ABBR) {
            current = no_abbr(&current);
        }
        if enabled_all || enabled.contains(K_SPACE_AROUND_LINKS) {
            current = space_around_links(&current);
        }
        if enabled_all || enabled.contains(K_CORNER_QUOTES) {
            current = corner_quotes(&current);
        }

        // 与 Python 保持一致：基础空格收尾规则始终执行（对应
        // _format_regular_text 末尾固定追加的四个函数）。
        current = cn_en_space(&current);
        current = cn_digit_space(&current);
        current = digit_unit_space(&current);
        current = fw_punct_no_space(&current);
        out.push(current);
    }

    let formatted = out.join("\n");
    let formatted = space_around_inline_placeholders(&formatted, &placeholders);
    let restored = restore(&formatted, &placeholders);
    Ok(restore_newlines(&restored, newline))
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
    static MIXED: OnceLock<Regex> = OnceLock::new();
    // 与 ccw_engine.py _r_no_repeat_punct 三步一致：
    // 1) [！？!?~～] 同字符折叠；2) [。，；：、] 同字符折叠；3) 混合叹问号 -> ？！
    let collapsed = collapse_repeated_runs(
        &collapse_repeated_runs(text, &['！', '？', '!', '?', '~', '～']),
        &['。', '，', '；', '：', '、'],
    );
    MIXED
        .get_or_init(|| Regex::new(r"[！？!?][！？!?]+").unwrap())
        .replace_all(&collapsed, "？！")
        .to_string()
}

/// 折叠同一字符的连续重复（模拟 Python 的 r"([..])\1+" backreference）。
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

fn fullwidth_chinese_punct(text: &str) -> String {
    // Python 版仅对含中文的行生效。
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

/// 仅转换直引号（规则 7 全角中文标点内部的行为，与 Python _conv_line 一致）。
fn straight_corner_quotes(text: &str) -> String {
    static DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SINGLE: OnceLock<Regex> = OnceLock::new();
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let text = DOUBLE
        .get_or_init(|| Regex::new(&format!(r#""([^"\n]*?{}[^"\n]*?)""#, cjk)).unwrap())
        .replace_all(text, "「$1」");
    SINGLE
        .get_or_init(|| Regex::new(&format!(r"'([^'\n]*?{}[^'\n]*?)'", cjk)).unwrap())
        .replace_all(&text, "『$1』")
        .to_string()
}

/// 13. 简体中文使用直角引号（争议规则；直引号 + 弯引号都转换）。
fn corner_quotes(text: &str) -> String {
    static SMART_DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SMART_SINGLE: OnceLock<Regex> = OnceLock::new();
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let text = straight_corner_quotes(text);
    let text = SMART_DOUBLE
        .get_or_init(|| Regex::new(&format!(r"“([^”\n]*?{}[^”\n]*?)”", cjk)).unwrap())
        .replace_all(&text, "「$1」");
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
    fn formats_protected_content_like_python_engine() {
        // LaTeX / Markdown 保护（对齐 test/test_ccw_engine.py）
        assert_eq!(
            format_text(&req("公式$E=mc^2$很重要")).unwrap(),
            "公式 $E=mc^2$ 很重要"
        );
        assert_eq!(
            format_text(&req(r"公式\( E=mc^2 \)很重要")).unwrap(),
            r"公式 \( E=mc^2 \) 很重要"
        );
        assert_eq!(
            format_text(&req(r"使用\frac{a}{b}计算")).unwrap(),
            r"使用 \frac{a}{b} 计算"
        );
        assert_eq!(format_text(&req(r"价格是\$100")).unwrap(), r"价格是\$100");

        let display_math = "如下：\n$$\nE=mc^2; github\n$$\n结束";
        assert_eq!(format_text(&req(display_math)).unwrap(), display_math);

        let latex_env = "如下：\n\\begin{align}\na&=b+c; github\n\\end{align}\n结束";
        assert_eq!(format_text(&req(latex_env)).unwrap(), latex_env);

        let fenced = "示例：\n```python\nprint('github; $x | y')\n```\n结束";
        assert_eq!(format_text(&req(fenced)).unwrap(), fenced);

        let indented = "命令：\n    npm install foo/bar; echo '$x|y'\n完成";
        assert_eq!(format_text(&req(indented)).unwrap(), indented);

        assert_eq!(
            format_text(&req("使用`a;b|c/$x`安装")).unwrap(),
            "使用 `a;b|c/$x` 安装"
        );
        assert_eq!(
            format_text(&req(
                "请看[GitHub链接](https://example.com/a;b?x=$1|y)然后继续"
            ))
            .unwrap(),
            "请看 [GitHub链接](https://example.com/a;b?x=$1|y) 然后继续"
        );
        assert_eq!(
            format_text(&req(r#"图片![alt text](image/path.png "title")很好"#)).unwrap(),
            r#"图片 ![alt text](image/path.png "title") 很好"#
        );
    }

    #[test]
    fn protected_cases_are_idempotent() {
        let cases = [
            "第一段\n\n第二段",
            "公式$E=mc^2$很重要",
            "示例：\n```\ngithub; $x | y\n```\n结束",
            r"路径是 <windows-user-home>，价格是\$100",
        ];
        for src in cases {
            let once = format_text(&req(src)).unwrap();
            assert_eq!(format_text(&req(&once)).unwrap(), once, "{src}");
        }
    }

    #[test]
    fn preserves_newline_style() {
        assert_eq!(
            format_text(&req("在LeanCloud上\r\n\r\n花了5000元")).unwrap(),
            "在 LeanCloud 上\r\n\r\n花了 5000 元"
        );
    }
}
