use std::ffi::{CStr, c_char};

/// Read C string pointer to String, return default if null
#[inline]
pub unsafe fn read_c_string(ptr: *const c_char, default: &str) -> String {
	if ptr.is_null() {
		default.to_string()
	} else {
		unsafe { CStr::from_ptr(ptr).to_string_lossy().to_string() }
	}
}

/// Read optional C string pointer to Option<String>
#[inline]
pub unsafe fn read_opt_c_string(ptr: *const c_char) -> Option<String> {
	if ptr.is_null() {
		None
	} else {
		Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().to_string() })
	}
}

/// Read C string pointer to String
#[inline]
pub unsafe fn read_c_str(ptr: *const c_char) -> Result<String, std::str::Utf8Error> {
	unsafe { CStr::from_ptr(ptr).to_str().map(|s| s.to_string()) }
}

/// Convert *const u8 pointer to u64 (for FFI 64-bit integer passing)
#[inline]
pub unsafe fn ptr_to_u64(ptr: *const u8) -> u64 {
	if ptr.is_null() {
		0
	} else {
		let buf = unsafe { std::slice::from_raw_parts(ptr, 8) };
		u64::from_le_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]])
	}
}

/// Validate result pointer and return mutable reference
#[inline]
pub unsafe fn validate_result_ptr<T>(ptr: *mut T, context: &str) -> Option<&mut T> {
	if ptr.is_null() {
		log::error!("{}: result pointer is null", context);
		None
	} else {
		unsafe { Some(&mut *ptr) }
	}
}
