//! Selection and text selection utilities
//!
//! Provides selection types and word/line selection algorithms.

use std::ops::Range;

use super::text_content::TextContent;

/// Selection anchor and head positions
///
/// - `anchor`: The fixed end of the selection (where selection started)
/// - `head`: The moving end of the selection (current cursor position)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selection {
	/// Fixed end of selection (where selection started)
	pub anchor: usize,
	/// Moving end of selection (cursor position)
	pub head:   usize,
}

impl Selection {
	/// Create a new selection
	pub fn new(anchor: usize, head: usize) -> Self { Self { anchor, head } }

	/// Create a cursor (selection with no range)
	pub fn cursor(offset: usize) -> Self { Self { anchor: offset, head: offset } }

	/// Check if this is an empty selection (just a cursor)
	pub fn is_empty(&self) -> bool { self.anchor == self.head }

	/// Get the start of the selection (smaller offset)
	pub fn start(&self) -> usize { self.anchor.min(self.head) }

	/// Get the end of the selection (larger offset)
	pub fn end(&self) -> usize { self.anchor.max(self.head) }

	/// Get the selection as a Range
	pub fn range(&self) -> Range<usize> { self.start()..self.end() }

	/// Check if selection is reversed (head before anchor)
	pub fn is_reversed(&self) -> bool { self.head < self.anchor }

	/// Extend selection to a new head position
	pub fn extend_to(&mut self, head: usize) { self.head = head; }

	/// Collapse selection to the head position
	pub fn collapse_to_head(&mut self) { self.anchor = self.head; }

	/// Collapse selection to the start
	pub fn collapse_to_start(&mut self) {
		let start = self.start();
		self.anchor = start;
		self.head = start;
	}

	/// Collapse selection to the end
	pub fn collapse_to_end(&mut self) {
		let end = self.end();
		self.anchor = end;
		self.head = end;
	}

	/// Convert to legacy tuple format (for compatibility)
	pub fn as_tuple(&self) -> Option<(usize, usize)> {
		if self.is_empty() { None } else { Some((self.start(), self.end())) }
	}

	/// Create from legacy tuple format
	pub fn from_tuple(tuple: Option<(usize, usize)>, cursor_position: usize) -> Self {
		match tuple {
			Some((start, end)) => Self::new(start, end),
			None => Self::cursor(cursor_position),
		}
	}
}

/// Character classification for word selection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharType {
	/// Word characters: a-z, A-Z, 0-9, _
	Word,
	/// Whitespace: space, tab, etc. (not newlines)
	Whitespace,
	/// Newlines: \n, \r
	Newline,
	/// Punctuation and other characters
	Punctuation,
}

impl CharType {
	/// Classify a character
	pub fn from_char(c: char) -> Self {
		match c {
			'_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => CharType::Word,
			'\n' | '\r' => CharType::Newline,
			c if c.is_whitespace() => CharType::Whitespace,
			_ => CharType::Punctuation,
		}
	}

	/// Check if this char type should extend a word selection
	pub fn extends_word(&self, other: CharType) -> bool {
		match (self, other) {
			(CharType::Word, CharType::Word) => true,
			(CharType::Whitespace, CharType::Whitespace) => true,
			(CharType::Punctuation, CharType::Punctuation) => true,
			_ => false,
		}
	}
}

/// Text selection utilities
pub struct TextSelector;

impl TextSelector {
	/// Find word boundaries for double-click selection
	pub fn word_range(content: &TextContent, offset: usize) -> Option<Range<usize>> {
		if content.is_empty() {
			return None;
		}

		let offset = offset.min(content.len().saturating_sub(1));
		let char_at = content.char_at(offset)?;
		let char_type = CharType::from_char(char_at);

		// Don't select newlines as words
		if char_type == CharType::Newline {
			return None;
		}

		// Expand left while same char type
		let mut start = offset;
		loop {
			if start == 0 {
				break;
			}
			let prev = content.prev_grapheme_boundary(start);
			if let Some(c) = content.char_at(prev) {
				if !char_type.extends_word(CharType::from_char(c)) {
					break;
				}
				start = prev;
			} else {
				break;
			}
		}

		// Expand right while same char type
		let mut end = content.next_grapheme_boundary(offset);
		loop {
			if end >= content.len() {
				break;
			}
			if let Some(c) = content.char_at(end) {
				if !char_type.extends_word(CharType::from_char(c)) {
					break;
				}
				end = content.next_grapheme_boundary(end);
			} else {
				break;
			}
		}

		if start < end { Some(start..end) } else { None }
	}

	/// Find line range for triple-click selection
	pub fn line_range(content: &TextContent, offset: usize) -> Range<usize> {
		let point = content.offset_to_point(offset);
		let start = content.line_start_offset(point.row);
		let end = if point.row + 1 < content.line_count() {
			// Include the newline
			content.line_start_offset(point.row + 1)
		} else {
			content.len()
		};
		start..end
	}

	/// Classify character at offset
	pub fn char_type(content: &TextContent, offset: usize) -> Option<CharType> {
		content.char_at(offset).map(CharType::from_char)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_selection_basic() {
		let sel = Selection::new(5, 10);
		assert_eq!(sel.start(), 5);
		assert_eq!(sel.end(), 10);
		assert!(!sel.is_empty());
		assert!(!sel.is_reversed());

		let sel = Selection::new(10, 5);
		assert_eq!(sel.start(), 5);
		assert_eq!(sel.end(), 10);
		assert!(sel.is_reversed());

		let cursor = Selection::cursor(5);
		assert!(cursor.is_empty());
		assert_eq!(cursor.start(), 5);
		assert_eq!(cursor.end(), 5);
	}

	#[test]
	fn test_selection_extend() {
		let mut sel = Selection::cursor(5);
		sel.extend_to(10);
		assert_eq!(sel.anchor, 5);
		assert_eq!(sel.head, 10);
		assert!(!sel.is_empty());
	}

	#[test]
	fn test_char_type() {
		assert_eq!(CharType::from_char('a'), CharType::Word);
		assert_eq!(CharType::from_char('Z'), CharType::Word);
		assert_eq!(CharType::from_char('5'), CharType::Word);
		assert_eq!(CharType::from_char('_'), CharType::Word);
		assert_eq!(CharType::from_char(' '), CharType::Whitespace);
		assert_eq!(CharType::from_char('\t'), CharType::Whitespace);
		assert_eq!(CharType::from_char('\n'), CharType::Newline);
		assert_eq!(CharType::from_char('.'), CharType::Punctuation);
		assert_eq!(CharType::from_char('!'), CharType::Punctuation);
	}

	#[test]
	fn test_word_range() {
		let content = TextContent::from_str("hello world");
		assert_eq!(TextSelector::word_range(&content, 0), Some(0..5));
		assert_eq!(TextSelector::word_range(&content, 2), Some(0..5));
		assert_eq!(TextSelector::word_range(&content, 5), Some(5..6)); // space
		assert_eq!(TextSelector::word_range(&content, 6), Some(6..11));

		let content = TextContent::from_str("hello_world");
		assert_eq!(TextSelector::word_range(&content, 0), Some(0..11));
	}

	#[test]
	fn test_line_range() {
		let content = TextContent::from_str("line1\nline2\nline3");
		assert_eq!(TextSelector::line_range(&content, 0), 0..6); // includes \n
		assert_eq!(TextSelector::line_range(&content, 6), 6..12);
		assert_eq!(TextSelector::line_range(&content, 12), 12..17); // last line, no \n
	}
}
