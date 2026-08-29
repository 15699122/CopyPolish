// engine/protection.rs
// =============================================================================
// 保护层：Markdown / LaTeX / URL / 邮箱 / 化学式 -> 私有区占位符。
// 占位符格式沿用历史约定：\u{E000}CCWPROTECTED{n}\u{E001}，
// 规则正则不会匹配私有区字符，因此受保护内容在处理期间不可被改写。
// =============================================================================

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

pub(crate) fn find_inline_html_tag_end(bytes: &[u8], open: usize) -> Option<usize> {
    let mut quote = None;
    for (index, &byte) in bytes.iter().enumerate().skip(open + 1) {
        match (quote, byte) {
            (Some(current), byte) if byte == current => quote = None,
            (None, b'"' | b'\'') => quote = Some(bytes[index]),
            (None, b'\n') => return None,
            (None, b'>') => return Some(index),
            _ => {}
        }
    }
    None
}

pub(crate) fn inline_html_tag_name(bytes: &[u8], open: usize) -> Option<&str> {
    let mut index = open + 1;
    if bytes.get(index) == Some(&b'/') {
        return None;
    }
    let start = index;
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_alphabetic())
    {
        index += 1;
    }
    (index > start).then(|| std::str::from_utf8(&bytes[start..index]).ok())?
}

pub(crate) fn is_self_closing_html_tag(bytes: &[u8], open: usize, end: usize) -> bool {
    bytes[open + 1..end]
        .iter()
        .rev()
        .find(|byte| !byte.is_ascii_whitespace())
        .is_some_and(|byte| *byte == b'/')
}

pub(crate) fn find_inline_html_closing_tag(
    bytes: &[u8],
    start: usize,
    name: &str,
) -> Option<usize> {
    let marker = format!("</{name}");
    let mut cursor = start;
    while cursor < bytes.len() {
        let close_start = find_ascii_case_insensitive(bytes, marker.as_bytes(), cursor)?;
        let boundary = bytes.get(close_start + marker.len()).copied();
        if matches!(boundary, Some(b'>') | Some(b' ') | Some(b'\t')) {
            return find_inline_html_tag_end(bytes, close_start);
        }
        cursor = close_start + marker.len();
    }
    None
}

fn find_ascii_case_insensitive(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start >= haystack.len() || needle.len() > haystack.len() - start {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| {
            window
                .iter()
                .zip(needle)
                .all(|(left, right)| left.eq_ignore_ascii_case(right))
        })
        .map(|offset| start + offset)
}

pub(crate) fn is_inline_html_tag(bytes: &[u8], open: usize, end: usize) -> bool {
    if bytes.get(open) != Some(&b'<') || bytes.get(end) != Some(&b'>') {
        return false;
    }
    let mut index = open + 1;
    if bytes.get(index) == Some(&b'/') {
        index += 1;
    }
    let name_start = index;
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_alphabetic())
    {
        index += 1;
    }
    if index == name_start {
        return false;
    }
    matches!(
        bytes.get(index),
        Some(b'>') | Some(b'/') | Some(b' ' | b'\t')
    )
}

/// 保护 Markdown 链接和图片链接，支持目标中的嵌套圆括号。
///
/// 扫描器只处理同一行且最终闭合的 `[label](target)` 结构；未闭合结构
/// 保留为普通文本，避免把后续正文整体吞入占位符。
fn protect_markdown_links(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;

    while cursor < bytes.len() {
        let Some(relative_open) = text[cursor..].find('[') else {
            out.push_str(&text[cursor..]);
            break;
        };
        let open = cursor + relative_open;
        let image_start = open.checked_sub(1).filter(|&index| bytes[index] == b'!');
        let structure_start = image_start.unwrap_or(open);

        let Some(label_end) = find_link_label_end(bytes, open) else {
            out.push_str(&text[cursor..]);
            break;
        };
        if bytes.get(label_end + 1) != Some(&b'(') {
            out.push_str(&text[cursor..label_end + 1]);
            cursor = label_end + 1;
            continue;
        }

        let target_open = label_end + 1;
        let Some(target_end) = find_link_target_end(bytes, target_open) else {
            out.push_str(&text[cursor..target_open + 1]);
            cursor = target_open + 1;
            continue;
        };

        out.push_str(&text[cursor..structure_start]);
        let ph = placeholder(placeholders.len());
        placeholders.push((
            ph.clone(),
            text[structure_start..target_end + 1].to_string(),
        ));
        out.push_str(&ph);
        cursor = target_end + 1;
    }

    out
}

