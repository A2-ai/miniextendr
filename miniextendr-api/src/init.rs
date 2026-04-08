//! Package initialization for miniextendr R packages.
//!
//! [`package_init`](crate::init::package_init) consolidates all initialization steps that were previously
//! scattered across `entrypoint.c.in`. The `miniextendr_init!` proc macro
//! generates the `R_init_*` entry point that calls this function.
//!
//! # Usage
//!
//! In your crate's `lib.rs`:
//!
//! ```ignore
//! miniextendr_init!(mypkg);
//! ```
//!
//! This expands to an `extern "C-unwind" fn R_init_mypkg(dll)` that calls
//! [`package_init`](crate::init::package_init) with the appropriate package name.

use crate::ffi::{DllInfo, R_forceSymbols, R_useDynamicSymbols, Rboolean};
use std::ffi::CStr;

/// Initialize a miniextendr R package.
///
/// This performs all initialization steps in the correct order:
///
/// 1. Install panic hook for better error messages
/// 2. Record main thread ID (and optionally spawn worker thread)
/// 3. Assert UTF-8 locale
/// 4. Set ALTREP package name
/// 5. Register mx_abi C-callables for cross-package trait dispatch
/// 6. Register all `#[miniextendr]` routines and ALTREP classes
/// 7. Lock down dynamic symbols
///
/// # Safety
///
/// Must be called from R's main thread during `R_init_*`.
/// `dll` must be a valid pointer provided by R.
/// `pkg_name` must be a valid null-terminated C string that lives for the
/// duration of the R session (typically a string literal).
pub unsafe fn package_init(dll: *mut DllInfo, pkg_name: &CStr) {
    unsafe {
        // When loaded as a cdylib for wrapper generation, skip full init.
        // Only routine registration is needed so .Call(miniextendr_write_wrappers) works.
        // The env var is set by Makevars before dyn.load().
        let wrapper_gen = std::env::var_os("MINIEXTENDR_CDYLIB_WRAPPERS").is_some();

        // 1. Record main thread ID (and optionally spawn worker thread)
        // Always needed: checked FFI variants (R_useDynamicSymbols, etc.)
        // route through with_r_thread() which requires runtime_init.
        crate::worker::miniextendr_runtime_init();

        if !wrapper_gen {
            // 2. Install panic hook for better error messages
            // Skipped during wrapper-gen: on Windows, set_hook during DLL init
            // can fail with "failed to initiate panic, error 5" because the
            // panic infrastructure isn't fully available during DLL loading.
            crate::backtrace::miniextendr_panic_hook();

            // 3. Assert UTF-8 locale
            crate::encoding::miniextendr_assert_utf8_locale();

            // 3b. Install R console logger (if log feature enabled)
            #[cfg(feature = "log")]
            crate::optionals::log_impl::install_r_logger();

            // 3c. Compute SEXPREC data offset (used by Arrow SEXP recovery, etc.)
            crate::r_memory::init_sexprec_data_offset();

            // 4. Set ALTREP package name and DllInfo
            crate::miniextendr_set_altrep_pkg_name(pkg_name.as_ptr());
            crate::set_altrep_dll_info(dll);

            // 5. Register mx_abi C-callables
            crate::mx_abi::mx_abi_register(pkg_name);
        }

        // 6. Register .Call routines (and ALTREP classes, unless wrapper-gen)
        crate::registry::miniextendr_register_routines(dll);

        // 7. Lock down dynamic symbols
        R_useDynamicSymbols(dll, Rboolean::FALSE);
        R_forceSymbols(dll, Rboolean::TRUE);
    }
}
