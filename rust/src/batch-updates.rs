use serde_json::Value;
use std::ffi::{c_char, CStr, CString};
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

use crate::element_store::{ReactElement, ELEMENT_MAP, ELEMENT_STYLE, ELEMENT_TREE};
use crate::ffi_types::FfiResult;

pub fn gpui_batch_update_elements(
    count_ptr: *const u8,
    elements_json_ptr: *const c_char,
    result: *mut FfiResult,
) {
    unsafe {
        if count_ptr.is_null() || elements_json_ptr.is_null() || result.is_null() {
            *result = FfiResult::error("count_ptr or elements_json_ptr or result is null");
            return;
        }

        let count = std::ptr::read_volatile(count_ptr) as u64;
        let elements_json_str = CStr::from_ptr(elements_json_ptr)
            .to_str()
            .map_err(|e| format!("Invalid UTF-8 in elements JSON: {}", e))
            .unwrap();

        let elements_value: Value = serde_json::from_str(&elements_json_str)
            .map_err(|e| format!("Failed to parse elements JSON: {}", e))
            .unwrap();

        let elements_array = elements_value
            .as_array()
            .map_err(|_| "Elements must be an array".to_string())
            .unwrap();

        eprintln!("Batch update: Processing {} elements", count);

        let mut element_map = ELEMENT_MAP.lock().unwrap();

        for (i, elem_value) in elements_array.iter().enumerate() {
            if let Some(elem_obj) = elem_value.as_object() {
                let global_id = elem_obj
                    .get("globalId")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| 0);

                let element_type = elem_obj
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else("".to_string());

                let text = elem_obj
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let style_value = elem_obj
                    .get("style")
                    .and_then(|v| {
                        let mut element_style = ELEMENT_STYLE.lock().unwrap();

                        if let Some(color) = v.get("textColor").and_then(|c| c.as_u64()) {
                            element_style.text_color = Some(color as u32);
                        }
                        if let Some(bg) = v.get("bgColor").and_then(|c| c.as_u64()) {
                            element_style.bg_color = Some(bg as u32);
                        }
                        if let Some(text_size) = v.get("textSize").and_then(|c| c.as_f64()) {
                            element_style.text_size = Some(text_size as f32);
                        }
                        if let Some(width) = v.get("width").and_then(|c| c.as_f64()) {
                            element_style.width = Some(width as f32);
                        }
                        if let Some(height) = v.get("height").and_then(|c| c.as_f64()) {
                            element_style.height = Some(height as f32);
                        }

                        drop(element_style);
                        Some(element_style)
                    })
                    .unwrap_or(ELEMENT_STYLE.lock().unwrap());

                let element = Arc::new(ReactElement {
                    global_id,
                    element_type,
                    text,
                    children: Vec::new(),
                    style,
                    event_handlers: None,
                });

                element_map.insert(global_id, element);

                eprintln!(
                    "Batch update: Updated element {} ({})",
                    global_id, element_type
                );
            }
        }

        drop(element_map);

        *result = FfiResult::success();
    }
}
