use std::{ops::Range, sync::Arc};

use gpui::{App, Bounds, DispatchPhase, Element, ElementId, GlobalElementId, HitboxBehavior, InputHandler, IntoElement, LayoutId, Modifiers, MouseDownEvent, Pixels, Point, SharedString, Size, TextRun, UTF16Selection, Window, fill, px, rgb};
use ropey::Rope;

use super::{ElementStyle, ReactElement, input::{CURSOR_WIDTH, InputMode, RopeExt, Selection}};
use crate::{event_types::{EventData, FocusEventData, InputEventData}, focus, global_state::GLOBAL_STATE, renderer::dispatch_event_to_js};

/// Root input handler that delegates to the focused input element.
/// This is used for IME support at the window level.
pub struct RootInputHandler {
	window_id: u64,
}

impl RootInputHandler {
	pub fn new(window_id: u64) -> Self { Self { window_id } }

	/// Get the currently focused input state, if any.
	fn get_focused_input_state(&self) -> Option<InputState> {
		let focused_id = focus::get_focused(self.window_id)?;
		let window = GLOBAL_STATE.get_window(self.window_id)?;

		let element_map =
			window.state().element_map.lock().expect("Failed to acquire element_map lock");
		let element = element_map.get(&focused_id)?;

		// Check if it's an input element
		if element.element_kind != super::ElementKind::Input {
			return None;
		}

		// Get the input state from the input state map
		let input_states =
			window.state().input_states.lock().expect("Failed to acquire input_states lock");
		input_states.get(&focused_id).cloned()
	}
}

impl InputHandler for RootInputHandler {
	fn selected_text_range(
		&mut self,
		_ignore_disabled_input: bool,
		_window: &mut Window,
		_cx: &mut App,
	) -> Option<UTF16Selection> {
		if let Some(state) = self.get_focused_input_state() {
			let range = state.selected_text_range();
			// Convert byte range to UTF-16 range
			let text = state.text.to_string();
			let start_utf16 = text[..range.start.min(text.len())].encode_utf16().count();
			let end_utf16 = text[..range.end.min(text.len())].encode_utf16().count();
			Some(UTF16Selection {
				range:    start_utf16..end_utf16,
				reversed: state.selection.is_reversed(),
			})
		} else {
			None
		}
	}

	fn marked_text_range(&mut self, _window: &mut Window, _cx: &mut App) -> Option<Range<usize>> {
		if let Some(state) = self.get_focused_input_state() { state.marked_text_range() } else { None }
	}

	fn text_for_range(
		&mut self,
		range_utf16: Range<usize>,
		_adjusted_range: &mut Option<Range<usize>>,
		_window: &mut Window,
		_cx: &mut App,
	) -> Option<String> {
		if let Some(state) = self.get_focused_input_state() {
			// Convert UTF-16 range to byte range
			let text = state.text.to_string();
			let chars: Vec<char> = text.chars().collect();
			let mut byte_start = 0;
			let mut byte_end = text.len();
			let mut utf16_idx = 0;

			for (i, ch) in chars.iter().enumerate() {
				if utf16_idx == range_utf16.start {
					byte_start = text.char_indices().nth(i).map(|(idx, _)| idx).unwrap_or(0);
				}
				utf16_idx += ch.len_utf16();
				if utf16_idx == range_utf16.end {
					byte_end = text.char_indices().nth(i + 1).map(|(idx, _)| idx).unwrap_or(text.len());
					break;
				}
			}

			Some(text[byte_start..byte_end].to_string())
		} else {
			None
		}
	}

	fn unmark_text(&mut self, window: &mut Window, cx: &mut App) {
		if let Some(mut state) = self.get_focused_input_state() {
			state.unmark_text(window, cx);
			self.dispatch_input_event(&state, "insertText");
		}
	}

