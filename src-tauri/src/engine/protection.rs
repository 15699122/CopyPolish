// engine/protection.rs
// =============================================================================
// 保护层：Markdown / LaTeX / URL / 邮箱 / 化学式 -> 私有区占位符。
// 占位符格式沿用历史约定：\u{E000}CCWPROTECTED{n}\u{E001}，
// 规则正则不会匹配私有区字符，因此受保护内容在处理期间不可被改写。
// =============================================================================

use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::sync::OnceLock;

pub const PH_START: char = '\u{E000}';

pub fn placeholder(idx: usize) -> String {
    format!("{PH_START}CCWPROTECTED{idx}\u{E001}")
}

fn placeholder_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(&format!(r"^{PH_START}CCWPROTECTED\d+\u{{E001}}$")).unwrap())
}

/// 全局保护模式（与历史实现一致，顺序也一致）。
fn protect_patterns() -> &'static Vec<FancyRegex> {
    static PATTERNS: OnceLock<Vec<FancyRegex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            // fenced code block（``` 或 ~~~，支持缩进闭合）
            r"(?s)(^|\n)([ \t]*)(`{3,}|~{3,})[^\n]*\n.*?\n\2\3[ \t]*(?=\n|$)",
            // HTML 注释（支持跨行，注释内部完全保持原样）
            r"(?s)<!--.*?-->",
            // LaTeX environment
            r"(?s)\\begin\{(equation\*?|align\*?|gather\*?|multline\*?|matrix|pmatrix|bmatrix|cases)\}.*?\\end\{\1\}",
            // LaTeX display \[...\]
            r"(?s)\\\[.*?\\\]",
            // LaTeX inline \(...\)
            r"(?s)\\\(.*?\\\)",
            // LaTeX display $$...$$（排除转义 \$）
            r"(?s)(?<!\\)\$\$(?!\$).*?(?<!\\)\$\$",
            // LaTeX inline $...$（排除转义与空白起始）
            r#"(?<!\\)\$(?!\s|\$)(?:\\.|[^$\n\\]){1,300}?(?<!\\)\$(?!\$)"#,
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
            // URL
            r#"(?i)https?://[^\s，。；：！？、（）《》【】「」“”‘’…—<>'"]+"#,
            // Email
            r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}",
        ]
        .iter()
        .map(|p| FancyRegex::new(p).expect("invalid protect pattern"))
        .collect()
    })
}

/// “链接之间增加空格”使用的保护模式子集。
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

/// 把受保护片段替换为占位符；placeholders 按创建顺序保存 (占位符, 原文)。
pub fn protect(text: &str, placeholders: &mut Vec<(String, String)>) -> Result<String, String> {
    protect_with(protect_patterns(), text, placeholders)
}

/// 与 `protect` 相同，但使用调用方提供的模式集合。
pub fn protect_with(
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

/// 把给定的字节区间（如化学式识别结果）替换为占位符。
/// 区间必须来自同一份原文、互不重叠且按出现顺序排列。
pub fn protect_byte_spans(
    text: &str,
    spans: &[(usize, usize)],
    placeholders: &mut Vec<(String, String)>,
) -> String {
    protect_byte_spans_with_offset(text, spans, placeholders, 0)
}

/// 与 `protect_byte_spans` 相同，但允许调用方为占位符编号提供偏移量，
/// 以便多个独立占位符集合可以安全地在同一文本中共存。
pub fn protect_byte_spans_with_offset(
    text: &str,
    spans: &[(usize, usize)],
    placeholders: &mut Vec<(String, String)>,
    offset: usize,
) -> String {
    if spans.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut last = 0usize;
    for &(s, e) in spans {
        out.push_str(&text[last..s]);
        let ph = placeholder(offset + placeholders.len());
        placeholders.push((ph.clone(), text[s..e].to_string()));
        out.push_str(&ph);
        last = e;
    }
    out.push_str(&text[last..]);
    out
}

/// “链接之间增加空格”：先按链接子集保护，再为占位符两侧补空格并还原。
pub fn space_around_links(text: &str) -> String {
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
pub fn protect_markdown_lines(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
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

/// 判断整行是否只是占位符（fenced block / 缩进代码行），此类行不做规则处理。
pub fn is_placeholder_line(line: &str) -> bool {
    placeholder_re().is_match(line)
}

/// 为行内保护片段补边界空格；整行/跨行保护块保持原样。
pub fn space_around_inline_placeholders(text: &str, placeholders: &[(String, String)]) -> String {
    let inline: Vec<String> = placeholders
        .iter()
        .filter(|(_, val)| !val.contains('\n'))
        .map(|(ph, _)| regex::escape(ph))
        .collect();
    if inline.is_empty() {
        return text.to_string();
    }
    let alt = inline.join("|");
    let before_re = Regex::new(&format!(r"(\S)({alt})")).unwrap();
    let after_re = Regex::new(&format!(r#"({alt})([^\s，。；：！？、）】》」』])"#)).unwrap();
    let before = before_re.replace_all(text, "$1 $2");
    after_re.replace_all(&before, "$1 $2").to_string()
}

/// 为数学表达式占位符补充仅限 Han 边界的空格。
///
/// 数学 token 不复用普通 Markdown 占位符的 `\S` 边界规则，避免在全角标点后
/// 产生额外空格；表达式内部和标点邻接关系均保持原样。
pub fn space_around_math_placeholders(text: &str, placeholders: &[(String, String)]) -> String {
    let inline: Vec<String> = placeholders
        .iter()
        .filter(|(_, val)| !val.contains('\n'))
        .map(|(ph, _)| regex::escape(ph))
        .collect();
    if inline.is_empty() {
        return text.to_string();
    }
    let alt = inline.join("|");
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let before_re = Regex::new(&format!(r"({cjk})({alt})")).unwrap();
    let after_re = Regex::new(&format!(r"({alt})({cjk})")).unwrap();
    let before = before_re.replace_all(text, "$1 $2");
    after_re.replace_all(&before, "$1 $2").to_string()
}

/// 按创建顺序的逆序还原占位符，保证嵌套内容（如链接中的 URL）正确还原。
pub fn restore(text: &str, placeholders: &[(String, String)]) -> String {
    let mut current = text.to_string();
    for (ph, val) in placeholders.iter().rev() {
        current = current.replace(ph.as_str(), val);
    }
    current
}
