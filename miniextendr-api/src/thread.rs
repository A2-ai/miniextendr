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

// =============================================================================
// Thread spawning with R-compatible settings
// =============================================================================

/// Default stack size for R-compatible threads (8 MiB).
///
/// R doesn't enforce a specific stack size - it uses whatever the OS provides:
/// - **Unix**: Typically 8 MiB from `ulimit -s`
/// - **Windows**: 64 MiB for the main thread (since R 4.2)
///
/// Since we disable R's stack checking via [`StackCheckGuard`], the size is about
/// practical needs rather than R enforcement. Deep recursion in R code (especially
/// recursive functions, `lapply` chains, or complex formulas) can use significant stack.
///
/// Rust's default thread stack is only 2 MiB, which may be insufficient for deep R calls.
/// We default to 8 MiB as a reasonable balance. Increase via [`RThreadBuilder::stack_size`]
/// if you encounter stack overflows.
#[cfg(feature = "nonapi")]
pub const DEFAULT_R_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Stack size matching Windows R (64 MiB).
///
/// Use this if your code involves very deep recursion or complex R operations.
/// Windows R uses 64 MiB for its main thread since R 4.2.
#[cfg(all(feature = "nonapi", windows))]
pub const WINDOWS_R_STACK_SIZE: usize = 64 * 1024 * 1024;

/// Spawn a new thread configured for calling R APIs.
///
/// This function:
/// 1. Sets a stack size appropriate for R (8 MiB by default)
/// 2. Automatically disables R's stack checking via [`StackCheckGuard`]
/// 3. Restores stack checking when the thread completes
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::thread::spawn_with_r;
///
/// let handle = spawn_with_r(|| {
///     // Safe to call R APIs here!
///     let result = unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) };
///     result
/// })?;
///
/// let sexp = handle.join().unwrap();
/// ```
///
/// # Panics
///
/// Returns an error if the thread cannot be spawned (e.g., resource exhaustion).
#[cfg(feature = "nonapi")]
pub fn spawn_with_r<F, T>(f: F) -> std::io::Result<std::thread::JoinHandle<T>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    RThreadBuilder::new().spawn(f)
}

/// Builder for spawning R-compatible threads with custom settings.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::thread::RThreadBuilder;
///
/// let handle = RThreadBuilder::new()
///     .stack_size(16 * 1024 * 1024)  // 16 MiB
///     .name("r-worker".to_string())
///     .spawn(|| {
///         // R API calls safe here
///     })?;
/// ```
#[cfg(feature = "nonapi")]
pub struct RThreadBuilder {
    stack_size: usize,
    name: Option<String>,
}

#[cfg(feature = "nonapi")]
impl Default for RThreadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "nonapi")]
impl RThreadBuilder {
    /// Create a new builder with default settings.
    ///
    /// Default stack size is [`DEFAULT_R_STACK_SIZE`] (8 MiB).
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack_size: DEFAULT_R_STACK_SIZE,
            name: None,
        }
    }

    /// Set the stack size for the thread.
    ///
    /// R typically requires more stack space than Rust's default 2 MiB.
    /// The default is 8 MiB to match typical R installations.
    #[must_use]
    pub fn stack_size(mut self, size: usize) -> Self {
        self.stack_size = size;
        self
    }

    /// Set the name for the thread (for debugging).
    #[must_use]
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Spawn the thread with the configured settings.
    ///
    /// The closure will automatically have R's stack checking disabled.
    pub fn spawn<F, T>(self, f: F) -> std::io::Result<std::thread::JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let mut builder = std::thread::Builder::new().stack_size(self.stack_size);

        if let Some(name) = self.name {
            builder = builder.name(name);
        }

        builder.spawn(move || {
            let _guard = StackCheckGuard::disable();
            f()
        })
    }

    /// Spawn and immediately join, returning the result.
    ///
    /// Convenience method for synchronous R calls on a separate thread.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = RThreadBuilder::new()
    ///     .spawn_join(|| unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) })
    ///     .unwrap();
    /// ```
    pub fn spawn_join<F, T>(self, f: F) -> std::thread::Result<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.spawn(f)
            .map_err(|e| Box::new(e) as Box<dyn std::any::Any + Send>)?
            .join()
    }
}

/// Spawn a scoped thread configured for calling R APIs.
///
/// Like [`spawn_with_r`] but uses scoped threads, allowing the closure to
/// borrow from the enclosing scope.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::thread::scope_with_r;
///
/// let data = vec![1, 2, 3];
///
/// std::thread::scope(|s| {
///     scope_with_r(s, |_| {
///         // Can borrow `data` here!
///         println!("data len: {}", data.len());
///         // R API calls also safe
///     });
/// });
/// ```
#[cfg(feature = "nonapi")]
pub fn scope_with_r<'scope, 'env, F, T>(
    scope: &'scope std::thread::Scope<'scope, 'env>,
    f: F,
) -> std::thread::ScopedJoinHandle<'scope, T>
where
    F: FnOnce(&'scope std::thread::Scope<'scope, 'env>) -> T + Send + 'scope,
    T: Send + 'scope,
{
    // Note: scoped threads don't support custom stack sizes in std
    // This is a known limitation. For custom stack sizes, use spawn_with_r.
    scope.spawn(move || {
        let _guard = StackCheckGuard::disable();
        f(scope)
    })
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
