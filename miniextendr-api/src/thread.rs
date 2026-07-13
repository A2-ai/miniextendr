//! Advanced controls for R's process-global C-stack bounds.
//!
//! R's stack checking mechanism causes segfaults when R API functions are called
//! from threads other than the main R thread. This module exposes legacy tools
//! that disable that one check, but doing so does **not** make R's API safe on a
//! secondary thread.
//!
//! # R API calls remain main-thread-only
//!
//! R's global state, garbage collector, and error signaling are not made
//! thread-safe by changing `R_CStackLimit`. Writing R Extensions requires
//! package R API calls to stay on R's main thread and specifically says packages
//! must not change these variables to call stack-checking internals on a
//! secondary thread. Do not use this module as an off-main R bridge in package
//! code. The misleading package-facing surface is tracked for removal or
//! relocation in #1352.
//!
//! # Prefer [`crate::worker::with_r_thread`] in normal code
//!
//! The supported bridge is [`crate::worker::with_r_thread`], which routes a
//! closure from miniextendr's dedicated worker context to the recorded R main
//! thread. Arbitrary Rayon or `std::thread` workers cannot call R directly and
//! cannot use `with_r_thread` outside that active worker context.
//!
//! `StackCheckGuard` is gated behind the `nonapi` feature because it
//! mutates `R_CStackStart` / `R_CStackLimit` / `R_CStackDir`, none of which
//! are part of R's public C API. The lint **MXL301** currently recognizes this
//! guard as an unchecked-FFI context, but that only reflects the existing API;
//! it does not override R's main-thread contract.
//!
//! # Don't use `Rf_error` here either
//!
//! A longjmp from a non-main thread is undefined behaviour even with the
//! stack check disabled. Panic, capture the message in your guard's
//! fallback (see [`crate::ffi_guard::guarded_ffi_call_with_fallback`]), and
//! surface the failure to the main thread before letting R see it. The lint
//! **MXL300** rejects direct `Rf_error` calls in user code.
//!
//! # Cross references
//!
//! - [`crate::worker::with_r_thread`] — preferred path for crossing back to R.
//! - [`crate::sys`] — checked vs `*_unchecked` FFI surface.
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
//! Setting `R_CStackLimit` to `usize::MAX` disables R's own stack-address
//! check. The OS still enforces its real stack limit, but all other R threading
//! invariants remain unchanged.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::thread::StackCheckGuard;
//!
//! // Advanced embedded-host bookkeeping only; not package R API access.
//! let _guard = StackCheckGuard::disable();
//! assert!(miniextendr_api::thread::is_stack_checking_disabled());
//! ```
//!
//! # Feature Gate
//!
//! This module requires the `nonapi` feature because it accesses non-API
//! R internals (`R_CStackLimit`, `R_CStackStart`, `R_CStackDir`).

#[cfg(feature = "nonapi")]
use crate::sys::nonapi_stack::{
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

/// RAII guard that disables R's process-global stack check and restores it on drop.
///
/// This does not make R API calls safe on a secondary thread. R package code
/// must keep R API work on the main thread and should not use this guard.
///
/// Multiple guards can be active concurrently. Stack checking is only restored
/// when the last guard is dropped.
///
/// # Example
///
/// ```ignore
/// let _guard = StackCheckGuard::disable();
/// assert!(miniextendr_api::thread::is_stack_checking_disabled());
/// // Original process-global limit is restored when `_guard` is dropped.
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
    /// # Process-global contract
    ///
    /// The caller must own the relevant embedded-R lifecycle, ensure R is
    /// initialized, and prevent conflicting access to the stack globals. This
    /// is not a supported way for an R package to call R from another thread.
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
/// This is only for an embedding host that owns the complete R process
/// lifecycle. It must not be used by an R package to enable secondary-thread
/// R API calls.
///
/// # Process-global contract
///
/// Call at most once during controlled embedded-R initialization, with no
/// concurrent access to the stack globals.
#[cfg(feature = "nonapi")]
pub fn disable_stack_checking_permanently() {
    unsafe {
        set_r_cstack_limit(usize::MAX);
    }
    // Pin the saved limit to `usize::MAX` so a later `StackCheckGuard` drop can't
    // silently re-enable checking. If a guard is already active, `ORIGINAL_STACK_LIMIT`
    // holds the real pre-disable limit, and the last guard's drop would restore it —
    // undoing the "permanent" disable. Overwriting it with `usize::MAX` makes that
    // restore a no-op, so the disable stays durable regardless of guard ordering.
    ORIGINAL_STACK_LIMIT.store(usize::MAX, Ordering::SeqCst);
}

/// Execute a closure with stack checking disabled.
///
/// This is a convenience wrapper around [`StackCheckGuard`].
///
/// This helper does not authorize R API calls from a secondary thread.
#[cfg(feature = "nonapi")]
pub fn with_stack_checking_disabled<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = StackCheckGuard::disable();
    f()
}

// region: Thread spawning with legacy R-sized stacks

