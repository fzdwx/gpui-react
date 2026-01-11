//! Input state management
//!
//! Handles text content, cursor position, and selection for input elements.

use unicode_segmentation::UnicodeSegmentation;

use super::{selection::{Selection, TextSelector}, text_content::{Point, TextContent}, text_wrapper::TextWrapper};

/// Input type for specialized behavior
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum InputType {
	#[default]
	Text,
	Password,
	Number,
	Email,
}

impl InputType {
	pub fn from_str(s: &str) -> Self {
		match s {
			"password" => InputType::Password,
			"number" => InputType::Number,
			"email" => InputType::Email,
			_ => InputType::Text,
		}
	}
}

/// Represents a text change for onChange event
#[derive(Clone, Debug)]
pub struct TextChange {
	pub old_value:       String,
	pub new_value:       String,
	pub cursor_position: usize,
	pub data:            Option<String>,
	pub input_type:      &'static str,
}

/// Text input state - tracks content, cursor, and selection
#[derive(Clone, Debug)]
pub struct InputState {
	/// The text content using rope-based storage
	pub content: TextContent,

	/// Current selection (includes cursor position as head)
	pub selection: Selection,

	/// Text wrapper for multi-line mode
	pub wrapper: Option<TextWrapper>,

	/// Whether multi-line mode is enabled
	pub multi_line: bool,

	/// Preferred column for up/down navigation (preserves horizontal position)
	pub preferred_column: Option<usize>,

	/// Scroll offset for horizontal scrolling when text exceeds bounds
	pub scroll_offset: f32,

	/// Marked range for IME composition (start_byte, end_byte)
	pub marked_range: Option<(usize, usize)>,

	/// Whether the input is disabled
	pub disabled: bool,

	/// Whether the input is read-only
	pub read_only: bool,

	/// Maximum length (in characters, not bytes)
	pub max_length: Option<usize>,

	/// Placeholder text
	pub placeholder: Option<String>,

	/// Input type for specialized behavior
	pub input_type: InputType,
}

impl Default for InputState {
	fn default() -> Self { Self::new() }
}

impl InputState {
	pub fn new() -> Self {
		Self {
			content:          TextContent::new(),
			selection:        Selection::cursor(0),
			wrapper:          None,
			multi_line:       false,
			preferred_column: None,
			scroll_offset:    0.0,
			marked_range:     None,
			disabled:         false,
			read_only:        false,
			max_length:       None,
			placeholder:      None,
			input_type:       InputType::Text,
		}
	}

	/// Create with initial content
	pub fn with_content(content_str: String) -> Self {
		let content = TextContent::from_str(&content_str);
		let cursor = content.len();
		Self { content, selection: Selection::cursor(cursor), ..Self::new() }
	}

	/// Enable multi-line mode
	pub fn set_multi_line(&mut self, enabled: bool) {
		self.multi_line = enabled;
		if enabled && self.wrapper.is_none() {
			self.wrapper = Some(TextWrapper::new(None));
			self.update_wrapper();
		} else if !enabled {
			self.wrapper = None;
		}
	}

	/// Update the text wrapper with current content
	fn update_wrapper(&mut self) {
		if let Some(ref mut wrapper) = self.wrapper {
			wrapper.update(&self.content);
		}
	}

	// ===========================================
	// Compatibility layer - maps to old API
	// ===========================================

	/// Get cursor position as byte offset (legacy compatibility)
	pub fn cursor_position(&self) -> usize { self.selection.head }

	/// Set cursor position (legacy compatibility)
	pub fn set_cursor_position(&mut self, pos: usize) {
		let pos = pos.min(self.content.len());
		self.selection = Selection::cursor(pos);
		self.preferred_column = None;
	}

	/// Get content as String (legacy compatibility)
	#[inline]
	pub fn content_string(&self) -> String { self.content.to_string() }

	/// Get the number of grapheme clusters (visible characters)
	pub fn grapheme_count(&self) -> usize { self.content.grapheme_count() }

