//! Input element implementation
//!
//! A text input element that supports text editing, cursor, and selection.

pub mod cursor;
pub mod state;

use std::{collections::HashMap, sync::{Arc, Mutex}};

pub use cursor::BlinkCursor;
use gpui::{AnyElement, App, BorderStyle, Bounds, Element, ElementId, GlobalElementId, Hitbox, HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId, PaintQuad, Pixels, Window, div, point, prelude::*, px, rgb, size};
use lazy_static::lazy_static;
pub use state::{InputState, InputType, TextChange};

use super::{ElementStyle, ReactElement, events::{EventHandlerFlags, register_event_handlers}};
use crate::{event_types::{EventData, InputEventData, types}, focus, renderer::dispatch_event_to_js};

lazy_static! {
	/// Global input state storage - persists across frames
	/// Key: element global_id
	static ref INPUT_STATES: Mutex<HashMap<u64, InputState>> = Mutex::new(HashMap::new());

	/// Global blink cursor storage
	static ref BLINK_CURSORS: Mutex<HashMap<u64, BlinkCursor>> = Mutex::new(HashMap::new());

	/// Track which elements are currently selecting with mouse
	static ref SELECTING: Mutex<HashMap<u64, bool>> = Mutex::new(HashMap::new());
}

/// Get or create input state for an element
pub fn get_or_create_input_state(
	element_id: u64,
	initial_value: Option<&str>,
	input_type: InputType,
) -> InputState {
	let mut states = INPUT_STATES.lock().unwrap();
	if let Some(state) = states.get(&element_id) {
		state.clone()
	} else {
		let mut state = if let Some(value) = initial_value {
			InputState::with_content(value.to_string())
		} else {
			InputState::new()
		};
		state.input_type = input_type;
		states.insert(element_id, state.clone());
		state
	}
}

/// Update input state for an element
pub fn update_input_state(element_id: u64, state: InputState) {
	let mut states = INPUT_STATES.lock().unwrap();
	states.insert(element_id, state);
}

/// Get input state by element ID
pub fn get_input_state(element_id: u64) -> Option<InputState> {
	let states = INPUT_STATES.lock().unwrap();
	states.get(&element_id).cloned()
}

/// Remove input state when element is removed
pub fn remove_input_state(element_id: u64) {
	INPUT_STATES.lock().unwrap().remove(&element_id);
	BLINK_CURSORS.lock().unwrap().remove(&element_id);
	SELECTING.lock().unwrap().remove(&element_id);
}

/// Get or create blink cursor for an element
pub fn get_or_create_blink_cursor(element_id: u64) -> BlinkCursor {
	let mut cursors = BLINK_CURSORS.lock().unwrap();
	if let Some(cursor) = cursors.get(&element_id) {
		cursor.clone()
	} else {
		let cursor = BlinkCursor::new();
		cursors.insert(element_id, cursor.clone());
		cursor
	}
}

/// Update blink cursor for an element
pub fn update_blink_cursor(element_id: u64, cursor: BlinkCursor) {
	let mut cursors = BLINK_CURSORS.lock().unwrap();
	cursors.insert(element_id, cursor);
}

/// Check if element is currently selecting
pub fn is_selecting(element_id: u64) -> bool {
	SELECTING.lock().unwrap().get(&element_id).copied().unwrap_or(false)
}

/// Set selecting state
pub fn set_selecting(element_id: u64, selecting: bool) {
	SELECTING.lock().unwrap().insert(element_id, selecting);
}

/// A React input element that implements GPUI's Element trait
pub struct ReactInputElement {
	element:      Arc<ReactElement>,
	window_id:    u64,
	parent_style: Option<ElementStyle>,
	text_child:   Option<AnyElement>,
}

/// State returned from request_layout
pub struct InputLayoutState {
	child_layout_id: Option<LayoutId>,
	text_width:      f32,
	input_type:      InputType,
}

/// State returned from prepaint
pub struct InputPrepaintState {
	hitbox:       Option<Hitbox>,
	event_flags:  EventHandlerFlags,
	input_state:  InputState,
	blink_cursor: BlinkCursor,
}

impl ReactInputElement {
	pub fn new(
		element: Arc<ReactElement>,
		window_id: u64,
		parent_style: Option<ElementStyle>,
	) -> Self {
		Self { element, window_id, parent_style, text_child: None }
	}

	/// Get the value from element style props
	fn get_value(&self) -> Option<&str> { self.element.style.value.as_deref() }

	/// Get placeholder text from element style props
	fn get_placeholder(&self) -> Option<&str> { self.element.style.placeholder.as_deref() }

