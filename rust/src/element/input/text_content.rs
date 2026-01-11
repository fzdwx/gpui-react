//! Rope-based text content with efficient O(log n) operations
//!
//! This module provides a text storage implementation using the ropey crate
//! for efficient text operations on large documents.

use ropey::{Rope, RopeSlice};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Point in text (row, column) - byte offsets within line
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
	pub row: usize,
	pub column: usize,
}

impl Point {
	pub fn new(row: usize, column: usize) -> Self {
		Self { row, column }
	}
}

/// Rope-based text content with efficient operations
#[derive(Clone, Debug)]
pub struct TextContent {
	/// The text stored as a rope
	rope: Rope,
}

impl Default for TextContent {
	fn default() -> Self {
		Self::new()
	}
}

impl TextContent {
	/// Create empty text content
	pub fn new() -> Self {
		Self { rope: Rope::new() }
	}

	/// Create from string
	pub fn from_str(s: &str) -> Self {
		Self { rope: Rope::from_str(s) }
	}

	/// Get the underlying rope (for advanced operations)
	pub fn rope(&self) -> &Rope {
		&self.rope
	}

	/// Get length in bytes
	pub fn len(&self) -> usize {
		self.rope.len_bytes()
	}

	/// Check if empty
	pub fn is_empty(&self) -> bool {
		self.rope.len_bytes() == 0
	}

	/// Get number of lines (including empty trailing line after final newline)
	pub fn line_count(&self) -> usize {
		self.rope.len_lines()
	}

	/// Get a line by index (0-based)
	pub fn line(&self, line_idx: usize) -> Option<RopeSlice> {
		if line_idx >= self.line_count() {
			None
		} else {
			Some(self.rope.line(line_idx))
		}
	}

	/// Get a slice of the text by byte range
	pub fn slice(&self, range: Range<usize>) -> RopeSlice {
		let start = range.start.min(self.len());
		let end = range.end.min(self.len());
		self.rope.byte_slice(start..end)
	}

	/// Insert text at byte offset - O(log n)
	pub fn insert(&mut self, byte_offset: usize, text: &str) {
		let offset = byte_offset.min(self.len());
		let char_idx = self.rope.byte_to_char(offset);
		self.rope.insert(char_idx, text);
	}

	/// Remove text in byte range - O(log n)
	pub fn remove(&mut self, range: Range<usize>) {
		let start = range.start.min(self.len());
		let end = range.end.min(self.len());
		if start < end {
			let start_char = self.rope.byte_to_char(start);
			let end_char = self.rope.byte_to_char(end);
			self.rope.remove(start_char..end_char);
		}
	}

	/// Replace text in byte range - O(log n)
	pub fn replace(&mut self, range: Range<usize>, text: &str) {
		self.remove(range.clone());
		self.insert(range.start, text);
	}

	/// Convert byte offset to Point (row, column)
	pub fn offset_to_point(&self, offset: usize) -> Point {
		let offset = offset.min(self.len());
		let line = self.rope.byte_to_line(offset);
		let line_start = self.rope.line_to_byte(line);
		Point { row: line, column: offset - line_start }
	}

	/// Convert Point (row, column) to byte offset
	pub fn point_to_offset(&self, point: Point) -> usize {
		if point.row >= self.line_count() {
			return self.len();
		}
		let line_start = self.rope.line_to_byte(point.row);
		let line_len = self.line_len(point.row);
		line_start + point.column.min(line_len)
	}

	/// Get the starting byte offset of a line
	pub fn line_start_offset(&self, line_idx: usize) -> usize {
		if line_idx >= self.line_count() {
			self.len()
		} else {
			self.rope.line_to_byte(line_idx)
		}
	}

