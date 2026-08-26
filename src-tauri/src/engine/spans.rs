//! 统一文本 span 模型与重叠仲裁。
//!
//! 本模块暂不替换现有占位符管线，只为后续 TextSpan/TextEdit 重构提供稳定的
//! 优先级和重叠规则。仲裁结果按原文位置排序，调用方可以安全地按字节区间消费。

use super::semantic_tokens::scan_math_expressions;
use super::tokenizer::detect_chemical_formulas;
use super::unit_lexicon::scan_measurements;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpanPriority {
    Editable = 0,
    SemanticAtomic = 1,
    OpaqueStructure = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpanKind {
    EditableText,
    ChemicalFormula,
    Measurement,
    Temperature,
    ScientificUnit,
    MathExpression,
    InlineCode,
    MarkdownLink,
    InlineHtml,
    HtmlBlock,
    FencedCode,
    FrontMatter,
    HtmlComment,
    ReferenceDefinition,
    IndentedCode,
    TableSeparator,
    LatexMath,
}

impl SpanKind {
    pub(crate) fn priority(self) -> SpanPriority {
        match self {
            Self::EditableText => SpanPriority::Editable,
            Self::ChemicalFormula
            | Self::Measurement
            | Self::Temperature
            | Self::ScientificUnit
            | Self::MathExpression => SpanPriority::SemanticAtomic,
            Self::InlineCode
            | Self::MarkdownLink
            | Self::InlineHtml
            | Self::HtmlBlock
            | Self::FencedCode
            | Self::FrontMatter
            | Self::HtmlComment
            | Self::ReferenceDefinition
            | Self::IndentedCode
            | Self::TableSeparator
            | Self::LatexMath => SpanPriority::OpaqueStructure,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextSpan {
    pub start: usize,
    pub end: usize,
    pub kind: SpanKind,
    pub priority: SpanPriority,
}

impl TextSpan {
    pub(crate) fn new(start: usize, end: usize, kind: SpanKind) -> Option<Self> {
        (start < end).then_some(Self {
            start,
            end,
            priority: kind.priority(),
            kind,
        })
    }

    pub(crate) fn len(self) -> usize {
        self.end - self.start
    }

    fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// 以“优先级高 > 同优先级更长 > 更早出现”为规则选择不重叠 span。
pub(crate) fn arbitrate_spans(mut candidates: Vec<TextSpan>) -> Vec<TextSpan> {
    candidates.sort_by_key(|span| {
        (
            std::cmp::Reverse(span.priority),
            span.start,
            std::cmp::Reverse(span.len()),
        )
    });

    let mut accepted: Vec<TextSpan> = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if accepted
            .iter()
            .copied()
            .all(|span| !span.overlaps(candidate))
        {
            accepted.push(candidate);
        }
    }
    accepted.sort_by_key(|span| (span.start, span.end));
    accepted
}

/// 将现有化学式、单位和数学扫描结果汇总为统一语义 span。
///
/// 结构保护 span 尚未接入这里；后续 Markdown scanner 可将结构 span 与本结果
/// 合并后再次调用 `arbitrate_spans`，从而让结构自动覆盖内部语义 token。
pub(crate) fn scan_semantic_spans(text: &str) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    for (start, end) in detect_chemical_formulas(text) {
        if let Some(span) = TextSpan::new(start, end, SpanKind::ChemicalFormula) {
            spans.push(span);
        }
    }
    for measurement in scan_measurements(text) {
        let kind = if measurement
            .unit_start
            .checked_sub(measurement.number_end)
            .is_some_and(|_| {
                matches!(
                    &text[measurement.unit_start..measurement.end],
                    "℃" | "℉" | "°C" | "°F"
                )
            }) {
            SpanKind::Temperature
        } else if text[measurement.unit_start..measurement.end]
            .chars()
            .any(|ch| matches!(ch, '·' | '⋅' | '⁰'..='⁹' | '₀'..='₉' | '⁺' | '⁻'))
        {
            SpanKind::ScientificUnit
        } else {
            SpanKind::Measurement
        };
        if let Some(span) = TextSpan::new(measurement.start, measurement.end, kind) {
            spans.push(span);
        }
    }
    for (start, end) in scan_math_expressions(text) {
        if let Some(span) = TextSpan::new(start, end, SpanKind::MathExpression) {
            spans.push(span);
        }
    }
    arbitrate_spans(spans)
}

/// 扫描当前保护层已经支持的结构 span。
///
/// 这是后续 placeholder → TextEdit 迁移的只读入口；当前不改变生产 pipeline。
pub(crate) fn scan_structure_spans(text: &str) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    scan_front_matter_spans(text, &mut spans);
    scan_fenced_code_spans(text, &mut spans);
    scan_html_comment_spans(text, &mut spans);
    scan_reference_definition_spans(text, &mut spans);
    scan_indented_code_spans(text, &mut spans);
    scan_table_separator_spans(text, &mut spans);
    scan_inline_code_spans(text, &mut spans);
    scan_markdown_link_spans(text, &mut spans);
    scan_html_block_spans(text, &mut spans);
    scan_inline_html_spans(text, &mut spans);
    scan_dollar_math_spans(text, &mut spans);
    arbitrate_spans(spans)
}

pub(crate) fn scan_all_spans(text: &str) -> Vec<TextSpan> {
    let mut spans = scan_semantic_spans(text);
    spans.extend(scan_structure_spans(text));
    arbitrate_spans(spans)
}

fn line_ranges(text: &str) -> Vec<(usize, usize, &str)> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    for line in text.split('\n') {
        let end = start + line.len();
        ranges.push((start, end, line));
        start = end.saturating_add(1);
    }
    ranges
}

fn scan_front_matter_spans(text: &str, output: &mut Vec<TextSpan>) {
    let lines = line_ranges(text);
    let Some(&(start, _, first)) = lines.first() else {
        return;
    };
    let first = first.strip_prefix('\u{FEFF}').unwrap_or(first);
    if first.trim() != "---" {
        return;
    }
    for &(end_start, end, line) in lines.iter().skip(1) {
        if line.trim() == "---" {
            if let Some(span) = TextSpan::new(start, end, SpanKind::FrontMatter) {
                output.push(span);
            }
            break;
        }
        let _ = end_start;
    }
}

fn scan_fenced_code_spans(text: &str, output: &mut Vec<TextSpan>) {
    let lines = line_ranges(text);
    let mut index = 0;
    while index < lines.len() {
        let (start, _, line) = lines[index];
        let trimmed = line.trim_start_matches([' ', '\t']);
        let indent = line.len() - trimmed.len();
        let bytes = trimmed.as_bytes();
        let marker = if bytes.starts_with(b"```") {
            Some(b'`')
        } else if bytes.starts_with(b"~~~") {
            Some(b'~')
        } else {
            None
        };
        if indent > 3 || marker.is_none() {
            index += 1;
            continue;
        }
        let marker = marker.unwrap();
        let mut marker_len = 0;
        while marker_len < trimmed.len() && trimmed.as_bytes()[marker_len] == marker {
            marker_len += 1;
        }
        if marker_len < 3 {
            index += 1;
            continue;
        }
        let mut end_line = None;
        for (candidate, (_, _, content)) in lines.iter().enumerate().skip(index + 1) {
            let close = content.trim_start_matches([' ', '\t']);
            let close_count = close.bytes().take_while(|byte| *byte == marker).count();
            if close_count >= marker_len && close[close_count..].trim().is_empty() {
                end_line = Some(candidate);
                break;
            }
        }
        if let Some(end_line) = end_line {
            let end = lines[end_line].1;
            if let Some(span) = TextSpan::new(start, end, SpanKind::FencedCode) {
                output.push(span);
            }
            index = end_line + 1;
        } else {
            index += 1;
        }
    }
}

fn scan_html_comment_spans(text: &str, output: &mut Vec<TextSpan>) {
    let mut cursor = 0;
    while let Some(relative) = text[cursor..].find("<!--") {
        let start = cursor + relative;
        let Some(relative_end) = text[start + 4..].find("-->") else {
            break;
        };
        let end = start + 4 + relative_end + 3;
        if let Some(span) = TextSpan::new(start, end, SpanKind::HtmlComment) {
            output.push(span);
        }
        cursor = end;
    }
}

fn scan_reference_definition_spans(text: &str, output: &mut Vec<TextSpan>) {
    for (start, end, line) in line_ranges(text) {
        let trimmed = line.trim_start_matches([' ', '\t']);
        let indent = line.len() - trimmed.len();
        if indent > 3 || !trimmed.starts_with('[') {
            continue;
        }
        let Some(label_end) = trimmed.find("]:") else {
            continue;
        };
        let target = trimmed[label_end + 2..].trim();
        if target.starts_with('<')
            || target.starts_with("http://")
            || target.starts_with("https://")
        {
            if let Some(span) = TextSpan::new(start, end, SpanKind::ReferenceDefinition) {
                output.push(span);
            }
        }
    }
}

fn scan_indented_code_spans(text: &str, output: &mut Vec<TextSpan>) {
    let lines = line_ranges(text);
    let mut index = 0;
    while index < lines.len() {
        if !(lines[index].2.starts_with("    ") || lines[index].2.starts_with('\t')) {
            index += 1;
            continue;
        }
        let start = lines[index].0;
        let mut end = lines[index].1;
        index += 1;
        while index < lines.len()
            && (lines[index].2.starts_with("    ") || lines[index].2.starts_with('\t'))
        {
            end = lines[index].1;
            index += 1;
        }
        if let Some(span) = TextSpan::new(start, end, SpanKind::IndentedCode) {
            output.push(span);
        }
    }
}

fn scan_table_separator_spans(text: &str, output: &mut Vec<TextSpan>) {
    for (start, end, line) in line_ranges(text) {
        let trimmed = line.trim();
        if !trimmed.contains('|') {
            continue;
        }
        let cells: Vec<&str> = trimmed
            .split('|')
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .collect();
        let valid = cells.len() >= 2
            && cells.iter().all(|cell| {
                let cell = cell.strip_prefix(':').unwrap_or(cell);
                let cell = cell.strip_suffix(':').unwrap_or(cell);
                cell.len() >= 3 && cell.bytes().all(|byte| byte == b'-')
            });
        if valid {
            if let Some(span) = TextSpan::new(start, end, SpanKind::TableSeparator) {
                output.push(span);
            }
        }
    }
}

fn scan_inline_code_spans(text: &str, output: &mut Vec<TextSpan>) {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(relative) = text[cursor..].find('`') else {
            break;
        };
        let start = cursor + relative;
        let mut delimiter_end = start;
        while delimiter_end < bytes.len() && bytes[delimiter_end] == b'`' {
            delimiter_end += 1;
        }
        let delimiter_len = delimiter_end - start;
        let mut search = delimiter_end;
        let line_end = text[delimiter_end..]
            .find('\n')
            .map(|offset| delimiter_end + offset)
            .unwrap_or(bytes.len());
        let mut close = None;
        while search < line_end {
            let Some(relative_tick) = text[search..line_end].find('`') else {
                break;
            };
            let candidate = search + relative_tick;
            let mut candidate_end = candidate;
            while candidate_end < line_end && bytes[candidate_end] == b'`' {
                candidate_end += 1;
            }
            if candidate_end - candidate == delimiter_len {
                close = Some(candidate_end);
                break;
            }
            search = candidate_end;
        }
        if let Some(end) = close {
            if let Some(span) = TextSpan::new(start, end, SpanKind::InlineCode) {
                output.push(span);
            }
            cursor = end;
        } else {
            cursor = delimiter_end;
        }
    }
}

