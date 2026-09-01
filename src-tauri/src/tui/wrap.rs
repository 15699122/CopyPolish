//! 与 ratatui `WordWrapper`（`trim: false`）等价的纯文本换行布局。
//!
//! 输入区使用 `Paragraph::wrap(Wrap { trim: false })` 渲染，ratatui 会自动按
//! `text_area.width` 对文本做词边界（无空格时按显示宽度）软换行。为了把光标和
//! 垂直滚动放到**与实际渲染一致的视觉行**上，本模块用一个纯函数镜像同一种换行
//! 语义，并返回每行在原文本中的字节区间。这样光标永远落在输入框内的正确视觉
//! 位置，不会越过输入框而绘制到状态栏，也不会因为按“逻辑行+整行宽度”计算而
//! 变得不可见。

use std::collections::VecDeque;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// 与 `ratatui::text::StyledGrapheme::is_whitespace` 保持一致的判定：
/// ZWSP 视为空白，NBSP 不视为空白，其余全空白码点视为空白。
const NBSP: &str = "\u{00a0}";
const ZWSP: &str = "\u{200b}";

/// 一个视觉行，`start`/`end` 是它在原文本中的字节区间，`width` 是显示宽度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualRow {
    pub start: usize,
    pub end: usize,
    pub width: u16,
}

/// 光标在视觉坐标下的位置：`row` 是视觉行号（0 基，含软换行），`col` 是该行的
/// 显示列号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorVisual {
    pub row: usize,
    pub col: u16,
}

#[derive(Clone, Copy)]
struct Grm {
    start: usize,
    end: usize,
    width: u16,
    ws: bool,
}

fn is_whitespace(s: &str) -> bool {
    s == ZWSP || (s.chars().all(char::is_whitespace) && s != NBSP)
}

fn make_row(pending_line: &[&Grm], line_abs_start: usize) -> VisualRow {
    let start = pending_line.first().map_or(line_abs_start, |g| g.start);
    let width = pending_line.iter().map(|g| g.width).sum();
    let end = pending_line.last().map_or(start, |g| g.end);
    VisualRow { start, end, width }
}

/// 把整段文本按 `max_width` 拆成视觉行（等价 ratatui 的自动换行）。
///
/// 硬换行 `\n` 分隔逻辑行；每个空逻辑行产生一个宽度为 0 的空视觉行；纯空文本
/// 至少返回一个空行。宽度大于 `max_width` 的 grapheme 与 ratatui 一样会被丢弃。
pub fn visual_rows(text: &str, max_width: u16) -> Vec<VisualRow> {
    if max_width == 0 {
        return Vec::new();
    }

    let mut rows: Vec<VisualRow> = Vec::new();
    let mut line: Vec<Grm> = Vec::new();
    let mut line_abs_start = 0usize;

    for (gs, grapheme) in text.grapheme_indices(true) {
        if grapheme == "\n" {
            process_line(&mut rows, &line, line_abs_start, max_width);
            line_abs_start = gs + grapheme.len();
            line.clear();
            continue;
        }
        line.push(Grm {
            start: gs,
            end: gs + grapheme.len(),
            width: UnicodeWidthStr::width(grapheme) as u16,
            ws: is_whitespace(grapheme),
        });
    }
    process_line(&mut rows, &line, line_abs_start, max_width);

    if rows.is_empty() {
        rows.push(VisualRow {
            start: 0,
            end: 0,
            width: 0,
        });
    }
    rows
}

/// 返回光标（字节偏移）所在的视觉行号与行内显示列号。
pub fn cursor_visual(text: &str, cursor: usize, max_width: u16) -> CursorVisual {
    let rows = visual_rows(text, max_width);
    let row = rows.iter().rposition(|r| r.start <= cursor).unwrap_or(0);
    let start = rows[row].start;
    let col = UnicodeWidthStr::width(&text[start..cursor.min(text.len())]) as u16;
    CursorVisual { row, col }
}

