//! TLS-backed convenience API for GC protection.
//!
//! Provides an **optional** convenience layer that maintains a thread-local
//! stack of scope pointers, allowing `tls::protect(x)` without passing
//! `&ProtectScope` explicitly.
//!
//! **Explicit `&ProtectScope` is recommended for most use cases.** It's simpler,
//! clearer about lifetimes, and doesn't rely on runtime state. Use this TLS
//! convenience only when threading scope references through deep call stacks
//! would be excessively verbose.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::gc_protect::tls;
//!
//! unsafe fn deep_helper(x: SEXP) -> SEXP {
//!     let y = tls::protect(allocate_something());
//!     combine(x, y.get())
//! }
//!
//! unsafe fn call_body(x: SEXP) -> SEXP {
//!     tls::with_protect_scope(|| {
//!         let x = tls::protect(x);
//!         deep_helper(x.get())
//!     })
//! }
//! ```
use super::ProtectScope;
use crate::ffi::SEXP;
use std::cell::RefCell;
use std::ptr::NonNull;

thread_local! {
    /// Stack of active protection scopes on this thread.
    static SCOPE_STACK: RefCell<Vec<NonNull<ProtectScope>>> = const { RefCell::new(Vec::new()) };
}

/// Execute a closure with a new protection scope as the current TLS scope.
///
/// The scope is pushed onto the thread-local stack, the closure runs, and
/// then the scope is popped and dropped (triggering `UNPROTECT(n)`).
///
/// # Safety
///
/// Must be called from the R main thread.
///
/// # Example
///
/// ```ignore
/// unsafe fn my_call(x: SEXP) -> SEXP {
///     tls::with_protect_scope(|| {
///         let x = tls::protect(x);
///         let y = tls::protect(allocate_something());
///         combine(x.get(), y.get())
///     })
/// }
/// ```
/// Guard that pops the TLS scope stack on drop (panic-safe cleanup).
struct TlsScopeGuard {
    scope_ptr: NonNull<ProtectScope>,
}

impl Drop for TlsScopeGuard {
    fn drop(&mut self) {
        SCOPE_STACK.with(|stack| {
            let popped = stack.borrow_mut().pop();
            debug_assert!(
                popped == Some(self.scope_ptr),
                "TLS scope stack corrupted: expected to pop same scope"
            );
        });
    }
}

/// Execute a closure with a protect scope that is accessible via TLS.
///
/// # Safety
///
/// Must be called from the R main thread.
#[inline]
pub unsafe fn with_protect_scope<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // SAFETY: caller guarantees R main thread
    let scope = unsafe { ProtectScope::new() };

    // Push scope pointer onto TLS stack
    // SAFETY: scope lives for the duration of this function call
    let scope_ptr = NonNull::from(&scope);
    SCOPE_STACK.with(|stack| {
        stack.borrow_mut().push(scope_ptr);
    });

    // Guard ensures TLS stack is popped even on panic.
    // The guard must be dropped BEFORE scope (declared after scope),
    // so the TLS stack is popped before UNPROTECT runs.
    let _guard = TlsScopeGuard { scope_ptr };

    // Run the user's closure - if it panics, _guard drops and pops TLS,
    // then scope drops and calls UNPROTECT(n)
    f()
}

/// Protect a value using the current TLS scope.
///
/// # Panics
///
/// Panics if called outside of a [`with_protect_scope`] block.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `x` must be a valid SEXP
/// - Must be called within a [`with_protect_scope`] block
///
/// # Example
///
/// ```ignore
/// tls::with_protect_scope(|| {
///     let x = tls::protect(some_sexp);
///     // use x...
/// })
/// ```
#[inline]
pub unsafe fn protect(x: SEXP) -> TlsRoot {
    let scope_ptr = SCOPE_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .copied()
            .expect("tls::protect called outside of with_protect_scope")
    });

    // SAFETY: scope_ptr is valid because we're inside with_protect_scope
    let scope: &ProtectScope = unsafe { scope_ptr.as_ref() };
    let root = unsafe { scope.protect(x) };

    TlsRoot { sexp: root.sexp }
}

/// Protect a value, returning the raw SEXP.
///
/// # Panics
///
/// Panics if called outside of a [`with_protect_scope`] block.
///
/// # Safety
///
/// Same as [`protect`].
#[inline]
pub unsafe fn protect_raw(x: SEXP) -> SEXP {
    let scope_ptr = SCOPE_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .copied()
            .expect("tls::protect_raw called outside of with_protect_scope")
    });

    let scope: &ProtectScope = unsafe { scope_ptr.as_ref() };
    unsafe { scope.protect_raw(x) }
}

/// Check if there is an active TLS scope.
#[inline]
pub fn has_active_scope() -> bool {
    SCOPE_STACK.with(|stack| !stack.borrow().is_empty())
}

/// Get the current scope's protection count.
///
/// Returns `None` if no scope is active.
#[inline]
pub fn current_count() -> Option<i32> {
    SCOPE_STACK.with(|stack| {
        stack.borrow().last().map(|ptr| {
            // SAFETY: pointer is valid while in with_protect_scope
            unsafe { ptr.as_ref() }.count()
        })
    })
}

/// Get the nesting depth of TLS scopes.
#[inline]
pub fn scope_depth() -> usize {
    SCOPE_STACK.with(|stack| stack.borrow().len())
}

/// A rooted SEXP from TLS protection.
///
/// This is similar to [`Root`] but without a compile-time lifetime tie to
/// the scope. The protection is valid as long as the enclosing
/// [`with_protect_scope`] block hasn't exited.
///
/// # Warning
///
/// Using a `TlsRoot` after its scope has exited is undefined behavior.
/// The compile-time lifetime checking of [`Root`] is safer; use TLS
/// convenience only when necessary.
#[derive(Clone, Copy)]
pub struct TlsRoot {
    sexp: SEXP,
}

impl TlsRoot {
    /// Get the underlying SEXP.
    #[inline]
    pub fn get(&self) -> SEXP {
        self.sexp
    }

    /// Consume and return the underlying SEXP.
    #[inline]
    pub fn into_raw(self) -> SEXP {
        self.sexp
    }
}

impl std::ops::Deref for TlsRoot {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sexp
    }
}
// endregion