	fn replace_text_in_range(
		&mut self,
		range: Option<Range<usize>>,
		text: &str,
		window: &mut Window,
		_cx: &mut App,
	) {
		log::debug!("[RootInputHandler] replace_text_in_range: range={:?}, text={:?}", range, text);

		let Some(focused_id) = focus::get_focused(self.window_id) else {
			log::debug!("[RootInputHandler] No focused element");
			return;
		};

		let Some(gpui_window) = GLOBAL_STATE.get_window(self.window_id) else {
			log::debug!("[RootInputHandler] Window not found");
			return;
		};

		// Check if it's an input element
		{
			let element_map =
				gpui_window.state().element_map.lock().expect("Failed to acquire element_map lock");

			if let Some(element) = element_map.get(&focused_id) {
				if element.element_kind != super::ElementKind::Input {
					log::debug!("[RootInputHandler] Focused element is not an input");
					return;
				}
			} else {
				log::debug!("[RootInputHandler] Focused element not found in map");
				return;
			}
		}

		// Get or create input state and modify it directly
		let value = {
			let mut input_states =
				gpui_window.state().input_states.lock().expect("Failed to acquire input_states lock");

			let state = input_states.entry(focused_id).or_insert_with(|| {
				let element_map =
					gpui_window.state().element_map.lock().expect("Failed to acquire element_map lock");
				let element = element_map.get(&focused_id).unwrap();
				InputState::new(focused_id)
					.with_text(element.style.value.as_deref().unwrap_or(""))
					.with_placeholder(element.style.placeholder.clone().unwrap_or_default())
					.with_mode(if element.style.multi_line == Some(true) {
						InputMode::multi_line()
					} else {
						InputMode::plain_text()
					})
					.with_masked(element.style.input_type.as_deref() == Some("password"))
					.with_disabled(element.style.disabled.unwrap_or(false))
			});

			// Modify state directly
			// For IME: prefer marked_range over selection when range is None
			let range =
				range.or_else(|| state.marked_range.clone()).unwrap_or_else(|| state.selection.range());
			let start = range.start.min(state.text.len_bytes());
			let end = range.end.min(state.text.len_bytes());

			log::debug!(
				"[RootInputHandler] replace_text_in_range: using range {:?}, marked_range={:?}",
				start..end,
				state.marked_range
			);

			state.text.replace(start..end, text);
			let new_cursor = start + text.len();
			state.selection = Selection::cursor(new_cursor);
			state.marked_range = None;

			log::debug!(
				"[RootInputHandler] Updated state: cursor={}, text_len={}",
				new_cursor,
				state.text.len_bytes()
			);

			state.text.to_string()
		};

		// Dispatch input event
		dispatch_event_to_js(
			self.window_id,
			focused_id,
			"input",
			EventData::Input(InputEventData {
				value,
				data: Some(text.to_string()),
				input_type: "insertText".to_string(),
				is_composing: false,
			}),
		);

		window.refresh();
	}

