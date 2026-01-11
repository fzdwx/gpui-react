//! Input handler for IME support
//!
//! Implements GPUI's InputHandler trait to enable IME (Input Method Editor)
//! support for Chinese, Japanese, and other input methods.

use std::ops::Range;

use gpui::{App, Bounds, InputHandler, Pixels, Point, UTF16Selection, Window, point, px};

use super::{get_input_state, get_text_shaping_info, selection::Selection, sync_input_state_from_props, text_content::TextContent, update_input_state};
use crate::{event_types::{EventData, InputEventData, types}, focus, renderer::dispatch_event_to_js};

/// Root-level input handler that delegates to the currently focused input
/// element This is used because we can't easily create per-element FocusHandles
pub struct RootInputHandler {
	pub window_id: u64,
}

impl RootInputHandler {
	pub fn new(window_id: u64) -> Self { Self { window_id } }

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
			element_id,
			range,
			utf16_range
		);

		Some(UTF16Selection { range: utf16_range, reversed: state.selection.is_reversed() })
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
			element_id,
			state.marked_range,
			result
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

		Some(state.content.slice(clamped_range).to_string())
	}

	fn replace_text_in_range(
		&mut self,
		replacement_range: Option<Range<usize>>,
		text: &str,
		window: &mut Window,
		_cx: &mut App,
	) {
		log::debug!(
			"[IME] replace_text_in_range: START replacement_range={:?}, text={:?}",
			replacement_range,
			text
		);

		let Some(element_id) = self.get_focused_input() else {
			log::warn!("[IME] replace_text_in_range: no focused input");
			return;
		};

		// Get current state first to check if we're in IME composition
		let Some(mut state) = get_input_state(element_id) else {
			log::warn!("[IME] replace_text_in_range: no state for element {}", element_id);
			return;
		};

		// Only sync from props if NOT in IME composition
		// During IME composition, Rust state is authoritative (not React's value prop)
		// because React may not have processed the compositionupdate event yet
		// Preserve marked range before sync to ensure correct replacement range after
		// sync
		let marked_range_before_sync = state.marked_range;
		if state.marked_range.is_none() && replacement_range.is_none() {
			sync_input_state_from_props(self.window_id, element_id);
			// Re-fetch state after sync
			let Some(synced_state) = get_input_state(element_id) else {
				log::warn!("[IME] replace_text_in_range: no state after sync for element {}", element_id);
				return;
			};
			state = synced_state;
		}
		// Restore marked range if it was saved before sync
		if marked_range_before_sync.is_some() && state.marked_range.is_none() {
			state.marked_range = marked_range_before_sync;
			log::debug!(
				"[IME] replace_text_in_range: Restored marked_range={:?} after sync",
				state.marked_range
			);
		}

		log::debug!(
			"[IME] replace_text_in_range: AFTER SYNC state.content={:?}, cursor_position={}, selection={:?}",
			state.content.to_string(),
			state.cursor_position(),
			state.selection_tuple()
		);

		// Determine the range to replace
		let range = if let Some(range_utf16) = replacement_range {
			state.range_from_utf16(&range_utf16)
		} else if let Some((start, end)) = state.marked_range {
			start..end
		} else if !state.selection.is_empty() {
			state.selection.range()
		} else {
			let cursor = state.cursor_position();
			cursor..cursor
		};

		log::debug!(
			"[IME] replace_text_in_range: computed range={:?}, content={:?}",
			range,
			state.content.to_string()
		);

		// Perform the replacement
		if let Some(change) = state.replace_in_range(range, text) {
			log::debug!(
				"[IME] replace_text_in_range: AFTER REPLACE content={:?}, cursor={}, change.new_value={:?}",
				state.content.to_string(),
				state.cursor_position(),
				change.new_value
			);

			// Clear marked range after committing text
			state.marked_range = None;

			update_input_state(element_id, state);

			// Dispatch input event to JS
			dispatch_event_to_js(
				self.window_id,
				element_id,
				types::INPUT,
				EventData::Input(InputEventData {
					value:        change.new_value,
					data:         change.data,
					input_type:   change.input_type.to_string(),
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
			"[IME] replace_and_mark_text_in_range: START range_utf16={:?}, new_text={:?}, new_selected_range_utf16={:?}",
			range_utf16,
			new_text,
			new_selected_range_utf16
		);

		let Some(element_id) = self.get_focused_input() else {
			log::warn!("[IME] replace_and_mark_text_in_range: no focused input");
			return;
		};

		// Get current state first to check if we're in IME composition
		let Some(mut state) = get_input_state(element_id) else {
			log::warn!("[IME] replace_and_mark_text_in_range: no state for element {}", element_id);
			return;
		};

		// Only sync from props if NOT in IME composition (first composition event)
		// During ongoing composition, Rust state is authoritative
		if state.marked_range.is_none() && range_utf16.is_none() {
			sync_input_state_from_props(self.window_id, element_id);
			// Re-fetch state after sync
			let Some(synced_state) = get_input_state(element_id) else {
				log::warn!(
					"[IME] replace_and_mark_text_in_range: no state after sync for element {}",
					element_id
				);
				return;
			};
			state = synced_state;
		}

		log::debug!(
			"[IME] replace_and_mark_text_in_range: AFTER SYNC content={:?}, cursor={}, selection={:?}, marked_range={:?}",
			state.content.to_string(),
			state.cursor_position(),
			state.selection_tuple(),
			state.marked_range
		);

		// Determine the range to replace
		let range = if let Some(range_utf16) = range_utf16.clone() {
			state.range_from_utf16(&range_utf16)
		} else if let Some((start, end)) = state.marked_range {
			start..end
		} else if !state.selection.is_empty() {
			state.selection.range()
		} else {
			let cursor = state.cursor_position();
			cursor..cursor
		};

		// Clamp range to valid boundaries
		let start = range.start.min(state.content.len());
		let end = range.end.min(state.content.len());

		// Clip to valid char boundaries
		let start = state.content.clip_offset(start);
		let end = state.content.clip_offset(end);

		log::debug!(
			"[IME] replace_and_mark_text_in_range: original_range={:?}, clamped={}..{}, content={:?}",
			range,
			start,
			end,
			state.content.to_string()
		);

		// Perform the replacement using TextContent
		state.content.replace(start..end, new_text);

		// Set marked range for the new text (showing composition underline)
		// marked_range uses byte offsets, not character counts
		if !new_text.is_empty() {
			state.marked_range = Some((start, start + new_text.len()));
		} else {
			state.marked_range = None;
		}

		// Set cursor position based on new_selected_range
		if let Some(selected_range_utf16) = new_selected_range_utf16 {
			let selected_range = state.range_from_utf16(&selected_range_utf16);
			// The selected range is relative to the new text, so add start offset
			let new_cursor = (start + selected_range.end).min(state.content.len());
			if selected_range.start != selected_range.end {
				state.selection = Selection::new(
					start + selected_range.start,
					(start + selected_range.end).min(state.content.len()),
				);
			} else {
				state.selection = Selection::cursor(new_cursor);
			}
		} else {
			// Use byte length for cursor positioning (consistent with rest of the system)
			state.selection = Selection::cursor((start + new_text.len()).min(state.content.len()));
		}

		log::debug!(
			"[IME] replace_and_mark_text_in_range: AFTER content={:?}, cursor={}, marked_range={:?}, selection={:?}",
			state.content.to_string(),
			state.cursor_position(),
			state.marked_range,
			state.selection_tuple()
		);

		update_input_state(element_id, state.clone());

		// Dispatch input event with isComposing = true
		dispatch_event_to_js(
			self.window_id,
			element_id,
			types::INPUT,
			EventData::Input(InputEventData {
				value:        state.content.to_string(),
				data:         Some(new_text.to_string()),
				input_type:   "insertCompositionText".to_string(),
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

		let start_slice = state.content.slice(0..range.start.min(state.content.len()));
		let end_slice = state.content.slice(0..range.end.min(state.content.len()));
		let start_char_count = start_slice.to_string().chars().count();
		let end_char_count = end_slice.to_string().chars().count();

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
		let content_str = state.content.to_string();
		let byte_offset: usize =
			content_str.char_indices().nth(char_index).map(|(idx, _)| idx).unwrap_or(state.content.len());

		Some(state.offset_to_utf16(byte_offset))
	}
}
