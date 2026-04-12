//! NNG message wrapper.

use super::error::{NngResult, check};
use super::ffi;
use core::ffi::c_void;
use std::mem::ManuallyDrop;

/// Owned NNG message.
///
/// `Drop` calls `nng_msg_free`. After sending via `NngSocket::send_msg()`,
/// the message is consumed (NNG takes ownership).
pub struct NngMsg {
    ptr: *mut ffi::nng_msg,
}

// NNG messages are thread-safe
unsafe impl Send for NngMsg {}

impl NngMsg {
    /// Allocate a new empty message.
    pub fn new() -> NngResult<Self> {
        let mut ptr: *mut ffi::nng_msg = std::ptr::null_mut();
        check(unsafe { ffi::nng_msg_alloc(&mut ptr, 0) })?;
        Ok(Self { ptr })
    }

    /// Create a message from bytes.
    pub fn from_bytes(data: &[u8]) -> NngResult<Self> {
        let mut msg = Self::new()?;
        msg.append(data)?;
        Ok(msg)
    }

    /// Get the message body as a byte slice.
    pub fn body(&self) -> &[u8] {
        let ptr = unsafe { ffi::nng_msg_body(self.ptr) };
        let len = unsafe { ffi::nng_msg_len(self.ptr) };
        if len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
        }
    }

    /// Get message body length.
    pub fn len(&self) -> usize {
        unsafe { ffi::nng_msg_len(self.ptr) }
    }

    /// Check if message is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Append bytes to the message body.
    pub fn append(&mut self, data: &[u8]) -> NngResult<()> {
        check(unsafe { ffi::nng_msg_append(self.ptr, data.as_ptr() as *const c_void, data.len()) })
    }

    /// Clear the message body.
    pub fn clear(&mut self) {
        unsafe { ffi::nng_msg_clear(self.ptr) }
    }

    /// Consume self and return the raw pointer.
    /// Caller (NNG send) takes ownership — no Drop.
    pub(crate) fn into_raw(self) -> *mut ffi::nng_msg {
        let msg = ManuallyDrop::new(self);
        msg.ptr
    }

    /// Wrap a raw pointer (from NNG recv).
    /// Caller must ensure the pointer is valid and owned.
    pub(crate) unsafe fn from_raw(ptr: *mut ffi::nng_msg) -> Self {
        Self { ptr }
    }
}

impl Drop for NngMsg {
    fn drop(&mut self) {
        unsafe { ffi::nng_msg_free(self.ptr) }
    }
}

impl AsRef<[u8]> for NngMsg {
    fn as_ref(&self) -> &[u8] {
        self.body()
    }
}