fn scan_markdown_link_spans(text: &str, output: &mut Vec<TextSpan>) {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(relative) = text[cursor..].find('[') else {
            break;
        };
        let label_start = cursor + relative;
        let structure_start = label_start
            .checked_sub(1)
            .filter(|index| bytes[*index] == b'!')
            .unwrap_or(label_start);
        let Some(label_end) = balanced_end(bytes, label_start, b'[', b']') else {
            cursor = label_start + 1;
            continue;
        };
        if bytes.get(label_end + 1) != Some(&b'(') {
            cursor = label_end + 1;
            continue;
        }
        let Some(target_end) = balanced_end(bytes, label_end + 1, b'(', b')') else {
            cursor = label_end + 1;
            continue;
        };
        if let Some(span) = TextSpan::new(structure_start, target_end + 1, SpanKind::MarkdownLink) {
            output.push(span);
        }
        cursor = target_end + 1;
    }
}

fn balanced_end(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0usize;
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        if bytes[cursor] == b'\\' {
            cursor = cursor.saturating_add(2);
            continue;
        }
        match bytes[cursor] {
            value if value == open => depth += 1,
            value if value == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => {}
        }
        cursor += 1;
    }
    None
}

fn scan_html_block_spans(text: &str, output: &mut Vec<TextSpan>) {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut offset = 0usize;
    let mut line = 0usize;
    while line < lines.len() {
        let trimmed = lines[line].trim_start_matches([' ', '\t']);
        let indent = lines[line].len() - trimmed.len();
        let Some(tag) = html_block_tag(trimmed, indent) else {
            offset += lines[line].len() + usize::from(line + 1 < lines.len());
            line += 1;
            continue;
        };
        let closing = format!("</{tag}");
        let mut end_line = None;
        for (candidate, content) in lines.iter().enumerate().skip(line) {
            if content.to_ascii_lowercase().contains(&closing) {
                end_line = Some(candidate);
                break;
            }
        }
        if let Some(end_line) = end_line {
            // 终点必须包含起始行到结束行之间的所有中间行与换行符，
            // 否则块内正文会漏出 span 被当作可编辑文本格式化。
            let mut end = offset;
            for content in lines.iter().take(end_line).skip(line) {
                end += content.len() + 1;
            }
            end += lines[end_line].len();
            if let Some(span) = TextSpan::new(offset, end, SpanKind::HtmlBlock) {
                output.push(span);
            }
            for content in lines.iter().take(end_line + 1).skip(line) {
                offset += content.len() + 1;
            }
            line = end_line + 1;
        } else {
            offset += lines[line].len() + usize::from(line + 1 < lines.len());
            line += 1;
        }
    }
}

