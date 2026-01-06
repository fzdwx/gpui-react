use std::ffi::{c_char, c_void};

use gpui::{App as GpuiAppContext, Application as GpuiApp, ClickEvent, Div, ElementId, Entity, MouseButton, Render, Window, div, prelude::*, px, rgb};

use crate::{element::render_to_gpui, global_state::GLOBAL_STATE, host_command};

/// Dispatch an event directly to JavaScript via the registered callback
fn dispatch_event_to_js(
	window_id: u64,
	element_id: u64,
	event_type: &str,
	_event_data: Option<&serde_json::Value>,
) {
	use crate::get_event_callback;

	let callback_ptr = match get_event_callback() {
		Some(ptr) => ptr,
		None => {
			log::warn!("[Rust] dispatch_event_to_js: No event callback registered");
			return;
		}
	};

	unsafe {
		log::info!(
			"[Rust] dispatch_event_to_js: window_id={}, element_id={}, event_type={}",
			window_id,
			element_id,
			event_type
		);

		// Prepare buffers on heap to ensure they live long enough
		let window_id_bytes = window_id.to_le_bytes();
		let element_id_bytes = element_id.to_le_bytes();

		let event_type_cstring = std::ffi::CString::new(event_type).unwrap_or_default();
		let event_type_ptr = event_type_cstring.as_ptr() as *mut c_char;
		let event_type_len = event_type_cstring.to_bytes().len() + 1; // include null terminator

		// Create event data (empty JSON object)
		let event_data_json = "{}";
		let event_data_bytes = event_data_json.as_bytes();
		let event_data_len = event_data_bytes.len();

		// Allocate buffers on heap
		let window_id_boxed = Box::new(window_id_bytes);
		let element_id_boxed = Box::new(element_id_bytes);
		let event_data_boxed = Box::new(event_data_bytes.to_vec());

		let window_id_ptr = Box::into_raw(window_id_boxed) as *mut c_void;
		let element_id_ptr = Box::into_raw(element_id_boxed) as *mut c_void;
		let event_data_ptr = Box::into_raw(event_data_boxed) as *mut u8;

		log::info!(
			"[Rust] dispatch_event_to_js: calling callback with pointers: window={:?}, element={:?}",
			window_id_ptr,
			element_id_ptr
		);

		let callback: extern "C" fn(
			*mut c_void,
			usize,
			*mut c_void,
			usize,
			*mut c_char,
			usize,
			*mut u8,
			usize,
		) = std::mem::transmute(callback_ptr);

		callback(
			window_id_ptr,
			8,
			element_id_ptr,
			8,
			event_type_ptr,
			event_type_len,
			event_data_ptr,
			event_data_len,
		);

		log::info!("[Rust] dispatch_event_to_js: callback returned");

		// Cleanup (JS should have copied the data by now)
		let _ = Box::from_raw(window_id_ptr as *mut [u8; 8]);
		let _ = Box::from_raw(element_id_ptr as *mut [u8; 8]);
		let _ = Box::from_raw(event_data_ptr);
	}
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
			return div().child("Window not found");
		};

		let tree = window
			.state()
			.element_tree
			.lock()
			.expect("Failed to acquire element_tree lock in RootView.render");

		log::debug!("RootView.render: window_id={}, has_tree={}", self.window_id, tree.is_some());
		let result = div().h_auto().w_auto().child(match &*tree {
			Some(element) => render_to_gpui(element, None, self.window_id),
			None => div().id("base").child("Waiting for React...").text_color(rgb(0x888888)),
		});

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
