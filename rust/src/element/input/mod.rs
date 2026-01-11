//! Input element implementation
//!
//! A text input element that supports text editing, cursor, and selection.

pub mod cursor;
pub mod handler;
pub mod state;

use std::{collections::HashMap, sync::{Arc, Mutex}};

pub use cursor::BlinkCursor;
use gpui::{App, BorderStyle, Bounds, Element, ElementId, Font, FontStyle, FontWeight, GlobalElementId, Hitbox, HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId, PaintQuad, Pixels, Point, ShapedLine, TextRun, Window, point, px, rgb, size};
pub use handler::RootInputHandler;
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

	/// Store text shaping info for click-to-cursor positioning
	/// Key: element global_id, Value: (display_text, font_size, font_weight, text_origin_x)
	static ref TEXT_SHAPING_INFO: Mutex<HashMap<u64, TextShapingInfo>> = Mutex::new(HashMap::new());
}

/// Text shaping info for click-to-cursor positioning
#[derive(Clone)]
pub struct TextShapingInfo {
	pub display_text: String,
	pub font_size: f32,
	pub font_weight: u32,
	pub text_origin_x: f32,
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
	let result = states.get(&element_id).cloned();
	if result.is_none() {
		log::trace!("[Input] get_input_state: no state for element_id={}", element_id);
	}
	result
}

/// Remove input state when element is removed
pub fn remove_input_state(element_id: u64) {
	INPUT_STATES.lock().unwrap().remove(&element_id);
	BLINK_CURSORS.lock().unwrap().remove(&element_id);
	SELECTING.lock().unwrap().remove(&element_id);
	TEXT_SHAPING_INFO.lock().unwrap().remove(&element_id);
}

/// Get text shaping info for an element
pub fn get_text_shaping_info(element_id: u64) -> Option<TextShapingInfo> {
	TEXT_SHAPING_INFO.lock().unwrap().get(&element_id).cloned()
}

/// Update text shaping info for an element
pub fn update_text_shaping_info(element_id: u64, info: TextShapingInfo) {
	TEXT_SHAPING_INFO.lock().unwrap().insert(element_id, info);
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
}

/// State returned from request_layout
pub struct InputLayoutState {
	child_layout_id:  Option<LayoutId>,
	input_type:       InputType,
	display_text:     String,
	/// The actual content (not display text) - used for cursor calculation
	content:          String,
	/// Cursor position at layout time - ensures consistency with display_text
	cursor_position:  usize,
	text_color:       u32,
	text_size:        f32,
	font_weight:      Option<u32>,
}

/// State returned from prepaint
pub struct InputPrepaintState {
	hitbox:       Option<Hitbox>,
	event_flags:  EventHandlerFlags,
	input_state:  InputState,
	blink_cursor: BlinkCursor,
	shaped_line:  Option<ShapedLine>,
	text_origin:  Point<Pixels>,
}

