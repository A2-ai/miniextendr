//! Thread safety utilities for calling R from non-main threads.
//!
//! R's stack checking mechanism causes segfaults when R API functions are called
//! from threads other than the main R thread. This module provides utilities to
//! safely disable stack checking when crossing thread boundaries.
//!
//! # Background
//!
//! R tracks three variables for stack overflow detection (all non-API):
//! - `R_CStackStart` - top of the main thread's stack
//! - `R_CStackLimit` - stack size limit
//! - `R_CStackDir` - stack growth direction
//!
//! When R API functions check the stack, they compare the current stack pointer
//! against these bounds. On a different thread, the stack is completely different,
//! causing false stack overflow detection.
//!
//! # Solution
//!
//! Setting `R_CStackLimit` to `usize::MAX` disables stack checking entirely.
//! This is safe because:
//! 1. The OS still enforces real stack limits
//! 2. R will still function correctly, just without its own overflow detection
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::thread::StackCheckGuard;
//!
//! std::thread::spawn(|| {
//!     // This would segfault without the guard!
//!     let _guard = StackCheckGuard::disable();
//!
//!     // Now safe to call R APIs
//!     unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) };
//!
//!     // Guard restores original limit on drop
//! });
//! ```
//!
//! # Feature Gate
//!
//! This module requires the `nonapi` feature because it accesses non-API
//! R internals (`R_CStackLimit`, `R_CStackStart`, `R_CStackDir`).

#[cfg(feature = "nonapi")]
use crate::ffi::nonapi_stack::{R_CStackDir, R_CStackLimit, R_CStackStart};

/// RAII guard that disables R's stack checking and restores it on drop.
///
/// Use this when calling R APIs from a thread other than the main R thread.
///
/// # Example
///
/// ```ignore
/// let _guard = StackCheckGuard::disable();
/// // R API calls are now safe on this thread
/// // Original limit restored when _guard is dropped
/// ```
#[cfg(feature = "nonapi")]
pub struct StackCheckGuard {
    saved_limit: usize,
}

#[cfg(feature = "nonapi")]
impl StackCheckGuard {
    /// Disable R's stack checking and return a guard that restores it on drop.
    ///
    /// # Safety
    ///
    /// This is safe to call, but the caller must ensure that:
    /// 1. R has been initialized (the variables exist)
    /// 2. No other code is concurrently modifying these variables
    #[must_use]
    pub fn disable() -> Self {
        let saved_limit = unsafe { R_CStackLimit };
        unsafe {
            R_CStackLimit = usize::MAX;
        }
        Self { saved_limit }
    }

    /// Get the saved limit value (for debugging).
    pub fn saved_limit(&self) -> usize {
        self.saved_limit
    }
}

#[cfg(feature = "nonapi")]
impl Drop for StackCheckGuard {
    fn drop(&mut self) {
        unsafe {
            R_CStackLimit = self.saved_limit;
        }
    }
}

/// Check if stack checking is currently disabled.
///
/// Returns `true` if `R_CStackLimit` is set to `usize::MAX`.
#[cfg(feature = "nonapi")]
pub fn is_stack_checking_disabled() -> bool {
    unsafe { R_CStackLimit == usize::MAX }
}

/// Get the current stack checking configuration (for debugging).
///
/// Returns `(start, limit, direction)`.
#[cfg(feature = "nonapi")]
pub fn get_stack_config() -> (usize, usize, i32) {
    unsafe { (R_CStackStart, R_CStackLimit, R_CStackDir) }
}

/// Disable stack checking permanently for the current session.
///
/// Unlike [`StackCheckGuard`], this does not restore the original value.
/// Use this at program startup if you know you'll be calling R from multiple threads.
///
/// # Safety
///
/// Safe to call, but should only be called once during initialization.
#[cfg(feature = "nonapi")]
pub fn disable_stack_checking_permanently() {
    unsafe {
        R_CStackLimit = usize::MAX;
    }
}

/// Execute a closure with stack checking disabled.
///
/// This is a convenience wrapper around [`StackCheckGuard`].
///
/// # Example
///
/// ```ignore
/// let result = with_stack_checking_disabled(|| {
///     unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) }
/// });
/// ```
#[cfg(feature = "nonapi")]
pub fn with_stack_checking_disabled<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = StackCheckGuard::disable();
    f()
}

#[cfg(test)]
#[cfg(feature = "nonapi")]
mod tests {
    use super::*;

    #[test]
    fn test_guard_saves_and_restores() {
        let original = unsafe { R_CStackLimit };

        {
            let guard = StackCheckGuard::disable();
            assert_eq!(guard.saved_limit(), original);
            assert!(is_stack_checking_disabled());
        }

        // After guard drops, should be restored
        assert_eq!(unsafe { R_CStackLimit }, original);
    }
}
