//! Shared event handling for all element types
//!
//! This module provides common event handling functionality that can be used
//! by div, span, img, text and other element types.

use gpui::{Bounds, DispatchPhase, Hitbox, HitboxBehavior, KeyDownEvent, KeyUpEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, ScrollWheelEvent, Window};

use crate::{event_types::{EventData, FocusEventData, KeyboardEventData, MouseEventData, ScrollEventData, props, types}, focus, hover::get_hover_state, renderer::dispatch_event_to_js};

/// Flags indicating which event handlers are registered
pub struct EventHandlerFlags {
	pub has_click:       bool,
	pub has_mouse_down:  bool,
	pub has_mouse_up:    bool,
	pub has_mouse_move:  bool,
	pub has_mouse_enter: bool,
	pub has_mouse_leave: bool,
	pub has_key_down:    bool,
	pub has_key_up:      bool,
	pub has_scroll:      bool,
	pub has_wheel:       bool,
	pub has_focus:       bool,
	pub has_blur:        bool,
	/// Tab index for focus management (-1 = programmatic only, 0+ = tab order)
	pub tab_index:       Option<i32>,
}

impl EventHandlerFlags {
	/// Create flags from event_handlers JSON value and tab_index
	pub fn from_handlers(event_handlers: Option<&serde_json::Value>, tab_index: Option<i32>) -> Self {
		let has = |prop: &str| -> bool { event_handlers.and_then(|v| v.get(prop)).is_some() };

		Self {
			has_click: has(props::ON_CLICK),
			has_mouse_down: has(props::ON_MOUSE_DOWN),
			has_mouse_up: has(props::ON_MOUSE_UP),
			has_mouse_move: has(props::ON_MOUSE_MOVE),
			has_mouse_enter: has(props::ON_MOUSE_ENTER),
			has_mouse_leave: has(props::ON_MOUSE_LEAVE),
			has_key_down: has(props::ON_KEY_DOWN),
			has_key_up: has(props::ON_KEY_UP),
			has_scroll: has(props::ON_SCROLL),
			has_wheel: has(props::ON_WHEEL),
			has_focus: has(props::ON_FOCUS),
			has_blur: has(props::ON_BLUR),
			tab_index,
		}
	}

	/// Check if any mouse event handler is registered
	pub fn has_any_mouse_handler(&self) -> bool {
		self.has_click
			|| self.has_mouse_down
			|| self.has_mouse_up
			|| self.has_mouse_move
			|| self.has_mouse_enter
			|| self.has_mouse_leave
	}

	/// Check if any scroll event handler is registered
	pub fn has_any_scroll_handler(&self) -> bool { self.has_scroll || self.has_wheel }

	/// Check if any handler requires a hitbox
	pub fn needs_hitbox(&self) -> bool {
		self.has_any_mouse_handler() || self.has_any_scroll_handler() || self.is_focusable()
	}

	/// Check if any keyboard handler is registered
	pub fn has_any_keyboard_handler(&self) -> bool { self.has_key_down || self.has_key_up }

	/// Check if element is focusable (has tabIndex)
	pub fn is_focusable(&self) -> bool { self.tab_index.is_some() }

	/// Check if focus-related event handlers or attributes are present
	pub fn needs_focus_handling(&self) -> bool {
		self.is_focusable() || self.has_any_keyboard_handler() || self.has_focus || self.has_blur
	}
}

/// Insert a hitbox if needed based on event handler flags
pub fn insert_hitbox_if_needed(
	flags: &EventHandlerFlags,
	bounds: Bounds<Pixels>,
	window: &mut Window,
) -> Option<Hitbox> {
	if flags.needs_hitbox() {
		Some(window.insert_hitbox(bounds, HitboxBehavior::Normal))
	} else {
		None
	}
}

/// Register all event handlers for an element
pub fn register_event_handlers(
	flags: &EventHandlerFlags,
	hitbox: Option<&Hitbox>,
	window_id: u64,
	element_id: u64,
	window: &mut Window,
) {
	// Register tab index for focus management
	if let Some(tab_index) = flags.tab_index {
		focus::register_tab_index(window_id, element_id, tab_index);
	}

	// Register mouse event handlers (require hitbox)
	if let Some(hitbox) = hitbox {
		register_mouse_handlers(flags, hitbox, window_id, element_id, window);
		register_scroll_handlers(flags, hitbox, window_id, element_id, window);
		register_hover_handlers(flags, hitbox, window_id, element_id, window);

		// Register focus-on-click for focusable elements
		if flags.is_focusable() {
			register_focus_on_click(flags, hitbox, window_id, element_id, window);
		}
	}

	// Note: Keyboard event handlers are now registered at the window level
	// via register_window_keyboard_handlers() in host_command.rs
}

