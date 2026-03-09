//! Tests for `r_name`, `r_entry`, `r_post_checks`, and `r_on_exit` attributes.

use miniextendr_api::{ExternalPtr, miniextendr};

// ── Standalone function: r_name ──

/// Check if an object is a widget.
///
/// @param x Value to check.
/// @export
#[miniextendr(r_name = "is.widget")]
pub fn is_widget(x: i32) -> bool {
    x > 0
}

// ── Standalone function: r_entry ──

/// Coerce x to integer before passing to Rust.
///
/// @param x Value to coerce.
/// @export
#[miniextendr(r_entry = "x <- as.integer(x)")]
pub fn r_entry_demo(x: i32) -> i32 {
    x * 2
}

// ── Standalone function: r_post_checks ──

/// Log a message after checks pass.
///
/// @param x Input value.
/// @export
#[miniextendr(r_post_checks = "message(\"validated\")")]
pub fn r_post_checks_demo(x: i32) -> i32 {
    x + 1
}

// ── Standalone function: all three combined ──

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

// ── Standalone function: r_on_exit (short form) ──

/// Test on.exit cleanup with short form.
///
/// @param x Input value.
/// @export
#[miniextendr(r_on_exit = "message(\"cleanup ran\")")]
pub fn on_exit_short(x: i32) -> i32 {
    x + 1
}

// ── Standalone function: r_on_exit (long form, add = false) ──

/// Test on.exit cleanup with add = false (overwrite previous).
///
/// @param x Input value.
/// @export
#[miniextendr(r_on_exit(expr = "message(\"cleanup overwrite\")", add = false))]
pub fn on_exit_no_add(x: i32) -> i32 {
    x + 2
}

// ── Standalone function: r_on_exit (long form, after = false) ──

/// Test on.exit cleanup with after = false (LIFO order).
///
/// @param x Input value.
/// @export
#[miniextendr(r_on_exit(expr = "message(\"cleanup lifo\")", after = false))]
pub fn on_exit_lifo(x: i32) -> i32 {
    x + 3
}

// ── R6 class with r_name on method ──

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
