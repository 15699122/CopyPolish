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
            // YAML front matter：仅匹配文档开头，可选 UTF-8 BOM，分隔线独占一行。
            r"(?s)\A(?:\u{FEFF})?---[ \t]*\n.*?\n---[ \t]*(?=\n|$)",
            // HTML 注释（支持跨行，注释内部完全保持原样）
            r"(?s)<!--.*?-->",
            // 引用式链接定义：整行保护，支持最多 3 个前导空格与可选标题。
            r##"(?m)^[ \t]{0,3}\[[^\]\n]+\]:[ \t]*(?:<[^>\n]+>|[^\s<>\n]+)(?:[ \t]+(?:"[^"]*"|'[^']*'|\([^\)]*\)))?[ \t]*(?=\n|$)"##,
            // LaTeX environment
            r"(?s)\\begin\{(equation\*?|align\*?|gather\*?|multline\*?|matrix|pmatrix|bmatrix|cases)\}.*?\\end\{\1\}",
            // LaTeX display \[...\]
            r"(?s)\\\[.*?\\\]",
            // LaTeX inline \(...\)
            r"(?s)\\\(.*?\\\)",
            // Markdown reference-style link usage
            r"\[[^\]\n]+\]\[[^\]\n]*\]",
            // autolink <https://...> / <mail@...>
            r"(?i)<(?:(?:https?://[^>\s]+)|(?:[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}))>",
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
    let linked = protect_markdown_links(text, placeholders);
    let table_protected = protect_markdown_table_lines(&linked, placeholders);
    let html_protected = protect_html_blocks(&table_protected, placeholders);
    let inline_html_protected = protect_inline_html_tags(&html_protected, placeholders);
    let math_protected = protect_dollar_math(&inline_html_protected, placeholders);
    let protected = protect_with(protect_patterns(), &math_protected, placeholders)?;
    let hard_breaks = protect_markdown_hard_breaks(&protected, placeholders);
    let escaped = protect_escaped_markdown(&hard_breaks, placeholders);
    Ok(protect_inline_code(&escaped, placeholders))
}

/// 保护美元定界的 LaTeX 数学表达式。
///
/// 展示数学使用 `$$...$$`，可跨行；行内数学使用 `$...$`，不跨行。
/// 扫描器显式处理反斜杠奇偶性，避免把 `\$` 误当作定界符，同时不限制
/// 行内表达式长度。对金额等缺少闭合定界符的普通文本保持原样。
fn protect_dollar_math(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;

    while cursor < bytes.len() {
        let Some(relative) = text[cursor..].find('$') else {
            output.push_str(&text[cursor..]);
            break;
        };
        let open = cursor + relative;
        if is_escaped_dollar(bytes, open) {
            output.push_str(&text[cursor..open + 1]);
            cursor = open + 1;
            continue;
        }

        let is_display = bytes.get(open + 1) == Some(&b'$');
        let delimiter_len = if is_display { 2 } else { 1 };
        if !is_display
            && (bytes
                .get(open + 1)
                .is_none_or(|byte| byte.is_ascii_whitespace())
                || bytes.get(open + 1) == Some(&b'$'))
        {
            output.push_str(&text[cursor..open + 1]);
            cursor = open + 1;
            continue;
        }

        let content_start = open + delimiter_len;
        let Some(close_start) = find_dollar_math_close(bytes, content_start, delimiter_len) else {
            output.push_str(&text[cursor..open + delimiter_len]);
            cursor = open + delimiter_len;
            continue;
        };
        if !is_display && bytes[content_start..close_start].contains(&b'\n') {
            output.push_str(&text[cursor..open + 1]);
            cursor = open + 1;
            continue;
        }

        let close_end = close_start + delimiter_len;
        let ph = placeholder(placeholders.len());
        placeholders.push((ph.clone(), text[open..close_end].to_string()));
        output.push_str(&text[cursor..open]);
        output.push_str(&ph);
        cursor = close_end;
    }

    output
}

fn find_dollar_math_close(bytes: &[u8], start: usize, delimiter_len: usize) -> Option<usize> {
    let mut cursor = start;
    while cursor < bytes.len() {
        if bytes[cursor] == b'$'
            && !is_escaped_dollar(bytes, cursor)
            && (delimiter_len == 1 || bytes.get(cursor + 1) == Some(&b'$'))
        {
            if delimiter_len == 1 && bytes.get(cursor + 1) == Some(&b'$') {
                cursor += 2;
                continue;
            }
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

fn is_escaped_dollar(bytes: &[u8], index: usize) -> bool {
    let mut backslashes = 0;
    let mut cursor = index;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        backslashes += 1;
        cursor -= 1;
    }
    backslashes % 2 == 1
}

/// 保护 Markdown 硬换行标记：行尾两个以上空格或行尾反斜杠。
/// 换行本身不占位，继续交给现有换行归一化/还原流程处理。
fn protect_markdown_hard_breaks(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    let mut output = String::with_capacity(text.len());
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            output.push('\n');
        }

        let trailing_spaces = line.len() - line.trim_end_matches(' ').len();
        let hard_break_len = if trailing_spaces >= 2 {
            trailing_spaces
        } else if line.ends_with('\\') {
            1
        } else {
            0
        };

        if hard_break_len == 0 {
            output.push_str(line);
            continue;
        }

        let content_end = line.len() - hard_break_len;
        output.push_str(&line[..content_end]);
        let marker = &line[content_end..];
        let ph = placeholder(placeholders.len());
        placeholders.push((ph.clone(), marker.to_string()));
        output.push_str(&ph);
    }
    output
}

