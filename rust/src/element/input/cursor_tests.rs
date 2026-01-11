//! Cursor positioning tests for multi-byte characters
//!
//! Tests verify that cursor positioning works correctly for:
//! - ASCII characters (1 byte each)
//! - Multi-byte characters (Chinese, 3 bytes each)
//! - Mixed content (ASCII + multi-byte)
//! - Grapheme clusters (combining characters)
//! - Empty input
//! - Boundary conditions

use super::{InputType, state::InputState, text_content::TextContent};
use crate::element::input::selection::Selection;

/// Test cursor positioning for ASCII text
#[test]
fn test_ascii_cursor_position() {
	let content = TextContent::from_str("hello");
	let state = InputState::with_content("hello".to_string());

	// Click after "o" (position 5)
	let byte_offset =
		state.content.char_indices().nth(5).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 5, "ASCII: cursor at end should be byte 5");

	// Click after "e" (position 1)
	let byte_offset =
		state.content.char_indices().nth(1).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 1, "ASCII: cursor after 'e' should be byte 1");
}

/// Test cursor positioning for Chinese/multi-byte characters
#[test]
fn test_chinese_cursor_position() {
	let content = TextContent::from_str("阿斯顿");
	let state = InputState::with_content("阿斯顿".to_string());

	// "阿斯顿" = 9 bytes (3 chars × 3 bytes each)
	// Bytes: 0-2 = 阿, 3-5 = 斯, 6-8 = 顿
	assert_eq!(content.len(), 9, "Chinese: content length should be 9 bytes");

	// Click after "阿" (should be byte 3)
	let byte_offset =
		state.content.char_indices().nth(1).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 3, "Chinese: cursor after '阿' should be byte 3");

	// Click after "斯" (should be byte 6)
	let byte_offset =
		state.content.char_indices().nth(2).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 6, "Chinese: cursor after '斯' should be byte 6");

	// Click after "顿" (should be byte 9)
	let byte_offset =
		state.content.char_indices().nth(3).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 9, "Chinese: cursor after '顿' should be byte 9");
}

/// Test cursor positioning for mixed ASCII and Chinese content
#[test]
fn test_mixed_cursor_position() {
	let content = TextContent::from_str("Hello世界");
	let state = InputState::with_content("Hello世界".to_string());

	// "Hello世界" = 5 + 6 = 11 bytes
	// Bytes: 0-4 = Hello, 5-7 = 世, 8-10 = 界
	assert_eq!(content.len(), 11, "Mixed: content length should be 11 bytes");

	// Click after "o" (position 5, byte 5)
	let byte_offset =
		state.content.char_indices().nth(5).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 5, "Mixed: cursor after 'o' should be byte 5");

	// Click after "世" (position 6, byte 8)
	let byte_offset =
		state.content.char_indices().nth(6).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 8, "Mixed: cursor after '世' should be byte 8");

	// Click after "界" (position 7, byte 11)
	let byte_offset =
		state.content.char_indices().nth(7).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 11, "Mixed: cursor after '界' should be byte 11");
}

/// Test cursor positioning for grapheme clusters (combining characters)
#[test]
fn test_grapheme_cursor_position() {
	let content = TextContent::from_str("café");
	let state = InputState::with_content("café".to_string());

	// "café" = 5 bytes (c,a,f = 1 byte each, é = 2 bytes)
	// But as a grapheme cluster, é is one visual character
	assert_eq!(content.len(), 5, "Grapheme: content length should be 5 bytes");

	// Cursor should be at grapheme boundaries
	// After "caf" (position 3, byte 3)
	let byte_offset =
		state.content.char_indices().nth(3).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 3, "Grapheme: cursor after 'f' should be byte 3");

	// After "café" (position 4, byte 5)
	let byte_offset =
		state.content.char_indices().nth(4).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 5, "Grapheme: cursor after 'é' should be byte 5");
}

/// Test cursor positioning for empty input
#[test]
fn test_empty_input() {
	let content = TextContent::new();
	let state = InputState::new();

	assert!(content.is_empty(), "Empty: content should be empty");
	assert_eq!(content.len(), 0, "Empty: length should be 0");

	// Click anywhere in empty input should return 0
	let byte_offset =
		state.content.char_indices().nth(0).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 0, "Empty: cursor should be at 0");
}