	fn replace_and_mark_text_in_range(
		&mut self,
		range: Option<Range<usize>>,
		new_text: &str,
		new_selected_range: Option<Range<usize>>,
		window: &mut Window,
		_cx: &mut App,
	) {
		log::debug!(
			"[RootInputHandler] replace_and_mark_text_in_range: range={:?}, text={:?}, selected={:?}",
			range,
			new_text,
			new_selected_range
		);

		let Some(focused_id) = focus::get_focused(self.window_id) else {
			return;
		};

		let Some(gpui_window) = GLOBAL_STATE.get_window(self.window_id) else {
			return;
		};

		// Check if it's an input element
		{
			let element_map =
				gpui_window.state().element_map.lock().expect("Failed to acquire element_map lock");

			if let Some(element) = element_map.get(&focused_id) {
				if element.element_kind != super::ElementKind::Input {
					return;
				}
			} else {
				return;
			}
		}

		// Get or create input state and modify it directly
		{
			let mut input_states =
				gpui_window.state().input_states.lock().expect("Failed to acquire input_states lock");

			let state = input_states.entry(focused_id).or_insert_with(|| {
				let element_map =
					gpui_window.state().element_map.lock().expect("Failed to acquire element_map lock");
				let element = element_map.get(&focused_id).unwrap();
				InputState {
					element_id:     focused_id,
					text:           Rope::from(element.style.value.as_deref().unwrap_or("")),
					selection:      Selection::cursor(
						element.style.value.as_ref().map(|v| v.len()).unwrap_or(0),
					),
					marked_range:   None,
					cursor_visible: true,
					mode:           if element.style.multi_line == Some(true) {
						InputMode::multi_line()
					} else {
						InputMode::plain_text()
					},
					masked:         element.style.input_type.as_deref() == Some("password"),
					placeholder:    element.style.placeholder.clone().unwrap_or_default(),
					disabled:       element.style.disabled.unwrap_or(false),
				}
			});

			// Modify state directly
			let range =
				range.or_else(|| state.marked_range.clone()).unwrap_or_else(|| state.selection.range());

			let start = range.start.min(state.text.len_bytes());
			let end = range.end.min(state.text.len_bytes());

			log::debug!(
				"[RootInputHandler] replace_and_mark: using range {:?}, text='{}', marked_range={:?}",
				start..end,
				new_text,
				state.marked_range
			);

			state.text.replace(start..end, new_text);

			let new_end = start + new_text.len();
			state.marked_range = if new_text.is_empty() { None } else { Some(start..new_end) };

			if let Some(sel) = new_selected_range {
				state.selection = Selection::new(start + sel.start, start + sel.end);
			} else {
				state.selection = Selection::cursor(new_end);
			}
		}

		// Refresh to show composition text
		window.refresh();
	}

	fn bounds_for_range(
		&mut self,
		range_utf16: Range<usize>,
		_window: &mut Window,
		_cx: &mut App,
	) -> Option<Bounds<Pixels>> {
		// Return bounds for IME candidate window positioning
		// This is a simplified implementation - ideally we'd compute actual text bounds
		if let Some(state) = self.get_focused_input_state() {
			// Get the input element bounds from the window state
			if let Some(gpui_window) = GLOBAL_STATE.get_window(self.window_id) {
				let element_map =
					gpui_window.state().element_map.lock().expect("Failed to acquire element_map lock");

				if let Some(element) = element_map.get(&state.element_id) {
					// Approximate position based on character width
					let char_width: f32 = 8.0;
					let x_offset = range_utf16.start as f32 * char_width;

					return Some(Bounds::new(
						Point::new(px(x_offset + 4.0), px(4.0)),
						Size::new(px(8.0), px(20.0)),
					));
				}
			}
		}
		None
	}

	fn character_index_for_point(
		&mut self,
		point: Point<Pixels>,
		_window: &mut Window,
		_cx: &mut App,
	) -> Option<usize> {
		// Convert screen position to character index
		// This is a simplified implementation
		let char_width: f32 = 8.0;
		let x: f32 = f32::from(point.x).max(0.0);
		Some((x / char_width) as usize)
	}
}

impl RootInputHandler {
	fn dispatch_input_event(&self, state: &InputState, input_type: &str) {
		let value = state.value();
		dispatch_event_to_js(
			self.window_id,
			state.element_id,
			"input",
			EventData::Input(InputEventData {
				value,
				data: None,
				input_type: input_type.to_string(),
				is_composing: false,
			}),
		);
	}
}

/// Input state for managing text editing in React input elements.
///
/// This struct holds the editing state (text, selection, IME composition) for
/// a single input element. State is stored in `WindowState.input_states` keyed
/// by element ID.
///
/// # Fields
///
/// * `element_id` - The React element ID this state belongs to
/// * `text` - The text content (ropey::Rope for efficient large-text
///   operations)
/// * `selection` - Current cursor/selection range in bytes
/// * `marked_range` - IME (Input Method Editor) composition range, used for
///   languages like Chinese, Japanese, Korean where composition happens before
///   text is finalized
/// * `cursor_visible` - Whether the cursor should be rendered (for blinking)
/// * `mode` - InputMode determining single/multi-line behavior
/// * `masked` - If true, display text as password bullets (•••)
/// * `placeholder` - Text shown when input is empty
/// * `disabled` - If true, user cannot edit the text
///
/// # Storage
///
/// Stored in `WindowState.input_states: Mutex<HashMap<u64, InputState>>`.
///
/// # Thread Safety
///
/// Safe for concurrent access via Mutex protection.
#[derive(Clone)]
pub struct InputState {
	pub element_id:     u64,
	pub text:           Rope,
	pub selection:      Selection,
	pub marked_range:   Option<Range<usize>>,
	pub cursor_visible: bool,
	pub mode:           InputMode,
	pub masked:         bool,
	pub placeholder:    String,
	pub disabled:       bool,
}