	/// Get input type from element style props
	fn get_input_type(&self) -> InputType {
		self.element.style.input_type.as_deref().map(InputType::from_str).unwrap_or(InputType::Text)
	}
}

impl Element for ReactInputElement {
	type PrepaintState = InputPrepaintState;
	type RequestLayoutState = InputLayoutState;

	fn id(&self) -> Option<ElementId> { Some(ElementId::Integer(self.element.global_id)) }

	fn source_location(&self) -> Option<&'static std::panic::Location<'static>> { None }

	fn request_layout(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		window: &mut Window,
		cx: &mut App,
	) -> (LayoutId, Self::RequestLayoutState) {
		let effective = self.element.effective_style(self.parent_style.as_ref());
		let input_type = self.get_input_type();

		// Get or create input state
		let mut input_state =
			get_or_create_input_state(self.element.global_id, self.get_value(), input_type);

		// Update input type in case it changed
		input_state.input_type = input_type;

		// Update input state from props if this is a controlled input
		if let Some(value) = self.get_value() {
			// Only update if different (avoid cursor reset)
			if input_state.content != value {
				input_state.content = value.to_string();
				input_state.cursor_position = input_state.cursor_position.min(value.len());
			}
		}

		// Update placeholder
		input_state.placeholder = self.get_placeholder().map(|s| s.to_string());

		// Update disabled/readonly
		if let Some(disabled) = self.element.style.disabled {
			input_state.disabled = disabled;
		}
		if let Some(read_only) = self.element.style.read_only {
			input_state.read_only = read_only;
		}
		if let Some(max_length) = self.element.style.max_length {
			input_state.max_length = Some(max_length);
		}

		// Store updated state
		update_input_state(self.element.global_id, input_state.clone());

		// Determine display text (masked for password)
		let display_text = if input_state.content.is_empty() {
			input_state.placeholder.clone().unwrap_or_default()
		} else {
			input_state.display_text()
		};

		// Build style
		let mut style = self.element.build_gpui_style(Some(0x333333)); // Dark background for input

		// Ensure padding for text and cursor
		let padding = px(8.0);
		style.padding.left = padding.into();
		style.padding.right = padding.into();
		style.padding.top = px(4.0).into();
		style.padding.bottom = px(4.0).into();

		// Ensure we have some minimum size if not specified
		if matches!(style.size.width, gpui::Length::Auto) {
			style.size.width = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(200.0)),
			));
		}
		if matches!(style.size.height, gpui::Length::Auto) {
			style.size.height = gpui::Length::Definite(gpui::DefiniteLength::Absolute(
				gpui::AbsoluteLength::Pixels(px(32.0)),
			));
		}

		// Calculate text width for cursor positioning
		let text_width = display_text.len() as f32 * 8.0; // Approximate

		// Create text child element
		let child_layout_id = if !display_text.is_empty() {
			let text_color = if input_state.content.is_empty() {
				// Placeholder color (dimmed)
				effective.text_color.unwrap_or(0x888888)
			} else {
				effective.text_color.unwrap_or(0xffffff)
			};
			let text_size = effective.text_size.unwrap_or(14.0);

			let mut text_element =
				div().text_color(rgb(text_color)).text_size(px(text_size)).child(display_text);

			if let Some(weight) = effective.font_weight {
				text_element = text_element.font_weight(gpui::FontWeight(weight as f32));
			}

			let mut child = text_element.into_any_element();
			let layout_id = child.request_layout(window, cx);
			self.text_child = Some(child);
			Some(layout_id)
		} else {
			None
		};

		// Request our own layout
		let layout_id = if let Some(child_id) = child_layout_id {
			window.request_layout(style, std::iter::once(child_id), cx)
		} else {
			window.request_layout(style, std::iter::empty(), cx)
		};

		(layout_id, InputLayoutState { child_layout_id, text_width, input_type })
	}

	fn prepaint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		request_layout: &mut Self::RequestLayoutState,
		window: &mut Window,
		cx: &mut App,
	) -> Self::PrepaintState {
		// Prepaint text child
		if let Some(ref mut child) = self.text_child {
			child.prepaint(window, cx);
		}

		// Get input state and blink cursor
		let input_state =
			get_or_create_input_state(self.element.global_id, None, request_layout.input_type);
		let mut blink_cursor = get_or_create_blink_cursor(self.element.global_id);

		// Update blink cursor with current time
		let current_time_ms = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_millis() as u64)
			.unwrap_or(0);
		blink_cursor.update(current_time_ms);
		update_blink_cursor(self.element.global_id, blink_cursor.clone());

		// Build event flags - input always needs hitbox for interaction
		let mut event_flags = EventHandlerFlags::from_handlers(
			self.element.event_handlers.as_ref(),
			self.element.style.tab_index,
		);
		// Input elements are always focusable
		if event_flags.tab_index.is_none() {
			event_flags.tab_index = Some(0);
		}

		// Insert hitbox
		let hitbox = Some(window.insert_hitbox(bounds, HitboxBehavior::Normal));

		InputPrepaintState { hitbox, event_flags, input_state, blink_cursor }
	}

	fn paint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		_request_layout: &mut Self::RequestLayoutState,
		prepaint: &mut Self::PrepaintState,
		window: &mut Window,
		cx: &mut App,
	) {
		let style = self.element.build_gpui_style(Some(0x333333));
		let is_focused = focus::is_focused(self.window_id, self.element.global_id);
		let effective = self.element.effective_style(self.parent_style.as_ref());
		let text_size = effective.text_size.unwrap_or(14.0);
		let char_width = text_size * 0.6; // Approximate character width
		let padding: f32 = 8.0;

		// Paint background
		style.paint(bounds, window, cx, |window, cx| {
			// Paint text child
			if let Some(ref mut child) = self.text_child {
				child.paint(window, cx);
			}

			// Only paint cursor and selection when focused
			if is_focused {
				// Paint selection highlight if any
				if let Some((start, end)) = prepaint.input_state.selection {
					let start_graphemes = prepaint.input_state.content[..start].chars().count();
					let end_graphemes = prepaint.input_state.content[..end].chars().count();

					let x_start = bounds.origin.x + px(start_graphemes as f32 * char_width + padding);
					let x_end = bounds.origin.x + px(end_graphemes as f32 * char_width + padding);

					let selection_bounds = Bounds {
						origin: point(x_start, bounds.origin.y + px(4.0)),
						size:   size(x_end - x_start, bounds.size.height - px(8.0)),
					};

					window.paint_quad(PaintQuad {
						bounds:        selection_bounds,
						background:    Hsla::from(rgb(0x264f78)).into(),
						corner_radii:  gpui::Corners::default(),
						border_color:  Hsla::transparent_black(),
						border_widths: gpui::Edges::default(),
						border_style:  BorderStyle::default(),
					});
				}

				// Paint cursor (always visible when focused, no blink for now)
				let cursor_graphemes = if prepaint.input_state.content.is_empty() {
					0
				} else {
					prepaint.input_state.content[..prepaint.input_state.cursor_position].chars().count()
				};
				let cursor_x = bounds.origin.x + px(cursor_graphemes as f32 * char_width + padding);

				let cursor_bounds = Bounds {
					origin: point(cursor_x, bounds.origin.y + px(4.0)),
					size:   size(px(cursor::CURSOR_WIDTH), bounds.size.height - px(8.0)),
				};

				window.paint_quad(PaintQuad {
					bounds:        cursor_bounds,
					background:    Hsla::from(rgb(0xffffff)).into(),
					corner_radii:  gpui::Corners::default(),
					border_color:  Hsla::transparent_black(),
					border_widths: gpui::Edges::default(),
					border_style:  BorderStyle::default(),
				});
			}
		});

		// Paint focus border
		if is_focused {
			let border_bounds = bounds;
			window.paint_quad(PaintQuad {
				bounds:        border_bounds,
				background:    gpui::Background::default(),
				corner_radii:  gpui::Corners::all(px(4.0)),
				border_color:  Hsla::from(rgb(0x4a9eff)),
				border_widths: gpui::Edges::all(px(2.0)),
				border_style:  BorderStyle::default(),
			});
		}

		// Register event handlers
		register_event_handlers(
			&prepaint.event_flags,
			prepaint.hitbox.as_ref(),
			self.window_id,
			self.element.global_id,
			window,
		);

		// Register input-specific event handlers
		self.register_input_handlers(prepaint.hitbox.as_ref(), bounds, window);
	}
}

