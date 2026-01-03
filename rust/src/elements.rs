use gpui::{div, prelude::*, px, rgb, IntoElement, SharedString};
use std::ffi::CStr;

use crate::ffi_types::ElementData;

pub fn build_element_tree(data: &ElementData) -> Result<String, String> {
    let type_str = unsafe {
        if data.type_ptr.is_null() {
            return Err("type_ptr is null".to_string());
        }
        CStr::from_ptr(data.type_ptr)
            .to_str()
            .map_err(|e| format!("Invalid UTF-8 in type: {}", e))?
    };

    let text_str = if !data.text_ptr.is_null() {
        Some(unsafe {
            CStr::from_ptr(data.text_ptr)
                .to_str()
                .map_err(|e| format!("Invalid UTF-8 in text: {}", e))?
        })
    } else {
        None
    };

    match type_str {
        "div" => {
            let mut result = format!("div(text={:?})", text_str);
            if let Some(text) = text_str {
                result = format!("div: {}", text);
            }
            Ok(result)
        }
        "text" => {
            let text = text_str.ok_or_else(|| "text element requires text content".to_string())?;
            Ok(format!("text: {}", text))
        }
        _ => Err(format!("Unknown element type: {}", type_str)),
    }
}