impl ReactInputElement {
	pub fn new(
		element: Arc<ReactElement>,
		window_id: u64,
		parent_style: Option<ElementStyle>,
	) -> Self {
		Self { element, window_id, parent_style }
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

		log::debug!(
			"[Input] request_layout: element_id={}, initial cursor={}, content={:?}",
			self.element.global_id, input_state.cursor_position, input_state.content
		);

		// Update input type in case it changed
		input_state.input_type = input_type;

		// Update input state from props if this is a controlled input
		if let Some(value) = self.get_value() {
			// Only update if different (avoid cursor reset)
			if input_state.content != value {
				let old_cursor = input_state.cursor_position;
				input_state.content = value.to_string();
				input_state.cursor_position = input_state.cursor_position.min(value.len());
				log::debug!(
					"[Input] request_layout: value prop changed content, old_cursor={}, new_cursor={}, value={:?}",
					old_cursor, input_state.cursor_position, value
				);
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
		let is_placeholder = input_state.content.is_empty();
		let display_text = if is_placeholder {
			input_state.placeholder.clone().unwrap_or_default()
		} else {
			input_state.display_text()
		};

		// Text styling
		let text_color = if is_placeholder {
			0x888888 // Gray for placeholder
		} else {
			effective.text_color.unwrap_or(0xffffff)
		};
		let text_size = effective.text_size.unwrap_or(14.0);
		let font_weight = effective.font_weight;

		// Build style
		let mut style = self.element.build_gpui_style(Some(0x333333));

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

		// Request layout without text child - we'll paint text directly
		let layout_id = window.request_layout(style, std::iter::empty(), cx);

		(layout_id, InputLayoutState {
			child_layout_id: None,
			input_type,
			display_text,
			content: input_state.content.clone(),
			cursor_position: input_state.cursor_position,
			text_color,
			text_size,
			font_weight,
		})
	}

	fn prepaint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		request_layout: &mut Self::RequestLayoutState,
		window: &mut Window,
		_cx: &mut App,
	) -> Self::PrepaintState {
		// Get input state and blink cursor - this is the CURRENT state
		let input_state =
			get_or_create_input_state(self.element.global_id, None, request_layout.input_type);
		let mut blink_cursor = get_or_create_blink_cursor(self.element.global_id);

		// Update request_layout with current state to ensure consistency
		// between shaped_line and cursor calculation in paint
		request_layout.content = input_state.content.clone();
		request_layout.cursor_position = input_state.cursor_position;

		// Recalculate display_text with current content
		let is_placeholder = input_state.content.is_empty();
		let current_display_text = if is_placeholder {
			input_state.placeholder.clone().unwrap_or_default()
		} else {
			input_state.display_text()
		};
		request_layout.display_text = current_display_text.clone();

		// Update blink cursor with current time
		let current_time_ms = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_millis() as u64)
			.unwrap_or(0);
		let needs_repaint = blink_cursor.update(current_time_ms);
		update_blink_cursor(self.element.global_id, blink_cursor.clone());

		// Schedule repaint for cursor blinking if focused
		let is_focused = focus::is_focused(self.window_id, self.element.global_id);
		if is_focused && needs_repaint {
			window.refresh();
		}

		// Shape the text for accurate cursor positioning
		let padding = px(8.0);
		let text_origin = point(bounds.origin.x + padding, bounds.origin.y + px(4.0));

		// Create font for text shaping
		let font_weight_val = request_layout.font_weight.unwrap_or(400);
		let font = Font {
			family: "Zed Plex Mono".into(),
			features: Default::default(),
			fallbacks: None,
			weight: FontWeight(font_weight_val as f32),
			style: FontStyle::Normal,
		};

		let font_size = px(request_layout.text_size);
		let text_color = Hsla::from(rgb(request_layout.text_color));

		// Create text run for the display text - use current_display_text
		let text_run = TextRun {
			len: current_display_text.len(),
			font,
			color: text_color,
			background_color: None,
			underline: None,
			strikethrough: None,
		};

		// Shape the line using GPUI's text system with current display text
		let shaped_line = if !current_display_text.is_empty() {
			Some(window
				.text_system()
				.shape_line(
					current_display_text.into(),
					font_size,
					&[text_run],
					None,
				))
		} else {
			None
		};

		// Store text shaping info for mouse handlers
		update_text_shaping_info(self.element.global_id, TextShapingInfo {
			display_text: request_layout.display_text.clone(),
			font_size: request_layout.text_size,
			font_weight: font_weight_val,
			text_origin_x: text_origin.x.into(),
		});

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

		InputPrepaintState { hitbox, event_flags, input_state, blink_cursor, shaped_line, text_origin }
	}