impl InputState {
	/// Create a new InputState with minimal defaults
	pub fn new(element_id: u64) -> Self {
		Self {
			element_id,
			text: Rope::from(""),
			selection: Selection::cursor(0),
			marked_range: None,
			cursor_visible: true,
			mode: InputMode::plain_text(),
			masked: false,
			placeholder: String::new(),
			disabled: false,
		}
	}

	/// Set text value (fluent)
	pub fn with_text(mut self, text: impl Into<String>) -> Self {
		self.text = Rope::from(text.into());
		self
	}

	/// Set placeholder (fluent)
	pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
		self.placeholder = placeholder.into();
		self
	}

	/// Set mode (fluent)
	pub fn with_mode(mut self, mode: InputMode) -> Self {
		self.mode = mode;
		self
	}

	/// Set masked state (fluent)
	pub fn with_masked(mut self, masked: bool) -> Self {
		self.masked = masked;
		self
	}

	/// Set disabled state (fluent)
	pub fn with_disabled(mut self, disabled: bool) -> Self {
		self.disabled = disabled;
		self
	}

	pub fn value(&self) -> String { self.text.to_string() }

	pub fn selected_text(&self) -> String {
		let range = self.selection.range();
		self.text.slice(range).to_string()
	}

	pub fn text_for_range(&self, range: Range<usize>) -> String {
		let start = range.start.min(self.text.len_bytes());
		let end = range.end.min(self.text.len_bytes());
		self.text.slice(start..end).to_string()
	}

	pub fn selected_text_range(&self) -> Range<usize> { self.selection.range() }

	pub fn marked_text_range(&self) -> Option<Range<usize>> { self.marked_range.clone() }

	pub fn unmark_text(&mut self, _window: &mut Window, _cx: &mut App) { self.marked_range = None; }

	pub fn replace_text_in_range(
		&mut self,
		range: Option<Range<usize>>,
		text: &str,
		_window: &mut Window,
		_cx: &mut App,
	) {
		let range = range.unwrap_or_else(|| self.selection.range());
		let start = range.start.min(self.text.len_bytes());
		let end = range.end.min(self.text.len_bytes());

		self.text.replace(start..end, text);
		let new_cursor = start + text.len();
		self.selection = Selection::cursor(new_cursor);
		self.marked_range = None;

		// Update global state
		self.sync_to_global_state();
	}

	pub fn replace_and_mark_text_in_range(
		&mut self,
		range: Option<Range<usize>>,
		new_text: &str,
		new_selected_range: Option<Range<usize>>,
		_window: &mut Window,
		_cx: &mut App,
	) {
		let range =
			range.or_else(|| self.marked_range.clone()).unwrap_or_else(|| self.selection.range());

		let start = range.start.min(self.text.len_bytes());
		let end = range.end.min(self.text.len_bytes());

		self.text.replace(start..end, new_text);

		let new_end = start + new_text.len();
		self.marked_range = if new_text.is_empty() { None } else { Some(start..new_end) };

		if let Some(sel) = new_selected_range {
			self.selection = Selection::new(start + sel.start, start + sel.end);
		} else {
			self.selection = Selection::cursor(new_end);
		}

		// Update global state
		self.sync_to_global_state();
	}

	fn sync_to_global_state(&self) {
		// This would update the global input state
		// For now, we rely on the element-level state
	}
}

