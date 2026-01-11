use gpui::{Application as GpuiApp, Entity, FocusHandle, InteractiveElement, KeyDownEvent, KeyUpEvent, Render, Task, Timer, Window, div, prelude::*, rgb};

use crate::{element::{create_element, input::{RootInputHandler, handle_input_key_event}}, event_types::{EventData, FocusEventData, KeyboardEventData, types}, focus, global_state::GLOBAL_STATE, host_command, window::EventMessage};

/// Dispatch an event to the event queue for JS polling
/// This is thread-safe and doesn't require calling JS directly from Rust
pub(crate) fn dispatch_event_to_js(
	window_id: u64,
	element_id: u64,
	event_type: &str,
	event_data: EventData,
) {
	let timestamp = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map(|d| d.as_millis() as u64)
		.unwrap_or(0);

	// Build JSON payload based on event data type
	let json_payload = match event_data {
		EventData::Mouse(data) => {
			serde_json::json!({
				"windowId": window_id,
				"elementId": element_id,
				"eventType": event_type,
				"clientX": data.client_x,
				"clientY": data.client_y,
				"offsetX": data.offset_x,
				"offsetY": data.offset_y,
				"button": data.button,
				"timestamp": timestamp
			})
		}
		EventData::Keyboard(data) => {
			serde_json::json!({
				"windowId": window_id,
				"elementId": element_id,
				"eventType": event_type,
				"key": data.key,
				"code": data.code,
				"repeat": data.repeat,
				"ctrlKey": data.ctrl,
				"shiftKey": data.shift,
				"altKey": data.alt,
				"metaKey": data.meta,
				"timestamp": timestamp
			})
		}
		EventData::Scroll(data) => {
			serde_json::json!({
				"windowId": window_id,
				"elementId": element_id,
				"eventType": event_type,
				"deltaX": data.delta_x,
				"deltaY": data.delta_y,
				"deltaMode": data.delta_mode,
				"timestamp": timestamp
			})
		}
		EventData::Focus(data) => {
			serde_json::json!({
				"windowId": window_id,
				"elementId": element_id,
				"eventType": event_type,
				"relatedTarget": data.related_target,
				"timestamp": timestamp
			})
		}
		EventData::Input(data) => {
			serde_json::json!({
				"windowId": window_id,
				"elementId": element_id,
				"eventType": event_type,
				"value": data.value,
				"data": data.data,
				"inputType": data.input_type,
				"isComposing": data.is_composing,
				"timestamp": timestamp
			})
		}
		EventData::None => {
			serde_json::json!({
				"windowId": window_id,
				"elementId": element_id,
				"eventType": event_type,
				"timestamp": timestamp
			})
		}
	};

	let json_str = json_payload.to_string();

	// Push event to window's event queue instead of calling JS directly
	if let Some(window) = GLOBAL_STATE.get_window(window_id) {
		window.state().push_event(EventMessage {
			window_id,
			element_id,
			event_type: event_type.to_string(),
			payload: json_str,
		});
		log::trace!(
			"[Rust] Event queued: window_id={}, element_id={}, event_type={}",
			window_id,
			element_id,
			event_type
		);
	} else {
		log::warn!("[Rust] dispatch_event_to_js: window {} not found", window_id);
	}
}

pub struct RootState {
	pub render_count: u64,
}

/// Cursor blink interval in milliseconds
const CURSOR_BLINK_INTERVAL_MS: u64 = 500;

pub struct RootView {
	state:             Entity<RootState>,
	last_render:       u64,
	window_id:         u64,
	focus_handle:      Option<FocusHandle>,
	focus_initialized: bool,
	/// Task for cursor blink timer
	blink_timer:       Option<Task<()>>,
	/// Track last focused element to detect focus changes
	last_focused:      Option<u64>,
}

impl RootView {
	pub fn new(state: Entity<RootState>, window_id: u64, _w: f32, _h: f32) -> RootView {
		return Self {
			state,
			last_render: 0,
			window_id,
			focus_handle: None,
			focus_initialized: false,
			blink_timer: None,
			last_focused: None,
		};
	}

	fn get_or_create_focus_handle(&mut self, cx: &mut Context<Self>) -> FocusHandle {
		if self.focus_handle.is_none() {
			let handle = cx.focus_handle();
			self.focus_handle = Some(handle);
		}
		self.focus_handle.clone().unwrap()
	}

	fn ensure_focus(&mut self, window: &mut Window) {
		if !self.focus_initialized {
			if let Some(ref handle) = self.focus_handle {
				window.focus(handle);
				log::info!("[Rust] Root element focused for keyboard input");
			}
			self.focus_initialized = true;
		}
	}

	fn update_state(&mut self, cx: &mut Context<Self>) {
		let Some(window) = GLOBAL_STATE.get_window(self.window_id) else {
			log::warn!("update_state: window {} not found", self.window_id);
			return;
		};
		let trigger = window.state().get_render_count();
		log::trace!(
			"update_state: window_id={}, trigger={}, last_render={}",
			self.window_id,
			trigger,
			self.last_render
		);

		if trigger != self.last_render {
			log::debug!("update_state: trigger changed from {} to {}", self.last_render, trigger);
			self.last_render = trigger;
			self.state.update(cx, |state, _cx| {
				state.render_count = trigger;
			});
		}
	}

