//! NNG error type.

use core::ffi::CStr;
use core::fmt;

/// NNG error code wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NngError(pub i32);

impl NngError {
    /// Get human-readable error message from NNG.
    pub fn message(&self) -> &'static str {
        let ptr = unsafe { super::ffi::nng_strerror(self.0) };
        if ptr.is_null() {
            "unknown NNG error"
        } else {
            unsafe { CStr::from_ptr(ptr) }
                .to_str()
                .unwrap_or("invalid UTF-8 in NNG error")
        }
    }
}

impl fmt::Display for NngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NNG error {}: {}", self.0, self.message())
    }
}

impl std::error::Error for NngError {}

/// Result type for NNG operations.
pub type NngResult<T> = Result<T, NngError>;

/// Check an NNG return code, converting non-zero to `Err(NngError)`.
#[inline]
pub(crate) fn check(rv: i32) -> NngResult<()> {
    if rv == 0 { Ok(()) } else { Err(NngError(rv)) }
}