/// Handle a key event for an input element.
/// Returns true if the event was handled.
pub fn handle_input_key_event(
	window_id: u64,
	element_id: u64,
	key: &str,
	modifiers: Modifiers,
	window: &mut Window,
) -> bool {
	let Some(gpui_window) = GLOBAL_STATE.get_window(window_id) else {
		return false;
	};

	let element_map =
		gpui_window.state().element_map.lock().expect("Failed to acquire element_map lock");

	let Some(element) = element_map.get(&element_id) else {
		return false;
	};

	// Check if it's an input element
	if element.element_kind != super::ElementKind::Input {
		return false;
	}

	// Check if disabled
	if element.style.disabled == Some(true) {
		return false;
	}

	drop(element_map);

	// Get or create input state
	let mut input_states =
		gpui_window.state().input_states.lock().expect("Failed to acquire input_states lock");

	let state = input_states.entry(element_id).or_insert_with(|| {
		let element_map =
			gpui_window.state().element_map.lock().expect("Failed to acquire element_map lock");
		let element = element_map.get(&element_id).unwrap();
		InputState::new(element_id)
			.with_text(element.style.value.as_deref().unwrap_or(""))
			.with_placeholder(element.style.placeholder.clone().unwrap_or_default())
			.with_mode(if element.style.multi_line == Some(true) {
				InputMode::multi_line()
			} else {
				InputMode::plain_text()
			})
			.with_masked(element.style.input_type.as_deref() == Some("password"))
			.with_disabled(element.style.disabled.unwrap_or(false))
	});

	let handled = match key {
		"backspace" => {
			if state.selection.is_empty() {
				let cursor = state.selection.end;
				if cursor > 0 {
					let prev = state.text.prev_grapheme_boundary(cursor);
					state.text.replace(prev..cursor, "");
					state.selection = Selection::cursor(prev);
				}
			} else {
				let range = state.selection.range();
				state.text.replace(range.clone(), "");
				state.selection = Selection::cursor(range.start);
			}
			true
		}
		"delete" => {
			if state.selection.is_empty() {
				let cursor = state.selection.end;
				if cursor < state.text.len_bytes() {
					let next = state.text.next_grapheme_boundary(cursor);
					state.text.replace(cursor..next, "");
				}
			} else {
				let range = state.selection.range();
				state.text.replace(range.clone(), "");
				state.selection = Selection::cursor(range.start);
			}
			true
		}
		"left" => {
			if modifiers.shift {
				let new_end = state.text.prev_grapheme_boundary(state.selection.end);
				state.selection = Selection::new(state.selection.start, new_end);
			} else if !state.selection.is_empty() {
				let (min, _) = state.selection.ordered();
				state.selection = Selection::cursor(min);
			} else {
				let new_pos = state.text.prev_grapheme_boundary(state.selection.end);
				state.selection = Selection::cursor(new_pos);
			}
			true
		}
		"right" => {
			if modifiers.shift {
				let new_end = state.text.next_grapheme_boundary(state.selection.end);
				state.selection = Selection::new(state.selection.start, new_end);
			} else if !state.selection.is_empty() {
				let (_, max) = state.selection.ordered();
				state.selection = Selection::cursor(max);
			} else {
				let new_pos = state.text.next_grapheme_boundary(state.selection.end);
				state.selection = Selection::cursor(new_pos);
			}
			true
		}
		"home" => {
			let point = state.text.offset_to_point(state.selection.end);
			let line_start = state.text.line_start_offset(point.row);
			if modifiers.shift {
				state.selection = Selection::new(state.selection.start, line_start);
			} else {
				state.selection = Selection::cursor(line_start);
			}
			true
		}
		"end" => {
			let point = state.text.offset_to_point(state.selection.end);
			let line_end = state.text.line_end_offset(point.row);
			if modifiers.shift {
				state.selection = Selection::new(state.selection.start, line_end);
			} else {
				state.selection = Selection::cursor(line_end);
			}
			true
		}
		"a" if modifiers.control || modifiers.platform => {
			// Select all
			state.selection = Selection::new(0, state.text.len_bytes());
			true
		}
		"enter" => {
			if state.mode.is_multi_line() {
				let range = state.selection.range();
				state.text.replace(range.clone(), "\n");
				state.selection = Selection::cursor(range.start + 1);
				true
			} else {
				// Single line - dispatch submit event
				false
			}
		}
		_ => false,
	};

	if handled {
		// Dispatch input event
		let value = state.text.to_string();
		drop(input_states);

		dispatch_event_to_js(
			window_id,
			element_id,
			"input",
			EventData::Input(InputEventData {
				value,
				data: Some(key.to_string()),
				input_type: "insertText".to_string(),
				is_composing: false,
			}),
		);

		window.refresh();
	}

	handled
}

