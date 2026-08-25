//! TextEdit 计划与冲突仲裁。
//!
//! 本模块是 Span → TextEdit 迁移基础设施，当前不接管生产 pipeline。编辑使用
//! 原文 UTF-8 字节区间，先校验边界，再按优先级仲裁，最后从后向前应用。

use super::semantic_tokens::scan_semantic_tokens;
use super::spans::{scan_all_spans, SpanKind, SpanPriority};
use super::tokenizer::{classify, CharKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EditPriority {
    Editable = 0,
    SemanticAtomic = 1,
    OpaqueStructure = 2,
}

impl From<SpanPriority> for EditPriority {
    fn from(priority: SpanPriority) -> Self {
        match priority {
            SpanPriority::Editable => Self::Editable,
            SpanPriority::SemanticAtomic => Self::SemanticAtomic,
            SpanPriority::OpaqueStructure => Self::OpaqueStructure,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub priority: EditPriority,
}

impl TextEdit {
    pub(crate) fn new(
        text: &str,
        start: usize,
        end: usize,
        replacement: impl Into<String>,
        priority: EditPriority,
    ) -> Result<Self, String> {
        if start > end {
            return Err(format!("invalid edit range: {start}..{end}"));
        }
        if end > text.len() {
            return Err(format!("edit range exceeds text: {start}..{end}"));
        }
        if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return Err(format!("edit range splits UTF-8 text: {start}..{end}"));
        }
        Ok(Self {
            start,
            end,
            replacement: replacement.into(),
            priority,
        })
    }

    fn len(&self) -> usize {
        self.end - self.start
    }

    fn conflicts(&self, other: &Self) -> bool {
        match (self.start == self.end, other.start == other.end) {
            (true, true) => self.start == other.start,
            (true, false) => self.start > other.start && self.start < other.end,
            (false, true) => other.start > self.start && other.start < self.end,
            (false, false) => self.start < other.end && other.start < self.end,
        }
    }
}

/// 选择不重叠编辑：优先级更高、同优先级区间更长、再按起点稳定排序。
pub(crate) fn arbitrate_edits(mut candidates: Vec<TextEdit>) -> Vec<TextEdit> {
    candidates.sort_by_key(|edit| {
        (
            std::cmp::Reverse(edit.priority),
            edit.start,
            std::cmp::Reverse(edit.len()),
        )
    });

    let mut accepted: Vec<TextEdit> = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if accepted.iter().all(|edit| !edit.conflicts(&candidate)) {
            accepted.push(candidate);
        }
    }
    accepted.sort_by_key(|edit| (edit.start, edit.end));
    accepted
}

/// 在原文上应用已仲裁的非重叠编辑。
pub(crate) fn apply_edits(text: &str, edits: &[TextEdit]) -> Result<String, String> {
    let mut ordered = edits.to_vec();
    ordered.sort_by_key(|edit| (edit.start, edit.end));

    for edit in &ordered {
        if edit.end > text.len()
            || edit.start > edit.end
            || !text.is_char_boundary(edit.start)
            || !text.is_char_boundary(edit.end)
        {
            return Err(format!("invalid edit range: {}..{}", edit.start, edit.end));
        }
    }
    if ordered.windows(2).any(|pair| pair[0].conflicts(&pair[1])) {
        return Err("overlapping edits must be arbitrated before application".to_string());
    }

    let mut output = text.to_string();
    for edit in ordered.into_iter().rev() {
        output.replace_range(edit.start..edit.end, &edit.replacement);
    }
    Ok(output)
}

/// 为已经仲裁的语义 span 生成边界编辑计划。
///
/// 该函数是生产 pipeline 迁移前的对照路径：只处理最终保留下来的
/// `Measurement` / `ScientificUnit` / `MathExpression` span，结构 span 内部
/// 不会生成编辑。温标的数字与符号间距继续保持 `25°C` / `4℃` 的既有语义。
pub(crate) fn plan_semantic_boundary_edits(text: &str) -> Vec<TextEdit> {
    let spans = scan_all_spans(text);
    let semantic_ranges: Vec<(usize, usize, SpanKind)> = spans
        .iter()
        .filter_map(|span| match span.kind {
            SpanKind::Measurement
            | SpanKind::Temperature
            | SpanKind::ScientificUnit
            | SpanKind::MathExpression => Some((span.start, span.end, span.kind)),
            _ => None,
        })
        .collect();

    let mut edits = Vec::new();
    for (start, end, kind) in semantic_ranges {
        if matches!(kind, SpanKind::Measurement | SpanKind::ScientificUnit) {
            for token in scan_semantic_tokens(text) {
                if token.start != start
                    || token.end != end
                    || token.kind == super::semantic_tokens::SemanticTokenKind::Temperature
                {
                    continue;
                }
                if token.number_end == token.unit_start
                    && next_char_range(text, token.unit_start).is_some()
                {
                    edits.push(
                        TextEdit::new(
                            text,
                            token.unit_start,
                            token.unit_start,
                            " ",
                            EditPriority::SemanticAtomic,
                        )
                        .expect("semantic unit edit must use valid UTF-8 boundaries"),
                    );
                }
                break;
            }
        }

        if kind == SpanKind::MathExpression {
            if let Some((before_start, before_end, before)) = previous_char_range(text, start) {
                if classify(before) == CharKind::Cjk {
                    edits.push(
                        TextEdit::new(
                            text,
                            before_start,
                            before_end,
                            format!("{} ", before),
                            EditPriority::SemanticAtomic,
                        )
                        .expect("math boundary edit must use valid UTF-8 boundaries"),
                    );
                }
            }
            if let Some((after_start, after_end, after)) = next_char_range(text, end) {
                if classify(after) == CharKind::Cjk {
                    edits.push(
                        TextEdit::new(
                            text,
                            after_start,
                            after_end,
                            format!(" {}", after),
                            EditPriority::SemanticAtomic,
                        )
                        .expect("math boundary edit must use valid UTF-8 boundaries"),
                    );
                }
            }
        }
    }
    arbitrate_edits(edits)
}

fn previous_char_range(text: &str, index: usize) -> Option<(usize, usize, char)> {
    text[..index]
        .char_indices()
        .next_back()
        .map(|(start, ch)| (start, index, ch))
}

fn next_char_range(text: &str, index: usize) -> Option<(usize, usize, char)> {
    let ch = text[index..].chars().next()?;
    Some((index, index + ch.len_utf8(), ch))
}

#[cfg(test)]
mod tests {
    use super::{
        apply_edits, arbitrate_edits, plan_semantic_boundary_edits, EditPriority, TextEdit,
    };

    fn edit(
        text: &str,
        start: usize,
        end: usize,
        replacement: &str,
        priority: EditPriority,
    ) -> TextEdit {
        TextEdit::new(text, start, end, replacement, priority).unwrap()
    }

    #[test]
    fn rejects_invalid_and_non_char_boundary_ranges() {
        assert!(TextEdit::new("中文", 0, 1, "x", EditPriority::Editable).is_err());
        assert!(TextEdit::new("abc", 3, 2, "x", EditPriority::Editable).is_err());
        assert!(TextEdit::new("abc", 0, 4, "x", EditPriority::Editable).is_err());
    }

    #[test]
    fn opaque_edit_wins_over_inner_semantic_edit() {
        let text = "代码10μm继续";
        let edits = arbitrate_edits(vec![
            edit(text, 6, 10, "10 μm", EditPriority::SemanticAtomic),
            edit(
                text,
                0,
                text.len(),
                "代码10μm继续",
                EditPriority::OpaqueStructure,
            ),
        ]);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].priority, EditPriority::OpaqueStructure);
    }

    #[test]
    fn longer_same_priority_edit_wins() {
        let text = "abcdef";
        let edits = arbitrate_edits(vec![
            edit(text, 0, 3, "left", EditPriority::SemanticAtomic),
            edit(text, 0, 5, "long", EditPriority::SemanticAtomic),
        ]);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].replacement, "long");
    }

    #[test]
    fn non_overlapping_edits_apply_from_right_to_left() {
        let text = "中文abc数字";
        let edits = vec![
            edit(text, 0, 6, "文本", EditPriority::Editable),
            edit(text, 9, 15, "数字", EditPriority::Editable),
        ];
        assert_eq!(apply_edits(text, &edits).unwrap(), "文本abc数字");
    }

    #[test]
    fn overlapping_edits_are_rejected_before_application() {
        let text = "abcdef";
        let edits = vec![
            edit(text, 0, 3, "x", EditPriority::Editable),
            edit(text, 2, 5, "y", EditPriority::Editable),
        ];
        assert!(apply_edits(text, &edits).is_err());
    }

    #[test]
    fn semantic_plan_formats_units_and_math_boundaries_without_touching_code() {
        let text = "样品10μm且计算∂f/∂x很重要，代码`10μm $x$`继续";
        let edits = plan_semantic_boundary_edits(text);
        let output = apply_edits(text, &edits).unwrap();
        assert_eq!(output, "样品10 μm且计算 ∂f/∂x 很重要，代码`10μm $x$`继续");
    }

    #[test]
    fn semantic_plan_keeps_temperature_number_boundary() {
        let text = "样品25°C保存，4℃冷藏";
        let output = apply_edits(text, &plan_semantic_boundary_edits(text)).unwrap();
        assert_eq!(output, "样品25°C保存，4℃冷藏");
    }
}
