mod element;
mod ffi_helpers;
mod ffi_types;
mod global_state;
mod host_command;
mod logging;
mod renderer;
mod window_state;

use std::{
    ffi::{c_char, CStr},
    sync::Arc,
};

use tokio::sync::oneshot;

use crate::{
    element::ReactElement,
    ffi_helpers::{ptr_to_u64, read_c_string, read_opt_c_string, validate_result_ptr},
    ffi_types::{FfiResult, WindowCreateResult, WindowOptions},
    global_state::GLOBAL_STATE,
    host_command::{is_bus_ready, send_host_command, HostCommand},
    renderer::start_gpui_thread,
};

#[no_mangle]
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

#[no_mangle]
pub extern "C" fn gpui_create_window(
    options_ptr: *const c_char,
    result: *mut WindowCreateResult,
) {
    let options_json = unsafe { read_c_string(options_ptr, "{}") };

    let options: WindowOptions = serde_json::from_str(&options_json)
        .map_err(|e| format!("Failed to parse window options JSON: {}", e))
        .unwrap_or_else(|e| {
            log::error!("JSON parse error: {}", e);
            WindowOptions::default()
        });

    let (response_tx, response_rx) = oneshot::channel();

    send_host_command(HostCommand::CreateWindow {
        options,
        response_tx,
    });

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

#[no_mangle]
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

        let window_state = GLOBAL_STATE.get_window_state(window_id);

        let element = Arc::new(ReactElement {
            global_id,
            element_type: element_type.clone(),
            text: text.clone(),
            children: Vec::new(),
            style: element::ElementStyle::default(),
            event_handlers: None,
        });

        let mut element_map = window_state
            .element_map
            .lock()
            .expect("Failed to acquire element_map lock in gpui_render_frame");
        element_map.insert(global_id, element.clone());

        for &child_id in &children {
            if !element_map.contains_key(&child_id) {
                let placeholder = Arc::new(ReactElement {
                    global_id: child_id,
                    element_type: "placeholder".to_string(),
                    text: None,
                    children: Vec::new(),
                    style: crate::element::ElementStyle::default(),
                    event_handlers: None,
                });
                element_map.insert(child_id, placeholder);
            }
        }

        drop(element_map);

        window_state.set_root_element_id(global_id);

        window_state.rebuild_tree(global_id, &children);

        window_state.update_element_tree();

        send_host_command(HostCommand::TriggerRender {
            window_id,
        });

        let result_buf = std::slice::from_raw_parts_mut(result_ptr as *mut u8, 8);
        result_buf[0] = 0;
        log::debug!("gpui_render_frame: completed successfully");
    }
}

#[no_mangle]
pub extern "C" fn gpui_trigger_render(
    window_id_ptr: *const u8,
    _result: *mut FfiResult,
) {
    unsafe {
        let window_id = ptr_to_u64(window_id_ptr);
        let window_state = GLOBAL_STATE.get_window_state(window_id);
        window_state.increment_render_count();
        send_host_command(HostCommand::TriggerRender {
            window_id,
        });
    }
}

#[no_mangle]
pub extern "C" fn gpui_batch_update_elements(
    window_id_ptr: *const u8,
    count_ptr: *const u8,
    elements_json_ptr: *const c_char,
    result: *mut FfiResult,
) {
    log::debug!("gpui_batch_update_elements: called");
    unsafe {
        let window_id = ptr_to_u64(window_id_ptr);
        let count = std::ptr::read_volatile(count_ptr) as u64;
        let elements_json_str = CStr::from_ptr(elements_json_ptr)
            .to_str()
            .map_err(|e| format!("Invalid UTF-8 in elements JSON: {}", e))
            .unwrap();

        let elements_value: serde_json::Value = serde_json::from_str(&elements_json_str)
            .map_err(|e| format!("Failed to parse elements JSON: {}", e))
            .unwrap();

        let elements_array = elements_value
            .as_array()
            .ok_or_else(|| "Elements must be an array".to_string())
            .unwrap();

        log::info!(
            "Batch update: Processing {} elements for window {}",
            count,
            window_id
        );

        let window_state = GLOBAL_STATE.get_window_state(window_id);

        let mut element_map = window_state
            .element_map
            .lock()
            .expect("Failed to acquire element_map lock in gpui_batch_update_elements");

        for (_i, elem_value) in elements_array.iter().enumerate() {
            if let Some(elem_obj) = elem_value.as_object() {
                let global_id = elem_obj
                    .get("globalId")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let element_type = elem_obj
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let text = elem_obj
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let style = elem_obj
                    .get("style")
                    .map(|s| element::ElementStyle::from_json(s))
                    .unwrap_or_default();

                let element = Arc::new(ReactElement {
                    global_id,
                    element_type,
                    text,
                    children: Vec::new(),
                    style,
                    event_handlers: None,
                });

                element_map.insert(global_id, element.clone());
            }
        }

        log::debug!("Updating children references...");

        for (_i, elem_value) in elements_array.iter().enumerate() {
            if let Some(elem_obj) = elem_value.as_object() {
                if let Some(global_id) = elem_obj.get("globalId").and_then(|v| v.as_u64()) {
                    if let Some(children_arr) = elem_obj.get("children").and_then(|v| v.as_array())
                    {
                        let children_ids: Vec<u64> =
                            children_arr.iter().filter_map(|c| c.as_u64()).collect();

                        let mut child_refs: Vec<Arc<ReactElement>> = Vec::new();

                        for &cid in &children_ids {
                            if let Some(child) = element_map.get(&cid) {
                                child_refs.push(child.clone());
                            }
                        }

                        if let Some(element) = element_map.get_mut(&global_id) {
                            let element_mut = Arc::make_mut(element);
                            element_mut.children = child_refs;
                        }
                    }
                }
            }
        }

        drop(element_map);
        log::debug!("Children updated for all elements");
        // window_state.update_element_tree();
        send_host_command(HostCommand::TriggerRender {
            window_id,
        });

        let trigger = window_state.get_render_count();
        log::debug!("Triggering render, current count: {}", trigger);

        *result = FfiResult::success();
        log::debug!("gpui_batch_update_elements: completed successfully");
    }
}

#[no_mangle]
pub extern "C" fn gpui_free_result(_result: FfiResult) {}

#[no_mangle]
pub extern "C" fn gpui_is_ready() -> bool {
    is_bus_ready()
}