/// React Input Element implementation.
pub struct ReactInputElement {
	element:      Arc<ReactElement>,
	window_id:    u64,
	parent_style: Option<ElementStyle>,
}

impl ReactInputElement {
	pub fn new(
		element: Arc<ReactElement>,
		window_id: u64,
		parent_style: Option<ElementStyle>,
	) -> Self {
		Self { element, window_id, parent_style }
	}
}

impl IntoElement for ReactInputElement {
	type Element = Self;

	fn into_element(self) -> Self::Element { self }
}

/// Prepaint state for input element.
pub struct InputPrepaintState {
	cursor_bounds:    Option<Bounds<Pixels>>,
	selection_bounds: Option<Bounds<Pixels>>,
}

impl Element for ReactInputElement {
	type PrepaintState = InputPrepaintState;
	type RequestLayoutState = ();

	fn id(&self) -> Option<ElementId> { Some(ElementId::from(self.element.global_id as usize)) }

	fn source_location(&self) -> Option<&'static std::panic::Location<'static>> { None }

	fn request_layout(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&gpui::InspectorElementId>,
		_window: &mut Window,
		_cx: &mut App,
	) -> (LayoutId, Self::RequestLayoutState) {
		let style = self.element.build_gpui_style(Some(0x333333));
		(_window.request_layout(style, [], _cx), ())
	}