	/// Check if max_length would be exceeded by inserting text
	fn would_exceed_max_length(&self, text: &str) -> bool {
		if let Some(max) = self.max_length {
			let current_count = self.grapheme_count();
			let selection_count = if !self.selection.is_empty() {
				self.content.slice(self.selection.range()).to_string().graphemes(true).count()
			} else {
				0
			};
			let insert_count = text.graphemes(true).count();
			current_count - selection_count + insert_count > max
		} else {
			false
		}
	}

	/// Insert text at cursor position, replacing selection if any
	pub fn insert_text(&mut self, text: &str) -> Option<TextChange> {
		if self.disabled || self.read_only {
			return None;
		}

		// Filter newlines in single-line mode
		let text =
			if self.multi_line { text.to_string() } else { text.replace('\n', "").replace('\r', "") };

		if self.would_exceed_max_length(&text) {
			return None;
		}

		let old_value = self.content.to_string();
		let range = self.selection.range();

		// Replace selection with new text
		self.content.replace(range.clone(), &text);

		let new_cursor = range.start + text.len();
		self.selection = Selection::cursor(new_cursor);
		self.preferred_column = None;
		self.update_wrapper();

		Some(TextChange {
			old_value,
			new_value: self.content.to_string(),
			cursor_position: self.selection.head,
			data: Some(text),
			input_type: "insertText",
		})
	}

	/// Insert a newline (for multi-line mode)
	pub fn insert_newline(&mut self) -> Option<TextChange> {
		if !self.multi_line {
			return None;
		}
		self.insert_text("\n")
	}

	/// Delete backward (backspace)
	pub fn backspace(&mut self) -> Option<TextChange> {
		if self.disabled || self.read_only {
			return None;
		}

		let old_value = self.content.to_string();

		// Delete selection if exists
		if !self.selection.is_empty() {
			let range = self.selection.range();
			self.content.remove(range.clone());
			self.selection = Selection::cursor(range.start);
			self.preferred_column = None;
			self.marked_range = None;
			self.update_wrapper();
			return Some(TextChange {
				old_value,
				new_value: self.content.to_string(),
				cursor_position: self.selection.head,
				data: None,
				input_type: "deleteContentBackward",
			});
		}

		// Delete one grapheme before cursor
		if self.selection.head > 0 {
			let prev_boundary = self.content.prev_grapheme_boundary(self.selection.head);
			self.content.remove(prev_boundary..self.selection.head);
			self.selection = Selection::cursor(prev_boundary);
			self.preferred_column = None;
			self.marked_range = None;
			self.update_wrapper();
			return Some(TextChange {
				old_value,
				new_value: self.content.to_string(),
				cursor_position: self.selection.head,
				data: None,
				input_type: "deleteContentBackward",
			});
		}

		None
	}

	/// Delete forward (delete key)
	pub fn delete(&mut self) -> Option<TextChange> {
		if self.disabled || self.read_only {
			return None;
		}

		let old_value = self.content.to_string();

		// Delete selection if exists
		if !self.selection.is_empty() {
			let range = self.selection.range();
			self.content.remove(range.clone());
			self.selection = Selection::cursor(range.start);
			self.preferred_column = None;
			self.update_wrapper();
			return Some(TextChange {
				old_value,
				new_value: self.content.to_string(),
				cursor_position: self.selection.head,
				data: None,
				input_type: "deleteContentForward",
			});
		}

		// Delete one grapheme after cursor
		if self.selection.head < self.content.len() {
			let next_boundary = self.content.next_grapheme_boundary(self.selection.head);
			self.content.remove(self.selection.head..next_boundary);
			self.preferred_column = None;
			self.update_wrapper();
			return Some(TextChange {
				old_value,
				new_value: self.content.to_string(),
				cursor_position: self.selection.head,
				data: None,
				input_type: "deleteContentForward",
			});
		}

		None
	}

	/// Move cursor left by one grapheme
	pub fn move_left(&mut self, extend_selection: bool) {
		let new_pos = self.content.prev_grapheme_boundary(self.selection.head);

		if extend_selection {
			self.selection.extend_to(new_pos);
		} else {
			// If there's a selection, move to the start of it
			if !self.selection.is_empty() {
				self.selection = Selection::cursor(self.selection.start());
			} else {
				self.selection = Selection::cursor(new_pos);
			}
		}
		self.preferred_column = None;
	}