	/// Get the ending byte offset of a line (before the newline)
	pub fn line_end_offset(&self, line_idx: usize) -> usize {
		if line_idx >= self.line_count() {
			return self.len();
		}
		let next_line_start = if line_idx + 1 >= self.line_count() {
			self.len()
		} else {
			self.rope.line_to_byte(line_idx + 1)
		};
		// Subtract newline if present
		if next_line_start > 0 {
			let prev_char_idx = self.rope.byte_to_char(next_line_start) - 1;
			let prev_char = self.rope.char(prev_char_idx);
			if prev_char == '\n' {
				return next_line_start - 1;
			}
		}
		next_line_start
	}

	/// Get line length in bytes (excluding newline)
	pub fn line_len(&self, line_idx: usize) -> usize {
		if line_idx >= self.line_count() {
			0
		} else {
			self.line_end_offset(line_idx) - self.line_start_offset(line_idx)
		}
	}

	/// Clip offset to valid byte boundary
	pub fn clip_offset(&self, offset: usize) -> usize {
		let offset = offset.min(self.len());
		// Ensure we're at a char boundary
		if offset == 0 || offset == self.len() {
			return offset;
		}
		// Find the char index and convert back to get valid byte offset
		let char_idx = self.rope.byte_to_char(offset);
		self.rope.char_to_byte(char_idx)
	}

	/// Find previous grapheme boundary
	pub fn prev_grapheme_boundary(&self, offset: usize) -> usize {
		if offset == 0 {
			return 0;
		}

		let offset = offset.min(self.len());

		// Get the line containing this offset
		let line_idx = self.rope.byte_to_line(offset);
		let line_start = self.rope.line_to_byte(line_idx);

		// If at line start and not first line, go to end of previous line
		if offset == line_start && line_idx > 0 {
			return self.line_end_offset(line_idx - 1);
		}

		// Search within current line for grapheme boundary
		let line = self.rope.line(line_idx);
		let line_str = line.to_string();
		let local_offset = offset - line_start;

		let mut prev_boundary = 0;
		for (idx, _) in line_str.grapheme_indices(true) {
			if idx >= local_offset {
				break;
			}
			prev_boundary = idx;
		}

		line_start + prev_boundary
	}

	/// Find next grapheme boundary
	pub fn next_grapheme_boundary(&self, offset: usize) -> usize {
		if offset >= self.len() {
			return self.len();
		}

		let line_idx = self.rope.byte_to_line(offset);
		let line_start = self.rope.line_to_byte(line_idx);
		let line_end = self.line_end_offset(line_idx);

		// If at line end, move to start of next line
		if offset >= line_end {
			if line_idx + 1 < self.line_count() {
				return self.line_start_offset(line_idx + 1);
			}
			return self.len();
		}

		// Search within current line for next grapheme boundary
		let line = self.rope.line(line_idx);
		let line_str = line.to_string();
		let local_offset = offset - line_start;

		for (idx, grapheme) in line_str.grapheme_indices(true) {
			if idx >= local_offset {
				return line_start + idx + grapheme.len();
			}
		}

		line_end
	}

	/// Get char at byte offset
	pub fn char_at(&self, offset: usize) -> Option<char> {
		if offset >= self.len() {
			return None;
		}
		let char_idx = self.rope.byte_to_char(offset);
		Some(self.rope.char(char_idx))
	}

	/// Convert to String
	pub fn to_string(&self) -> String {
		self.rope.to_string()
	}

	/// Get number of grapheme clusters (visible characters)
	pub fn grapheme_count(&self) -> usize {
		self.rope.to_string().graphemes(true).count()
	}

	/// Convert UTF-8 byte offset to UTF-16 offset
	pub fn offset_to_utf16(&self, byte_offset: usize) -> usize {
		let byte_offset = byte_offset.min(self.len());
		let char_idx = self.rope.byte_to_char(byte_offset);
		// Count UTF-16 code units
		let mut utf16_offset = 0;
		for ch in self.rope.chars().take(char_idx) {
			utf16_offset += ch.len_utf16();
		}
		utf16_offset
	}

