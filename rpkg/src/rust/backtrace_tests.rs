//! Test fixtures for backtrace module.

use miniextendr_api::prelude::*;

/// Register the configurable panic hook. This replaces the default hook.
/// With MINIEXTENDR_BACKTRACE unset, backtraces are suppressed.
#[miniextendr]
pub fn backtrace_install_hook() {
    miniextendr_api::backtrace::miniextendr_panic_hook();
}
