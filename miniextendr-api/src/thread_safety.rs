//! Thread safety utilities for R API calls.
//!
//! R is single-threaded - most R API calls must happen on the main thread.

use std::sync::OnceLock;
use std::thread::ThreadId;

static R_MAIN_THREAD_ID: OnceLock<ThreadId> = OnceLock::new();

/// Initialize the main thread ID. Call this once during R package init.
#[doc(hidden)]
pub fn init_main_thread() {
    let _ = R_MAIN_THREAD_ID.set(std::thread::current().id());
}

/// Check if the current thread is R's main thread.
#[inline]
pub fn is_r_main_thread() -> bool {
    R_MAIN_THREAD_ID
        .get()
        .map(|&id| id == std::thread::current().id())
        .unwrap_or(true) // If not initialized, assume we're on main thread
}