/// Register mouse event handlers
fn register_mouse_handlers(
	flags: &EventHandlerFlags,
	hitbox: &Hitbox,
	window_id: u64,
	element_id: u64,
	window: &mut Window,
) {
	let has_click = flags.has_click;
	let has_mouse_down = flags.has_mouse_down;
	let has_mouse_up = flags.has_mouse_up;
	let has_mouse_move = flags.has_mouse_move;

	// MouseDown handler
	if has_mouse_down {
		let hitbox = hitbox.clone();
		window.on_mouse_event(move |event: &MouseDownEvent, phase, window, _cx| {
			if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
				let position = event.position;
				let bounds = hitbox.bounds;
				let client_x: f32 = position.x.into();
				let client_y: f32 = position.y.into();
				let offset_x: f32 = (position.x - bounds.origin.x).into();
				let offset_y: f32 = (position.y - bounds.origin.y).into();

				let event_data = EventData::Mouse(MouseEventData {
					client_x,
					client_y,
					offset_x,
					offset_y,
					button: mouse_button_to_u8(event.button),
				});

				log::debug!(
					"[Rust] onMouseDown: window_id={}, element_id={}, position=({}, {}), offset=({}, {})",
					window_id,
					element_id,
					client_x,
					client_y,
					offset_x,
					offset_y
				);
				dispatch_event_to_js(window_id, element_id, types::MOUSEDOWN, event_data);
			}
		});
	}

	// MouseUp and Click handlers (both use MouseUpEvent)
	if has_mouse_up || has_click {
		let hitbox = hitbox.clone();
		window.on_mouse_event(move |event: &MouseUpEvent, phase, window, _cx| {
			if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
				let position = event.position;
				let bounds = hitbox.bounds;
				let client_x: f32 = position.x.into();
				let client_y: f32 = position.y.into();
				let offset_x: f32 = (position.x - bounds.origin.x).into();
				let offset_y: f32 = (position.y - bounds.origin.y).into();

				let event_data = EventData::Mouse(MouseEventData {
					client_x,
					client_y,
					offset_x,
					offset_y,
					button: mouse_button_to_u8(event.button),
				});

				// Dispatch mouseup event
				if has_mouse_up {
					log::debug!(
						"[Rust] onMouseUp: window_id={}, element_id={}, position=({}, {}), offset=({}, {})",
						window_id,
						element_id,
						client_x,
						client_y,
						offset_x,
						offset_y
					);
					dispatch_event_to_js(window_id, element_id, types::MOUSEUP, event_data.clone());
				}

				// Dispatch click event (only for left button)
				if has_click && event.button == MouseButton::Left {
					log::info!(
						"[Rust] onClick: window_id={}, element_id={}, position=({}, {}), offset=({}, {})",
						window_id,
						element_id,
						client_x,
						client_y,
						offset_x,
						offset_y
					);
					dispatch_event_to_js(window_id, element_id, types::CLICK, event_data);
				}
			}
		});
	}

	// MouseMove handler
	if has_mouse_move {
		let hitbox = hitbox.clone();
		window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, _cx| {
			if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
				let position = event.position;
				let bounds = hitbox.bounds;
				let client_x: f32 = position.x.into();
				let client_y: f32 = position.y.into();
				let offset_x: f32 = (position.x - bounds.origin.x).into();
				let offset_y: f32 = (position.y - bounds.origin.y).into();

				let event_data = EventData::Mouse(MouseEventData {
					client_x,
					client_y,
					offset_x,
					offset_y,
					button: 0, // No button for move events
				});

				log::trace!(
					"[Rust] onMouseMove: window_id={}, element_id={}, position=({}, {}), offset=({}, {})",
					window_id,
					element_id,
					client_x,
					client_y,
					offset_x,
					offset_y
				);
				dispatch_event_to_js(window_id, element_id, types::MOUSEMOVE, event_data);
			}
		});
	}
}