impl ReactInputElement {
	/// Register input-specific mouse handlers
	fn register_input_handlers(
		&self,
		hitbox: Option<&Hitbox>,
		bounds: Bounds<Pixels>,
		window: &mut Window,
	) {
		let Some(hitbox) = hitbox else { return };

		let hitbox_clone = hitbox.clone();
		let element_id = self.element.global_id;
		let window_id = self.window_id;
		let effective = self.element.effective_style(self.parent_style.as_ref());
		let text_size = effective.text_size.unwrap_or(14.0);
		let char_width = text_size * 0.6;
		let padding: f32 = 8.0;

		// Mouse down - set cursor position
		window.on_mouse_event(move |event: &gpui::MouseDownEvent, phase, window, _cx| {
			if phase == gpui::DispatchPhase::Bubble && hitbox_clone.is_hovered(window) {
				// Calculate character position from click
				let local_x: f32 = (event.position.x - bounds.origin.x - px(padding)).into();
				let char_pos = (local_x / char_width).round().max(0.0) as usize;

				// Update input state
				if let Some(mut state) = get_input_state(element_id) {
					// Convert char position to byte offset
					let byte_offset = state
						.content
						.char_indices()
						.nth(char_pos)
						.map(|(idx, _)| idx)
						.unwrap_or(state.content.len());

					state.set_cursor_from_offset(byte_offset);
					update_input_state(element_id, state);

					// Set selecting
					set_selecting(element_id, true);

					// Pause blink cursor
					let mut cursor = get_or_create_blink_cursor(element_id);
					cursor.pause_and_show();
					update_blink_cursor(element_id, cursor);
				}

				// Set focus
				focus::set_focus(window_id, element_id);

				// Request refresh
				window.refresh();
			}
		});

		// Mouse move - extend selection
		let hitbox_clone = hitbox.clone();
		window.on_mouse_event(move |event: &gpui::MouseMoveEvent, phase, window, _cx| {
			if phase == gpui::DispatchPhase::Bubble && is_selecting(element_id) {
				let local_x: f32 = (event.position.x - bounds.origin.x - px(padding)).into();
				let char_pos = (local_x / char_width).round().max(0.0) as usize;

				if let Some(mut state) = get_input_state(element_id) {
					let byte_offset = state
						.content
						.char_indices()
						.nth(char_pos)
						.map(|(idx, _)| idx)
						.unwrap_or(state.content.len());

					state.extend_selection_to(byte_offset);
					update_input_state(element_id, state);
					window.refresh();
				}
			}
			// Suppress unused warning
			let _ = hitbox_clone;
		});

		// Mouse up - end selection
		window.on_mouse_event(move |_event: &gpui::MouseUpEvent, phase, _window, _cx| {
			if phase == gpui::DispatchPhase::Bubble {
				set_selecting(element_id, false);
			}
		});
	}
}

