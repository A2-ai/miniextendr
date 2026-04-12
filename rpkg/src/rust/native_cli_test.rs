// Test module for cli native R package FFI integration.
//
// Demonstrates calling cli's C progress bar API from Rust via bindgen-generated
// FFI bindings. The cli R package must be installed and loaded.

use crate::native::cli_ffi;
use miniextendr_api::prelude::*;

/// Returns the number of currently active cli progress bars.
///
/// @importFrom cli cli_progress_bar
#[miniextendr]
pub fn cli_active_progress_bars() -> i32 {
    // SAFETY: cli_progress_num() is resolved via R_GetCCallable at first call.
    // Must be called on the R main thread (which #[miniextendr] guarantees).
    unsafe { cli_ffi::cli_progress_num() }
}
