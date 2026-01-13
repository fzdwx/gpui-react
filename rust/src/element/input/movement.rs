use ropey::Rope;

use super::{RopeExt, Selection, TextPoint};

/// Direction for vertical movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

/// Move cursor left by one grapheme.
pub fn move_left(text: &Rope, selection: &Selection) -> Selection {
    if !selection.is_empty() {
        // Collapse to start
        let (min, _) = selection.ordered();
        Selection::cursor(min)
    } else {
        let new_pos = text.prev_grapheme_boundary(selection.end);
        Selection::cursor(new_pos)
    }
}

/// Move cursor right by one grapheme.
pub fn move_right(text: &Rope, selection: &Selection) -> Selection {
    if !selection.is_empty() {
        // Collapse to end
        let (_, max) = selection.ordered();
        Selection::cursor(max)
    } else {
        let new_pos = text.next_grapheme_boundary(selection.end);
        Selection::cursor(new_pos)
    }
}

/// Move cursor to the start of the line.
pub fn move_to_line_start(text: &Rope, selection: &Selection) -> Selection {
    let point = text.offset_to_point(selection.end);
    let line_start = text.line_start_offset(point.row);
    Selection::cursor(line_start)
}

/// Move cursor to the end of the line.
pub fn move_to_line_end(text: &Rope, selection: &Selection) -> Selection {
    let point = text.offset_to_point(selection.end);
    let line_end = text.line_end_offset(point.row);
    Selection::cursor(line_end)
}

/// Move cursor to the start of the text.
pub fn move_to_start(_text: &Rope, _selection: &Selection) -> Selection {
    Selection::cursor(0)
}

/// Move cursor to the end of the text.
pub fn move_to_end(text: &Rope, _selection: &Selection) -> Selection {
    Selection::cursor(text.len_bytes())
}

/// Move cursor up by one line.
pub fn move_up(text: &Rope, selection: &Selection, preferred_column: Option<usize>) -> Selection {
    let point = text.offset_to_point(selection.end);
    if point.row == 0 {
        return Selection::cursor(0);
    }

    let new_row = point.row - 1;
    let target_column = preferred_column.unwrap_or(point.column);
    let line = text.slice_line(new_row);
    let new_column = target_column.min(line.len());

    let new_offset = text.point_to_offset(TextPoint::new(new_row, new_column));
    Selection::cursor(new_offset)
}

/// Move cursor down by one line.
pub fn move_down(text: &Rope, selection: &Selection, preferred_column: Option<usize>) -> Selection {
    let point = text.offset_to_point(selection.end);
    let total_lines = text.lines_len();

    if point.row >= total_lines.saturating_sub(1) {
        return Selection::cursor(text.len_bytes());
    }

    let new_row = point.row + 1;
    let target_column = preferred_column.unwrap_or(point.column);
    let line = text.slice_line(new_row);
    let new_column = target_column.min(line.len());

    let new_offset = text.point_to_offset(TextPoint::new(new_row, new_column));
    Selection::cursor(new_offset)
}

/// Move cursor to the previous word boundary.
pub fn move_to_prev_word(text: &Rope, selection: &Selection) -> Selection {
    let mut offset = if selection.is_empty() {
        selection.end
    } else {
        selection.ordered().0
    };

    if offset == 0 {
        return Selection::cursor(0);
    }

    // Skip whitespace
    while offset > 0 {
        let prev = text.prev_grapheme_boundary(offset);
        if let Some(c) = text.char_at(prev) {
            if !c.is_whitespace() {
                break;
            }
            offset = prev;
        } else {
            break;
        }
    }

    // Skip word characters
    let char_at = text.char_at(text.prev_grapheme_boundary(offset));
    let is_word_char = char_at.map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);

    while offset > 0 {
        let prev = text.prev_grapheme_boundary(offset);
        if let Some(c) = text.char_at(prev) {
            let curr_is_word = c.is_alphanumeric() || c == '_';
            if curr_is_word != is_word_char {
                break;
            }
            offset = prev;
        } else {
            break;
        }
    }

    Selection::cursor(offset)
}