/// 扫描行内 HTML 标签/元素（与 protection.rs 的占位符行为对齐）：
/// 单个标签（含自闭合）或成对元素的完整区间；跨行标签不匹配。
fn scan_inline_html_spans(text: &str, output: &mut Vec<TextSpan>) {
    use super::protection::{
        find_inline_html_closing_tag, find_inline_html_tag_end, inline_html_tag_name,
        is_inline_html_tag, is_self_closing_html_tag,
    };

    let bytes = text.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let Some(relative_open) = text[cursor..].find('<') else {
            break;
        };
        let open = cursor + relative_open;
        let Some(end) = find_inline_html_tag_end(bytes, open) else {
            break;
        };
        if !is_inline_html_tag(bytes, open, end) {
            cursor = open + 1;
            continue;
        }
        let mut close_end = None;
        if let Some(name) = inline_html_tag_name(bytes, open) {
            if !is_self_closing_html_tag(bytes, open, end) {
                if let Some(found) = find_inline_html_closing_tag(bytes, end + 1, name) {
                    close_end = Some(found);
                }
            }
        }
        let span_end = close_end.unwrap_or(end) + 1;
        if let Some(span) = TextSpan::new(open, span_end, SpanKind::InlineHtml) {
            output.push(span);
        }
        cursor = span_end;
    }
}