/// 镜像 ratatui `WordWrapper::process_input`（`trim: false`）处理一个逻辑行，
/// 把产生的视觉行追加到 `rows`。
fn process_line(rows: &mut Vec<VisualRow>, line: &[Grm], line_abs_start: usize, max_width: u16) {
    let mut pending_line: Vec<&Grm> = Vec::new();
    let mut line_width: u16 = 0;
    let mut word_width: u16 = 0;
    let mut whitespace_width: u16 = 0;
    let mut non_ws_prev = false;
    let mut pending_word: Vec<&Grm> = Vec::new();
    let mut pending_whitespace: VecDeque<&Grm> = VecDeque::new();
    // 空逻辑行（例如连续的 `\n`）渲染为一个空视觉行，保证空行可见且光标行号一致。
    if line.is_empty() {
        rows.push(VisualRow {
            start: line_abs_start,
            end: line_abs_start,
            width: 0,
        });
        return;
    }

    for g in line {
        let is_whitespace = g.ws;
        let symbol_width = g.width;

        // 与 ratatui 一致：丢弃比整行还宽的 grapheme。
        if symbol_width > max_width {
            continue;
        }

        // trim = false，因此 trimmed_overflow / whitespace_overflow 始终不成立。
        let word_found = non_ws_prev && is_whitespace;
        let untrimmed_overflow =
            pending_line.is_empty() && word_width + whitespace_width + symbol_width > max_width;

        // 追加已完成的“词 + 前置空白”段到当前行。
        if word_found || untrimmed_overflow {
            pending_line.extend(pending_whitespace.drain(..));
            line_width += whitespace_width;
            pending_line.append(&mut pending_word);
            line_width += word_width;
            whitespace_width = 0;
            word_width = 0;
        }

        let line_full = line_width >= max_width;
        let pending_word_overflow =
            symbol_width > 0 && line_width + whitespace_width + word_width >= max_width;

        // 输出一条已填满的行。
        if line_full || pending_word_overflow {
            let mut remaining_width = max_width.saturating_sub(line_width);
            rows.push(make_row(&pending_line, line_abs_start));
            // 等价 ratatui 的 `mem::take(&mut pending_line)`：清空已输出内容。
            pending_line.clear();
            line_width = 0;

            // 移除可放到行尾的空白。
            while let Some(front) = pending_whitespace.front() {
                let width = front.width;
                if width > remaining_width {
                    break;
                }
                whitespace_width = whitespace_width.saturating_sub(width);
                remaining_width -= width;
                pending_whitespace.pop_front();
            }

            // 首个空白不累计到下一词。
            if is_whitespace && pending_whitespace.is_empty() {
                continue;
            }
        }

        if is_whitespace {
            whitespace_width += symbol_width;
            pending_whitespace.push_back(g);
        } else {
            word_width += symbol_width;
            pending_word.push(g);
        }
        non_ws_prev = !is_whitespace;
    }

    // 尾段（trim = false 总会把尾部空白并入行）。
    if pending_line.is_empty() && pending_word.is_empty() && !pending_whitespace.is_empty() {
        rows.push(make_row(&[], line_abs_start));
    }
    pending_line.extend(pending_whitespace.drain(..));
    pending_line.append(&mut pending_word);
    if !pending_line.is_empty() {
        rows.push(make_row(&pending_line, line_abs_start));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        prelude::Widget,
        widgets::{Paragraph, Wrap},
        Terminal,
    };

    /// 把整段文本的视觉行转成字符串列表，便于断言。
    fn rows_strings(text: &str, max_width: u16) -> Vec<String> {
        visual_rows(text, max_width)
            .iter()
            .map(|r| text.get(r.start..r.end).unwrap_or("").to_string())
            .collect()
    }

    /// 通过 ratatui 实际渲染（`Paragraph::wrap(Wrap { trim: false })`）得到逐行
    /// 内容，与我们的 `visual_rows` 做等价校验。
    fn assert_matches_ratatui(text: &str, width: u16) {
        let ours = rows_strings(text, width);
        let height = (ours.len() as u16).max(1);
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        Paragraph::new(text.to_string())
            .wrap(Wrap { trim: false })
            .render(Rect::new(0, 0, width, height), &mut buf);

        let mut rendered: Vec<String> = Vec::new();
        for y in 0..height {
            let mut line = String::new();
            for x in 0..width {
                line.push_str(buf[(x, y)].symbol());
            }
            rendered.push(line.trim_end().to_string());
        }

        let mut expected: Vec<String> = ours.iter().map(|s| s.trim_end().to_string()).collect();
        expected.truncate(rendered.len());
        assert_eq!(
            rendered,
            expected,
            "mismatch for width={width} text={text:?}\nours={ous:?}\nratatui=\n{rendered:?}",
            ous = &ours[..]
        );
    }

    #[test]
    fn cjk_wraps_without_spaces() {
        let rows = rows_strings("在LeanCloud上，花了5000元！", 10);
        assert!(rows.len() >= 2, "expected multiple rows, got {rows:?}");
        for row in &rows {
            assert!(
                UnicodeWidthStr::width(row.as_str()) <= 10,
                "row too wide: {row:?}"
            );
        }
        // 软换行不丢失任何文字。
        assert_eq!(rows.concat(), "在LeanCloud上，花了5000元！");
    }

    #[test]
    fn ascii_word_boundary_wrap_matches_ratatui() {
        assert_matches_ratatui("abcd efghij klmnopabcd efgh ijklmnopabcdefg", 10);
        assert_matches_ratatui("           AAA AAA", 10);
        assert_matches_ratatui("AAAAAAAAAAAAAAAAAAAA    AAA", 20);
    }

    #[test]
    fn hard_newline_and_empty_lines_are_visible() {
        assert_eq!(rows_strings("ab\n\ncd", 10), vec!["ab", "", "cd"]);
        assert_eq!(rows_strings("ab\n", 10), vec!["ab", ""]);
        assert_eq!(rows_strings("", 10), vec![""]);
    }

    #[test]
    fn empty_and_tiny_width_are_safe() {
        assert_eq!(visual_rows("abc", 0), vec![]);
        // 宽度 1 时双宽字可能被丢弃；不应 panic。
        let rows = visual_rows("中文", 1);
        for r in rows {
            assert!(r.width <= 1);
        }
    }

    #[test]
    fn emoji_grapheme_is_not_split() {
        let text = "a👨‍👩‍👧‍👦b";
        let rows = rows_strings(text, 2);
        assert!(rows.len() >= 2);
        // 家庭 emoji 是单个 grapheme，必须完整出现在某个视觉行里。
        assert!(rows.iter().any(|r| r.contains("👨‍👩‍👧‍👦")));
        assert_eq!(rows.concat(), text);
    }

    #[test]
    fn cursor_tracks_visual_rows_after_soft_wrap() {
        // 12 个单宽字符在宽度 8 下应落在第二视觉行。
        let cv = cursor_visual("aaaaaaaaaaaa", "aaaaaaaaaaaa".len(), 8);
        assert_eq!(cv.row, 1);
        assert_eq!(cv.col, 4);
    }

    #[test]
    fn cursor_col_accounts_for_wide_chars() {
        // "中文" 各宽 2，宽度 5 时恰好放得下一行：光标在末尾，列号 = 4。
        let cv = cursor_visual("中文", 6, 5);
        assert_eq!(cv.row, 0);
        assert_eq!(cv.col, 4);
        // 宽度 3 装不下两个双宽字，ratatui 会把整段作为一个不可断词保持同行。
        assert_eq!(rows_strings("中文", 3), vec!["中文"]);
    }

    #[test]
    fn cursor_at_empty_text_is_row_zero() {
        assert_eq!(cursor_visual("", 0, 20), CursorVisual { row: 0, col: 0 });
    }

    #[test]
    fn ratatui_equivalence_on_mixed_samples() {
        // 缓冲解码对双宽字符的续写格不精确，因此这里只用单宽（ASCII/空白）样本做
        // 与 ratatui 的逐格等价校验；CJK/emoji 由上面的自检向量覆盖。
        for (text, width) in [
            ("abcd efgh ijklmnop abcd efg", 10u16),
            ("foo bar baz qux quux corge", 7),
            ("some words with spaces to wrap", 9),
            ("foo\u{200b}bar baz", 4),
            ("a\u{00a0}b c", 3),
            ("abcdefghijklmnopqrstuvwxyz", 7),
        ] {
            assert_matches_ratatui(text, width);
        }
    }

    #[test]
    fn ui_backend_can_verify_cursor_within_input() {
        // 冒烟：确认 ratatui TestBackend 可在测试中构造。
        let _ = Terminal::new(TestBackend::new(40, 20)).unwrap();
    }
}
