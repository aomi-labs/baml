use std::os::raw::c_void;
use std::sync::Arc;

use crate::ffi;

/// Shared handle to an underlying `baml_runtime::BamlRuntime` instance.
///
/// Internally wraps the raw pointer returned by the CFFI layer and ensures the
/// runtime is destroyed exactly once when the last reference is dropped.
#[derive(Debug)]
pub(crate) struct RuntimeHandle {
    ptr: *const c_void,
}

impl RuntimeHandle {
    /// Create a new handle from a non-null runtime pointer.
    pub(crate) fn new(ptr: *const c_void) -> Self {
        Self { ptr }
    }

    /// Get the raw pointer for FFI calls.
    pub(crate) fn ptr(&self) -> *const c_void {
        self.ptr
    }
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // The pointer originates from `create_baml_runtime` and we ensure
            // destruction happens exactly once when the final handle is
            // dropped.
            ffi::destroy_baml_runtime(self.ptr);
        }
    }
}

unsafe impl Send for RuntimeHandle {}
unsafe impl Sync for RuntimeHandle {}

pub(crate) type RuntimeHandleArc = Arc<RuntimeHandle>;
