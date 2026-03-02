//! Tests for `r_name`, `r_entry`, and `r_post_checks` attributes.

use miniextendr_api::{miniextendr, miniextendr_module, ExternalPtr};

// ── Standalone function: r_name ──

/// Check if an object is a widget.
#[miniextendr(r_name = "is.widget")]
pub fn is_widget(x: i32) -> bool {
    x > 0
}

// ── Standalone function: r_entry ──

/// Coerce x to integer before passing to Rust.
#[miniextendr(r_entry = "x <- as.integer(x)")]
pub fn r_entry_demo(x: i32) -> i32 {
    x * 2
}

// ── Standalone function: r_post_checks ──

/// Log a message after checks pass.
#[miniextendr(r_post_checks = "message(\"validated\")")]
pub fn r_post_checks_demo(x: i32) -> i32 {
    x + 1
}

// ── Standalone function: all three combined ──

/// Combined test: r_name + r_entry + r_post_checks.
#[miniextendr(r_name = "widget.create", r_entry = "n <- as.integer(n)", r_post_checks = "stopifnot(n > 0L)")]
pub fn create_widget(n: i32) -> i32 {
    n * 10
}

// ── R6 class with r_name on method ──

#[derive(ExternalPtr)]
pub struct WrapperDemo {
    value: i32,
}

#[miniextendr(r6)]
impl WrapperDemo {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    #[miniextendr(r_name = "add_one")]
    pub fn increment(&mut self) {
        self.value += 1;
    }

    #[miniextendr(r_entry = "by <- as.integer(by)")]
    pub fn add_by(&mut self, by: i32) {
        self.value += by;
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }
}

miniextendr_module! {
    mod r_wrapper_attrs;

    fn is_widget;
    fn r_entry_demo;
    fn r_post_checks_demo;
    fn create_widget;

    impl WrapperDemo;
}
