use unicode_segmentation::UnicodeSegmentation;

/// 面向 grapheme cluster 的最小多行编辑器。
#[derive(Clone, Debug, Default)]
pub struct TextEditor {
    text: String,
    cursor: usize,
}

impl TextEditor {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            cursor: text.len(),
            text,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// 返回当前光标所在的零基行号和 grapheme 列号。
    pub fn line_column(&self) -> (usize, usize) {
        let start = self.line_start(self.cursor);
        (
            self.text[..start]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count(),
            self.text[start..self.cursor].graphemes(true).count(),
        )
    }

    pub fn insert(&mut self, value: &str) {
        self.text.insert_str(self.cursor, value);
        self.cursor += value.len();
    }

    pub fn backspace(&mut self) {
        let Some(start) = self.previous_boundary(self.cursor) else {
            return;
        };
        self.text.drain(start..self.cursor);
        self.cursor = start;
    }

    pub fn delete(&mut self) {
        let Some(end) = self.next_boundary(self.cursor) else {
            return;
        };
        self.text.drain(self.cursor..end);
    }

    pub fn move_left(&mut self) {
        if let Some(start) = self.previous_boundary(self.cursor) {
            self.cursor = start;
        }
    }

    pub fn move_right(&mut self) {
        if let Some(end) = self.next_boundary(self.cursor) {
            self.cursor = end;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = self.line_start(self.cursor);
    }

    pub fn move_end(&mut self) {
        self.cursor = self.line_end(self.cursor);
    }

    pub fn move_up(&mut self) {
        let (line_start, column) = self.line_position();
        if line_start == 0 {
            return;
        }
        let previous_end = line_start - 1;
        let previous_start = self.line_start(previous_end);
        self.cursor = self.offset_at_grapheme_column(previous_start, previous_end, column);
    }

    pub fn move_down(&mut self) {
        let current_end = self.line_end(self.cursor);
        if current_end == self.text.len() {
            return;
        }
        let next_start = current_end + 1;
        let next_end = self.line_end(next_start);
        let (_, column) = self.line_position();
        self.cursor = self.offset_at_grapheme_column(next_start, next_end, column);
    }

    fn previous_boundary(&self, offset: usize) -> Option<usize> {
        self.text[..offset]
            .grapheme_indices(true)
            .next_back()
            .map(|(start, _)| start)
    }

    fn next_boundary(&self, offset: usize) -> Option<usize> {
        self.text[offset..]
            .grapheme_indices(true)
            .nth(1)
            .map(|(index, _)| offset + index)
    }

    fn line_start(&self, offset: usize) -> usize {
        self.text[..offset].rfind('\n').map_or(0, |index| index + 1)
    }

    fn line_end(&self, offset: usize) -> usize {
        self.text[offset..]
            .find('\n')
            .map_or(self.text.len(), |index| offset + index)
    }

    fn line_position(&self) -> (usize, usize) {
        let start = self.line_start(self.cursor);
        (start, self.text[start..self.cursor].graphemes(true).count())
    }

    fn offset_at_grapheme_column(&self, start: usize, end: usize, column: usize) -> usize {
        self.text[start..end]
            .grapheme_indices(true)
            .nth(column)
            .map_or(end, |(offset, _)| start + offset)
    }
}

#[cfg(test)]
mod tests {
    use super::TextEditor;

    #[test]
    fn deletes_a_whole_emoji_grapheme() {
        let mut editor = TextEditor::new("a👨‍👩‍👧‍👦b");
        editor.move_left();
        editor.backspace();
        assert_eq!(editor.text(), "ab");
    }

    #[test]
    fn moves_by_grapheme_not_byte() {
        let mut editor = TextEditor::new("中文");
        editor.move_left();
        assert_eq!(&editor.text()[..editor.cursor()], "中");
        editor.move_left();
        assert_eq!(editor.cursor(), 0);
    }

    #[test]
    fn moves_between_lines() {
        let mut editor = TextEditor::new("ab\n一二三");
        editor.move_home();
        editor.move_up();
        assert_eq!(editor.cursor(), 0);
        editor.move_down();
        assert_eq!(editor.cursor(), 3);
        assert_eq!(editor.line_column(), (1, 0));
    }

    #[test]
    fn reports_grapheme_column_for_combining_text() {
        let mut editor = TextEditor::new("e\u{301}中文");
        editor.move_end();
        assert_eq!(editor.line_column(), (0, 3));
    }
}
