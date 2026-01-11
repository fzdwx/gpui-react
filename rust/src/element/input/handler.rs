//! Input handler for IME support
//!
//! Implements GPUI's InputHandler trait to enable IME (Input Method Editor)
//! support for Chinese, Japanese, and other input methods.

use std::ops::Range;

use gpui::{App, Bounds, InputHandler, Pixels, Point, UTF16Selection, Window, point, px};

use super::{get_input_state, get_text_shaping_info, update_input_state};
use crate::{event_types::{EventData, InputEventData, types}, focus, renderer::dispatch_event_to_js};

/// Snap a byte position to the nearest valid UTF-8 char boundary
fn snap_to_char_boundary(s: &str, pos: usize) -> usize {
	if pos >= s.len() {
		return s.len();
	}
	if s.is_char_boundary(pos) {
		return pos;
	}
	// Find the nearest char boundary by moving forward
	for i in pos..=s.len() {
		if s.is_char_boundary(i) {
			return i;
		}
	}
	s.len()
}

/// Root-level input handler that delegates to the currently focused input element
/// This is used because we can't easily create per-element FocusHandles
pub struct RootInputHandler {
	pub window_id: u64,
}

impl RootInputHandler {
	pub fn new(window_id: u64) -> Self {
		Self { window_id }
	}

	/// Get the currently focused input element ID
	fn get_focused_input(&self) -> Option<u64> {
		let focused = focus::get_focused(self.window_id);
		log::trace!("[IME] get_focused_input: window_id={}, focused={:?}", self.window_id, focused);
		focused
	}
}

impl InputHandler for RootInputHandler {
	fn selected_text_range(
		&mut self,
		_ignore_disabled_input: bool,
		_window: &mut Window,
		_cx: &mut App,
	) -> Option<UTF16Selection> {
		let element_id = self.get_focused_input()?;
		let state = get_input_state(element_id)?;

		let range = state.selection_range();
		let utf16_range = state.range_to_utf16(&range);

		log::debug!(
			"[IME] selected_text_range: element_id={}, range={:?}, utf16_range={:?}",
			element_id, range, utf16_range
		);

		Some(UTF16Selection { range: utf16_range, reversed: state.selection_reversed })
	}

	fn marked_text_range(&mut self, _window: &mut Window, _cx: &mut App) -> Option<Range<usize>> {
		let element_id = self.get_focused_input()?;
		let state = get_input_state(element_id)?;

		let result = state.marked_range.map(|(start, end)| {
			let utf16_start = state.offset_to_utf16(start);
			let utf16_end = state.offset_to_utf16(end);
			utf16_start..utf16_end
		});

		log::debug!(
			"[IME] marked_text_range: element_id={}, marked_range={:?}, result={:?}",
			element_id, state.marked_range, result
		);

		result
	}

	fn text_for_range(
		&mut self,
		range_utf16: Range<usize>,
		adjusted_range: &mut Option<Range<usize>>,
		_window: &mut Window,
		_cx: &mut App,
	) -> Option<String> {
		let element_id = self.get_focused_input()?;
		let state = get_input_state(element_id)?;

		let range = state.range_from_utf16(&range_utf16);
		let clamped_range = range.start.min(state.content.len())..range.end.min(state.content.len());

		*adjusted_range = Some(state.range_to_utf16(&clamped_range));

		Some(state.content[clamped_range].to_string())
	}

	fn replace_text_in_range(
		&mut self,
		replacement_range: Option<Range<usize>>,
		text: &str,
		window: &mut Window,
		_cx: &mut App,
	) {
		log::debug!(
			"[IME] replace_text_in_range: replacement_range={:?}, text={:?}",
			replacement_range, text
		);

		let Some(element_id) = self.get_focused_input() else {
			log::warn!("[IME] replace_text_in_range: no focused input");
			return;
		};
		let Some(mut state) = get_input_state(element_id) else {
			log::warn!("[IME] replace_text_in_range: no state for element {}", element_id);
			return;
		};

		// Determine the range to replace
		let range = if let Some(range_utf16) = replacement_range {
			state.range_from_utf16(&range_utf16)
		} else if let Some((start, end)) = state.marked_range {
			start..end
		} else if let Some((start, end)) = state.selection {
			start..end
		} else {
			state.cursor_position..state.cursor_position
		};

		log::debug!(
			"[IME] replace_text_in_range: computed range={:?}, current content={:?}",
			range, state.content
		);

		// Perform the replacement
		if let Some(change) = state.replace_in_range(range, text) {
			// Clear marked range after committing text
			state.marked_range = None;

			update_input_state(element_id, state);

			// Dispatch input event to JS
			dispatch_event_to_js(
				self.window_id,
				element_id,
				types::INPUT,
				EventData::Input(InputEventData {
					value: change.new_value,
					data: change.data,
					input_type: change.input_type.to_string(),
					is_composing: false,
				}),
			);

			window.refresh();
		}
	}

