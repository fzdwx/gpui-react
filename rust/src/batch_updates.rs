use serde_json::Value;
use std::ffi::{c_char, CStr};
use std::sync::Arc;

use crate::element_store::ReactElement;
use crate::ffi_types::FfiResult;

#[no_mangle]
pub extern "C" fn gpui_batch_update_elements(
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
            .ok_or_else(|| "Elements must be an array".to_string())
            .unwrap();

        eprintln!("Batch update: Processing {} elements", count);

        let mut element_map = crate::ELEMENT_MAP.lock().unwrap();

        for (_i, elem_value) in elements_array.iter().enumerate() {
            if let Some(elem_obj) = elem_value.as_object() {
                let global_id = elem_obj
                    .get("globalId")
                    .and_then(|v: &Value| v.as_u64())
                    .unwrap_or(0);

                let element_type = elem_obj
                    .get("type")
                    .and_then(|v: &Value| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let text = elem_obj
                    .get("text")
                    .and_then(|v: &Value| v.as_str())
                    .map(|s| s.to_string());

                let style = if let Some(style_obj) = elem_obj.get("style") {
                    crate::element_store::ElementStyle {
                        text_color: style_obj
                            .get("textColor")
                            .and_then(|v: &Value| v.as_u64())
                            .map(|v| v as u32),
                        bg_color: style_obj
                            .get("bgColor")
                            .and_then(|v: &Value| v.as_u64())
                            .map(|v| v as u32),
                        border_color: style_obj
                            .get("borderColor")
                            .and_then(|v: &Value| v.as_u64())
                            .map(|v| v as u32),
                        text_size: style_obj
                            .get("textSize")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        width: style_obj
                            .get("width")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        height: style_obj
                            .get("height")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        margin_top: style_obj
                            .get("marginTop")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        margin_right: style_obj
                            .get("marginRight")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        margin_bottom: style_obj
                            .get("marginBottom")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        margin_left: style_obj
                            .get("marginLeft")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        padding_top: style_obj
                            .get("paddingTop")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        padding_right: style_obj
                            .get("paddingRight")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        padding_bottom: style_obj
                            .get("paddingBottom")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        padding_left: style_obj
                            .get("paddingLeft")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        display: style_obj
                            .get("display")
                            .and_then(|v: &Value| v.as_str())
                            .map(|s| s.to_string()),
                        flex_direction: style_obj
                            .get("flexDirection")
                            .and_then(|v: &Value| v.as_str())
                            .map(|s| s.to_string()),
                        justify_content: style_obj
                            .get("justifyContent")
                            .and_then(|v: &Value| v.as_str())
                            .map(|s| s.to_string()),
                        align_items: style_obj
                            .get("alignItems")
                            .and_then(|v: &Value| v.as_str())
                            .map(|s| s.to_string()),
                        gap: style_obj
                            .get("gap")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        border_radius: style_obj
                            .get("borderRadius")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        opacity: style_obj
                            .get("opacity")
                            .and_then(|v: &Value| v.as_f64())
                            .map(|v| v as f32),
                        src: style_obj
                            .get("src")
                            .and_then(|v: &Value| v.as_str())
                            .map(|s| s.to_string()),
                        alt: style_obj
                            .get("alt")
                            .and_then(|v: &Value| v.as_str())
                            .map(|s| s.to_string()),
                    }
                } else {
                    crate::element_store::ElementStyle::default()
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

        eprintln!("Updating children references...");

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
                                child_refs.push(child.clone() as Arc<ReactElement>);
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

        eprintln!("Children updated for all elements");

        *result = FfiResult::success();
    }
}
