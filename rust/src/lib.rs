mod app;
mod batch_updates;
mod element_store;
mod elements;
mod ffi_types;

use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::app::start_gpui_thread;
use crate::element_store::{ReactElement, ELEMENT_TREE, RENDER_TRIGGER};
use crate::ffi_types::FfiResult;

static GPUI_INITIALIZED: AtomicBool = AtomicBool::new(false);
static RENDER_COUNT: AtomicU64 = AtomicU64::new(0);
pub static GPUI_THREAD_STARTED: AtomicBool = AtomicBool::new(false);
static ROOT_ELEMENT_ID: AtomicU64 = AtomicU64::new(0);

lazy_static::lazy_static! {
    pub static ref ELEMENT_MAP: std::sync::Mutex<HashMap<u64, Arc<ReactElement>>> = std::sync::Mutex::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn gpui_init(width: f32, height: f32, result: *mut FfiResult) {
    unsafe {
        eprintln!("gpui_init: checking initialization...");

        if GPUI_INITIALIZED.load(Ordering::SeqCst) {
            eprintln!("gpui_init: already initialized");
            *result = FfiResult::success();
            return;
        }

        eprintln!("gpui_init: starting GPUI thread...");
        start_gpui_thread(width,height);
        GPUI_INITIALIZED.store(true, Ordering::SeqCst);

        // Wait a bit for the thread to start
        std::thread::sleep(std::time::Duration::from_millis(500));

        if GPUI_THREAD_STARTED.load(Ordering::SeqCst) {
            eprintln!("gpui_init: GPUI thread started successfully");
        } else {
            eprintln!("gpui_init: warning - GPUI thread may not have started");
        }

        *result = FfiResult::success();
    }
}

#[no_mangle]
pub extern "C" fn gpui_create_window(_width: f32, _height: f32, result: *mut FfiResult) {
    unsafe {
        *result = FfiResult::success();
    }
}

#[no_mangle]
pub extern "C" fn gpui_render_frame(
    global_id_ptr: *const u8,
    type_ptr: *const std::os::raw::c_char,
    text_ptr: *const std::os::raw::c_char,
    child_count_ptr: *const u8,
    children_ptr: *const u64,
    result_ptr: *mut FfiResult,
) {
    unsafe {
        if result_ptr.is_null() {
            return;
        }

        // Read global_id (little-endian u64 from 8-byte buffer)
        let global_id = if global_id_ptr.is_null() {
            0
        } else {
            let buf = std::slice::from_raw_parts(global_id_ptr, 8);
            u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ])
        };

        // Read child_count (little-endian u64 from 8-byte buffer)
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

        eprintln!(
            "gpui_render_frame: id={}, type={}, text={:?}, child_count={}, children={:?}",
            global_id, element_type, text, child_count, children
        );

        // Add ALL elements to the map (not just the root)
        // The root element gets full data, children get placeholder entries
        let element = Arc::new(ReactElement {
            global_id,
            element_type: element_type.clone(),
            text: text.clone(),
            children: Vec::new(),
            style: crate::element_store::ElementStyle::default(),
            event_handlers: None,
        });

        let mut element_map = ELEMENT_MAP.lock().unwrap();
        element_map.insert(global_id, element.clone());

        for &child_id in &children {
            if !element_map.contains_key(&child_id) {
                let placeholder = Arc::new(ReactElement {
                    global_id: child_id,
                    element_type: "placeholder".to_string(),
                    text: None,
                    children: Vec::new(),
                    style: crate::element_store::ElementStyle::default(),
                    event_handlers: None,
                });
                element_map.insert(child_id, placeholder);
            }
        }

        drop(element_map);

        // Store the root element ID
        ROOT_ELEMENT_ID.store(global_id, Ordering::SeqCst);

        rebuild_tree(global_id, &children);

        let mut tree = ELEMENT_TREE.lock().unwrap();
        *tree = Some(get_root_element());

        RENDER_TRIGGER.fetch_add(1, Ordering::SeqCst);

        let result_buf = std::slice::from_raw_parts_mut(result_ptr as *mut u8, 8);
        result_buf[0] = 0;
    }
}

