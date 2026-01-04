use std::ffi::CString;
use std::os::raw::c_char;

#[repr(C)]
pub struct FfiResult {
    pub status: i32,
    pub error_msg: *mut c_char,
}

impl FfiResult {
    pub fn success() -> Self {
        FfiResult {
            status: 0,
            error_msg: std::ptr::null_mut(),
        }
    }

    pub fn error(message: &str) -> Self {
        FfiResult {
            status: 1,
            error_msg: CString::new(message).unwrap().into_raw(),
        }
    }
}

#[repr(C)]
pub struct WindowCreateResult {
    pub status: i32,
    pub window_id: u64,
    pub error_msg: *mut c_char,
}

impl WindowCreateResult {
    pub fn success(window_id: u64) -> Self {
        WindowCreateResult {
            status: 0,
            window_id,
            error_msg: std::ptr::null_mut(),
        }
    }

    pub fn error(message: &str) -> Self {
        WindowCreateResult {
            status: 1,
            window_id: 0,
            error_msg: CString::new(message).unwrap().into_raw(),
        }
    }
}

#[repr(C)]
pub struct ElementData {
    pub global_id: u64,
    pub type_ptr: *const c_char,
    pub text_ptr: *const c_char,
    pub child_count: u32,
    pub _padding: u32,
    pub children_ptr: *const u64,
}
