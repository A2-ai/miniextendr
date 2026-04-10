//! Tests for trait method options alignment: r_entry, r_post_checks, r_on_exit, lifecycle, strict.
//!
//! Verifies that these options (previously only available on standalone functions
//! and impl methods) now work correctly on trait impl methods.

use miniextendr_api::miniextendr;

// region: Trait definition

/// A trait for testing method-level options on trait impls.
#[miniextendr]
pub trait OptionsDemo {
    /// Basic method — no special options.
    fn basic_value(&self) -> i32;

    /// Method that will use r_entry in impl.
    fn with_entry(&self) -> i32;

    /// Method that will use r_on_exit in impl.
    fn with_exit(&self) -> i32;

    /// Method that will use r_post_checks in impl.
    fn with_checks(&self, n: i32) -> i32;

    /// Method that will use lifecycle in impl.
    fn deprecated_method(&self) -> i32;
}

// endregion

// region: Concrete type

#[derive(miniextendr_api::ExternalPtr)]
pub struct OptsTarget {
    v: i32,
}

#[miniextendr]
impl OptsTarget {
    pub fn new(v: i32) -> Self {
        Self { v }
    }
}

// endregion

// region: Trait impl with options

#[miniextendr(env)]
impl OptionsDemo for OptsTarget {
    fn basic_value(&self) -> i32 {
        self.v
    }

    /// The r_entry injects R code at the top of the wrapper body.
    #[miniextendr(r_entry = ".__entry_ran__ <- TRUE")]
    fn with_entry(&self) -> i32 {
        self.v
    }

    /// The r_on_exit registers cleanup via on.exit().
    #[miniextendr(r_on_exit = ".__exit_ran__ <- TRUE")]
    fn with_exit(&self) -> i32 {
        self.v
    }

    /// The r_post_checks injects validation before .Call().
    #[miniextendr(r_post_checks = "stopifnot(is.integer(n))")]
    fn with_checks(&self, n: i32) -> i32 {
        self.v + n
    }

    /// Lifecycle marks the method as deprecated.
    #[miniextendr(lifecycle = "deprecated")]
    fn deprecated_method(&self) -> i32 {
        self.v
    }
}

// endregion
