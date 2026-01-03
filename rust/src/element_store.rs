use std::ffi::CStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, PartialEq, Eq)]
pub struct ReactElement {
    pub global_id: u64,
    pub element_type: String,
    pub text: Option<String>,
    pub children: Vec<Arc<ReactElement>>,
}

lazy_static::lazy_static! {
    pub static ref ELEMENT_TREE: Arc<Mutex<Option<Arc<ReactElement>>>> = Arc::new(Mutex::new(None));
    pub static ref RENDER_TRIGGER: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    pub static ref THREAD_STARTED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub fn parse_element(data: &crate::ffi_types::ElementData) -> Result<Arc<ReactElement>, String> {
    let element_type = unsafe {
        if data.type_ptr.is_null() {
            return Err("type_ptr is null".to_string());
        }
        CStr::from_ptr(data.type_ptr)
            .to_str()
            .map_err(|e| format!("Invalid UTF-8 in type: {}", e))?
            .to_string()
    };

    let text = if !data.text_ptr.is_null() {
        Some(unsafe {
            CStr::from_ptr(data.text_ptr)
                .to_str()
                .map_err(|e| format!("Invalid UTF-8 in text: {}", e))?
                .to_string()
        })
    } else {
        None
    };

    let mut children = Vec::new();
    for i in 0..data.child_count {
        let child_id = unsafe { *data.children_ptr.offset(i as isize) };
        let child_tree = ELEMENT_TREE.lock().unwrap();
        if let Some(ref root) = *child_tree {
            if let Some(child) = find_element_by_id(root, child_id) {
                children.push(child.clone());
            }
        }
    }

    Ok(Arc::new(ReactElement {
        global_id: data.global_id,
        element_type,
        text,
        children,
    }))
}

fn find_element_by_id(element: &Arc<ReactElement>, id: u64) -> Option<Arc<ReactElement>> {
    if element.global_id == id {
        return Some(element.clone());
    }
    for child in &element.children {
        if let Some(found) = find_element_by_id(child, id) {
            return Some(found);
        }
    }
    None
}

pub fn set_element_tree(data: &crate::ffi_types::ElementData) {
    match parse_element(data) {
        Ok(element) => {
            let mut tree = ELEMENT_TREE.lock().unwrap();
            *tree = Some(element);
            RENDER_TRIGGER.fetch_add(1, Ordering::SeqCst);
        }
        Err(e) => {
            eprintln!("Failed to parse element: {}", e);
        }
    }
}

pub fn get_render_trigger() -> u64 {
    RENDER_TRIGGER.load(Ordering::SeqCst)
}