/// Register hover event handlers (mouseenter/mouseleave)
fn register_hover_handlers(
	flags: &EventHandlerFlags,
	hitbox: &Hitbox,
	window_id: u64,
	element_id: u64,
	window: &mut Window,
) {
	let has_mouse_enter = flags.has_mouse_enter;
	let has_mouse_leave = flags.has_mouse_leave;

	if !has_mouse_enter && !has_mouse_leave {
		return;
	}

	let hitbox = hitbox.clone();

	// Use MouseMove event to track hover state changes
	window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, _cx| {
		if phase != DispatchPhase::Bubble {
			return;
		}

		let is_hovered = hitbox.is_hovered(window);
		let hover_state = get_hover_state();

		// Lock and check/update hover state
		if let Ok(mut state) = hover_state.lock() {
			let was_hovered = state.is_hovered(element_id);

			if is_hovered && !was_hovered {
				// Mouse entered
				state.set_hovered(element_id);
				if has_mouse_enter {
					let position = event.position;
					let bounds = hitbox.bounds;
					let event_data = EventData::Mouse(MouseEventData {
						client_x: position.x.into(),
						client_y: position.y.into(),
						offset_x: (position.x - bounds.origin.x).into(),
						offset_y: (position.y - bounds.origin.y).into(),
						button:   0,
					});
					log::debug!("[Rust] onMouseEnter: window_id={}, element_id={}", window_id, element_id);
					dispatch_event_to_js(window_id, element_id, types::MOUSEENTER, event_data);
				}
			} else if !is_hovered && was_hovered {
				// Mouse left
				state.set_not_hovered(element_id);
				if has_mouse_leave {
					let position = event.position;
					let bounds = hitbox.bounds;
					let event_data = EventData::Mouse(MouseEventData {
						client_x: position.x.into(),
						client_y: position.y.into(),
						offset_x: (position.x - bounds.origin.x).into(),
						offset_y: (position.y - bounds.origin.y).into(),
						button:   0,
					});
					log::debug!("[Rust] onMouseLeave: window_id={}, element_id={}", window_id, element_id);
					dispatch_event_to_js(window_id, element_id, types::MOUSELEAVE, event_data);
				}
			}
		}
	});
}

/// Register focus-on-click handler for focusable elements
fn register_focus_on_click(
	flags: &EventHandlerFlags,
	hitbox: &Hitbox,
	window_id: u64,
	element_id: u64,
	window: &mut Window,
) {
	let hitbox = hitbox.clone();
	let has_focus = flags.has_focus;
	let has_blur = flags.has_blur;

	window.on_mouse_event(move |_event: &MouseDownEvent, phase, window, _cx| {
		if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
			// Set focus to this element
			let (blur_id, focus_id) = focus::set_focus(window_id, element_id);

			// Dispatch blur event to previously focused element
			if let Some(blur_element_id) = blur_id {
				if blur_element_id != element_id {
					log::debug!("[Rust] onBlur: window_id={}, element_id={}", window_id, blur_element_id);
					dispatch_event_to_js(
						window_id,
						blur_element_id,
						types::BLUR,
						EventData::Focus(FocusEventData { related_target: Some(element_id) }),
					);
				}
			}

			// Dispatch focus event to this element
			if let Some(focus_element_id) = focus_id {
				if has_focus && (blur_id.is_none() || blur_id != Some(element_id)) {
					log::debug!("[Rust] onFocus: window_id={}, element_id={}", window_id, focus_element_id);
					dispatch_event_to_js(
						window_id,
						focus_element_id,
						types::FOCUS,
						EventData::Focus(FocusEventData { related_target: blur_id }),
					);
				}
			}

			// Suppress unused variable warning
			let _ = has_blur;
		}
	});
}

/// Register scroll/wheel event handlers
fn register_scroll_handlers(
	flags: &EventHandlerFlags,
	hitbox: &Hitbox,
	window_id: u64,
	element_id: u64,
	window: &mut Window,
) {
	let has_scroll = flags.has_scroll;
	let has_wheel = flags.has_wheel;

	if !has_scroll && !has_wheel {
		return;
	}

	let hitbox = hitbox.clone();
	window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, _cx| {
		if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
			let (delta_x, delta_y, delta_mode): (f32, f32, u8) = match &event.delta {
				gpui::ScrollDelta::Pixels(point) => (point.x.into(), point.y.into(), 0),
				gpui::ScrollDelta::Lines(point) => (point.x, point.y, 1),
			};

			let event_data = EventData::Scroll(ScrollEventData { delta_x, delta_y, delta_mode });

			if has_scroll {
				log::debug!(
					"[Rust] onScroll: window_id={}, element_id={}, delta=({}, {})",
					window_id,
					element_id,
					delta_x,
					delta_y
				);
				dispatch_event_to_js(window_id, element_id, types::SCROLL, event_data.clone());
			}

			if has_wheel {
				log::debug!(
					"[Rust] onWheel: window_id={}, element_id={}, delta=({}, {})",
					window_id,
					element_id,
					delta_x,
					delta_y
				);
				dispatch_event_to_js(window_id, element_id, types::WHEEL, event_data);
			}
		}
	});
}