pub(crate) fn find_link_label_end(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() && bytes[index] != b'\n' {
        if bytes[index] == b'\\' {
            index += 2;
            continue;
        }
        match bytes[index] {
            b'[' => depth += 1,
            b']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn find_link_target_end(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() && bytes[index] != b'\n' {
        if bytes[index] == b'\\' {
            index += 2;
            continue;
        }
        match bytes[index] {
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
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
    let mut protected = protect_markdown_links(text, &mut phs);
    for pattern in [
        r"(?i)<(?:(?:https?://[^>\s]+)|(?:[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}))>",
        r#"(?i)https?://[^\s，。；：！？、（）《》【】「」“”‘’…—<>'"]+"#,
        r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}",
    ] {
        let pattern = Regex::new(pattern).expect("invalid link protect pattern");
        let mut output = String::with_capacity(protected.len());
        let mut last = 0;
        for matched in pattern.find_iter(&protected) {
            let (start, end) = (matched.start(), matched.end());
            let ph = placeholder(phs.len());
            phs.push((ph.clone(), protected[start..end].to_string()));
            output.push_str(&protected[last..start]);
            output.push_str(&ph);
            last = end;
        }
        output.push_str(&protected[last..]);
        protected = output;
    }
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
///
/// 使用单一通用占位符模式配合成员集合判断，而不是把每个占位符 escape 后
/// 用 `|` 拼接：大量占位符（如 1 MB 文本）会让拼接出的正则超过编译大小
/// 上限并直接 panic（roadmap §8「正则上限可控」）。
pub fn space_around_inline_placeholders(text: &str, placeholders: &[(String, String)]) -> String {
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    static AFTER: OnceLock<Regex> = OnceLock::new();
    let inline: std::collections::HashSet<&str> = placeholders
        .iter()
        .filter(|(_, val)| {
            !val.contains('\n') && !is_escaped_markdown_value(val) && !is_hard_break_value(val)
        })
        .map(|(ph, _)| ph.as_str())
        .collect();
    if inline.is_empty() {
        return text.to_string();
    }
    let before = BEFORE
        .get_or_init(|| Regex::new(&format!(r"(\S)({PH_START}CCWPROTECTED\d+\u{{E001}})")).unwrap())
        .replace_all(text, |caps: &regex::Captures| {
            if inline.contains(&caps[2]) {
                format!("{} {}", &caps[1], &caps[2])
            } else {
                caps[0].to_string()
            }
        });
    AFTER
        .get_or_init(|| {
            Regex::new(&format!(
                r#"({PH_START}CCWPROTECTED\d+\u{{E001}})([^\s，。；：！？、）】》」』])"#
            ))
            .unwrap()
        })
        .replace_all(&before, |caps: &regex::Captures| {
            if inline.contains(&caps[1]) {
                format!("{} {}", &caps[1], &caps[2])
            } else {
                caps[0].to_string()
            }
        })
        .to_string()
}

fn is_escaped_markdown_value(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 2 && bytes[0] == b'\\'
}

fn is_hard_break_value(value: &str) -> bool {
    (value.len() >= 2 && value.bytes().all(|byte| byte == b' ')) || value == "\\"
}

pub fn restore_escaped_markdown_adjacency(text: &str, placeholders: &[(String, String)]) -> String {
    let mut current = text.to_string();
    for (_, value) in placeholders
        .iter()
        .filter(|(_, value)| is_escaped_markdown_value(value))
    {
        current = current.replace(&format!(" {value}"), value);
        current = current.replace(&format!("{value} "), value);
    }
    current
}

/// 为数学表达式占位符补充仅限 Han 边界的空格。
///
/// 数学 token 不复用普通 Markdown 占位符的 `\S` 边界规则，避免在全角标点后
/// 产生额外空格；表达式内部和标点邻接关系均保持原样。
pub fn space_around_math_placeholders(text: &str, placeholders: &[(String, String)]) -> String {
    static BEFORE: OnceLock<Regex> = OnceLock::new();
    static AFTER: OnceLock<Regex> = OnceLock::new();
    // 与 space_around_inline_placeholders 相同：单一通用模式 + 成员集合，
    // 避免按占位符拼接正则导致编译超限 panic。
    let inline: std::collections::HashSet<&str> = placeholders
        .iter()
        .filter(|(_, val)| !val.contains('\n'))
        .map(|(ph, _)| ph.as_str())
        .collect();
    if inline.is_empty() {
        return text.to_string();
    }
    let cjk = r"[\u{3400}-\u{4dbf}\u{4e00}-\u{9fff}\u{f900}-\u{faff}]";
    let before = BEFORE
        .get_or_init(|| {
            Regex::new(&format!(r"({cjk})({PH_START}CCWPROTECTED\d+\u{{E001}})")).unwrap()
        })
        .replace_all(text, |caps: &regex::Captures| {
            if inline.contains(&caps[2]) {
                format!("{} {}", &caps[1], &caps[2])
            } else {
                caps[0].to_string()
            }
        });
    AFTER
        .get_or_init(|| {
            Regex::new(&format!(r"({PH_START}CCWPROTECTED\d+\u{{E001}})({cjk})")).unwrap()
        })
        .replace_all(&before, |caps: &regex::Captures| {
            if inline.contains(&caps[1]) {
                format!("{} {}", &caps[1], &caps[2])
            } else {
                caps[0].to_string()
            }
        })
        .to_string()
}

/// 按创建顺序的逆序还原占位符，保证嵌套内容（如链接中的 URL）正确还原。
///
/// 实现为单遍正则替换 + 哈希查找：占位符编号互不重叠、值来自原文切片
/// （不含占位符 token），因此一遍替换与逐个 `replace` 的逆序循环等价；
/// 后者对 1 MB 级文本是 O(placeholders × len) 的热点（roadmap §8）。
pub fn restore(text: &str, placeholders: &[(String, String)]) -> String {
    static PH_RE: OnceLock<Regex> = OnceLock::new();
    if placeholders.is_empty() {
        return text.to_string();
    }
    let map: std::collections::HashMap<&str, &str> = placeholders
        .iter()
        .map(|(ph, val)| (ph.as_str(), val.as_str()))
        .collect();
    let re = PH_RE
        .get_or_init(|| Regex::new(&format!("{PH_START}CCWPROTECTED\\d+\\u{{E001}}")).unwrap());
    re.replace_all(text, |caps: &regex::Captures| {
        let token: &str = caps[0].as_ref();
        map.get(token).copied().unwrap_or(token).to_string()
    })
    .to_string()
}