/// Move cursor to the next word boundary.
pub fn move_to_next_word(text: &Rope, selection: &Selection) -> Selection {
    let mut offset = if selection.is_empty() {
        selection.end
    } else {
        selection.ordered().1
    };

    let text_len = text.len_bytes();
    if offset >= text_len {
        return Selection::cursor(text_len);
    }

    // Skip current word characters
    let char_at = text.char_at(offset);
    let is_word_char = char_at.map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);

    while offset < text_len {
        if let Some(c) = text.char_at(offset) {
            let curr_is_word = c.is_alphanumeric() || c == '_';
            if curr_is_word != is_word_char {
                break;
            }
            offset = text.next_grapheme_boundary(offset);
        } else {
            break;
        }
    }

    // Skip whitespace
    while offset < text_len {
        if let Some(c) = text.char_at(offset) {
            if !c.is_whitespace() {
                break;
            }
            offset = text.next_grapheme_boundary(offset);
        } else {
            break;
        }
    }

    Selection::cursor(offset)
}

/// Extend selection left.
pub fn select_left(text: &Rope, selection: &Selection) -> Selection {
    let new_end = text.prev_grapheme_boundary(selection.end);
    Selection::new(selection.start, new_end)
}

/// Extend selection right.
pub fn select_right(text: &Rope, selection: &Selection) -> Selection {
    let new_end = text.next_grapheme_boundary(selection.end);
    Selection::new(selection.start, new_end)
}

/// Extend selection up.
pub fn select_up(text: &Rope, selection: &Selection, preferred_column: Option<usize>) -> Selection {
    let new_cursor = move_up(text, selection, preferred_column);
    Selection::new(selection.start, new_cursor.end)
}

/// Extend selection down.
pub fn select_down(
    text: &Rope,
    selection: &Selection,
    preferred_column: Option<usize>,
) -> Selection {
    let new_cursor = move_down(text, selection, preferred_column);
    Selection::new(selection.start, new_cursor.end)
}

/// Extend selection to line start.
pub fn select_to_line_start(text: &Rope, selection: &Selection) -> Selection {
    let new_cursor = move_to_line_start(text, selection);
    Selection::new(selection.start, new_cursor.end)
}

/// Extend selection to line end.
pub fn select_to_line_end(text: &Rope, selection: &Selection) -> Selection {
    let new_cursor = move_to_line_end(text, selection);
    Selection::new(selection.start, new_cursor.end)
}

/// Extend selection to previous word.
pub fn select_to_prev_word(text: &Rope, selection: &Selection) -> Selection {
    let target = move_to_prev_word(text, &Selection::cursor(selection.end));
    Selection::new(selection.start, target.end)
}

/// Extend selection to next word.
pub fn select_to_next_word(text: &Rope, selection: &Selection) -> Selection {
    let target = move_to_next_word(text, &Selection::cursor(selection.end));
    Selection::new(selection.start, target.end)
}

/// Extend selection to start of text.
pub fn select_to_start(_text: &Rope, selection: &Selection) -> Selection {
    Selection::new(selection.start, 0)
}

/// Extend selection to end of text.
pub fn select_to_end(text: &Rope, selection: &Selection) -> Selection {
    Selection::new(selection.start, text.len_bytes())
}

/// Select all text.
pub fn select_all(text: &Rope) -> Selection {
    Selection::new(0, text.len_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_left_right() {
        let rope = Rope::from("hello");
        let sel = Selection::cursor(3);

        let left = move_left(&rope, &sel);
        assert_eq!(left.end, 2);

        let right = move_right(&rope, &sel);
        assert_eq!(right.end, 4);
    }

    #[test]
    fn test_move_up_down() {
        let rope = Rope::from("line1\nline2\nline3");
        let sel = Selection::cursor(8); // In "line2"

        let up = move_up(&rope, &sel, None);
        assert_eq!(rope.offset_to_point(up.end).row, 0);

        let down = move_down(&rope, &sel, None);
        assert_eq!(rope.offset_to_point(down.end).row, 2);
    }

    #[test]
    fn test_word_movement() {
        let rope = Rope::from("hello world test");
        let sel = Selection::cursor(6); // Start of "world"

        let prev = move_to_prev_word(&rope, &sel);
        assert_eq!(prev.end, 0);

        let next = move_to_next_word(&rope, &sel);
        assert_eq!(next.end, 12);
    }

    #[test]
    fn test_select_all() {
        let rope = Rope::from("hello world");
        let sel = select_all(&rope);
        assert_eq!(sel.start, 0);
        assert_eq!(sel.end, 11);
    }
}
