use std::ops::{Range, RangeBounds};

use ropey::Rope;

use super::RopeExt;

/// Represents a text selection with start and end byte offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
	/// Start byte offset (inclusive).
	pub start: usize,
	/// End byte offset (exclusive).
	pub end:   usize,
}

impl Selection {
	/// Create a new selection.
	pub fn new(start: usize, end: usize) -> Self { Self { start, end } }

	/// Create a cursor (empty selection) at the given offset.
	pub fn cursor(offset: usize) -> Self { Self { start: offset, end: offset } }

	/// Get the length of the selection.
	pub fn len(&self) -> usize {
		if self.start <= self.end { self.end - self.start } else { self.start - self.end }
	}

	/// Check if the selection is empty (cursor).
	pub fn is_empty(&self) -> bool { self.start == self.end }

	/// Check if the selection contains the given offset.
	pub fn contains(&self, offset: usize) -> bool {
		let (min, max) = self.ordered();
		offset >= min && offset < max
	}

	/// Get ordered (min, max) offsets.
	pub fn ordered(&self) -> (usize, usize) {
		if self.start <= self.end { (self.start, self.end) } else { (self.end, self.start) }
	}

	/// Get the selection as a Range.
	pub fn range(&self) -> Range<usize> {
		let (min, max) = self.ordered();
		min..max
	}

	/// Get the cursor position (end of selection).
	pub fn head(&self) -> usize { self.end }

	/// Check if selection is reversed (start > end).
	pub fn is_reversed(&self) -> bool { self.start > self.end }

	/// Normalize the selection so start <= end.
	pub fn normalized(&self) -> Self {
		let (min, max) = self.ordered();
		Self { start: min, end: max }
	}
}

impl From<Range<usize>> for Selection {
	fn from(range: Range<usize>) -> Self { Self { start: range.start, end: range.end } }
}

impl RangeBounds<usize> for Selection {
	fn start_bound(&self) -> std::ops::Bound<&usize> {
		let (min, _) = self.ordered();
		if self.start <= self.end {
			std::ops::Bound::Included(&self.start)
		} else {
			std::ops::Bound::Included(&self.end)
		}
	}

	fn end_bound(&self) -> std::ops::Bound<&usize> {
		if self.start <= self.end {
			std::ops::Bound::Excluded(&self.end)
		} else {
			std::ops::Bound::Excluded(&self.start)
		}
	}
}

/// Character type for word selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharType {
	Word,
	Whitespace,
	Newline,
	Other,
}

impl CharType {
	/// Classify a character.
	pub fn of(c: char) -> Self {
		if c == '\n' || c == '\r' {
			CharType::Newline
		} else if c.is_whitespace() {
			CharType::Whitespace
		} else if c.is_alphanumeric() || c == '_' {
			CharType::Word
		} else {
			CharType::Other
		}
	}
}

/// Select word at the given offset (for double-click).
pub fn select_word(text: &Rope, offset: usize) -> Selection {
	if text.len_bytes() == 0 {
		return Selection::cursor(0);
	}

	let offset = offset.min(text.len_bytes().saturating_sub(1));

	// Get the character at offset
	let char_at_offset = text.char_at(offset);
	let char_type = char_at_offset.map(CharType::of).unwrap_or(CharType::Other);

	// Find word boundaries
	let mut start = offset;
	let mut end = offset;

	// Expand backwards
	while start > 0 {
		let prev_offset = text.prev_grapheme_boundary(start);
		if let Some(c) = text.char_at(prev_offset) {
			if CharType::of(c) == char_type {
				start = prev_offset;
			} else {
				break;
			}
		} else {
			break;
		}
	}

	// Expand forwards
	while end < text.len_bytes() {
		if let Some(c) = text.char_at(end) {
			if CharType::of(c) == char_type {
				end = text.next_grapheme_boundary(end);
			} else {
				break;
			}
		} else {
			break;
		}
	}

	Selection::new(start, end)
}

/// Get word range at the given offset.
pub fn word_range(text: &Rope, offset: usize) -> Range<usize> {
	let selection = select_word(text, offset);
	selection.range()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_selection() {
		let sel = Selection::new(5, 10);
		assert_eq!(sel.len(), 5);
		assert!(!sel.is_empty());
		assert!(sel.contains(7));
		assert!(!sel.contains(10));
	}

	#[test]
	fn test_reversed_selection() {
		let sel = Selection::new(10, 5);
		assert_eq!(sel.len(), 5);
		assert!(sel.is_reversed());
		assert_eq!(sel.ordered(), (5, 10));
	}

	#[test]
	fn test_select_word() {
		let rope = Rope::from("hello world");
		let sel = select_word(&rope, 2);
		assert_eq!(sel.range(), 0..5);
	}

	#[test]
	fn test_char_type() {
		assert_eq!(CharType::of('a'), CharType::Word);
		assert_eq!(CharType::of('_'), CharType::Word);
		assert_eq!(CharType::of(' '), CharType::Whitespace);
		assert_eq!(CharType::of('\n'), CharType::Newline);
		assert_eq!(CharType::of('.'), CharType::Other);
	}
}
