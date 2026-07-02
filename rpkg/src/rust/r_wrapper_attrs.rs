//! Tests for `r_name`, `r_entry`, `r_post_checks`, and `r_on_exit` attributes.

use miniextendr_api::{ExternalPtr, miniextendr};

// region: Standalone function: r_name

/// Check if an object is a widget.
///
/// @param x Value to check.
/// @export
#[miniextendr(r_name = "is.widget")]
pub fn is_widget(x: i32) -> bool {
    x > 0
}
// endregion

// region: Standalone function: r_entry

/// Coerce x to integer before passing to Rust.
///
/// @param x Value to coerce.
/// @export
#[miniextendr(r_entry = "x <- as.integer(x)")]
pub fn r_entry_demo(x: i32) -> i32 {
    x * 2
}
// endregion

// region: Standalone function: r_post_checks

/// Log a message after checks pass.
///
/// @param x Input value.
/// @export
#[miniextendr(r_post_checks = "message(\"validated\")")]
pub fn r_post_checks_demo(x: i32) -> i32 {
    x + 1
}
// endregion

// region: Standalone function: all three combined

/// Combined test: r_name + r_entry + r_post_checks.
///
/// @param n Number of widgets.
/// @export
#[miniextendr(
    r_name = "widget.create",
    r_entry = "n <- as.integer(n)",
    r_post_checks = "stopifnot(n > 0L)"
)]
pub fn create_widget(n: i32) -> i32 {
    n * 10
}
// endregion

// region: Standalone function: r_on_exit (short form)

/// Test on.exit cleanup with short form.
///
/// @param x Input value.
/// @export
#[miniextendr(r_on_exit = "message(\"cleanup ran\")")]
pub fn on_exit_short(x: i32) -> i32 {
    x + 1
}
// endregion

// region: Standalone function: r_on_exit (long form, add = false)

/// Test on.exit cleanup with add = false (overwrite previous).
///
/// @param x Input value.
/// @export
#[miniextendr(r_on_exit(expr = "message(\"cleanup overwrite\")", add = false))]
pub fn on_exit_no_add(x: i32) -> i32 {
    x + 2
}
// endregion

// region: Standalone function: r_on_exit (long form, after = false)

/// Test on.exit cleanup with after = false (LIFO order).
///
/// @param x Input value.
/// @export
#[miniextendr(r_on_exit(expr = "message(\"cleanup lifo\")", after = false))]
pub fn on_exit_lifo(x: i32) -> i32 {
    x + 3
}
// endregion

// region: R6 class with r_name on method

#[derive(ExternalPtr)]
pub struct WrapperDemo {
    value: i32,
}

/// Demo class for testing R wrapper attributes on methods.
#[miniextendr(r6)]
impl WrapperDemo {
    /// Create a new WrapperDemo.
    /// @param value Initial integer value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// Increment the value by one.
    #[miniextendr(r_name = "add_one")]
    pub fn increment(&mut self) {
        self.value += 1;
    }

    /// Add a custom amount to the value.
    /// @param by Amount to add.
    #[miniextendr(r_entry = "by <- as.integer(by)")]
    pub fn add_by(&mut self, by: i32) {
        self.value += by;
    }

    /// Get the current value.
    #[miniextendr(r_on_exit = "message(\"method cleanup\")")]
    pub fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: Standalone function: c_symbol (audit A10)

/// Custom C symbol name via `c_symbol = "..."`. The symbol itself is an
/// implementation detail — the test is that codegen + R dispatch still work.
///
/// @param x Input value.
/// @export
#[miniextendr(c_symbol = "mx_custom_c_symbol_fixture")]
pub fn c_symbol_demo(x: i32) -> i32 {
    x + 41
}
// endregion

// region: track_caller (audit A10, docs/TRACK_CALLER.md)

/// Pin the automatic `#[track_caller]` added by `#[miniextendr]`
/// (docs/TRACK_CALLER.md): with the attribute in effect,
/// `Location::caller()` inside this function resolves to the *caller's*
/// call site (in the generated wrapper), not to the line below. Returns
/// TRUE when the attribute is active.
///
/// @export
#[miniextendr]
pub fn track_caller_is_active() -> bool {
    let here = line!();
    let loc = std::panic::Location::caller();
    // Without the auto-added attribute, caller() would report exactly the
    // line above (here + 1) in this file.
    loc.line() != here + 1
}

/// Report the location `Location::caller()` resolves to through a
/// `#[track_caller]` helper chain (mirrors the "propagation through call
/// chains" section of docs/TRACK_CALLER.md). Returned as "file:line" for
/// R-side inspection.
///
/// @export
#[miniextendr]
pub fn track_caller_chain_location() -> String {
    #[track_caller]
    fn where_from() -> String {
        let loc = std::panic::Location::caller();
        format!("{}:{}", loc.file(), loc.line())
    }
    where_from()
}
// endregion
