use std::{ffi::CString, os::raw::c_char};

#[repr(C)]
pub struct FfiResult {
	pub status:    i32,
	pub error_msg: *mut c_char,
}

impl FfiResult {
	pub fn success() -> Self { FfiResult { status: 0, error_msg: std::ptr::null_mut() } }

	pub fn error(message: &str) -> Self {
		FfiResult { status: 1, error_msg: CString::new(message).unwrap().into_raw() }
	}
}

#[repr(C)]
pub struct WindowCreateResult {
	pub status:    i32,
	pub window_id: u64,
	pub error_msg: *mut c_char,
}

impl WindowCreateResult {
	pub fn success(window_id: u64) -> Self {
		WindowCreateResult { status: 0, window_id, error_msg: std::ptr::null_mut() }
	}

	pub fn error(message: &str) -> Self {
		WindowCreateResult {
			status:    1,
			window_id: 0,
			error_msg: CString::new(message).unwrap().into_raw(),
		}
	}
}

#[repr(C)]
pub struct ElementData {
	pub global_id:    u64,
	pub type_ptr:     *const c_char,
	pub text_ptr:     *const c_char,
	pub child_count:  u32,
	pub _padding:     u32,
	pub children_ptr: *const u64,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct WindowOptions {
	pub width:      f32,
	pub height:     f32,
	pub title:      Option<String>,
	pub x:          Option<f32>,
	pub y:          Option<f32>,
	pub resizable:  Option<bool>,
	pub fullscreen: Option<bool>,
}

impl Default for WindowOptions {
	fn default() -> Self {
		WindowOptions {
			width:      800.0,
			height:     600.0,
			title:      Some("React-GPUI".to_string()),
			x:          None,
			y:          None,
			resizable:  None,
			fullscreen: None,
		}
	}
}

impl From<WindowOptions> for gpui::WindowOptions {
	fn from(opts: WindowOptions) -> Self {
		let title = opts.title.unwrap_or_else(|| "React-GPUI".to_string());
		let origin =
			gpui::Point { x: gpui::px(opts.x.unwrap_or(100.0)), y: gpui::px(opts.y.unwrap_or(100.0)) };
		let size = gpui::Size { width: gpui::px(opts.width), height: gpui::px(opts.height) };
		let bounds = gpui::Bounds { origin, size };

		let window_bounds_type = if opts.fullscreen == Some(true) {
			gpui::WindowBounds::Fullscreen(bounds)
		} else {
			gpui::WindowBounds::Windowed(bounds)
		};

		gpui::WindowOptions {
			window_bounds: Some(window_bounds_type),
			titlebar: Some(gpui::TitlebarOptions { title: Some(title.into()), ..Default::default() }),
			is_resizable: opts.resizable.unwrap_or(true),
			..Default::default()
		}
	}
}

#[derive(Debug, Clone)]
pub struct WindowBounds {
	pub x:      Option<f32>,
	pub y:      Option<f32>,
	pub width:  f32,
	pub height: f32,
}
