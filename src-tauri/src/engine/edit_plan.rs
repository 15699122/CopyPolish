//! TextEdit 计划与冲突仲裁。
//!
//! 本模块是 Span → TextEdit 迁移基础设施，当前不接管生产 pipeline。编辑使用
//! 原文 UTF-8 字节区间，先校验边界，再按优先级仲裁，最后从后向前应用。

use super::spans::SpanPriority;

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
        if start >= end {
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

    fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
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
        if accepted.iter().all(|edit| !edit.overlaps(&candidate)) {
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
            || edit.start >= edit.end
            || !text.is_char_boundary(edit.start)
            || !text.is_char_boundary(edit.end)
        {
            return Err(format!("invalid edit range: {}..{}", edit.start, edit.end));
        }
    }
    if ordered.windows(2).any(|pair| pair[0].overlaps(&pair[1])) {
        return Err("overlapping edits must be arbitrated before application".to_string());
    }

    let mut output = text.to_string();
    for edit in ordered.into_iter().rev() {
        output.replace_range(edit.start..edit.end, &edit.replacement);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{apply_edits, arbitrate_edits, EditPriority, TextEdit};

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
        assert!(TextEdit::new("abc", 2, 2, "x", EditPriority::Editable).is_err());
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
}
