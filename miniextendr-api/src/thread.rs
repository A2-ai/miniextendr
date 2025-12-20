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
//!     unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(42) };
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
use crate::ffi::nonapi_stack::{
    get_r_cstack_dir, get_r_cstack_limit, get_r_cstack_start, set_r_cstack_limit,
};

#[cfg(feature = "nonapi")]
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global refcount for active stack check guards.
/// When count > 0, stack checking is disabled.
#[cfg(feature = "nonapi")]
static STACK_GUARD_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Original R_CStackLimit value before any guards were created.
/// Only valid when STACK_GUARD_COUNT > 0.
#[cfg(feature = "nonapi")]
static ORIGINAL_STACK_LIMIT: AtomicUsize = AtomicUsize::new(0);

/// RAII guard that disables R's stack checking and restores it on drop.
///
/// Use this when calling R APIs from a thread other than the main R thread.
///
/// Multiple guards can be active concurrently. Stack checking is only restored
/// when the last guard is dropped.
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
    // Unit struct - state is in global atomics
    _private: (),
}

#[cfg(feature = "nonapi")]
impl StackCheckGuard {
    /// Disable R's stack checking and return a guard that restores it on drop.
    ///
    /// Multiple guards can be created concurrently (even from different threads).
    /// Stack checking is only restored when the last guard is dropped.
    ///
    /// # Safety
    ///
    /// This is safe to call, but the caller must ensure that R has been
    /// initialized (the R_CStackLimit variable exists).
    #[must_use]
    pub fn disable() -> Self {
        // Atomically increment guard count and save original limit if we're the first
        let prev_count = STACK_GUARD_COUNT.fetch_add(1, Ordering::SeqCst);
        if prev_count == 0 {
            // We're the first guard - save the original limit
            let original = get_r_cstack_limit();
            ORIGINAL_STACK_LIMIT.store(original, Ordering::SeqCst);
            // Disable stack checking
            unsafe {
                set_r_cstack_limit(usize::MAX);
            }
        }
        Self { _private: () }
    }

    /// Get the original limit value that will be restored (for debugging).
    pub fn original_limit() -> usize {
        ORIGINAL_STACK_LIMIT.load(Ordering::SeqCst)
    }

    /// Get the current number of active guards (for debugging).
    pub fn active_count() -> usize {
        STACK_GUARD_COUNT.load(Ordering::SeqCst)
    }
}

#[cfg(feature = "nonapi")]
impl Drop for StackCheckGuard {
    fn drop(&mut self) {
        // Atomically decrement guard count and restore limit if we're the last
        let prev_count = STACK_GUARD_COUNT.fetch_sub(1, Ordering::SeqCst);
        if prev_count == 1 {
            // We were the last guard - restore the original limit
            let original = ORIGINAL_STACK_LIMIT.load(Ordering::SeqCst);
            unsafe {
                set_r_cstack_limit(original);
            }
        }
    }
}

/// Check if stack checking is currently disabled.
///
/// Returns `true` if `R_CStackLimit` is set to `usize::MAX`.
#[cfg(feature = "nonapi")]
pub fn is_stack_checking_disabled() -> bool {
    get_r_cstack_limit() == usize::MAX
}

/// Get the current stack checking configuration (for debugging).
///
/// Returns `(start, limit, direction)`.
#[cfg(feature = "nonapi")]
pub fn get_stack_config() -> (usize, usize, i32) {
    (
        get_r_cstack_start(),
        get_r_cstack_limit(),
        get_r_cstack_dir(),
    )
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
        set_r_cstack_limit(usize::MAX);
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
///     unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(42) }
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
/// Since we disable R's stack checking via `StackCheckGuard`, the size is about
/// practical needs rather than R enforcement. Deep recursion in R code (especially
/// recursive functions, `lapply` chains, or complex formulas) can use significant stack.
///
/// Rust's default thread stack is only 2 MiB, which may be insufficient for deep R calls.
/// We default to 8 MiB as a reasonable balance. Increase via [`RThreadBuilder::stack_size`]
/// if you encounter stack overflows.
pub const DEFAULT_R_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Stack size matching Windows R (64 MiB).
///
/// Use this if your code involves very deep recursion or complex R operations.
/// Windows R uses 64 MiB for its main thread since R 4.2.
#[cfg(windows)]
pub const WINDOWS_R_STACK_SIZE: usize = 64 * 1024 * 1024;

/// Spawn a new thread configured for calling R APIs.
///
/// This function:
/// 1. Sets a stack size appropriate for R (8 MiB by default)
/// 2. Automatically disables R's stack checking via `StackCheckGuard`
/// 3. Restores stack checking when the thread completes
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::thread::spawn_with_r;
///
/// let handle = spawn_with_r(|| {
///     // Safe to call R APIs here!
///     let result = unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(42) };
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

/// Builder for spawning threads with R-appropriate stack sizes.
///
/// This builder is always available and configures threads with stack sizes
/// suitable for R workloads (8 MiB default, vs Rust's 2 MiB default).
///
/// When the `nonapi` feature is enabled, spawned threads also automatically
/// disable R's stack checking via `StackCheckGuard`, allowing R API calls
/// from the thread.
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
///         // With `nonapi`: R API calls safe here
///         // Without `nonapi`: Just a thread with correct stack size
///     })?;
/// ```
pub struct RThreadBuilder {
    stack_size: usize,
    name: Option<String>,
}

impl Default for RThreadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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
    /// With `nonapi` feature: automatically disables R's stack checking.
    /// Without `nonapi` feature: just spawns with the configured stack size.
    pub fn spawn<F, T>(self, f: F) -> std::io::Result<std::thread::JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let mut builder = std::thread::Builder::new().stack_size(self.stack_size);

        if let Some(name) = self.name {
            builder = builder.name(name);
        }

        #[cfg(feature = "nonapi")]
        {
            builder.spawn(move || {
                let _guard = StackCheckGuard::disable();
                f()
            })
        }

        #[cfg(not(feature = "nonapi"))]
        {
            builder.spawn(f)
        }
    }

    /// Spawn and immediately join, returning the result.
    ///
    /// Convenience method for synchronous R calls on a separate thread.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = RThreadBuilder::new()
    ///     .spawn_join(|| unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(42) })
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
        let original = get_r_cstack_limit();

        {
            let _guard = StackCheckGuard::disable();
            // The original limit is saved in ORIGINAL_STACK_LIMIT
            assert_eq!(ORIGINAL_STACK_LIMIT.load(Ordering::SeqCst), original);
            assert!(is_stack_checking_disabled());
        }

        // After guard drops, should be restored
        assert_eq!(get_r_cstack_limit(), original);
    }
}