	fn paint(
		&mut self,
		_id: Option<&GlobalElementId>,
		_inspector_id: Option<&InspectorElementId>,
		bounds: Bounds<Pixels>,
		request_layout: &mut Self::RequestLayoutState,
		prepaint: &mut Self::PrepaintState,
		window: &mut Window,
		cx: &mut App,
	) {
		let style = self.element.build_gpui_style(Some(0x333333));
		let is_focused = focus::is_focused(self.window_id, self.element.global_id);
		let text_origin = prepaint.text_origin;

		// Paint background
		style.paint(bounds, window, cx, |window, cx| {
			// Paint text using shaped line
			if let Some(ref shaped_line) = prepaint.shaped_line {
				// Calculate vertical center for text using window's line height
				let line_height = window.line_height();
				let text_y = bounds.origin.y + (bounds.size.height - line_height) / 2.0;
				let paint_origin = point(text_origin.x, text_y);

				let _ = shaped_line.paint(paint_origin, line_height, window, cx);
			}

			// Only paint cursor and selection when focused
			if is_focused {
				// Paint selection highlight if any
				if let Some((start, end)) = prepaint.input_state.selection {
					// Get x positions from shaped line
					// Use request_layout.content for consistency with shaped_line
					let (x_start, x_end) = if let Some(ref shaped_line) = prepaint.shaped_line {
						let content = &request_layout.content;
						// Snap to valid char boundaries
						let safe_start = {
							let pos = start.min(content.len());
							let mut p = pos;
							while p > 0 && !content.is_char_boundary(p) { p -= 1; }
							p
						};
						let safe_end = {
							let pos = end.min(content.len());
							let mut p = pos;
							while p > 0 && !content.is_char_boundary(p) { p -= 1; }
							p
						};
						// Convert byte positions to character indices
						let start_chars = content[..safe_start].chars().count();
						let end_chars = content[..safe_end].chars().count();
						let start_x = shaped_line.x_for_index(start_chars);
						let end_x = shaped_line.x_for_index(end_chars);
						(text_origin.x + start_x, text_origin.x + end_x)
					} else {
						(text_origin.x, text_origin.x)
					};

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

				// Paint IME composition underline (marked range)
				if let Some((start, end)) = prepaint.input_state.marked_range {
					if let Some(ref shaped_line) = prepaint.shaped_line {
						// Use request_layout.content for consistency with shaped_line
						let content = &request_layout.content;
						// Snap to valid char boundaries
						let safe_start = {
							let pos = start.min(content.len());
							let mut p = pos;
							while p > 0 && !content.is_char_boundary(p) { p -= 1; }
							p
						};
						let safe_end = {
							let pos = end.min(content.len());
							let mut p = pos;
							while p > 0 && !content.is_char_boundary(p) { p -= 1; }
							p
						};
						// Convert byte positions to character indices
						let start_chars = content[..safe_start].chars().count();
						let end_chars = content[..safe_end].chars().count();
						let start_x = shaped_line.x_for_index(start_chars);
						let end_x = shaped_line.x_for_index(end_chars);

						let underline_y = bounds.origin.y + bounds.size.height - px(6.0);
						let underline_bounds = Bounds {
							origin: point(text_origin.x + start_x, underline_y),
							size:   size(end_x - start_x, px(2.0)),
						};

						window.paint_quad(PaintQuad {
							bounds:        underline_bounds,
							background:    Hsla::from(rgb(0x4a9eff)).into(),
							corner_radii:  gpui::Corners::default(),
							border_color:  Hsla::transparent_black(),
							border_widths: gpui::Edges::default(),
							border_style:  BorderStyle::default(),
						});
					}
				}

				// Paint cursor only when visible (blink animation)
				if prepaint.blink_cursor.is_visible() {
					// If content is empty (showing placeholder), cursor should be at the start
					// Use request_layout data for consistency with shaped_line
					let cursor_x = if request_layout.content.is_empty() {
						text_origin.x
					} else if let Some(ref shaped_line) = prepaint.shaped_line {
						let cursor_pos = request_layout.cursor_position;
						let content = &request_layout.content;

						// Convert byte position to character count for x_for_index
						// x_for_index expects a character/glyph index, not a byte index
						// First, ensure cursor_pos is on a valid char boundary
						let safe_cursor_pos = {
							let pos = cursor_pos.min(content.len());
							// Find a valid char boundary at or before pos
							let mut safe_pos = pos;
							while safe_pos > 0 && !content.is_char_boundary(safe_pos) {
								safe_pos -= 1;
							}
							safe_pos
						};

						let char_index = if request_layout.input_type == InputType::Password {
							// For password, each character becomes one bullet
							content[..safe_cursor_pos]
								.chars()
								.count()
						} else {
							// Count characters up to cursor byte position
							content[..safe_cursor_pos]
								.chars()
								.count()
						};

						log::debug!(
							"[Input] paint cursor: cursor_pos={}, safe_cursor_pos={}, content_len={}, char_index={}, content={:?}",
							cursor_pos, safe_cursor_pos, content.len(), char_index, content
						);

						// Use x_for_index with character index
						text_origin.x + shaped_line.x_for_index(char_index)
					} else {
						text_origin.x
					};

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
		_bounds: Bounds<Pixels>,
		window: &mut Window,
	) {
		let Some(hitbox) = hitbox else { return };

		let hitbox_clone = hitbox.clone();
		let element_id = self.element.global_id;
		let window_id = self.window_id;

		// Mouse down - set cursor position
		window.on_mouse_event(move |event: &gpui::MouseDownEvent, phase, window, _cx| {
			if phase == gpui::DispatchPhase::Bubble && hitbox_clone.is_hovered(window) {
				// Get text shaping info for accurate positioning
				let shaping_info = get_text_shaping_info(element_id);
				let click_x: f32 = event.position.x.into();

				// Update input state
				if let Some(mut state) = get_input_state(element_id) {
					// Find cursor position from click
					let byte_offset = if let Some(info) = shaping_info {
						// Get local x relative to text origin
						let local_x = click_x - info.text_origin_x;

						if local_x <= 0.0 || info.display_text.is_empty() {
							0
						} else {
							// Reshape text to get accurate positions
							let font = Font {
								family: "Zed Plex Mono".into(),
								features: Default::default(),
								fallbacks: None,
								weight: FontWeight(info.font_weight as f32),
								style: FontStyle::Normal,
							};
							let font_size = px(info.font_size);
							let text_run = TextRun {
								len: info.display_text.len(),
								font,
								color: Hsla::white(),
								background_color: None,
								underline: None,
								strikethrough: None,
							};

							let shaped_line = window.text_system().shape_line(
								info.display_text.clone().into(),
								font_size,
								&[text_run],
								None,
							);

							// Use GPUI's built-in closest_index_for_x for accurate positioning
							let closest_idx = shaped_line.closest_index_for_x(px(local_x));

							// For password input, map display position back to content position
							if state.input_type == InputType::Password {
								// Each character is one bullet, so char index = byte position in content
								let char_count = info.display_text[..closest_idx.min(info.display_text.len())].chars().count();
								state.content.char_indices()
									.nth(char_count)
									.map(|(idx, _)| idx)
									.unwrap_or(state.content.len())
							} else {
								closest_idx
							}
						}
					} else {
						0
					};

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
				log::debug!("[Input] MouseDown: setting focus window_id={}, element_id={}", window_id, element_id);
				focus::set_focus(window_id, element_id);

				// Request refresh
				window.refresh();
			}
		});

		// Mouse move - extend selection
		let hitbox_clone = hitbox.clone();
		window.on_mouse_event(move |event: &gpui::MouseMoveEvent, phase, window, _cx| {
			if phase == gpui::DispatchPhase::Bubble && is_selecting(element_id) {
				let shaping_info = get_text_shaping_info(element_id);
				let click_x: f32 = event.position.x.into();

				if let Some(mut state) = get_input_state(element_id) {
					let byte_offset = if let Some(info) = shaping_info {
						let local_x = click_x - info.text_origin_x;

						if local_x <= 0.0 || info.display_text.is_empty() {
							0
						} else {
							// Reshape text for accurate positioning
							let font = Font {
								family: "Zed Plex Mono".into(),
								features: Default::default(),
								fallbacks: None,
								weight: FontWeight(info.font_weight as f32),
								style: FontStyle::Normal,
							};
							let font_size = px(info.font_size);
							let text_run = TextRun {
								len: info.display_text.len(),
								font,
								color: Hsla::white(),
								background_color: None,
								underline: None,
								strikethrough: None,
							};

							let shaped_line = window.text_system().shape_line(
								info.display_text.clone().into(),
								font_size,
								&[text_run],
								None,
							);

							// Use GPUI's built-in closest_index_for_x for accurate positioning
							let closest_idx = shaped_line.closest_index_for_x(px(local_x));

							if state.input_type == InputType::Password {
								let char_count = info.display_text[..closest_idx.min(info.display_text.len())].chars().count();
								state.content.char_indices()
									.nth(char_count)
									.map(|(idx, _)| idx)
									.unwrap_or(state.content.len())
							} else {
								closest_idx
							}
						}
					} else {
						0
					};

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

		// NOTE: Regular character input is NOT handled here!
		// It goes through the InputHandler (replace_text_in_range) for proper IME support.
		// This includes single characters and space.

		_ => {
			// Key not handled by input element, return false to let it bubble up
			return false;
		}
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