	/// Convert UTF-16 offset to UTF-8 byte offset
	pub fn offset_from_utf16(&self, utf16_offset: usize) -> usize {
		let mut utf16_count = 0;
		let mut byte_offset = 0;
		for ch in self.rope.chars() {
			if utf16_count >= utf16_offset {
				break;
			}
			utf16_count += ch.len_utf16();
			byte_offset += ch.len_utf8();
		}
		byte_offset
	}

	/// Check if a byte offset is on a char boundary
	pub fn is_char_boundary(&self, offset: usize) -> bool {
		if offset > self.len() {
			return false;
		}
		if offset == 0 || offset == self.len() {
			return true;
		}
		// Try to convert - if it works, it's valid
		let char_idx = self.rope.byte_to_char(offset);
		self.rope.char_to_byte(char_idx) == offset
	}

	/// Iterator over (byte_index, char) pairs - for password input positioning
	pub fn char_indices(&self) -> impl Iterator<Item = (usize, char)> + '_ {
		self.rope.chars().scan(0usize, |byte_offset, ch| {
			let current_offset = *byte_offset;
			*byte_offset += ch.len_utf8();
			Some((current_offset, ch))
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_basic_operations() {
		let mut content = TextContent::from_str("hello");
		assert_eq!(content.len(), 5);
		assert_eq!(content.to_string(), "hello");

		content.insert(5, " world");
		assert_eq!(content.to_string(), "hello world");

		content.remove(5..6);
		assert_eq!(content.to_string(), "helloworld");

		content.replace(5..10, " universe");
		assert_eq!(content.to_string(), "hello universe");
	}

	#[test]
	fn test_line_operations() {
		let content = TextContent::from_str("line1\nline2\nline3");
		assert_eq!(content.line_count(), 3);

		assert_eq!(content.line_start_offset(0), 0);
		assert_eq!(content.line_end_offset(0), 5);

		assert_eq!(content.line_start_offset(1), 6);
		assert_eq!(content.line_end_offset(1), 11);

		assert_eq!(content.line_start_offset(2), 12);
		assert_eq!(content.line_end_offset(2), 17);
	}

	#[test]
	fn test_offset_to_point() {
		let content = TextContent::from_str("hello\nworld");
		assert_eq!(content.offset_to_point(0), Point::new(0, 0));
		assert_eq!(content.offset_to_point(5), Point::new(0, 5));
		assert_eq!(content.offset_to_point(6), Point::new(1, 0));
		assert_eq!(content.offset_to_point(11), Point::new(1, 5));
	}

	#[test]
	fn test_point_to_offset() {
		let content = TextContent::from_str("hello\nworld");
		assert_eq!(content.point_to_offset(Point::new(0, 0)), 0);
		assert_eq!(content.point_to_offset(Point::new(0, 5)), 5);
		assert_eq!(content.point_to_offset(Point::new(1, 0)), 6);
		assert_eq!(content.point_to_offset(Point::new(1, 5)), 11);
	}

	#[test]
	fn test_grapheme_boundaries() {
		let content = TextContent::from_str("hello");
		assert_eq!(content.prev_grapheme_boundary(5), 4);
		assert_eq!(content.prev_grapheme_boundary(1), 0);
		assert_eq!(content.prev_grapheme_boundary(0), 0);

		assert_eq!(content.next_grapheme_boundary(0), 1);
		assert_eq!(content.next_grapheme_boundary(4), 5);
		assert_eq!(content.next_grapheme_boundary(5), 5);
	}

	#[test]
	fn test_unicode() {
		let content = TextContent::from_str("你好世界");
		assert_eq!(content.len(), 12); // 3 bytes per Chinese character
		assert_eq!(content.grapheme_count(), 4);

		assert_eq!(content.next_grapheme_boundary(0), 3);
		assert_eq!(content.next_grapheme_boundary(3), 6);
		assert_eq!(content.prev_grapheme_boundary(3), 0);
		assert_eq!(content.prev_grapheme_boundary(6), 3);
	}
}