	fn replace_and_mark_text_in_range(
		&mut self,
		range_utf16: Option<Range<usize>>,
		new_text: &str,
		new_selected_range_utf16: Option<Range<usize>>,
		window: &mut Window,
		_cx: &mut App,
	) {
		log::debug!(
			"[IME] replace_and_mark_text_in_range: range_utf16={:?}, new_text={:?}, new_selected_range={:?}",
			range_utf16, new_text, new_selected_range_utf16
		);

		let Some(element_id) = self.get_focused_input() else {
			log::warn!("[IME] replace_and_mark_text_in_range: no focused input");
			return;
		};
		let Some(mut state) = get_input_state(element_id) else {
			log::warn!("[IME] replace_and_mark_text_in_range: no state for element {}", element_id);
			return;
		};

		// Determine the range to replace
		let range = if let Some(range_utf16) = range_utf16.clone() {
			state.range_from_utf16(&range_utf16)
		} else if let Some((start, end)) = state.marked_range {
			start..end
		} else if let Some((start, end)) = state.selection {
			start..end
		} else {
			state.cursor_position..state.cursor_position
		};

		// Clamp range to valid UTF-8 boundaries
		let start = range.start.min(state.content.len());
		let end = range.end.min(state.content.len());

		// Ensure we're at valid UTF-8 char boundaries
		let start = snap_to_char_boundary(&state.content, start);
		let end = snap_to_char_boundary(&state.content, end);

		log::debug!(
			"[IME] replace_and_mark_text_in_range: range={:?}, snapped to {:?}..{:?}, content before={:?}, cursor_before={}",
			range, start, end, state.content, state.cursor_position
		);

		// Perform the replacement
		state.content.replace_range(start..end, new_text);

		// Set marked range for the new text (showing composition underline)
		if !new_text.is_empty() {
			state.marked_range = Some((start, start + new_text.len()));
		} else {
			state.marked_range = None;
		}

		// Set cursor position based on new_selected_range
		if let Some(selected_range_utf16) = new_selected_range_utf16 {
			let selected_range = state.range_from_utf16(&selected_range_utf16);
			// The selected range is relative to the new text, so add start offset
			state.cursor_position = (start + selected_range.end).min(state.content.len());
			if selected_range.start != selected_range.end {
				state.selection = Some((
					start + selected_range.start,
					(start + selected_range.end).min(state.content.len()),
				));
			} else {
				state.selection = None;
			}
		} else {
			state.cursor_position = start + new_text.len();
			state.selection = None;
		}

		log::debug!(
			"[IME] replace_and_mark_text_in_range: content after={:?}, marked_range={:?}, cursor={}",
			state.content, state.marked_range, state.cursor_position
		);

		update_input_state(element_id, state.clone());

		// Dispatch input event with isComposing = true
		dispatch_event_to_js(
			self.window_id,
			element_id,
			types::INPUT,
			EventData::Input(InputEventData {
				value: state.content.clone(),
				data: Some(new_text.to_string()),
				input_type: "insertCompositionText".to_string(),
				is_composing: true,
			}),
		);

		window.refresh();
	}

	fn unmark_text(&mut self, _window: &mut Window, _cx: &mut App) {
		let Some(element_id) = self.get_focused_input() else {
			return;
		};
		if let Some(mut state) = get_input_state(element_id) {
			state.marked_range = None;
			update_input_state(element_id, state);
		}
	}

	fn bounds_for_range(
		&mut self,
		range_utf16: Range<usize>,
		_window: &mut Window,
		_cx: &mut App,
	) -> Option<Bounds<Pixels>> {
		let element_id = self.get_focused_input()?;
		let state = get_input_state(element_id)?;
		let shaping_info = get_text_shaping_info(element_id)?;

		let range = state.range_from_utf16(&range_utf16);

		// Get x positions for the range using stored shaping info
		// Note: This is an approximation since we don't have the exact shaped line
		// A more accurate implementation would reshape the text here
		let char_width = shaping_info.font_size * 0.6; // Approximate character width

		let start_char_count = state.content[..range.start.min(state.content.len())].chars().count();
		let end_char_count = state.content[..range.end.min(state.content.len())].chars().count();

		let start_x = px(shaping_info.text_origin_x + start_char_count as f32 * char_width);
		let end_x = px(shaping_info.text_origin_x + end_char_count as f32 * char_width);

		// We need bounds, but we don't have them stored. Return approximate bounds.
		// This is used for IME candidate window positioning.
		Some(Bounds::from_corners(
			point(start_x, px(0.0)),
			point(end_x, px(32.0)), // Approximate height
		))
	}

	fn character_index_for_point(
		&mut self,
		point: Point<Pixels>,
		_window: &mut Window,
		_cx: &mut App,
	) -> Option<usize> {
		let element_id = self.get_focused_input()?;
		let state = get_input_state(element_id)?;
		let shaping_info = get_text_shaping_info(element_id)?;

		// Calculate local x position
		let local_x: f32 = point.x.into();
		let text_origin_x = shaping_info.text_origin_x;

		if local_x <= text_origin_x {
			return Some(0);
		}

		// Approximate character width
		let char_width = shaping_info.font_size * 0.6;
		let offset = (local_x - text_origin_x) / char_width;
		let char_index = offset as usize;

		// Convert character index to byte offset, then to UTF-16
		let byte_offset: usize = state.content
			.char_indices()
			.nth(char_index)
			.map(|(idx, _)| idx)
			.unwrap_or(state.content.len());

		Some(state.offset_to_utf16(byte_offset))
	}
}
