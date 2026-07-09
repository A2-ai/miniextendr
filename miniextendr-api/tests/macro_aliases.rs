//! #1105: `rust_error!` / `rust_condition!` collision-free macro aliases.
//!
//! Importing the crate-root modules `error` / `condition` (which happens under
//! any `use miniextendr_api::*;` glob) shadows the same-named `error!` /
//! `condition!` macros. This test proves that the `rust_*` aliases remain
//! usable alongside those module imports, with the same grammar as the
//! originals.
//!
//! Everything here is compile-only: each macro use lives in a closure that is
//! never called (the macros expand to `panic_any(...)`), so nothing panics at
//! runtime.

#![allow(dead_code)]

// These module imports are exactly what shadow the bare `error!`/`condition!`
// macros; bringing in the `rust_*` aliases below must still work.
use miniextendr_api::{condition, error};
use miniextendr_api::{rust_condition, rust_error};

fn _rust_error_grammars(name: &str, value: i32) {
    let _a = || rust_error!("plain message: {}", 1);
    let _b = || rust_error!("interpolated {name}");
    let _c = || rust_error!(class = "my_error", "missing field: {name}");
    let _d = || {
        rust_error!(
            class = "range_error",
            data = ("value", value),
            "bad {value}"
        )
    };
    let _e = || rust_error!(data = ("value", value), "bad {value}");
}

fn _rust_condition_grammars(n: i32) {
    let _a = || rust_condition!("progress {}", n);
    let _b = || rust_condition!(class = "my_progress", "processed {n} items");
    let _c = || rust_condition!(class = "my_progress", data = ("n", n), "processed {n}");
    let _d = || rust_condition!(data = ("n", n), "processed {n}");
}

// Reference the shadowing module imports so they are not "unused" — proving the
// modules and the alias macros coexist in the same scope.
#[allow(unused_imports)]
use condition::AsRError as _AsRError;
#[allow(unused_imports)]
use error::r_warning as _r_warning;

#[test]
fn aliases_compile_alongside_module_imports() {
    // The real assertion is that this file compiles at all.
}
