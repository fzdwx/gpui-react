//! Shared event handling for all element types
//!
//! This module provides common event handling functionality that can be used
//! by div, span, img, text and other element types.

use gpui::{
	Bounds, DispatchPhase, Hitbox, HitboxBehavior, KeyDownEvent, KeyUpEvent, MouseButton,
	MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, ScrollWheelEvent, Window,
};

use crate::event_types::{props, types};
use crate::renderer::{
	EventData, KeyboardEventData, MouseEventData, ScrollEventData, dispatch_event_to_js,
};

/// Flags indicating which event handlers are registered
pub struct EventHandlerFlags {
	pub has_click: bool,
	pub has_mouse_down: bool,
	pub has_mouse_up: bool,
	pub has_mouse_move: bool,
	pub has_mouse_enter: bool,
	pub has_mouse_leave: bool,
	pub has_key_down: bool,
	pub has_key_up: bool,
	pub has_scroll: bool,
	pub has_wheel: bool,
}

impl EventHandlerFlags {
	/// Create flags from event_handlers JSON value
	pub fn from_handlers(event_handlers: Option<&serde_json::Value>) -> Self {
		let has = |prop: &str| -> bool {
			event_handlers
				.and_then(|v| v.get(prop))
				.is_some()
		};

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
	pub fn has_any_scroll_handler(&self) -> bool {
		self.has_scroll || self.has_wheel
	}

	/// Check if any handler requires a hitbox
	pub fn needs_hitbox(&self) -> bool {
		self.has_any_mouse_handler() || self.has_any_scroll_handler()
	}

	/// Check if any keyboard handler is registered
	pub fn has_any_keyboard_handler(&self) -> bool {
		self.has_key_down || self.has_key_up
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
	// Register mouse event handlers (require hitbox)
	if let Some(hitbox) = hitbox {
		register_mouse_handlers(flags, hitbox, window_id, element_id, window);
		register_scroll_handlers(flags, hitbox, window_id, element_id, window);
	}

	// Register keyboard event handlers (no hitbox needed)
	register_keyboard_handlers(flags, window_id, element_id, window);
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
				let client_x: f32 = position.x.into();
				let client_y: f32 = position.y.into();

				let event_data = EventData::Mouse(MouseEventData {
					client_x,
					client_y,
					button: mouse_button_to_u8(event.button),
				});

				log::debug!(
					"[Rust] onMouseDown: window_id={}, element_id={}, position=({}, {})",
					window_id, element_id, client_x, client_y
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
				let client_x: f32 = position.x.into();
				let client_y: f32 = position.y.into();

				let event_data = EventData::Mouse(MouseEventData {
					client_x,
					client_y,
					button: mouse_button_to_u8(event.button),
				});

				// Dispatch mouseup event
				if has_mouse_up {
					log::debug!(
						"[Rust] onMouseUp: window_id={}, element_id={}, position=({}, {})",
						window_id, element_id, client_x, client_y
					);
					dispatch_event_to_js(window_id, element_id, types::MOUSEUP, event_data.clone());
				}

				// Dispatch click event (only for left button)
				if has_click && event.button == MouseButton::Left {
					log::info!(
						"[Rust] onClick: window_id={}, element_id={}, position=({}, {})",
						window_id, element_id, client_x, client_y
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
				let client_x: f32 = position.x.into();
				let client_y: f32 = position.y.into();

				let event_data = EventData::Mouse(MouseEventData {
					client_x,
					client_y,
					button: 0, // No button for move events
				});

				log::trace!(
					"[Rust] onMouseMove: window_id={}, element_id={}, position=({}, {})",
					window_id, element_id, client_x, client_y
				);
				dispatch_event_to_js(window_id, element_id, types::MOUSEMOVE, event_data);
			}
		});
	}

	// TODO: MouseEnter/MouseLeave require tracking hover state changes
}

/// Register keyboard event handlers
fn register_keyboard_handlers(
	flags: &EventHandlerFlags,
	window_id: u64,
	element_id: u64,
	window: &mut Window,
) {
	let has_key_down = flags.has_key_down;
	let has_key_up = flags.has_key_up;

	if !has_key_down && !has_key_up {
		return;
	}

	// KeyDown handler
	if has_key_down {
		window.on_key_event(move |event: &KeyDownEvent, phase, _window, _cx| {
			if phase == DispatchPhase::Bubble {
				let keystroke = &event.keystroke;
				let event_data = EventData::Keyboard(KeyboardEventData {
					key: keystroke.key.clone(),
					code: keystroke.key.clone(),
					repeat: false,
					ctrl: keystroke.modifiers.control,
					shift: keystroke.modifiers.shift,
					alt: keystroke.modifiers.alt,
					meta: keystroke.modifiers.platform,
				});

				log::debug!(
					"[Rust] onKeyDown: window_id={}, element_id={}, key={}",
					window_id, element_id, keystroke.key
				);
				dispatch_event_to_js(window_id, element_id, types::KEYDOWN, event_data);
			}
		});
	}

	// KeyUp handler
	if has_key_up {
		window.on_key_event(move |event: &KeyUpEvent, phase, _window, _cx| {
			if phase == DispatchPhase::Bubble {
				let keystroke = &event.keystroke;
				let event_data = EventData::Keyboard(KeyboardEventData {
					key: keystroke.key.clone(),
					code: keystroke.key.clone(),
					repeat: false,
					ctrl: keystroke.modifiers.control,
					shift: keystroke.modifiers.shift,
					alt: keystroke.modifiers.alt,
					meta: keystroke.modifiers.platform,
				});

				log::debug!(
					"[Rust] onKeyUp: window_id={}, element_id={}, key={}",
					window_id, element_id, keystroke.key
				);
				dispatch_event_to_js(window_id, element_id, types::KEYUP, event_data);
			}
		});
	}
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

			let event_data = EventData::Scroll(ScrollEventData {
				delta_x,
				delta_y,
				delta_mode,
			});

			if has_scroll {
				log::debug!(
					"[Rust] onScroll: window_id={}, element_id={}, delta=({}, {})",
					window_id, element_id, delta_x, delta_y
				);
				dispatch_event_to_js(window_id, element_id, types::SCROLL, event_data.clone());
			}

			if has_wheel {
				log::debug!(
					"[Rust] onWheel: window_id={}, element_id={}, delta=({}, {})",
					window_id, element_id, delta_x, delta_y
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
