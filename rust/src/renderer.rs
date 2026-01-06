use std::ffi::{c_char, c_void};

use gpui::{App as GpuiAppContext, Application as GpuiApp, ClickEvent, Div, ElementId, Entity, MouseButton, Render, Window, div, prelude::*, px, rgb};

use crate::{element::{ElementStyle, ReactElement}, global_state::GLOBAL_STATE, host_command};

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
	pub state:       Entity<RootState>,
	pub last_render: u64,
	pub window_id:   u64,
}

impl RootView {
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
		log::debug!(
			"RootView.render: window_id={}, tree={:?}",
			self.window_id,
			tree.as_ref().map(|e| (e.global_id, &e.element_type))
		);

		let result = div().size(px(800.0)).bg(rgb(0x1e1e1e)).child(match &*tree {
			Some(element) => render_element_to_gpui(&element, None, self.window_id),
			None => div().id("base").child("Waiting for React...").text_color(rgb(0x888888)),
		});

		let render_duration = render_start.elapsed();
		log::debug!("RootView.render completed in {:?}", render_duration);

		result
	}
}

fn render_element_to_gpui(
	element: &ReactElement,
	parent_style: Option<&ElementStyle>,
	window_id: u64,
) -> gpui::Stateful<Div> {
	log::debug!(
		"render_element_to_gpui: type={}, global_id={}, text={:?}, has_handlers={}",
		element.element_type,
		element.global_id,
		element.text,
		element.event_handlers.is_some()
	);

	if let Some(ref handlers) = element.event_handlers {
		log::debug!("  Element {} handlers: {:?}", element.global_id, handlers);
	}

	// Helper to get effective style (own style or inherited from parent for text
	// properties)
	let effective_text_color = element.style.text_color.or(parent_style.and_then(|s| s.text_color));
	let effective_text_size = element.style.text_size.or(parent_style.and_then(|s| s.text_size));

	match element.element_type.as_str() {
		"div" => {
			// Pass current element's style as parent for children
			let children: Vec<gpui::Stateful<Div>> = element
				.children
				.iter()
				.map(|c| render_element_to_gpui(c, Some(&element.style), window_id))
				.collect();
			log::trace!("  div has {} children", children.len());

			let is_flex = element.style.display.as_ref().map(|s| s.as_str()) == Some("flex");

			let mut div = div().id(element.global_id as usize);
			div = div.when(is_flex, |div| div.flex());

			div = match element.style.flex_direction.as_ref().map(|s| s.as_str()) {
				Some("row") => div.flex_row(),
				Some("column") => div.flex_col(),
				_ => div,
			};

			div = match element.style.justify_content.as_ref().map(|s| s.as_str()) {
				Some("flex-start") => div.justify_start(),
				Some("center") => div.justify_center(),
				Some("flex-end") => div.justify_end(),
				Some("space-between") => div.justify_between(),
				Some("space-around") => div.justify_around(),
				_ => div,
			};

			div = match element.style.align_items.as_ref().map(|s| s.as_str()) {
				Some("flex-start") => div.items_start(),
				Some("center") => div.items_center(),
				Some("flex-end") => div.items_end(),
				_ => div,
			};

			if let Some(bg) = element.style.bg_color {
				div = div.bg(rgb(bg));
			} else {
				div = div.bg(rgb(0x2d2d2d));
			}

			if let Some(width) = element.style.width {
				div = div.w(px(width));
			}

			if let Some(height) = element.style.height {
				div = div.h(px(height));
			}

			if let (Some(pt), Some(pr), Some(pb), Some(pl)) = (
				element.style.padding_top,
				element.style.padding_right,
				element.style.padding_bottom,
				element.style.padding_left,
			) {
				div = div.pt(px(pt)).pr(px(pr)).pb(px(pb)).pl(px(pl));
			}

			if let (Some(mt), Some(mr), Some(mb), Some(ml)) = (
				element.style.margin_top,
				element.style.margin_right,
				element.style.margin_bottom,
				element.style.margin_left,
			) {
				div = div.mt(px(mt)).mr(px(mr)).mb(px(mb)).ml(px(ml));
			}

			if let Some(border_radius) = element.style.border_radius {
				div = div.rounded(px(border_radius));
			}

			if let Some(gap) = element.style.gap {
				div = div.gap(px(gap));
			}

			if let Some(opacity) = element.style.opacity {
				div = div.opacity(opacity as f32);
			}

			if element.event_handlers.as_ref().and_then(|v| v.get("onClick")).is_some() {
				div = div.on_click(|v, av, bv| println!("bbbbbbbbbbbbbbbbbbbbbbb"))
			}

			div.children(children)
		}
		"text" => {
			let text = element.text.clone().unwrap_or_default();
			log::trace!("  rendering text: '{}'", text);

			let mut text_element = div().id(element.global_id as usize).child(text);

			// Use effective style (inherited from parent div if not set on text element)
			if let Some(color) = effective_text_color {
				text_element = text_element.text_color(rgb(color));
			} else {
				text_element = text_element.text_color(rgb(0xffffff));
			}

			if let Some(size) = effective_text_size {
				text_element = text_element.text_size(px(size));
			}

			text_element
		}
		"span" => {
			let text = if let Some(ref t) = element.text {
				t.clone()
			} else {
				element
					.children
					.iter()
					.filter(|c| c.element_type == "text")
					.filter_map(|c| c.text.as_ref())
					.cloned()
					.collect::<Vec<_>>()
					.join("")
			};
			log::trace!("  rendering span (inline text): '{}'", text);

			let mut span_element = div().id(element.global_id as usize).child(text);

			// Use effective style (inherited from parent if not set on span)
			if let Some(color) = effective_text_color {
				span_element = span_element.text_color(rgb(color));
			} else {
				span_element = span_element.text_color(rgb(0xffffff));
			}

			if let Some(size) = effective_text_size {
				span_element = span_element.text_size(px(size));
			}

			span_element
		}
		"img" => {
			log::trace!("  rendering img");
			let div = div().id(element.global_id as usize);
			let mut img_element = if let Some(ref src) = element.style.src {
				div.child(format!("[Image: {}]", src))
			} else if let Some(ref alt) = element.style.alt {
				div.child(format!("[Image: {}]", alt))
			} else {
				div.child("[Image]")
			};

			if let Some(width) = element.style.width {
				img_element = img_element.w(px(width));
			}

			if let Some(height) = element.style.height {
				img_element = img_element.h(px(height));
			}

			img_element
		}
		_ => div().id(element.global_id as usize).child(format!("[Unknown: {}]", element.element_type)),
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