	/// Move cursor right by one grapheme
	pub fn move_right(&mut self, extend_selection: bool) {
		let new_pos = self.content.next_grapheme_boundary(self.selection.head);

		if extend_selection {
			self.selection.extend_to(new_pos);
		} else {
			// If there's a selection, move to the end of it
			if !self.selection.is_empty() {
				self.selection = Selection::cursor(self.selection.end());
			} else {
				self.selection = Selection::cursor(new_pos);
			}
		}
		self.preferred_column = None;
	}

	/// Move cursor up by one line (multi-line mode only)
	pub fn move_up(&mut self, extend_selection: bool) {
		if !self.multi_line {
			return;
		}

		let Some(ref wrapper) = self.wrapper else { return };

		// Remember column on first vertical move
		if self.preferred_column.is_none() {
			let point = wrapper.offset_to_display_point(&self.content, self.selection.head);
			self.preferred_column = Some(point.column);
		}

		let new_pos = wrapper.move_up(&self.content, self.selection.head, self.preferred_column);

		if extend_selection {
			self.selection.extend_to(new_pos);
		} else {
			self.selection = Selection::cursor(new_pos);
		}
	}

	/// Move cursor down by one line (multi-line mode only)
	pub fn move_down(&mut self, extend_selection: bool) {
		if !self.multi_line {
			return;
		}

		let Some(ref wrapper) = self.wrapper else { return };

		// Remember column on first vertical move
		if self.preferred_column.is_none() {
			let point = wrapper.offset_to_display_point(&self.content, self.selection.head);
			self.preferred_column = Some(point.column);
		}

		let new_pos = wrapper.move_down(&self.content, self.selection.head, self.preferred_column);

		if extend_selection {
			self.selection.extend_to(new_pos);
		} else {
			self.selection = Selection::cursor(new_pos);
		}
	}

	/// Move cursor to start of current line
	pub fn move_to_line_start(&mut self, extend_selection: bool) {
		let point = self.content.offset_to_point(self.selection.head);
		let new_pos = self.content.line_start_offset(point.row);

		if extend_selection {
			self.selection.extend_to(new_pos);
		} else {
			self.selection = Selection::cursor(new_pos);
		}
		self.preferred_column = None;
	}

	/// Move cursor to end of current line
	pub fn move_to_line_end(&mut self, extend_selection: bool) {
		let point = self.content.offset_to_point(self.selection.head);
		let new_pos = self.content.line_end_offset(point.row);

		if extend_selection {
			self.selection.extend_to(new_pos);
		} else {
			self.selection = Selection::cursor(new_pos);
		}
		self.preferred_column = None;
	}

	/// Move cursor to start of text
	pub fn move_to_start(&mut self, extend_selection: bool) {
		if extend_selection {
			self.selection.extend_to(0);
		} else {
			self.selection = Selection::cursor(0);
		}
		self.preferred_column = None;
	}

	/// Move cursor to end of text
	pub fn move_to_end(&mut self, extend_selection: bool) {
		let end = self.content.len();
		if extend_selection {
			self.selection.extend_to(end);
		} else {
			self.selection = Selection::cursor(end);
		}
		self.preferred_column = None;
	}

	/// Select all text
	pub fn select_all(&mut self) {
		if self.content.is_empty() {
			return;
		}
		self.selection = Selection::new(0, self.content.len());
		self.preferred_column = None;
	}

	/// Select word at current position (for double-click)
	pub fn select_word(&mut self) {
		if let Some(range) = TextSelector::word_range(&self.content, self.selection.head) {
			self.selection = Selection::new(range.start, range.end);
		}
		self.preferred_column = None;
	}

	/// Select line at current position (for triple-click)
	pub fn select_line(&mut self) {
		let range = TextSelector::line_range(&self.content, self.selection.head);
		self.selection = Selection::new(range.start, range.end);
		self.preferred_column = None;
	}

	/// Get selected text, if any
	pub fn selected_text(&self) -> Option<String> {
		if self.selection.is_empty() {
			None
		} else {
			Some(self.content.slice(self.selection.range()).to_string())
		}
	}

