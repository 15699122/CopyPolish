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
            // LaTeX display $$...$$（排除转义 \$）
            r"(?s)(?<!\\)\$\$(?!\$).*?(?<!\\)\$\$",
            // LaTeX inline $...$（排除转义与空白起始）
            r#"(?<!\\)\$(?!\s|\$)(?:\\.|[^$\n\\]){1,300}?(?<!\\)\$(?!\$)"#,
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
    let protected = protect_with(protect_patterns(), &html_protected, placeholders)?;
    Ok(protect_inline_code(&protected, placeholders))
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

fn find_link_label_end(bytes: &[u8], open: usize) -> Option<usize> {
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
