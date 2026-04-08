//! Test fixtures for `#[miniextendr(error_in_r)]` mode.
//!
//! These functions use error_in_r to transport Rust failures as tagged SEXP values
//! instead of raising immediate R errors. The generated R wrapper inspects the
//! tagged value and raises a proper R error condition past the Rust boundary.

use miniextendr_api::miniextendr;

// region: Standalone functions

/// Standalone function that panics -- error_in_r catches this.
///
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_panic() -> String {
    panic!("test panic from error_in_r")
}

/// Standalone function that returns Result::Err -- becomes error value.
///
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_result_err() -> Result<String, String> {
    Err("test result error".to_string())
}

/// Standalone function that returns Result::Ok -- works normally.
///
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_result_ok() -> Result<String, String> {
    Ok("success".to_string())
}

/// Standalone function that returns Option::None (unit) -- becomes error value.
///
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_option_none() -> Option<()> {
    None
}

/// Standalone function that returns Option::Some(()) -- returns NULL.
///
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_option_some() -> Option<()> {
    Some(())
}

/// Normal function (no error) -- works fine.
///
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_normal() -> String {
    "all good".to_string()
}

/// Function returning i32 -- tests numeric return.
///
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_i32_ok() -> i32 {
    42
}

/// Function returning Result<i32, String> with Err -- becomes error value.
///
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_i32_err() -> Result<i32, String> {
    Err("integer conversion failed".to_string())
}

/// Panic with custom message.
///
/// @param msg Custom panic message.
/// @export
#[miniextendr(error_in_r)]
pub fn error_in_r_panic_custom(msg: String) -> String {
    panic!("{}", msg)
}
// endregion

// region: Env class with error_in_r methods

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

    /// Get the current value -- should always succeed.
    #[miniextendr(error_in_r)]
    fn get(&self) -> i32 {
        self.value
    }

    /// Increment -- mutable method, should succeed and allow chaining.
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
// endregion

// region: R6 class with error_in_r methods

/// Widget for testing error_in_r with R6 class system.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ErrorInRR6Widget {
    name: String,
}

#[miniextendr(r6)]
impl ErrorInRR6Widget {
    /// Create a new widget.
    /// @param name Name for the widget.
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Get the name -- should always succeed.
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
// endregion

// region: S7 class with error_in_r methods

/// Gauge for testing error_in_r with S7 class system.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ErrorInRS7Gauge {
    level: f64,
}

/// S7 class for testing error_in_r with S7 class system.
/// @param level Numeric gauge level.
#[miniextendr(s7)]
impl ErrorInRS7Gauge {
    /// Create a new gauge.
    /// @param level Numeric gauge level.
    pub fn new(level: f64) -> Self {
        Self { level }
    }

    /// Read the level -- should always succeed.
    #[miniextendr(error_in_r)]
    pub fn read_level(&self) -> f64 {
        self.level
    }

    /// Set the level -- mutable, chainable.
    #[miniextendr(error_in_r)]
    pub fn set_level(&mut self, level: f64) {
        self.level = level;
    }

    /// Deliberately panic.
    #[miniextendr(error_in_r)]
    pub fn panic_method(&self) -> f64 {
        panic!("S7 panic in error_in_r")
    }

    /// Return Result::Err.
    #[miniextendr(error_in_r)]
    pub fn failing_result(&self) -> Result<f64, String> {
        Err("S7 result error".to_string())
    }
}
// endregion

// region: Trait with error_in_r methods

/// Trait for testing error_in_r on trait impls.
#[miniextendr]
pub trait Fallible {
    /// Get a value -- should succeed.
    fn get_value(&self) -> i32;

    /// Deliberately panic.
    fn will_panic(&self) -> i32;
}

/// Concrete type implementing Fallible with error_in_r on the trait impl.
#[derive(miniextendr_api::ExternalPtr)]
pub struct FallibleImpl {
    value: i32,
}

#[miniextendr]
impl FallibleImpl {
    /// Create a new FallibleImpl.
    fn new(value: i32) -> Self {
        Self { value }
    }

    /// Get value via inherent method.
    fn inherent_value(&self) -> i32 {
        self.value
    }
}

/// Fallible trait implementation for FallibleImpl with error_in_r on each method.
#[miniextendr]
impl Fallible for FallibleImpl {
    #[miniextendr(error_in_r)]
    fn get_value(&self) -> i32 {
        self.value
    }

    #[miniextendr(error_in_r)]
    fn will_panic(&self) -> i32 {
        panic!("trait panic in error_in_r")
    }
}
// endregion
