mod app;
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

// Global map to store all elements by ID
lazy_static::lazy_static! {
    static ref ELEMENT_MAP: std::sync::Mutex<HashMap<u64, Arc<ReactElement>>> = std::sync::Mutex::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn gpui_init(_width: f32, _height: f32, result: *mut FfiResult) {
    unsafe {
        if !GPUI_INITIALIZED.load(Ordering::SeqCst) {
            start_gpui_thread();
            GPUI_INITIALIZED.store(true, Ordering::SeqCst);
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

        let global_id = if !global_id_ptr.is_null() {
            let buf = std::slice::from_raw_parts(global_id_ptr, 8);
            u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ])
        } else {
            0
        };

        let child_count = if !child_count_ptr.is_null() {
            let buf = std::slice::from_raw_parts(child_count_ptr, 8);
            u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]) as usize
        } else {
            0
        };

        let element_type = if !type_ptr.is_null() {
            CStr::from_ptr(type_ptr).to_string_lossy().to_string()
        } else {
            String::from("unknown")
        };

        let text = if !text_ptr.is_null() {
            Some(CStr::from_ptr(text_ptr).to_string_lossy().to_string())
        } else {
            None
        };

        let children: Vec<u64> = if !children_ptr.is_null() && child_count > 0 {
            let children_slice = std::slice::from_raw_parts(children_ptr, child_count);
            children_slice.to_vec()
        } else {
            Vec::new()
        };

        eprintln!(
            "gpui_render_frame: id={}, type={}, text={:?}, children={:?}",
            global_id, element_type, text, children
        );

        let element = Arc::new(ReactElement {
            global_id,
            element_type: element_type.clone(),
            text: text.clone(),
            children: Vec::new(),
        });

        let mut element_map = ELEMENT_MAP.lock().unwrap();
        element_map.insert(global_id, element.clone());

        drop(element_map);

        rebuild_tree(global_id, &children);

        let mut tree = ELEMENT_TREE.lock().unwrap();
        *tree = Some(get_root_element());

        RENDER_TRIGGER.fetch_add(1, Ordering::SeqCst);

        let result_buf = std::slice::from_raw_parts_mut(result_ptr as *mut u8, 8);
        result_buf[0] = 0;
    }
}

fn get_root_element() -> Arc<ReactElement> {
    let element_map = ELEMENT_MAP.lock().unwrap();

    let all_ids: std::collections::HashSet<u64> = element_map.keys().cloned().collect();

    for id in element_map.keys() {
        let mut is_root = true;
        for element in element_map.values() {
            if element.children.iter().any(|c| c.global_id == *id) {
                is_root = false;
                break;
            }
        }
        if is_root {
            return element_map.get(id).unwrap().clone();
        }
    }

    element_map.values().next().cloned().unwrap_or_else(|| {
        Arc::new(ReactElement {
            global_id: 0,
            element_type: "empty".to_string(),
            text: None,
            children: Vec::new(),
        })
    })
}

fn rebuild_tree(root_id: u64, children: &[u64]) {
    let element_map = ELEMENT_MAP.lock().unwrap();

    if let Some(root) = element_map.get(&root_id) {
        let child_elements: Vec<Arc<ReactElement>> = children
            .iter()
            .filter_map(|id| element_map.get(id).cloned())
            .collect();

        drop(element_map);

        let mut element_map = ELEMENT_MAP.lock().unwrap();
        if let Some(root) = element_map.get_mut(&root_id) {
            let root_mut = Arc::make_mut(root);
            root_mut.children = child_elements;
        }
    }
}

#[no_mangle]
pub extern "C" fn gpui_free_result(_result: FfiResult) {}

#[no_mangle]
pub extern "C" fn gpui_trigger_render(_result: *mut FfiResult) {
    RENDER_COUNT.fetch_add(1, Ordering::SeqCst);
}