fn html_block_tag(line: &str, indent: usize) -> Option<&'static str> {
    if indent > 3 || !line.starts_with('<') {
        return None;
    }
    const TAGS: [&str; 18] = [
        "div", "section", "article", "aside", "header", "footer", "nav", "main", "table", "thead",
        "tbody", "tr", "ul", "ol", "li", "pre", "script", "style",
    ];
    TAGS.into_iter().find(|tag| {
        let prefix = format!("<{tag}");
        line.get(..prefix.len())
            .is_some_and(|value| value.eq_ignore_ascii_case(&prefix))
            && line[prefix.len()..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_whitespace() || matches!(ch, '>' | '/'))
    })
}

fn scan_dollar_math_spans(text: &str, output: &mut Vec<TextSpan>) {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(relative) = text[cursor..].find('$') else {
            break;
        };
        let start = cursor + relative;
        if escaped_dollar(bytes, start) {
            cursor = start + 1;
            continue;
        }
        let display = bytes.get(start + 1) == Some(&b'$');
        let delimiter_len = if display { 2 } else { 1 };
        if !display
            && bytes
                .get(start + 1)
                .is_none_or(|byte| byte.is_ascii_whitespace())
        {
            cursor = start + 1;
            continue;
        }
        let content_start = start + delimiter_len;
        let mut search = content_start;
        let mut close = None;
        while search < bytes.len() {
            if bytes[search] == b'$'
                && !escaped_dollar(bytes, search)
                && (delimiter_len == 2 || bytes.get(search + 1) != Some(&b'$'))
                && (delimiter_len == 1 || bytes.get(search + 1) == Some(&b'$'))
            {
                close = Some(search + delimiter_len);
                break;
            }
            if !display && bytes[search] == b'\n' {
                break;
            }
            search += 1;
        }
        if let Some(end) = close {
            if let Some(span) = TextSpan::new(start, end, SpanKind::LatexMath) {
                output.push(span);
            }
            cursor = end;
        } else {
            cursor = start + delimiter_len;
        }
    }
}