	fn prepaint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&gpui::InspectorElementId>,
		bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		window: &mut Window,
		cx: &mut App,
	) -> Self::PrepaintState {
		let element_id = self.element.global_id;
		let line_height = window.line_height();

		// Get or create input state
		let Some(gpui_window) = GLOBAL_STATE.get_window(self.window_id) else {
			return InputPrepaintState { cursor_bounds: None, selection_bounds: None };
		};

		let mut input_states =
			gpui_window.state().input_states.lock().expect("Failed to acquire input_states lock");

		let state = input_states.entry(element_id).or_insert_with(|| {
			InputState::new(element_id)
				.with_text(self.element.style.value.as_deref().unwrap_or(""))
				.with_placeholder(self.element.style.placeholder.clone().unwrap_or_default())
				.with_mode(if self.element.style.multi_line == Some(true) {
					InputMode::multi_line()
				} else {
					InputMode::plain_text()
				})
				.with_masked(self.element.style.input_type.as_deref() == Some("password"))
				.with_disabled(self.element.style.disabled.unwrap_or(false))
		});

		// Calculate cursor bounds
		let cursor = state.selection.end;
		let cursor_x = cursor as f32 * 8.0; // Approximate char width
		let cursor_bounds = Some(Bounds::new(
			bounds.origin + Point::new(px(cursor_x + 4.0), px(4.0)),
			Size::new(CURSOR_WIDTH, line_height),
		));

		// Calculate selection bounds if there's a selection
		let selection_bounds = if !state.selection.is_empty() {
			let (start, end) = state.selection.ordered();
			let start_x = start as f32 * 8.0;
			let end_x = end as f32 * 8.0;
			Some(Bounds::new(
				bounds.origin + Point::new(px(start_x + 4.0), px(4.0)),
				Size::new(px(end_x - start_x), line_height),
			))
		} else {
			None
		};

		InputPrepaintState { cursor_bounds, selection_bounds }
	}

	fn paint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&gpui::InspectorElementId>,
		bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		prepaint: &mut Self::PrepaintState,
		window: &mut Window,
		cx: &mut App,
	) {
		let element_id = self.element.global_id;
		let window_id = self.window_id;

		// Create hitbox for mouse events
		let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);

		// Register mouse click event for focus
		{
			let hitbox = hitbox.clone();
			window.on_mouse_event(move |_event: &MouseDownEvent, phase, window, _cx| {
				if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
					log::debug!("[Input] MouseDown: window_id={}, element_id={}", window_id, element_id);

					// Set focus to this input element
					let (blur_id, focus_id) = focus::set_focus(window_id, element_id);

					// Dispatch blur event to previously focused element
					if let Some(blur_element_id) = blur_id {
						if blur_element_id != element_id {
							dispatch_event_to_js(
								window_id,
								blur_element_id,
								"blur",
								EventData::Focus(FocusEventData { related_target: Some(element_id) }),
							);
						}
					}

					// Dispatch focus event to this element
					if let Some(focus_element_id) = focus_id {
						if blur_id.is_none() || blur_id != Some(element_id) {
							dispatch_event_to_js(
								window_id,
								focus_element_id,
								"focus",
								EventData::Focus(FocusEventData { related_target: blur_id }),
							);
						}
					}

					window.refresh();
				}
			});
		}

		// Paint background
		let style = self.element.build_gpui_style(Some(0x333333));
		if let Some(bg) = &style.background {
			// Extract the Background from Fill
			let background = match bg {
				gpui::Fill::Color(color) => color.clone(),
			};
			window.paint_quad(fill(bounds, background));
		}

		// Paint border - skip for now as AbsoluteLength conversion is complex
		// We'll simplify and just draw the content

		// Get input state
		let Some(gpui_window) = GLOBAL_STATE.get_window(self.window_id) else {
			return;
		};

		let input_states =
			gpui_window.state().input_states.lock().expect("Failed to acquire input_states lock");

		let Some(state) = input_states.get(&element_id) else {
			return;
		};

		// Clone the data we need before dropping the lock
		let is_empty = state.text.len_bytes() == 0;
		let is_disabled = state.disabled;
		let display_text = if is_empty {
			state.placeholder.clone()
		} else if state.masked {
			"*".repeat(state.text.chars().count())
		} else {
			state.text.to_string()
		};

		let text_color = if is_empty {
			rgb(0x888888) // Placeholder color
		} else {
			rgb(0xffffff) // Text color
		};

		drop(input_states);

		// Paint selection background (before text)
		if let Some(selection_bounds) = prepaint.selection_bounds {
			window.paint_quad(fill(selection_bounds, rgb(0x264f78))); // Blue selection color
		}

		// Paint text
		if !display_text.is_empty() {
			let text_style = window.text_style();
			let font_size = text_style.font_size.to_pixels(window.rem_size());
			let line_height = window.line_height();

			let shaped_line = window.text_system().shape_line(
				SharedString::from(display_text.clone()),
				font_size,
				&[TextRun {
					len:              display_text.len(),
					font:             text_style.font(),
					color:            text_color.into(),
					background_color: None,
					underline:        None,
					strikethrough:    None,
				}],
				None,
			);

			let text_origin = bounds.origin + Point::new(px(4.0), px(4.0));
			let _ = shaped_line.paint(text_origin, line_height, window, cx);
		}

		// Paint cursor if focused
		let is_focused = focus::get_focused(self.window_id) == Some(element_id);
		if is_focused && !is_disabled {
			// Get cursor blink state
			let cursor_visible = {
				let blink_epoch = std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)
					.map(|d| d.as_millis() / 500)
					.unwrap_or(0);
				blink_epoch % 2 == 0
			};

			if cursor_visible {
				if let Some(cursor_bounds) = prepaint.cursor_bounds {
					window.paint_quad(fill(cursor_bounds, rgb(0xffffff)));
				}
			}
		}
	}
}
