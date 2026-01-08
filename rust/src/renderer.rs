use std::ffi::{CString, c_char};

use gpui::{Application as GpuiApp, Entity, Render, Window, div, prelude::*, rgb};

use crate::{element::create_element, global_state::GLOBAL_STATE, host_command};

/// Event data structure for enhanced event information
pub struct EventData {
	pub client_x: Option<f32>,
	pub client_y: Option<f32>,
	pub button:   Option<u8>,
}

impl Default for EventData {
	fn default() -> Self { Self { client_x: None, client_y: None, button: None } }
}

/// Dispatch an event directly to JavaScript via the registered callback
/// Now supports additional event data like mouse coordinates
pub(crate) fn dispatch_event_to_js(
	window_id: u64,
	element_id: u64,
	event_type: &str,
	event_data: Option<EventData>,
) {
	use crate::get_event_callback;

	let callback_ptr = match get_event_callback() {
		Some(ptr) => ptr,
		None => {
			log::warn!("[Rust] dispatch_event_to_js: No event callback registered");
			return;
		}
	};

	log::info!(
		"[Rust] dispatch_event_to_js: window_id={}, element_id={}, event_type={}",
		window_id,
		element_id,
		event_type
	);

	// Build JSON payload with optional event data
	let json_payload = if let Some(data) = event_data {
		serde_json::json!({
			"windowId": window_id,
			"elementId": element_id,
			"eventType": event_type,
			"clientX": data.client_x,
			"clientY": data.client_y,
			"button": data.button,
			"timestamp": std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_millis() as u64)
				.unwrap_or(0)
		})
	} else {
		serde_json::json!({
			"windowId": window_id,
			"elementId": element_id,
			"eventType": event_type,
			"timestamp": std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_millis() as u64)
				.unwrap_or(0)
		})
	};

	let json_str = json_payload.to_string();
	let c_string = CString::new(json_str).unwrap();
	let len = c_string.count_bytes();
	// Use into_raw() to transfer ownership to JavaScript.
	// JS will call gpui_free_event_string() after reading the data.
	let raw_ptr = c_string.into_raw();

	log::info!("[Rust] dispatch_event_to_js: calling callback with JSON pointer");

	unsafe {
		let callback: extern "C" fn(*mut c_char, u32) = std::mem::transmute(callback_ptr);
		callback(raw_ptr, len as u32);
	}

	log::info!("[Rust] dispatch_event_to_js: callback returned");
}

pub struct RootState {
	pub render_count: u64,
}

pub struct RootView {
	state:       Entity<RootState>,
	last_render: u64,
	window_id:   u64,
	w:           f32,
	h:           f32,
}

impl RootView {
	pub fn new(state: Entity<RootState>, window_id: u64, w: f32, h: f32) -> RootView {
		return Self { state, last_render: 0, window_id, w, h };
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
}

impl Render for RootView {
	fn render(
		&mut self,
		_window: &mut Window,
		cx: &mut gpui::Context<Self>,
	) -> impl gpui::IntoElement {
		let render_start = std::time::Instant::now();
		self.update_state(cx);

		let Some(window) = GLOBAL_STATE.get_window(self.window_id) else {
			log::warn!("RootView.render: window {} not found", self.window_id);
			return div().child("Window not found").into_any_element();
		};

		let tree = window
			.state()
			.element_tree
			.lock()
			.expect("Failed to acquire element_tree lock in RootView.render");

		log::debug!("RootView.render: window_id={}, has_tree={}", self.window_id, tree.is_some());
		let result = match &*tree {
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

		result
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