/// 保护反斜杠与 Markdown 可转义标点的组合，避免转义标记被规则改写。
///
/// 该扫描在 LaTeX / HTML / 链接等完整结构之后执行，因此不会截断已有的
/// `\\[...\\]`、`\\(...\\)` 或 `\\frac{...}` 保护片段。转义占位符在边界
/// 补空格阶段会被排除，保持 `\\*text\\*` 的原始邻接关系。
fn protect_escaped_markdown(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    // `[` / `]` / `(` / `)` 与现有 LaTeX display/inline 语法重叠，
    // 保留给 LaTeX 保护层处理，避免改变既有公式语义。
    const ESCAPABLE: &[u8] = b"\\`*_{}#+-.!|>";
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;

    while cursor < bytes.len() {
        if bytes[cursor] == b'\\'
            && bytes
                .get(cursor + 1)
                .is_some_and(|byte| ESCAPABLE.contains(byte))
        {
            let ph = placeholder(placeholders.len());
            placeholders.push((ph.clone(), text[cursor..cursor + 2].to_string()));
            output.push_str(&ph);
            cursor += 2;
        } else {
            if bytes[cursor] == b'\\' {
                output.push('\\');
                cursor += 1;
            } else {
                let next = text[cursor..]
                    .find('\\')
                    .map(|offset| cursor + offset)
                    .unwrap_or(bytes.len());
                output.push_str(&text[cursor..next]);
                cursor = next;
            }
        }
    }

    output
}

/// 保护行内 HTML 标签本身，但不保护标签之间的普通文本。
///
/// 只接受带 ASCII 标签名且在同一行闭合的 `<tag ...>` / `</tag>` / `<tag />`
/// 结构；属性值中的 `>` 仅在引号外作为标签结束符。比较表达式和 autolink
/// 不满足标签边界，不会进入此保护路径。
fn protect_inline_html_tags(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;

    while cursor < bytes.len() {
        let Some(relative_open) = text[cursor..].find('<') else {
            output.push_str(&text[cursor..]);
            break;
        };
        let open = cursor + relative_open;
        let Some(end) = find_inline_html_tag_end(bytes, open) else {
            output.push_str(&text[cursor..]);
            break;
        };
        if !is_inline_html_tag(bytes, open, end) {
            output.push_str(&text[cursor..open + 1]);
            cursor = open + 1;
            continue;
        }

        output.push_str(&text[cursor..open]);
        if let Some(name) = inline_html_tag_name(bytes, open) {
            if !is_self_closing_html_tag(bytes, open, end) {
                if let Some(close_end) = find_inline_html_closing_tag(bytes, end + 1, name) {
                    let ph = placeholder(placeholders.len());
                    placeholders.push((ph.clone(), text[open..=close_end].to_string()));
                    output.push_str(&ph);
                    cursor = close_end + 1;
                    continue;
                }
            }
        }

        let ph = placeholder(placeholders.len());
        placeholders.push((ph.clone(), text[open..=end].to_string()));
        output.push_str(&ph);
        cursor = end + 1;
    }

    output
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
    let text = std::str::from_utf8(bytes).ok()?;
    let mut cursor = start;
    while cursor < bytes.len() {
        let relative = text[cursor..].find(&marker)?;
        let close_start = cursor + relative;
        let boundary = bytes.get(close_start + marker.len()).copied();
        if matches!(boundary, Some(b'>') | Some(b' ') | Some(b'\t')) {
            return find_inline_html_tag_end(bytes, close_start);
        }
        cursor = close_start + marker.len();
    }
    None
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

/// 保护常见 Markdown HTML block，避免 block 内部文本被普通规则改写。
///
/// 仅从行首（允许最多 3 个空格）识别块级标签；行内标签如
/// `<span>GitHub</span>` 不会进入此保护路径。未闭合 block 保持普通文本，
/// 避免把后续文档内容整体吞入占位符。
fn protect_html_blocks(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut output = Vec::with_capacity(lines.len());
    let mut index = 0;

    while index < lines.len() {
        let Some(tag) = html_block_opening_tag(lines[index]) else {
            output.push(lines[index].to_string());
            index += 1;
            continue;
        };

        let Some(end) = find_html_block_end(&lines, index, tag) else {
            output.push(lines[index].to_string());
            index += 1;
            continue;
        };

        let block = lines[index..=end].join("\n");
        let ph = placeholder(placeholders.len());
        placeholders.push((ph.clone(), block));
        output.push(ph);
        index = end + 1;
    }

    output.join("\n")
}

fn html_block_opening_tag(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    if line.len() - trimmed.len() > 3 || !trimmed.starts_with('<') {
        return None;
    }

    const TAGS: [&str; 18] = [
        "div", "section", "article", "aside", "header", "footer", "nav", "main", "table", "thead",
        "tbody", "tr", "ul", "ol", "li", "pre", "script", "style",
    ];
    TAGS.into_iter().find(|tag| {
        let prefix_len = 1 + tag.len();
        let Some(prefix) = trimmed.get(..prefix_len) else {
            return false;
        };
        let Some(rest) = trimmed.get(prefix_len..) else {
            return false;
        };
        prefix.eq_ignore_ascii_case(&format!("<{tag}"))
            && rest
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_whitespace() || ch == '>' || ch == '/')
    })
}