	/// Delete selected text and return it
	pub fn cut_selection(&mut self) -> Option<String> {
		if self.disabled || self.read_only {
			return None;
		}

		if self.selection.is_empty() {
			return None;
		}

		let range = self.selection.range();
		let cut_text = self.content.slice(range.clone()).to_string();
		self.content.remove(range.clone());
		self.selection = Selection::cursor(range.start);
		self.preferred_column = None;
		self.update_wrapper();
		Some(cut_text)
	}

	/// Set cursor from click position (byte offset)
	pub fn set_cursor_from_offset(&mut self, byte_offset: usize) {
		let byte_offset = byte_offset.min(self.content.len());
		let byte_offset = self.content.clip_offset(byte_offset);
		self.selection = Selection::cursor(byte_offset);
		self.preferred_column = None;
	}

	/// Extend selection from current anchor to given byte offset
	pub fn extend_selection_to(&mut self, byte_offset: usize) {
		let byte_offset = byte_offset.min(self.content.len());
		let byte_offset = self.content.clip_offset(byte_offset);
		self.selection.extend_to(byte_offset);
	}

	/// Get display text (handles password masking)
	pub fn display_text(&self) -> String {
		if self.input_type == InputType::Password {
			"*".repeat(self.grapheme_count())
		} else {
			self.content.to_string()
		}
	}

	// ===========================================
	// UTF-16 conversion helpers for IME support
	// ===========================================

	/// Convert UTF-8 byte offset to UTF-16 offset
	pub fn offset_to_utf16(&self, byte_offset: usize) -> usize {
		self.content.offset_to_utf16(byte_offset)
	}

	/// Convert UTF-16 offset to UTF-8 byte offset
	pub fn offset_from_utf16(&self, utf16_offset: usize) -> usize {
		self.content.offset_from_utf16(utf16_offset)
	}

	/// Convert UTF-8 byte range to UTF-16 range
	pub fn range_to_utf16(&self, range: &std::ops::Range<usize>) -> std::ops::Range<usize> {
		self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
	}

	/// Convert UTF-16 range to UTF-8 byte range
	pub fn range_from_utf16(&self, range_utf16: &std::ops::Range<usize>) -> std::ops::Range<usize> {
		self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
	}

	/// Replace text in a range (for IME support)
	pub fn replace_in_range(
		&mut self,
		range: std::ops::Range<usize>,
		new_text: &str,
	) -> Option<TextChange> {
		if self.disabled || self.read_only {
			return None;
		}

		let old_value = self.content.to_string();

		// Clamp range to valid bounds
		let start = range.start.min(self.content.len());
		let end = range.end.min(self.content.len());

		log::debug!(
			"[InputState] replace_in_range: range={:?}, snapped to {}..{}, new_text={:?} (len={}), content_before={:?}",
			range,
			start,
			end,
			new_text,
			new_text.len(),
			self.content.to_string()
		);

		self.content.replace(start..end, new_text);
		let new_cursor = start + new_text.len();
		self.selection = Selection::cursor(new_cursor);
		self.preferred_column = None;
		self.update_wrapper();

		log::debug!(
			"[InputState] replace_in_range: after replacement, content={:?}, cursor_position={}",
			self.content.to_string(),
			self.selection.head
		);

		Some(TextChange {
			old_value,
			new_value: self.content.to_string(),
			cursor_position: self.selection.head,
			data: Some(new_text.to_string()),
			input_type: "insertText",
		})
	}

	/// Set marked range for IME composition
	pub fn set_marked_range(&mut self, range: Option<(usize, usize)>) { self.marked_range = range; }

	/// Get the selection range as a Range (legacy compatibility)
	pub fn selection_range(&self) -> std::ops::Range<usize> { self.selection.range() }

	/// Get selection as legacy tuple format
	pub fn selection_tuple(&self) -> Option<(usize, usize)> { self.selection.as_tuple() }

	// ===========================================
	// Multi-line helpers
	// ===========================================

	/// Get the text wrapper (for rendering)
	pub fn wrapper(&self) -> Option<&TextWrapper> { self.wrapper.as_ref() }

