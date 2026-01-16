use std::ops::Range;

use ropey::{LineType, Rope, RopeSlice};
use unicode_segmentation::UnicodeSegmentation;

/// Check if a character is a word character (letter, digit, or underscore).
fn is_word_char(c: char) -> bool { c.is_alphanumeric() || c == '_' }

/// Line type to use for line operations.
const LT: LineType = LineType::LF;

/// A point in text representing row and column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextPoint {
	pub row:    usize,
	pub column: usize,
}

impl TextPoint {
	pub fn new(row: usize, column: usize) -> Self { Self { row, column } }
}

/// Extension methods for Rope text operations.
pub trait RopeExt {
	/// Get the byte length of the text.
	fn len_bytes(&self) -> usize;

	/// Get the start byte offset of the given row (0-based).
	fn line_start_offset(&self, row: usize) -> usize;

	/// Get the end byte offset of the given row (0-based), excluding the `\n`.
	fn line_end_offset(&self, row: usize) -> usize;

	/// Get line by row, returns a RopeSlice without the ending `\n`.
	fn slice_line(&self, row: usize) -> RopeSlice;

	/// Get multiple lines by row range (not including end).
	fn slice_lines(&self, range: Range<usize>) -> RopeSlice;

	/// Get the total number of lines.
	fn lines_len(&self) -> usize;

	/// Iterator over lines.
	fn iter_lines(&self) -> RopeLines<'_>;

	/// Convert byte offset to (row, column) point.
	fn offset_to_point(&self, offset: usize) -> TextPoint;

	/// Convert (row, column) point to byte offset.
	fn point_to_offset(&self, point: TextPoint) -> usize;

	/// Convert char index to byte offset.
	fn char_index_to_offset(&self, char_index: usize) -> usize;

	/// Convert byte offset to char index.
	fn offset_to_char_index(&self, offset: usize) -> usize;

	/// Get the character at the given byte offset.
	fn char_at(&self, offset: usize) -> Option<char>;

	/// Clip offset to valid range with bias.
	fn clip_offset(&self, offset: usize, bias: Bias) -> usize;

	/// Find the next grapheme cluster boundary after the given offset.
	fn next_grapheme_boundary(&self, offset: usize) -> usize;

	/// Find the previous grapheme cluster boundary before the given offset.
	fn prev_grapheme_boundary(&self, offset: usize) -> usize;

	/// Find the next word boundary after the given offset.
	fn next_word_boundary(&self, offset: usize) -> usize;

	/// Find the previous word boundary before the given offset.
	fn prev_word_boundary(&self, offset: usize) -> usize;

	/// Replace text in the given range with new text.
	fn replace(&mut self, range: Range<usize>, text: &str);
}

/// Bias for clipping offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bias {
	Left,
	Right,
}

impl RopeExt for Rope {
	fn len_bytes(&self) -> usize { self.len() }

	fn line_start_offset(&self, row: usize) -> usize {
		let total_lines = self.len_lines(LT);
		if row >= total_lines {
			return self.len();
		}
		self.line_to_byte_idx(row, LT)
	}

	fn line_end_offset(&self, row: usize) -> usize {
		let total_lines = self.len_lines(LT);
		if row >= total_lines {
			return self.len();
		}

		let start = self.line_to_byte_idx(row, LT);
		let line = self.line(row, LT);
		let line_len = line.len();

		// Check if line ends with \n or \r\n
		let end = start + line_len;
		if line_len > 0 {
			let last_char = line.char(line.len_chars().saturating_sub(1));
			if last_char == '\n' {
				return end - 1;
			}
		}
		end
	}

	fn slice_line(&self, row: usize) -> RopeSlice {
		let total_lines = self.len_lines(LT);
		if row >= total_lines {
			return self.slice(self.len()..self.len());
		}

		let start = self.line_start_offset(row);
		let end = self.line_end_offset(row);
		self.slice(start..end)
	}

	fn slice_lines(&self, range: Range<usize>) -> RopeSlice {
		let start = self.line_start_offset(range.start);
		let end = if range.end > 0 { self.line_end_offset(range.end.saturating_sub(1)) } else { start };
		self.slice(start..end.min(self.len()))
	}

	fn lines_len(&self) -> usize { self.len_lines(LT) }

	fn iter_lines(&self) -> RopeLines<'_> { RopeLines { rope: self, current_row: 0 } }

	fn offset_to_point(&self, offset: usize) -> TextPoint {
		let offset = offset.min(self.len());
		let row = self.byte_to_line_idx(offset, LT);
		let line_start = self.line_to_byte_idx(row, LT);
		let column = offset - line_start;
		TextPoint::new(row, column)
	}

	fn point_to_offset(&self, point: TextPoint) -> usize {
		let total_lines = self.len_lines(LT);
		let row = point.row.min(total_lines.saturating_sub(1));
		let line_start = self.line_to_byte_idx(row, LT);
		let line = self.line(row, LT);
		let column = point.column.min(line.len());
		line_start + column
	}

	fn char_index_to_offset(&self, char_index: usize) -> usize {
		self.char_to_byte_idx(char_index.min(self.len_chars()))
	}