fn find_html_block_end(lines: &[&str], start: usize, tag: &str) -> Option<usize> {
    let closing = format!("</{tag}");
    for (index, line) in lines.iter().enumerate().skip(start) {
        if line.to_ascii_lowercase().contains(&closing) {
            return Some(index);
        }
    }
    None
}

/// 保护 Markdown 表格分隔行，避免分隔符中的短横线被标点规则改写。
///
/// 只接受包含至少两个分隔单元的行；每个单元必须是 `---`、`:---`、
/// `---:` 或 `:---:` 形式。单独的水平分隔线和普通表格正文不受影响。
fn protect_markdown_table_lines(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    text.split('\n')
        .map(|line| {
            if is_table_separator_line(line) {
                let ph = placeholder(placeholders.len());
                placeholders.push((ph.clone(), line.to_string()));
                ph
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains('|') {
        return false;
    }

    let cells: Vec<&str> = trimmed
        .split('|')
        .map(str::trim)
        .filter(|cell| !cell.is_empty())
        .collect();
    cells.len() >= 2 && cells.iter().all(|cell| is_table_separator_cell(cell))
}

fn is_table_separator_cell(cell: &str) -> bool {
    let cell = cell
        .strip_prefix(':')
        .unwrap_or(cell)
        .strip_suffix(':')
        .unwrap_or_else(|| cell.strip_prefix(':').unwrap_or(cell));
    cell.len() >= 3 && cell.bytes().all(|byte| byte == b'-')
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

/// 保护任意长度反引号 delimiter 的行内代码。
///
/// 只在同一行找到长度完全相同的闭合 delimiter 时保护完整代码；较短的
/// 反引号可以出现在代码内容中。未闭合 delimiter 仅保护反引号串本身，
/// 避免吞掉后续普通正文。
fn protect_inline_code(text: &str, placeholders: &mut Vec<(String, String)>) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;

    while cursor < bytes.len() {
        if bytes[cursor] != b'`' {
            let next = text[cursor..]
                .find('`')
                .map(|offset| cursor + offset)
                .unwrap_or(bytes.len());
            out.push_str(&text[cursor..next]);
            cursor = next;
            continue;
        }

        let opening_start = cursor;
        let opening_end = run_end(bytes, opening_start);
        let delimiter_len = opening_end - opening_start;
        let mut search = opening_end;
        let mut closing = None;

        while search < bytes.len() {
            let line_end = text[search..]
                .find('\n')
                .map(|offset| search + offset)
                .unwrap_or(bytes.len());
            let Some(relative_tick) = text[search..line_end].find('`') else {
                break;
            };
            let candidate_start = search + relative_tick;
            let candidate_end = run_end(bytes, candidate_start);
            if candidate_end - candidate_start == delimiter_len {
                closing = Some((candidate_start, candidate_end));
                break;
            }
            search = candidate_end;
        }

        let Some((closing_start, closing_end)) = closing else {
            let ph = placeholder(placeholders.len());
            placeholders.push((ph.clone(), text[opening_start..opening_end].to_string()));
            out.push_str(&ph);
            cursor = opening_end;
            continue;
        };

        let ph = placeholder(placeholders.len());
        placeholders.push((ph.clone(), text[opening_start..closing_end].to_string()));
        out.push_str(&ph);
        cursor = closing_end;
        let _ = closing_start;
    }

    out
}

fn run_end(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && bytes[end] == b'`' {
        end += 1;
    }
    end
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
    let linked = protect_markdown_links(text, &mut phs);
    let protected = match protect_with(link_patterns(), &linked, &mut phs) {
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
pub fn restore(text: &str, placeholders: &[(String, String)]) -> String {
    let mut current = text.to_string();
    for (ph, val) in placeholders.iter().rev() {
        current = current.replace(ph.as_str(), val);
    }
    current
}