	/// Get number of display rows
	pub fn display_row_count(&self) -> usize {
		self.wrapper.as_ref().map(|w| w.display_row_count()).unwrap_or(1)
	}

	/// Get the point (row, column) for the cursor
	pub fn cursor_point(&self) -> Point { self.content.offset_to_point(self.selection.head) }

	/// Get total line count
	pub fn line_count(&self) -> usize { self.content.line_count() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_insert_text() {
		let mut state = InputState::new();
		let change = state.insert_text("hello").unwrap();
		assert_eq!(state.content.to_string(), "hello");
		assert_eq!(state.selection.head, 5);
		assert_eq!(change.new_value, "hello");
	}

	#[test]
	fn test_backspace() {
		let mut state = InputState::with_content("hello".to_string());
		state.backspace();
		assert_eq!(state.content.to_string(), "hell");
		assert_eq!(state.selection.head, 4);
	}

	#[test]
	fn test_delete() {
		let mut state = InputState::with_content("hello".to_string());
		state.selection = Selection::cursor(0);
		state.delete();
		assert_eq!(state.content.to_string(), "ello");
		assert_eq!(state.selection.head, 0);
	}

	#[test]
	fn test_selection() {
		let mut state = InputState::with_content("hello world".to_string());
		state.selection = Selection::cursor(0);
		state.move_right(true); // h
		state.move_right(true); // he
		state.move_right(true); // hel
		state.move_right(true); // hell
		state.move_right(true); // hello
		assert_eq!(state.selected_text(), Some("hello".to_string()));
	}

	#[test]
	fn test_backspace_with_selection() {
		let mut state = InputState::with_content("hello".to_string());
		state.selection = Selection::new(1, 4); // "ell" selected
		state.backspace();
		assert_eq!(state.content.to_string(), "ho");
	}

	#[test]
	fn test_select_all() {
		let mut state = InputState::with_content("hello world".to_string());
		state.select_all();
		assert_eq!(state.selection, Selection::new(0, 5));
	}

	#[test]

	#[test]
	fn test_max_length() {
		let mut state = InputState::new();
		state.max_length = Some(5);
		state.insert_text("hello");
		assert_eq!(state.content.to_string(), "hello");
		// Should not insert more
		let result = state.insert_text("!");
		assert!(result.is_none());
		assert_eq!(state.content.to_string(), "hello");
	}

	#[test]
	fn test_password_display() {
		let mut state = InputState::with_content("secret".to_string());
		state.input_type = InputType::Password;
		assert_eq!(state.display_text(), "******");
	}

	#[test]
	fn test_multi_line_mode() {
		let mut state = InputState::new();
		state.set_multi_line(true);
		assert!(state.multi_line);
		assert!(state.wrapper.is_some());

		// Insert text with newlines
		state.insert_text("line1\nline2\nline3");
		assert_eq!(state.line_count(), 3);
		assert_eq!(state.display_row_count(), 3);
	}

	#[test]
	fn test_move_up_down() {
		let mut state = InputState::new();
		state.set_multi_line(true);
		state.insert_text("hello\nworld\ntest");

		// Cursor at end of "test"
		assert_eq!(state.cursor_position(), 16);

		// Move up to "world" line
		state.move_up(false);
		assert_eq!(state.cursor_point().row, 1);

		// Move up to "hello" line
		state.move_up(false);
		assert_eq!(state.cursor_point().row, 0);

		// Move down back to "world"
		state.move_down(false);
		assert_eq!(state.cursor_point().row, 1);
	}

	#[test]
	fn test_single_line_filters_newlines() {
		let mut state = InputState::new();
		state.insert_text("hello\nworld");
		// Newlines should be filtered in single-line mode
		assert_eq!(state.content.to_string(), "helloworld");
	}

	#[test]
	fn test_word_selection() {
		let mut state = InputState::with_content("hello world".to_string());
		state.selection = Selection::cursor(2); // in "hello"
		state.select_word();
		assert_eq!(state.selected_text(), Some("hello".to_string()));

		state.selection = Selection::cursor(8); // in "world"
		state.select_word();
		assert_eq!(state.selected_text(), Some("world".to_string()));
	}
}