/// Default stack size for the legacy R-sized thread builder (8 MiB).
///
/// R doesn't enforce a specific stack size - it uses whatever the OS provides:
/// - **Unix**: Typically 8 MiB from `ulimit -s`
/// - **Windows**: 64 MiB for the main thread (since R 4.2)
///
/// Rust's default thread stack is only 2 MiB. The legacy builder reserves
/// 8 MiB to match common R-hosting configurations; this sizing choice does not
/// make R API access from the spawned thread supported.
pub const DEFAULT_R_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Stack size matching Windows R (64 MiB).
///
/// Use this for a deep pure-Rust workload that needs a reservation comparable
/// to Windows R's main thread. It does not authorize R API calls on the spawned
/// thread. Windows R uses 64 MiB for its main thread since R 4.2.
///
/// # Why larger than [`DEFAULT_R_STACK_SIZE`]
///
/// This is 8x the [`DEFAULT_R_STACK_SIZE`] used on other platforms. Two factors
/// motivate the larger reservation on Windows:
///
/// - Newly spawned Windows threads get a comparatively small *committed* stack by
///   default, and the OS does not grow a thread stack the way `ulimit -s` allows
///   on typical Unix configurations — so the size we ask for up front is closer to
///   the size we actually get.
/// - R's own choice of 64 MiB for its Windows main thread (since R 4.2) provides
///   a conservative reference point for stack-heavy native workloads.
///
/// This is a conservative reservation, not a precise measurement of any single
/// Rust call's stack frame.
#[cfg(windows)]
pub const WINDOWS_R_STACK_SIZE: usize = 64 * 1024 * 1024;

/// Spawn a thread with the legacy R-oriented stack configuration.
///
/// This function:
/// 1. Sets a stack size appropriate for R (8 MiB by default)
/// 2. With `nonapi`, disables R's process-global stack check via `StackCheckGuard`
/// 3. Restores stack checking when the thread completes
///
/// This does **not** make R API calls safe on the spawned thread. Keep package
/// R calls on the main thread; this misleading surface is tracked in #1352.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::thread::spawn_with_r;
///
/// let handle = spawn_with_r(|| expensive_pure_rust_computation())?;
///
/// let result = handle.join().unwrap();
/// ```
///
/// # Errors
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

/// Builder for spawning pure-Rust threads with a legacy R-sized stack.
///
/// This builder is always available and configures threads with stack sizes
/// suitable for stack-heavy Rust workloads (8 MiB default, vs Rust's 2 MiB default).
///
/// When the `nonapi` feature is enabled, spawned threads also automatically
/// disable R's stack checking via `StackCheckGuard`. That changes only the
/// stack-address check and does not make R API calls from the thread safe.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::thread::RThreadBuilder;
///
/// let handle = RThreadBuilder::new()
///     .stack_size(16 * 1024 * 1024)  // 16 MiB
///     .name("r-worker".to_string())
///     .spawn(expensive_pure_rust_computation)?;
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
    /// The default is 8 MiB, matching the legacy R-sized configuration rather
    /// than Rust's typical 2 MiB spawned-thread default.
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
    /// Convenience method for synchronously joining a configured thread.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = RThreadBuilder::new().spawn_join(|| 40 + 2).unwrap();
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

/// Spawn a scoped thread with the legacy R-oriented stack configuration.
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
///         // Keep the scoped work Rust-only; R API calls remain main-thread-only.
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
    use std::sync::Mutex;

    // These tests mutate process-global state (`R_CStackLimit`, `STACK_GUARD_COUNT`,
    // `ORIGINAL_STACK_LIMIT`), so they must not run concurrently. Serialize them.
    static STACK_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_guard_saves_and_restores() {
        let _serial = STACK_TEST_LOCK.lock().unwrap();
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

    /// Regression test for the "permanent disable silently undone by a guard drop" bug.
    ///
    /// Ordering (a): a `StackCheckGuard` is active (so `ORIGINAL_STACK_LIMIT` holds the
    /// real pre-disable limit) when `disable_stack_checking_permanently()` is called.
    /// Before the fix, the guard's `Drop` restored that saved limit, re-enabling stack
    /// checking and undoing the "permanent" disable. After the fix,
    /// `disable_stack_checking_permanently()` also pins `ORIGINAL_STACK_LIMIT` to
    /// `usize::MAX`, so the drop's restore is a no-op and the limit stays at `usize::MAX`.
    #[test]
    fn test_permanent_disable_survives_guard_drop() {
        let _serial = STACK_TEST_LOCK.lock().unwrap();
        let original = get_r_cstack_limit();

        {
            // Guard active: ORIGINAL_STACK_LIMIT now holds the real pre-disable limit.
            let _guard = StackCheckGuard::disable();
            assert_eq!(ORIGINAL_STACK_LIMIT.load(Ordering::SeqCst), original);

            // Permanent disable while the guard is still alive.
            disable_stack_checking_permanently();
            // The saved limit is pinned to MAX so a later restore can't re-enable checking.
            assert_eq!(ORIGINAL_STACK_LIMIT.load(Ordering::SeqCst), usize::MAX);
        }
        // Guard dropped: limit must remain disabled, not bounce back to `original`.
        assert!(is_stack_checking_disabled());
        assert_eq!(get_r_cstack_limit(), usize::MAX);

        // Restore the real limit so we don't leak the permanent disable into other tests.
        unsafe {
            set_r_cstack_limit(original);
        }
        ORIGINAL_STACK_LIMIT.store(0, Ordering::SeqCst);
    }
}
// endregion