	/// Start or restart the cursor blink timer
	fn start_blink_timer(&mut self, _gpui_window: &mut Window, cx: &mut Context<Self>) {
		use std::time::Duration;

		// Cancel existing timer
		self.blink_timer = None;

		let window_id = self.window_id;

		// Start new timer that refreshes window periodically for cursor blinking
		let task = cx.spawn(async move |_this, mut async_cx| {
			loop {
				Timer::after(Duration::from_millis(CURSOR_BLINK_INTERVAL_MS)).await;
				// Refresh the window to update cursor blink state
				let result = async_cx.update(|cx| {
					if let Some(window) = GLOBAL_STATE.get_window(window_id) {
						// Trigger a refresh through the window handle
						if let Err(e) = cx.update_window(window.handle(), |_view, w, _cx| {
							w.refresh();
						}) {
							log::trace!("Blink timer: failed to update window: {:?}", e);
							return false;
						}
					}
					true
				});
				if result.is_err() || result.unwrap_or(true) == false {
					// Window closed or update failed, stop timer
					break;
				}
			}
		});
		self.blink_timer = Some(task);
	}

	/// Update blink timer based on focus state
	fn update_blink_timer(&mut self, gpui_window: &mut Window, cx: &mut Context<Self>) {
		let current_focused = focus::get_focused(self.window_id);

		// Check if focus changed
		if current_focused != self.last_focused {
			self.last_focused = current_focused;

			if current_focused.is_some() {
				// An element is focused - start blink timer
				self.start_blink_timer(gpui_window, cx);
			} else {
				// No element focused - stop blink timer
				self.blink_timer = None;
			}
		}
	}
}

impl Render for RootView {
	fn render(
		&mut self,
		gpui_window: &mut Window,
		cx: &mut gpui::Context<Self>,
	) -> impl gpui::IntoElement {
		let render_start = std::time::Instant::now();
		self.update_state(cx);
		self.update_blink_timer(gpui_window, cx);

		let focus_handle = self.get_or_create_focus_handle(cx);
		self.ensure_focus(gpui_window);
		let window_id = self.window_id;

		let Some(window_state) = GLOBAL_STATE.get_window(self.window_id) else {
			log::warn!("RootView.render: window {} not found", self.window_id);
			return div().child("Window not found").into_any_element();
		};

		let tree = window_state
			.state()
			.element_tree
			.lock()
			.expect("Failed to acquire element_tree lock in RootView.render");

		log::debug!("RootView.render: window_id={}, has_tree={}", self.window_id, tree.is_some());
		let child_element = match &*tree {
			Some(element) => {
				// Use the new Element trait implementation
				create_element(element.clone(), self.window_id, None)
			}
			None => {
				div().id("base").child("Waiting for React...").text_color(rgb(0x888888)).into_any_element()
			}
		};

		let render_duration = render_start.elapsed();
		log::debug!("RootView.render completed in {:?}", render_duration);

		// Register the input handler for IME support
		// This uses the root focus handle, and RootInputHandler delegates to the
		// focused input
		gpui_window.handle_input(&focus_handle, RootInputHandler::new(self.window_id), cx);

		// Wrap in a focusable div that handles keyboard events at the window level
		div()
			.id("gpui-root")
			.size_full()
			.track_focus(&focus_handle)
			.on_key_down(move |event: &KeyDownEvent, window, _cx| {
				let keystroke = &event.keystroke;
				log::debug!(
					"[Rust] Window {} KeyDown: key={}, shift={}",
					window_id,
					keystroke.key,
					keystroke.modifiers.shift
				);

				// Get the currently focused element for this window
				let focused_element = focus::get_focused(window_id);

				log::debug!(
					"[Rust] KeyDown: window_id={}, focused_element={:?}, key={}",
					window_id,
					focused_element,
					keystroke.key
				);

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

					log::debug!(
						"[Rust] Focus navigation result: blur_id={:?}, focus_id={:?}",
						blur_id,
						focus_id
					);

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

					// Refresh to show focus changes
					window.refresh();

					return; // Don't dispatch Tab as keydown to the element
				}

				// Dispatch keydown event to the focused element
				if let Some(element_id) = focused_element {
					log::debug!(
						"[Rust] KeyDown: element_id={}, key={}, shift={}",
						element_id,
						keystroke.key,
						keystroke.modifiers.shift
					);
					// Try to handle as input element first
					if handle_input_key_event(
						window_id,
						element_id,
						&keystroke.key,
						keystroke.modifiers,
						window,
					) {
						// Input element handled the key
						log::debug!("[Rust] KeyDown: element_id={} handled key={}", element_id, keystroke.key);
						return;
					}

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
			})
			.on_key_up(move |event: &KeyUpEvent, _window, _cx| {
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

					log::debug!(
						"[Rust] Dispatching onKeyUp to element_id={}, key={}",
						element_id,
						keystroke.key
					);
					dispatch_event_to_js(window_id, element_id, types::KEYUP, event_data);
				}
			})
			.child(child_element)
			.into_any_element()
	}
}

pub fn start_gpui_thread() {
	log::info!("start_gpui_thread: spawning thread...");

	std::thread::spawn(move || {
		log::info!("GPUI thread: starting...");
		GLOBAL_STATE.set_thread_started(true);

		let app = GpuiApp::new();
		log::debug!("GPUI thread: app created");

		app.run(move |cx: &mut gpui::App| {
			log::debug!("GPUI thread: app.run() callback entered");
			host_command::init(cx);

			log::info!("GPUI thread: initialized, window creation via gpui_create_window");
		});

		log::debug!("GPUI thread: app.run() returned");
	});

	log::info!("start_gpui_thread: thread spawned");
}
