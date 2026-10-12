//! None diagnostics use the R entry point's name (#2063).
//!
//! The worker fixtures need a worker feature, which rpkg's default build
//! (`just test-rpkg`) leaves off; the R install enables `worker-thread`.

use miniextendr_api::{ExternalPtr, miniextendr};

/// @noRd
#[miniextendr(noexport, no_worker, r_name = "none_named_main")]
pub fn none_rust_main() -> Option<()> {
    None
}

/// @noRd
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(noexport, worker, r_name = "none_named_worker")]
pub fn none_rust_worker() -> Option<()> {
    None
}

/// @noRd
#[miniextendr(noexport, no_worker, postfix = "_entry")]
pub fn none_postfix_main() -> Option<()> {
    None
}

/// @noRd
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(noexport, worker, postfix = "_entry")]
pub fn none_postfix_worker() -> Option<()> {
    None
}

/// @noRd
#[miniextendr(noexport, r_name = "none_named_scalar")]
pub fn none_rust_scalar() -> Option<i32> {
    None
}

#[derive(ExternalPtr)]
pub struct NoneNameLookup;

/// @noRd
#[miniextendr(env, noexport)]
impl NoneNameLookup {
    pub fn new() -> Self {
        Self
    }

    #[miniextendr(r_name = "find_value")]
    pub fn rust_find(&self) -> Option<i32> {
        None
    }

    #[miniextendr(postfix = "_entry")]
    pub fn find(&self) -> Option<i32> {
        None
    }
}

// The worker methods, in a second impl block of the same class (#1776),
// which repeats the class settings.
/// @noRd
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(env, noexport)]
impl NoneNameLookup {
    #[miniextendr(env(worker), r_name = "load_value")]
    pub fn rust_load() -> Option<i32> {
        #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
        assert!(!miniextendr_api::worker::is_r_main_thread());
        None
    }

    #[miniextendr(env(worker), postfix = "_entry")]
    pub fn load() -> Option<i32> {
        #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
        assert!(!miniextendr_api::worker::is_r_main_thread());
        None
    }
}
