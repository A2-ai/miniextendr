//! Test fixtures for `#[miniextendr(error_in_r)]` mode.
//!
//! These functions use error_in_r to transport Rust failures as tagged SEXP values
//! instead of raising immediate R errors. The generated R wrapper inspects the
//! tagged value and raises a proper R error condition past the Rust boundary.

use miniextendr_api::{miniextendr, miniextendr_module};

// =============================================================================
// Standalone functions
// =============================================================================

/// Standalone function that panics — error_in_r catches this.
#[miniextendr(error_in_r)]
pub fn error_in_r_panic() -> String {
    panic!("test panic from error_in_r")
}

/// Standalone function that returns Result::Err — becomes error value.
#[miniextendr(error_in_r)]
pub fn error_in_r_result_err() -> Result<String, String> {
    Err("test result error".to_string())
}

/// Standalone function that returns Result::Ok — works normally.
#[miniextendr(error_in_r)]
pub fn error_in_r_result_ok() -> Result<String, String> {
    Ok("success".to_string())
}

/// Standalone function that returns Option::None (unit) — becomes error value.
#[miniextendr(error_in_r)]
pub fn error_in_r_option_none() -> Option<()> {
    None
}

/// Standalone function that returns Option::Some(()) — returns NULL.
#[miniextendr(error_in_r)]
pub fn error_in_r_option_some() -> Option<()> {
    Some(())
}

/// Normal function (no error) — works fine.
#[miniextendr(error_in_r)]
pub fn error_in_r_normal() -> String {
    "all good".to_string()
}

/// Function returning i32 — tests numeric return.
#[miniextendr(error_in_r)]
pub fn error_in_r_i32_ok() -> i32 {
    42
}

/// Function returning Result<i32, String> with Err — becomes error value.
#[miniextendr(error_in_r)]
pub fn error_in_r_i32_err() -> Result<i32, String> {
    Err("integer conversion failed".to_string())
}

/// Panic with custom message.
#[miniextendr(error_in_r)]
pub fn error_in_r_panic_custom(msg: String) -> String {
    panic!("{}", msg)
}

// =============================================================================
// Env class with error_in_r methods
// =============================================================================

/// Counter for testing error_in_r with env class system.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ErrorInRCounter {
    value: i32,
}

#[miniextendr]
impl ErrorInRCounter {
    /// Create a new counter.
    fn new() -> Self {
        Self { value: 0 }
    }

    /// Get the current value — should always succeed.
    #[miniextendr(error_in_r)]
    fn get(&self) -> i32 {
        self.value
    }

    /// Increment — mutable method, should succeed and allow chaining.
    #[miniextendr(error_in_r)]
    fn inc(&mut self) {
        self.value += 1;
    }

    /// Deliberately panic in a method.
    #[miniextendr(error_in_r)]
    fn panic_method(&self) -> i32 {
        panic!("method panic in error_in_r")
    }

    /// Return Result::Err in a method.
    #[miniextendr(error_in_r)]
    fn failing_method(&self) -> Result<i32, String> {
        Err("method error".to_string())
    }
}

// =============================================================================
// R6 class with error_in_r methods
// =============================================================================

/// Widget for testing error_in_r with R6 class system.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ErrorInRR6Widget {
    name: String,
}

#[miniextendr(r6)]
impl ErrorInRR6Widget {
    /// Create a new widget.
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Get the name — should always succeed.
    #[miniextendr(error_in_r)]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Deliberately panic.
    #[miniextendr(error_in_r)]
    pub fn panic_method(&self) -> String {
        panic!("R6 panic in error_in_r")
    }

    /// Return Result::Err.
    #[miniextendr(error_in_r)]
    pub fn failing_result(&self) -> Result<String, String> {
        Err("R6 result error".to_string())
    }
}

miniextendr_module! {
    mod error_in_r_tests;

    fn error_in_r_panic;
    fn error_in_r_result_err;
    fn error_in_r_result_ok;
    fn error_in_r_option_none;
    fn error_in_r_option_some;
    fn error_in_r_normal;
    fn error_in_r_i32_ok;
    fn error_in_r_i32_err;
    fn error_in_r_panic_custom;

    impl ErrorInRCounter;
    impl ErrorInRR6Widget;
}
