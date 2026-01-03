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

// ElementData struct with explicit padding to match JavaScript layout
// JavaScript layout (40 bytes total):
// 0-8: global_id (u64)
// 8-16: type_ptr (u64)
// 16-24: text_ptr (u64)
// 24-28: child_count (u32)
// 28-32: padding (4 bytes, for 8-byte alignment)
// 32-40: children_ptr (u64)
#[repr(C)]
pub struct ElementData {
    pub global_id: u64,
    pub type_ptr: *const c_char,
    pub text_ptr: *const c_char,
    pub child_count: u32,
    pub _padding: u32, // Explicit padding for alignment
    pub children_ptr: *const u64,
}