fn escaped_dollar(bytes: &[u8], index: usize) -> bool {
    let mut count = 0usize;
    let mut cursor = index;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        count += 1;
        cursor -= 1;
    }
    count % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::{
        arbitrate_spans, scan_all_spans, scan_semantic_spans, scan_structure_spans, SpanKind,
        SpanPriority, TextSpan,
    };

    #[test]
    fn opaque_structure_wins_over_inner_semantic_span() {
        let spans = arbitrate_spans(vec![
            TextSpan::new(2, 7, SpanKind::Measurement).unwrap(),
            TextSpan::new(0, 12, SpanKind::InlineCode).unwrap(),
        ]);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].kind, SpanKind::InlineCode);
        assert_eq!(spans[0].priority, SpanPriority::OpaqueStructure);
    }

    #[test]
    fn longer_same_priority_span_wins() {
        let spans = arbitrate_spans(vec![
            TextSpan::new(0, 5, SpanKind::MathExpression).unwrap(),
            TextSpan::new(0, 8, SpanKind::ScientificUnit).unwrap(),
        ]);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].end, 8);
    }

    #[test]
    fn semantic_scanners_share_one_arbitration_result() {
        let spans = scan_semantic_spans("样品Fe²⁺厚度10μm且计算∂f/∂x");
        let kinds: Vec<SpanKind> = spans.iter().map(|span| span.kind).collect();
        assert_eq!(
            kinds,
            vec![
                SpanKind::ChemicalFormula,
                SpanKind::Measurement,
                SpanKind::MathExpression
            ]
        );
        assert!(spans.windows(2).all(|pair| pair[0].end <= pair[1].start));
    }

    #[test]
    fn invalid_empty_span_is_rejected() {
        assert!(TextSpan::new(4, 4, SpanKind::EditableText).is_none());
        assert!(TextSpan::new(8, 3, SpanKind::EditableText).is_none());
    }

    #[test]
    fn structure_scanners_cover_precedence_fixture_shapes() {
        let text = "代码`10μm $x$ https://example.com`查看[价格](https://example.com/a_(b))";
        let spans = scan_structure_spans(text);
        let kinds: Vec<SpanKind> = spans.iter().map(|span| span.kind).collect();
        assert!(kinds.contains(&SpanKind::InlineCode));
        assert!(kinds.contains(&SpanKind::MarkdownLink));
        assert!(!kinds.contains(&SpanKind::LatexMath));
    }

    #[test]
    fn structure_span_wins_when_combined_with_semantic_spans() {
        let text = "代码`10μm $x$ https://example.com`继续";
        let spans = scan_all_spans(text);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].kind, SpanKind::InlineCode);
        assert_eq!(
            &text[spans[0].start..spans[0].end],
            "`10μm $x$ https://example.com`"
        );
    }

    #[test]
    fn block_structure_scanners_cover_supported_protection_shapes() {
        let text = "---\ntitle: 10μm\n---\n<!-- 3mg/mL -->\n```text\n$x$\n```\n    Fe²⁺\n| --- | --- |\n[home]: https://example.com\n正文";
        let spans = scan_structure_spans(text);
        let kinds: Vec<SpanKind> = spans.iter().map(|span| span.kind).collect();
        for kind in [
            SpanKind::FrontMatter,
            SpanKind::HtmlComment,
            SpanKind::FencedCode,
            SpanKind::IndentedCode,
            SpanKind::TableSeparator,
            SpanKind::ReferenceDefinition,
        ] {
            assert!(kinds.contains(&kind), "missing span kind: {kind:?}");
        }
    }

    #[test]
    fn unclosed_block_structures_do_not_consume_following_text() {
        let text = "---\ntitle: 10μm\n正文在GitHub上发布";
        let spans = scan_structure_spans(text);
        assert!(!spans.iter().any(|span| span.kind == SpanKind::FrontMatter));

        let text = "```text\n10μm\n正文在GitHub上发布";
        let spans = scan_structure_spans(text);
        assert!(!spans.iter().any(|span| span.kind == SpanKind::FencedCode));
    }

    /// HTML block span 必须覆盖起始行到结束行之间的全部中间行；
    /// 否则块内正文会漏出 span 被当作可编辑文本格式化。
    #[test]
    fn html_block_span_covers_interior_lines() {
        let text = "<div class=\"notice\">\n在GitHub上发布5000元\n</div>\n正文在GitHub上发布";
        let spans = scan_structure_spans(text);
        let span = spans
            .iter()
            .find(|span| span.kind == SpanKind::HtmlBlock)
            .expect("html block span must be detected");
        assert_eq!(
            &text[span.start..span.end],
            "<div class=\"notice\">\n在GitHub上发布5000元\n</div>"
        );
    }
}
