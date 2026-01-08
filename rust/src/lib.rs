extern crate core;

mod element;
mod event_types;
mod ffi_helpers;
mod ffi_types;
mod global_state;
mod host_command;
mod logging;
mod renderer;
mod window;

use std::{ffi::{CStr, CString, c_char, c_void}, sync::atomic::{AtomicPtr, Ordering}};

use tokio::sync::oneshot;

use crate::{ffi_helpers::{ptr_to_u64, read_c_string, read_opt_c_string, validate_result_ptr}, ffi_types::{FfiResult, WindowCreateResult, WindowOptions}, global_state::GLOBAL_STATE, host_command::{HostCommand, is_bus_ready, send_host_command}, renderer::start_gpui_thread};

/// Global event callback pointer for routing events to JavaScript (thread-safe
/// using AtomicPtr)
static EVENT_CALLBACK: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

pub(crate) fn get_event_callback() -> Option<*mut c_void> {
	let ptr = EVENT_CALLBACK.load(Ordering::Acquire);
	if ptr.is_null() { None } else { Some(ptr) }
}

#[unsafe(no_mangle)]
pub extern "C" fn gpui_init(result: *mut FfiResult) {
	unsafe {
		logging::init_logging();
		log::info!("gpui_init: checking initialization...");

		if GLOBAL_STATE.is_initialized() {
			log::info!("gpui_init: already initialized");
			*result = FfiResult::success();
			return;
		}

		log::info!("gpui_init: starting GPUI thread...");
		start_gpui_thread();
		GLOBAL_STATE.set_initialized(true);

		if GLOBAL_STATE.is_thread_started() {
			log::info!("gpui_init: GPUI thread started successfully");
		} else {
			log::warn!("gpui_init: warning - GPUI thread may not have started");
		}

		*result = FfiResult::success();
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn gpui_create_window(options_ptr: *const c_char, result: *mut WindowCreateResult) {
	let options_json = unsafe { read_c_string(options_ptr, "{}") };

	let options: WindowOptions = serde_json::from_str(&options_json)
		.map_err(|e| format!("Failed to parse window options JSON: {}", e))
		.unwrap_or_else(|e| {
			log::error!("JSON parse error: {}", e);
			WindowOptions::default()
		});

	let (response_tx, response_rx) = oneshot::channel();

	send_host_command(HostCommand::CreateWindow { options, response_tx });

	let real_window_id: u64 = match response_rx.blocking_recv() {
		Ok(id) => id,
		Err(e) => {
			log::error!("Failed to receive window ID: {}", e);
			unsafe {
				if let Some(result_ref) = validate_result_ptr(result, "gpui_create_window") {
					*result_ref = WindowCreateResult::error("Failed to get window ID from GPUI");
				}
			}
			return;
		}
	};

	unsafe {
		if let Some(result_ref) = validate_result_ptr(result, "gpui_create_window") {
			*result_ref = WindowCreateResult::success(real_window_id);
		}
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn gpui_render_frame(
	window_id_ptr: *const u8,
	global_id_ptr: *const u8,
	type_ptr: *const std::os::raw::c_char,
	text_ptr: *const std::os::raw::c_char,
	child_count_ptr: *const u8,
	children_ptr: *const u64,
	result_ptr: *mut FfiResult,
) {
	log::debug!("gpui_render_frame: called");
	unsafe {
		if result_ptr.is_null() {
			log::error!("gpui_render_frame: result_ptr is null");
			return;
		}

		let window_id = ptr_to_u64(window_id_ptr);
		let global_id = ptr_to_u64(global_id_ptr);
		let child_count = ptr_to_u64(child_count_ptr) as usize;

		let element_type = read_c_string(type_ptr, "unknown");
		let text = read_opt_c_string(text_ptr);

		let children: Vec<u64> = if children_ptr.is_null() || child_count == 0 {
			Vec::new()
		} else {
			let slice = std::slice::from_raw_parts(children_ptr, child_count);
			slice.to_vec()
		};

		log::debug!(
			"gpui_render_frame: window_id={}, id={}, type={}, text={:?}, child_count={}, children={:?}",
			window_id,
			global_id,
			element_type,
			text,
			child_count,
			children
		);

		send_host_command(HostCommand::UpdateElement {
			window_id,
			global_id,
			element_type,
			text,
			children,
		});

		let result_buf = std::slice::from_raw_parts_mut(result_ptr as *mut u8, 8);
		result_buf[0] = 0;
		log::debug!("gpui_render_frame: completed successfully");
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn gpui_trigger_render(window_id_ptr: *const u8, _result: *mut FfiResult) {
	unsafe {
		let window_id = ptr_to_u64(window_id_ptr);
		send_host_command(HostCommand::TriggerRender { window_id });
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn gpui_batch_update_elements(
	window_id_ptr: *const u8,
	count_ptr: *const u8,
	elements_json_ptr: *const c_char,
	result: *mut FfiResult,
) {
	log::debug!("gpui_batch_update_elements: called");
	unsafe {
		let window_id = ptr_to_u64(window_id_ptr);
		let _count = std::ptr::read_volatile(count_ptr) as u64;

		// Safe UTF-8 conversion with error handling
		let elements_json_str = match CStr::from_ptr(elements_json_ptr).to_str() {
			Ok(s) => s,
			Err(e) => {
				log::error!("Invalid UTF-8 in elements JSON: {}", e);
				*result = FfiResult::error(&format!("Invalid UTF-8 in elements JSON: {}", e));
				return;
			}
		};

		// Safe JSON parsing with error handling
		let elements_value: serde_json::Value = match serde_json::from_str(elements_json_str) {
			Ok(v) => v,
			Err(e) => {
				log::error!("Failed to parse elements JSON: {}", e);
				*result = FfiResult::error(&format!("Failed to parse elements JSON: {}", e));
				return;
			}
		};

		let _ = GLOBAL_STATE.get_window(window_id);

		send_host_command(HostCommand::BatchUpdateElements { window_id, elements: elements_value });

		*result = FfiResult::success();
		log::debug!("gpui_batch_update_elements: completed successfully");
	}
}

/// Free the memory allocated for FfiResult's error message
#[unsafe(no_mangle)]
pub extern "C" fn gpui_free_result(result: FfiResult) {
	if !result.error_msg.is_null() {
		unsafe {
			let _ = CString::from_raw(result.error_msg);
		}
	}
}

/// Free the memory allocated for WindowCreateResult's error message
#[unsafe(no_mangle)]
pub extern "C" fn gpui_free_window_result(result: WindowCreateResult) {
	if !result.error_msg.is_null() {
		unsafe {
			let _ = CString::from_raw(result.error_msg);
		}
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn gpui_is_ready() -> bool { is_bus_ready() }

/// Free a string pointer that was passed to JavaScript via event callback
#[unsafe(no_mangle)]
pub extern "C" fn gpui_free_event_string(ptr: *mut c_char) {
	if !ptr.is_null() {
		unsafe {
			let _ = CString::from_raw(ptr);
		}
	}
}

/// Set the event callback for receiving events from Rust to JavaScript
#[unsafe(no_mangle)]
pub extern "C" fn set_event_callback(callback_ptr: *mut c_void) {
	EVENT_CALLBACK.store(callback_ptr, Ordering::Release);
	log::info!("Event callback registered: {:p}", callback_ptr);
}
