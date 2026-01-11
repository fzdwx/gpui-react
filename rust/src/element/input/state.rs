//! Input state management
//!
//! Handles text content, cursor position, and selection for input elements.

use unicode_segmentation::UnicodeSegmentation;

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
	/// The text content (UTF-8)
	pub content: String,

	/// Cursor position as byte offset in content
	pub cursor_position: usize,

	/// Selection range (start_byte, end_byte), None if no selection
	pub selection: Option<(usize, usize)>,

	/// Whether selection direction is reversed (selecting leftward)
	pub selection_reversed: bool,

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
			content:            String::new(),
			cursor_position:    0,
			selection:          None,
			selection_reversed: false,
			scroll_offset:      0.0,
			marked_range:       None,
			disabled:           false,
			read_only:          false,
			max_length:         None,
			placeholder:        None,
			input_type:         InputType::Text,
		}
	}

	/// Create with initial content
	pub fn with_content(content: String) -> Self {
		let cursor_position = content.len();
		Self { content, cursor_position, ..Self::new() }
	}

	/// Get the number of grapheme clusters (visible characters)
	pub fn grapheme_count(&self) -> usize { self.content.graphemes(true).count() }

	/// Check if max_length would be exceeded by inserting text
	fn would_exceed_max_length(&self, text: &str) -> bool {
		if let Some(max) = self.max_length {
			let current_count = self.grapheme_count();
			let selection_count =
				self.selection.map(|(s, e)| self.content[s..e].graphemes(true).count()).unwrap_or(0);
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

		if self.would_exceed_max_length(text) {
			return None;
		}

		let old_value = self.content.clone();

		// Delete selection first if exists
		if let Some((start, end)) = self.selection.take() {
			self.content.replace_range(start..end, "");
			self.cursor_position = start;
		}

		// Insert text at cursor
		self.content.insert_str(self.cursor_position, text);
		self.cursor_position += text.len();

		Some(TextChange {
			old_value,
			new_value: self.content.clone(),
			cursor_position: self.cursor_position,
			data: Some(text.to_string()),
			input_type: "insertText",
		})
	}

	/// Delete backward (backspace)
	pub fn backspace(&mut self) -> Option<TextChange> {
		if self.disabled || self.read_only {
			return None;
		}

		let old_value = self.content.clone();

		// Delete selection if exists
		if let Some((start, end)) = self.selection.take() {
			self.content.replace_range(start..end, "");
			self.cursor_position = start;
			return Some(TextChange {
				old_value,
				new_value: self.content.clone(),
				cursor_position: self.cursor_position,
				data: None,
				input_type: "deleteContentBackward",
			});
		}

		// Delete one grapheme before cursor
		if self.cursor_position > 0 {
			let prev_boundary = self.prev_grapheme_boundary(self.cursor_position);
			self.content.replace_range(prev_boundary..self.cursor_position, "");
			self.cursor_position = prev_boundary;
			return Some(TextChange {
				old_value,
				new_value: self.content.clone(),
				cursor_position: self.cursor_position,
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

		let old_value = self.content.clone();

		// Delete selection if exists
		if let Some((start, end)) = self.selection.take() {
			self.content.replace_range(start..end, "");
			self.cursor_position = start;
			return Some(TextChange {
				old_value,
				new_value: self.content.clone(),
				cursor_position: self.cursor_position,
				data: None,
				input_type: "deleteContentForward",
			});
		}

		// Delete one grapheme after cursor
		if self.cursor_position < self.content.len() {
			let next_boundary = self.next_grapheme_boundary(self.cursor_position);
			self.content.replace_range(self.cursor_position..next_boundary, "");
			return Some(TextChange {
				old_value,
				new_value: self.content.clone(),
				cursor_position: self.cursor_position,
				data: None,
				input_type: "deleteContentForward",
			});
		}

		None
	}

	/// Move cursor left by one grapheme
	pub fn move_left(&mut self, extend_selection: bool) {
		let new_pos = self.prev_grapheme_boundary(self.cursor_position);

		if extend_selection {
			self.extend_selection_to(new_pos);
		} else {
			// If there's a selection, move to the start of it
			if let Some((start, _end)) = self.selection.take() {
				self.cursor_position = start;
			} else {
				self.cursor_position = new_pos;
			}
		}
	}

	/// Move cursor right by one grapheme
	pub fn move_right(&mut self, extend_selection: bool) {
		let new_pos = self.next_grapheme_boundary(self.cursor_position);

		if extend_selection {
			self.extend_selection_to(new_pos);
		} else {
			// If there's a selection, move to the end of it
			if let Some((_start, end)) = self.selection.take() {
				self.cursor_position = end;
			} else {
				self.cursor_position = new_pos;
			}
		}
	}

	/// Move cursor to start of line
	pub fn move_to_start(&mut self, extend_selection: bool) {
		if extend_selection {
			self.extend_selection_to(0);
		} else {
			self.selection = None;
			self.cursor_position = 0;
		}
	}

	/// Move cursor to end of line
	pub fn move_to_end(&mut self, extend_selection: bool) {
		let end = self.content.len();
		if extend_selection {
			self.extend_selection_to(end);
		} else {
			self.selection = None;
			self.cursor_position = end;
		}
	}

	/// Select all text
	pub fn select_all(&mut self) {
		if self.content.is_empty() {
			return;
		}
		self.selection = Some((0, self.content.len()));
		self.selection_reversed = false;
		self.cursor_position = self.content.len();
	}

	/// Get selected text, if any
	pub fn selected_text(&self) -> Option<&str> {
		self.selection.map(|(start, end)| &self.content[start..end])
	}

	/// Delete selected text and return it
	pub fn cut_selection(&mut self) -> Option<String> {
		if self.disabled || self.read_only {
			return None;
		}

		if let Some((start, end)) = self.selection.take() {
			let cut_text = self.content[start..end].to_string();
			self.content.replace_range(start..end, "");
			self.cursor_position = start;
			Some(cut_text)
		} else {
			None
		}
	}

	/// Set cursor from click position (byte offset)
	pub fn set_cursor_from_offset(&mut self, byte_offset: usize) {
		// Clamp to valid range
		let byte_offset = byte_offset.min(self.content.len());
		// Snap to grapheme boundary
		let byte_offset = self.snap_to_grapheme_boundary(byte_offset);

		self.cursor_position = byte_offset;
		self.selection = None;
	}

	/// Extend selection from current anchor to given byte offset
	pub fn extend_selection_to(&mut self, byte_offset: usize) {
		let byte_offset = byte_offset.min(self.content.len());
		let byte_offset = self.snap_to_grapheme_boundary(byte_offset);

		match self.selection {
			Some((start, end)) => {
				// Determine anchor based on selection direction
				let anchor = if self.selection_reversed { end } else { start };

				if byte_offset < anchor {
					self.selection = Some((byte_offset, anchor));
					self.selection_reversed = true;
				} else {
					self.selection = Some((anchor, byte_offset));
					self.selection_reversed = false;
				}
			}
			None => {
				// Start new selection from cursor
				let anchor = self.cursor_position;
				if byte_offset < anchor {
					self.selection = Some((byte_offset, anchor));
					self.selection_reversed = true;
				} else if byte_offset > anchor {
					self.selection = Some((anchor, byte_offset));
					self.selection_reversed = false;
				}
			}
		}

		self.cursor_position = byte_offset;
	}

	/// Get display text (handles password masking)
	pub fn display_text(&self) -> String {
		if self.input_type == InputType::Password {
			"*".repeat(self.grapheme_count())
		} else {
			self.content.clone()
		}
	}

	/// Find the previous grapheme cluster boundary
	fn prev_grapheme_boundary(&self, byte_offset: usize) -> usize {
		let mut prev = 0;
		for (idx, _) in self.content.grapheme_indices(true) {
			if idx >= byte_offset {
				break;
			}
			prev = idx;
		}
		if byte_offset > 0 && prev == 0 {
			// Check if we need to return 0 or find the actual previous
			for (idx, _) in self.content.grapheme_indices(true) {
				if idx >= byte_offset {
					return prev;
				}
				prev = idx;
			}
		}
		prev
	}

	/// Find the next grapheme cluster boundary
	fn next_grapheme_boundary(&self, byte_offset: usize) -> usize {
		for (idx, grapheme) in self.content.grapheme_indices(true) {
			if idx >= byte_offset {
				return idx + grapheme.len();
			}
		}
		self.content.len()
	}

	/// Snap a byte offset to the nearest grapheme boundary
	fn snap_to_grapheme_boundary(&self, byte_offset: usize) -> usize {
		if byte_offset == 0 || byte_offset >= self.content.len() {
			return byte_offset.min(self.content.len());
		}

		// Find the nearest grapheme boundary
		let mut prev = 0;
		for (idx, _) in self.content.grapheme_indices(true) {
			if idx >= byte_offset {
				// Return the closer boundary
				if byte_offset - prev < idx - byte_offset {
					return prev;
				}
				return idx;
			}
			prev = idx;
		}
		self.content.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_insert_text() {
		let mut state = InputState::new();
		let change = state.insert_text("hello").unwrap();
		assert_eq!(state.content, "hello");
		assert_eq!(state.cursor_position, 5);
		assert_eq!(change.new_value, "hello");
	}

	#[test]
	fn test_backspace() {
		let mut state = InputState::with_content("hello".to_string());
		state.backspace();
		assert_eq!(state.content, "hell");
		assert_eq!(state.cursor_position, 4);
	}

	#[test]
	fn test_delete() {
		let mut state = InputState::with_content("hello".to_string());
		state.cursor_position = 0;
		state.delete();
		assert_eq!(state.content, "ello");
		assert_eq!(state.cursor_position, 0);
	}

	#[test]
	fn test_selection() {
		let mut state = InputState::with_content("hello world".to_string());
		state.cursor_position = 0;
		state.move_right(true); // h
		state.move_right(true); // he
		state.move_right(true); // hel
		state.move_right(true); // hell
		state.move_right(true); // hello
		assert_eq!(state.selected_text(), Some("hello"));
	}

	#[test]
	fn test_backspace_with_selection() {
		let mut state = InputState::with_content("hello".to_string());
		state.selection = Some((1, 4)); // "ell" selected
		state.backspace();
		assert_eq!(state.content, "ho");
	}

	#[test]
	fn test_select_all() {
		let mut state = InputState::with_content("hello".to_string());
		state.select_all();
		assert_eq!(state.selection, Some((0, 5)));
	}

	#[test]
	fn test_max_length() {
		let mut state = InputState::new();
		state.max_length = Some(5);
		state.insert_text("hello");
		assert_eq!(state.content, "hello");
		// Should not insert more
		let result = state.insert_text("!");
		assert!(result.is_none());
		assert_eq!(state.content, "hello");
	}

	#[test]
	fn test_password_display() {
		let mut state = InputState::with_content("secret".to_string());
		state.input_type = InputType::Password;
		assert_eq!(state.display_text(), "******");
	}
}