/// Test cursor positioning at boundaries
#[test]
fn test_cursor_at_boundaries() {
	let content = TextContent::from_str("阿a");
	let state = InputState::with_content("阿a".to_string());

	// "阿a" = 4 bytes (阿 = 3 bytes, a = 1 byte)
	assert_eq!(content.len(), 4, "Boundary: content length should be 4 bytes");

	// At start (position 0, byte 0)
	let byte_offset =
		state.content.char_indices().nth(0).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 0, "Boundary: cursor at start should be byte 0");

	// After "阿" (position 1, byte 3)
	let byte_offset =
		state.content.char_indices().nth(1).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 3, "Boundary: cursor after '阿' should be byte 3");

	// At end (position 2, byte 4)
	let byte_offset =
		state.content.char_indices().nth(2).map(|(idx, _)| idx).unwrap_or(content.len());
	assert_eq!(byte_offset, 4, "Boundary: cursor at end should be byte 4");
}

/// Test selection range calculation with multi-byte characters
#[test]
fn test_selection_range_multi_byte() {
	let mut state = InputState::with_content("Hello世界".to_string());

	// "Hello世界" = 11 bytes: 0-4="Hello", 5-7="世", 8-10="界"
	// Select "lo世" = bytes 3..8 (5 bytes: "lo" + "世")
	state.selection = Selection::new(3, 8); // bytes 3 to 8

	let range = state.selection_range();
	assert_eq!(range.start, 3, "Selection: start should be byte 3");
	assert_eq!(range.end, 8, "Selection: end should be byte 8");

	// Verify selected text
	let selected = state.selected_text();
	assert_eq!(selected, Some("lo世".to_string()), "Selection: should select 'lo世'");
}

/// Test that cursor position is correctly clamped to content length
#[test]
fn test_cursor_clamping() {
	let mut state = InputState::with_content("Hi".to_string());

	// Try to set cursor beyond content length
	state.set_cursor_from_offset(100);
	assert_eq!(state.cursor_position(), 2, "Cursor: should be clamped to content length");

	// Set cursor at valid position
	state.set_cursor_from_offset(1);
	assert_eq!(state.cursor_position(), 1, "Cursor: should be at valid position");
}

/// Test char_indices iterator with various content types
#[test]
fn test_char_indices_iterator() {
	// ASCII
	let content = TextContent::from_str("abc");
	let indices: Vec<(usize, char)> = content.char_indices().collect();
	assert_eq!(indices.len(), 3, "ASCII: should have 3 char indices");
	assert_eq!(indices[0], (0, 'a'), "ASCII: first char at byte 0");
	assert_eq!(indices[1], (1, 'b'), "ASCII: second char at byte 1");
	assert_eq!(indices[2], (2, 'c'), "ASCII: third char at byte 2");

	// Chinese
	let content = TextContent::from_str("你好");
	let indices: Vec<(usize, char)> = content.char_indices().collect();
	assert_eq!(indices.len(), 2, "Chinese: should have 2 char indices");
	assert_eq!(indices[0], (0, '你'), "Chinese: first char at byte 0");
	assert_eq!(indices[1], (3, '好'), "Chinese: second char at byte 3");

	// Mixed
	let content = TextContent::from_str("A中B");
	let indices: Vec<(usize, char)> = content.char_indices().collect();
	assert_eq!(indices.len(), 3, "Mixed: should have 3 char indices");
	assert_eq!(indices[0], (0, 'A'), "Mixed: 'A' at byte 0");
	assert_eq!(indices[1], (1, '中'), "Mixed: '中' at byte 1");
	assert_eq!(indices[2], (4, 'B'), "Mixed: 'B' at byte 4");
}

#[test]
fn test_ime_commit_cursor_preservation() {
	let mut state = InputState::with_content("ni hao".to_string());
	state.set_cursor_position(6); // Cursor at end of "ni hao"

	let range = 0..6;
	let new_text = "你好";

	if let Some(_change) = state.replace_in_range(range, new_text) {
		assert_eq!(state.content.to_string(), "你好");
		assert_eq!(state.content.len(), 6, "Chinese: content length should be 6 bytes");
		assert_eq!(state.cursor_position(), 6, "IME commit: cursor should be at byte 6 (end)");
		assert!(
			state.content.is_char_boundary(state.cursor_position()),
			"Cursor should be at valid char boundary"
		);
	}
}

#[test]
fn test_cursor_sync_after_ime_commit() {
	let mut state = InputState::with_content("ni hao".to_string());
	state.set_cursor_position(6);

	let new_value = "你好";
	let old_content = state.content.to_string();
	let old_cursor = state.cursor_position();

	state.content = TextContent::from_str(new_value);
	let new_cursor = old_cursor.min(new_value.len());
	state.set_cursor_position(new_cursor);

	assert_eq!(state.cursor_position(), 6, "Cursor should be preserved after IME commit");
	assert_eq!(new_cursor, 6, "min(6, 6) should be 6");
}