/// Convert GPUI MouseButton to u8 (0=left, 1=middle, 2=right)
fn mouse_button_to_u8(button: MouseButton) -> u8 {
	match button {
		MouseButton::Left => 0,
		MouseButton::Middle => 1,
		MouseButton::Right => 2,
		MouseButton::Navigate(_) => 3,
	}
}

/// Register window-level keyboard event handlers
/// This should be called once when a window is created
/// Note: GPUI's on_key_event does not return a Subscription, the handler lives
/// for the duration of the Window's scope
pub fn register_window_keyboard_handlers(window_id: u64, window: &mut Window) {
	log::info!("[Rust] Registering window-level keyboard handlers for window {}", window_id);

	// KeyDown handler - handles Tab navigation and dispatches keydown to focused
	// element
	window.on_key_event(move |event: &KeyDownEvent, phase, _window, _cx| {
		if phase != DispatchPhase::Bubble {
			return;
		}

		let keystroke = &event.keystroke;
		log::debug!(
			"[Rust] Window {} received KeyDown: key={}, shift={}",
			window_id,
			keystroke.key,
			keystroke.modifiers.shift
		);

		// Get the currently focused element for this window
		let focused_element = focus::get_focused(window_id);

		// Handle Tab key for focus navigation
		if keystroke.key == "tab" {
			log::debug!(
				"[Rust] Tab key pressed, current focused={:?}, shift={}",
				focused_element,
				keystroke.modifiers.shift
			);

			let (blur_id, focus_id) = if keystroke.modifiers.shift {
				focus::focus_prev(window_id)
			} else {
				focus::focus_next(window_id)
			};

			log::debug!("[Rust] Focus navigation result: blur_id={:?}, focus_id={:?}", blur_id, focus_id);

			// Dispatch blur event
			if let Some(blur_element_id) = blur_id {
				dispatch_event_to_js(
					window_id,
					blur_element_id,
					types::BLUR,
					EventData::Focus(FocusEventData { related_target: focus_id }),
				);
			}

			// Dispatch focus event
			if let Some(focus_element_id) = focus_id {
				dispatch_event_to_js(
					window_id,
					focus_element_id,
					types::FOCUS,
					EventData::Focus(FocusEventData { related_target: blur_id }),
				);
			}

			return; // Don't dispatch Tab as keydown to the element
		}

		// Dispatch keydown event to the focused element
		if let Some(element_id) = focused_element {
			let event_data = EventData::Keyboard(KeyboardEventData {
				key:    keystroke.key.clone(),
				code:   keystroke.key.clone(),
				repeat: event.is_held,
				ctrl:   keystroke.modifiers.control,
				shift:  keystroke.modifiers.shift,
				alt:    keystroke.modifiers.alt,
				meta:   keystroke.modifiers.platform,
			});

			log::debug!(
				"[Rust] Dispatching onKeyDown to element_id={}, key={}",
				element_id,
				keystroke.key
			);
			dispatch_event_to_js(window_id, element_id, types::KEYDOWN, event_data);
		}
	});

	// KeyUp handler - dispatches keyup to focused element
	window.on_key_event(move |event: &KeyUpEvent, phase, _window, _cx| {
		if phase != DispatchPhase::Bubble {
			return;
		}

		// Get the currently focused element for this window
		let focused_element = focus::get_focused(window_id);

		// Dispatch keyup event to the focused element
		if let Some(element_id) = focused_element {
			let keystroke = &event.keystroke;
			let event_data = EventData::Keyboard(KeyboardEventData {
				key:    keystroke.key.clone(),
				code:   keystroke.key.clone(),
				repeat: false,
				ctrl:   keystroke.modifiers.control,
				shift:  keystroke.modifiers.shift,
				alt:    keystroke.modifiers.alt,
				meta:   keystroke.modifiers.platform,
			});

			log::debug!("[Rust] Dispatching onKeyUp to element_id={}, key={}", element_id, keystroke.key);
			dispatch_event_to_js(window_id, element_id, types::KEYUP, event_data);
		}
	});
}