	fn offset_to_char_index(&self, offset: usize) -> usize {
		self.byte_to_char_idx(offset.min(self.len()))
	}

	fn char_at(&self, offset: usize) -> Option<char> {
		if offset >= self.len() {
			return None;
		}
		let char_idx = self.byte_to_char_idx(offset);
		Some(self.char(char_idx))
	}

	fn clip_offset(&self, offset: usize, bias: Bias) -> usize {
		let offset = offset.min(self.len());
		if offset == 0 || offset == self.len() {
			return offset;
		}

		// Check if we're at a valid char boundary
		if self.is_char_boundary(offset) {
			return offset;
		}

		// Not at char boundary, adjust based on bias
		match bias {
			Bias::Left => self.floor_char_boundary(offset),
			Bias::Right => self.ceil_char_boundary(offset),
		}
	}

	fn next_grapheme_boundary(&self, offset: usize) -> usize {
		if offset >= self.len() {
			return self.len();
		}

		let text = self.to_string();
		let mut grapheme_offset = 0;

		for grapheme in text.graphemes(true) {
			let grapheme_end = grapheme_offset + grapheme.len();
			if grapheme_offset >= offset {
				return grapheme_end.min(self.len());
			}
			grapheme_offset = grapheme_end;
		}

		self.len()
	}

	fn prev_grapheme_boundary(&self, offset: usize) -> usize {
		if offset == 0 {
			return 0;
		}

		let text = self.to_string();
		let mut grapheme_offset = 0;
		let mut prev_offset = 0;

		for grapheme in text.graphemes(true) {
			if grapheme_offset >= offset {
				return prev_offset;
			}
			prev_offset = grapheme_offset;
			grapheme_offset += grapheme.len();
		}

		prev_offset
	}

	fn next_word_boundary(&self, offset: usize) -> usize {
		if offset >= self.len() {
			return self.len();
		}

		let text = self.to_string();
		let chars: Vec<char> = text.chars().collect();
		let mut pos = self.offset_to_char_index(offset);

		// Find word boundary based on word characters
		while pos < chars.len() {
			let c = chars[pos];
			if !is_word_char(c) {
				break;
			}
			pos += 1;
		}

		// Skip non-word characters
		while pos < chars.len() && !is_word_char(chars[pos]) {
			pos += 1;
		}

		self.char_index_to_offset(pos)
	}

	fn prev_word_boundary(&self, offset: usize) -> usize {
		if offset == 0 {
			return 0;
		}

		let text = self.to_string();
		let chars: Vec<char> = text.chars().collect();
		let mut pos = self.offset_to_char_index(offset);

		// Skip non-word characters
		while pos > 0 && !is_word_char(chars[pos - 1]) {
			pos -= 1;
		}

		// Find word boundary
		while pos > 0 && is_word_char(chars[pos - 1]) {
			pos -= 1;
		}

		self.char_index_to_offset(pos)
	}

	fn replace(&mut self, range: Range<usize>, text: &str) {
		let start = range.start.min(self.len());
		let end = range.end.min(self.len());

		if start < end {
			self.remove(start..end);
		}
		if !text.is_empty() {
			self.insert(start, text);
		}
	}
}

/// Iterator over lines in a Rope.
pub struct RopeLines<'a> {
	rope:        &'a Rope,
	current_row: usize,
}

impl<'a> Iterator for RopeLines<'a> {
	type Item = RopeSlice<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current_row >= self.rope.lines_len() {
			return None;
		}

		let line = self.rope.slice_line(self.current_row);
		self.current_row += 1;
		Some(line)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_line_offsets() {
		let rope = Rope::from_str("Hello\nWorld\n");

		assert_eq!(rope.line_start_offset(0), 0);
		assert_eq!(rope.line_end_offset(0), 5);

		assert_eq!(rope.line_start_offset(1), 6);
		assert_eq!(rope.line_end_offset(1), 11);
	}

	#[test]
	fn test_offset_to_point() {
		let rope = Rope::from_str("Hello\nWorld");

		assert_eq!(rope.offset_to_point(0), TextPoint::new(0, 0));
		assert_eq!(rope.offset_to_point(5), TextPoint::new(0, 5));
		assert_eq!(rope.offset_to_point(6), TextPoint::new(1, 0));
		assert_eq!(rope.offset_to_point(11), TextPoint::new(1, 5));
	}

	#[test]
	fn test_chinese_characters() {
		let rope = Rope::from_str("你好世界");

		assert_eq!(rope.len_bytes(), 12); // 4 chars * 3 bytes
		assert_eq!(rope.offset_to_char_index(0), 0);
		assert_eq!(rope.offset_to_char_index(3), 1);
		assert_eq!(rope.char_index_to_offset(1), 3);
	}

	#[test]
	fn test_grapheme_boundaries() {
		let rope = Rope::from_str("Hello");

		assert_eq!(rope.next_grapheme_boundary(0), 1);
		assert_eq!(rope.next_grapheme_boundary(4), 5);
		assert_eq!(rope.prev_grapheme_boundary(5), 4);
		assert_eq!(rope.prev_grapheme_boundary(1), 0);
	}
}