fn get_root_element() -> Arc<ReactElement> {
    let root_id = ROOT_ELEMENT_ID.load(Ordering::SeqCst);

    let element_map = ELEMENT_MAP.lock().unwrap();

    if root_id == 0 {
        return element_map.values().next().cloned().unwrap_or_else(|| {
            Arc::new(ReactElement {
                global_id: 0,
                element_type: "empty".to_string(),
                text: None,
                children: Vec::new(),
                style: crate::element_store::ElementStyle::default(),
                event_handlers: None,
            })
        });
    }

    element_map.get(&root_id).cloned().unwrap_or_else(|| {
        Arc::new(ReactElement {
            global_id: 0,
            element_type: "empty".to_string(),
            text: None,
            children: Vec::new(),
            style: crate::element_store::ElementStyle::default(),
            event_handlers: None,
        })
    })
}

fn rebuild_tree(root_id: u64, children: &[u64]) {
    eprintln!("rebuild_tree: root_id={}, children={:?}", root_id, children);
    let element_map = ELEMENT_MAP.lock().unwrap();

    eprintln!("  element_map has {} entries", element_map.len());
    for (id, elem) in element_map.iter() {
        eprintln!("    id={}, type={}", id, elem.element_type);
    }

    if let Some(root) = element_map.get(&root_id) {
        eprintln!(
            "  found root element: id={}, type={}",
            root.global_id, root.element_type
        );
        let child_elements: Vec<Arc<ReactElement>> = children
            .iter()
            .filter_map(|id| {
                eprintln!("    looking up child id={}", id);
                element_map.get(id).cloned()
            })
            .collect();
        eprintln!("  found {} child elements", child_elements.len());

        drop(element_map);

        let mut element_map = ELEMENT_MAP.lock().unwrap();
        if let Some(root) = element_map.get_mut(&root_id) {
            eprintln!(
                "  updating root children to {} elements",
                child_elements.len()
            );
            let root_mut = Arc::make_mut(root);
            root_mut.children = child_elements;
            root_mut.style = crate::element_store::ElementStyle::default();
        }
    } else {
        eprintln!("  root element not found!");
    }
}

#[no_mangle]
pub extern "C" fn gpui_free_result(_result: FfiResult) {}

#[no_mangle]
pub extern "C" fn gpui_update_element(
    json_ptr: *const std::os::raw::c_char,
    result_ptr: *mut FfiResult,
) {
    unsafe {
        if result_ptr.is_null() || json_ptr.is_null() {
            return;
        }

        let json_str = CStr::from_ptr(json_ptr).to_string_lossy();
        eprintln!("gpui_update_element: {}", json_str);

        match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(json) => {
                if let Some(id) = json.get("globalId").and_then(|v| v.as_u64()) {
                    let element_type = json
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let text = json
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let style = if let Some(style_obj) = json.get("style") {
                        crate::element_store::ElementStyle {
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
                        crate::element_store::ElementStyle::default()
                    };

                    let mut element_map = ELEMENT_MAP.lock().unwrap();

                    let element_type_clone = element_type.clone();
                    let text_clone = text.clone();
                    let children_data: Vec<u64> = json
                        .get("children")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|c| c.as_u64()).collect())
                        .unwrap_or_default();

                    let new_element = Arc::new(ReactElement {
                        global_id: id,
                        element_type: element_type_clone,
                        text: text_clone,
                        children: Vec::new(),
                        style: style.clone(),
                        event_handlers: None,
                    });
                    element_map.insert(id, new_element);

                    for &child_id in &children_data {
                        if !element_map.contains_key(&child_id) {
                            let placeholder = Arc::new(ReactElement {
                                global_id: child_id,
                                element_type: "placeholder".to_string(),
                                text: None,
                                children: Vec::new(),
                                style: crate::element_store::ElementStyle::default(),
                                event_handlers: None,
                            });
                            element_map.insert(child_id, placeholder);
                        }
                    }

                    let mut child_refs: Vec<Arc<ReactElement>> = Vec::new();
                    for &cid in &children_data {
                        if let Some(child) = element_map.get(&cid) {
                            child_refs.push(child.clone());
                        }
                    }

                    if let Some(existing) = element_map.get_mut(&id) {
                        let mut updated = (**existing).clone();
                        updated.children = child_refs;
                        updated.style = style.clone();
                        *existing = Arc::new(updated);
                    }

                    drop(element_map);

                    let result_buf = std::slice::from_raw_parts_mut(result_ptr as *mut u8, 8);
                    result_buf[0] = 0;
                }
            }
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn gpui_trigger_render(_result: *mut FfiResult) {
    RENDER_COUNT.fetch_add(1, Ordering::SeqCst);
}