impl IntoElement for ReactInputElement {
	type Element = Self;

	fn into_element(self) -> Self::Element { self }
}

/// Handle keyboard input for a focused input element
/// Called from the window-level keyboard handler
pub fn handle_input_key_event(
	window_id: u64,
	element_id: u64,
	key: &str,
	modifiers: gpui::Modifiers,
	window: &mut Window,
) -> bool {
	let Some(mut state) = get_input_state(element_id) else {
		return false;
	};

	let change: Option<TextChange> = match key {
		// Navigation
		"left" => {
			state.move_left(modifiers.shift);
			None
		}
		"right" => {
			state.move_right(modifiers.shift);
			None
		}
		"home" => {
			state.move_to_start(modifiers.shift);
			None
		}
		"end" => {
			state.move_to_end(modifiers.shift);
			None
		}

		// Editing
		"backspace" => state.backspace(),
		"delete" => state.delete(),

		// Select all (Ctrl/Cmd+A)
		"a" if modifiers.platform || modifiers.control => {
			state.select_all();
			None
		}

		// Copy (Ctrl/Cmd+C) - handled separately
		// Cut (Ctrl/Cmd+X)
		"x" if modifiers.platform || modifiers.control => {
			if let Some(_text) = state.cut_selection() {
				// TODO: Write to clipboard
				Some(TextChange {
					old_value:       state.content.clone(),
					new_value:       state.content.clone(),
					cursor_position: state.cursor_position,
					data:            None,
					input_type:      "deleteByCut",
				})
			} else {
				None
			}
		}

		// Regular character input
		key if key.len() == 1 && !modifiers.control && !modifiers.alt => state.insert_text(key),

		// Space
		"space" => state.insert_text(" "),

		_ => None,
	};

	// Update cursor state
	let mut cursor = get_or_create_blink_cursor(element_id);
	cursor.pause_and_show();
	update_blink_cursor(element_id, cursor);

	// Update input state
	update_input_state(element_id, state.clone());

	// Request window refresh to show changes immediately
	window.refresh();

	// Dispatch input event if text changed
	if let Some(change) = change {
		dispatch_event_to_js(
			window_id,
			element_id,
			types::INPUT,
			EventData::Input(InputEventData {
				value:        change.new_value,
				data:         change.data,
				input_type:   change.input_type.to_string(),
				is_composing: false,
			}),
		);
		return true;
	}

	// Return true to indicate we handled the key (even navigation keys need
	// refresh)
	true
}
