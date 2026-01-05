mod element;
mod ffi_types;
mod global_state;
mod host_command;
mod logging;
mod renderer;
mod window_state;

use std::ffi::{c_char, CStr};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::element::ReactElement;
use crate::ffi_types::{FfiResult, WindowCreateResult};
use crate::global_state::GLOBAL_STATE;
use crate::host_command::{is_bus_ready, send_host_command, HostCommand};
use crate::renderer::start_gpui_thread;

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
    width: f32,
    height: f32,
    title_ptr: *const c_char,
    result: *mut WindowCreateResult,
) {
    // 从 C 字符串读取 title
    let title = unsafe {
        if title_ptr.is_null() {
            String::from("React-GPUI")
        } else {
            CStr::from_ptr(title_ptr).to_string_lossy().to_string()
        }
    };

    let window_id = crate::renderer::NEXT_WINDOW_ID.fetch_add(1, Ordering::SeqCst);

    let _ = GLOBAL_STATE.get_window_state(window_id);

    send_host_command(HostCommand::CreateWindow {
        width,
        height,
        window_id,
        title,
    });

    unsafe {
        if result.is_null() {
            log::error!("gpui_create_window: result is null");
            return;
        }
        *result = WindowCreateResult::success(window_id);
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

        let window_id = if window_id_ptr.is_null() {
            0
        } else {
            let buf = std::slice::from_raw_parts(window_id_ptr, 8);
            u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ])
        };

        let global_id = if global_id_ptr.is_null() {
            0
        } else {
            let buf = std::slice::from_raw_parts(global_id_ptr, 8);
            u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ])
        };

        let child_count = if child_count_ptr.is_null() {
            0
        } else {
            let buf = std::slice::from_raw_parts(child_count_ptr, 8);
            u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]) as usize
        };

        let element_type = if type_ptr.is_null() {
            String::from("unknown")
        } else {
            CStr::from_ptr(type_ptr).to_string_lossy().to_string()
        };

        let text = if text_ptr.is_null() {
            None
        } else {
            Some(CStr::from_ptr(text_ptr).to_string_lossy().to_string())
        };

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

        send_host_command(HostCommand::TriggerRender);

        let result_buf = std::slice::from_raw_parts_mut(result_ptr as *mut u8, 8);
        result_buf[0] = 0;
        log::debug!("gpui_render_frame: completed successfully");
    }
}

#[no_mangle]
pub extern "C" fn gpui_free_result(_result: FfiResult) {}

#[no_mangle]
pub extern "C" fn gpui_is_ready() -> bool {
    is_bus_ready()
}

#[no_mangle]
pub extern "C" fn gpui_trigger_render(window_id_ptr: *const u8, _result: *mut FfiResult) {
    unsafe {
        let window_id = if window_id_ptr.is_null() {
            0
        } else {
            let buf = std::slice::from_raw_parts(window_id_ptr, 8);
            u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ])
        };

        let window_state = GLOBAL_STATE.get_window_state(window_id);
        window_state.increment_render_count();
    }
    send_host_command(HostCommand::TriggerRender);
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
        if count_ptr.is_null() || elements_json_ptr.is_null() || result.is_null() {
            log::error!("gpui_batch_update_elements: null pointer detected");
            *result = FfiResult::error("count_ptr or elements_json_ptr or result is null");
            return;
        }

        let window_id = if window_id_ptr.is_null() {
            0
        } else {
            let buf = std::slice::from_raw_parts(window_id_ptr, 8);
            u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ])
        };

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

        log::info!("Batch update: Processing {} elements for window {}", count, window_id);

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

                let style = if let Some(style_obj) = elem_obj.get("style") {
                    crate::element::ElementStyle {
                        text_color: style_obj
                            .get("textColor")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32),
                        bg_color: style_obj
                            .get("bgColor")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32),
                        border_color: style_obj
                            .get("borderColor")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32),
                        text_size: style_obj
                            .get("textSize")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        width: style_obj
                            .get("width")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        height: style_obj
                            .get("height")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        margin_top: style_obj
                            .get("marginTop")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        margin_right: style_obj
                            .get("marginRight")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        margin_bottom: style_obj
                            .get("marginBottom")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        margin_left: style_obj
                            .get("marginLeft")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        padding_top: style_obj
                            .get("paddingTop")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        padding_right: style_obj
                            .get("paddingRight")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        padding_bottom: style_obj
                            .get("paddingBottom")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        padding_left: style_obj
                            .get("paddingLeft")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        display: style_obj
                            .get("display")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        flex_direction: style_obj
                            .get("flexDirection")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        justify_content: style_obj
                            .get("justifyContent")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        align_items: style_obj
                            .get("alignItems")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        gap: style_obj
                            .get("gap")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        border_radius: style_obj
                            .get("borderRadius")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        opacity: style_obj
                            .get("opacity")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        src: style_obj
                            .get("src")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        alt: style_obj
                            .get("alt")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    }
                } else {
                    crate::element::ElementStyle::default()
                };

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

        window_state.update_element_tree();

        send_host_command(HostCommand::TriggerRender);

        let trigger = window_state.get_render_count();
        log::debug!(
            "Triggering render, current count: {}",
            trigger
        );

        *result = FfiResult::success();
        log::debug!("gpui_batch_update_elements: completed successfully");
    }
}