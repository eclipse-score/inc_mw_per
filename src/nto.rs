#[no_mangle]
pub extern "C" fn __my_thread_exit(_value_ptr: *mut *const core::ffi::c_void) {}
