//! Fixtures exercising the `#[miniextendr(no_preconditions)]` option.
//!
//! These mirror the canonical identity-style `conv_i32_arg` / `conv_i32_ret`
//! fns from `conversions.rs` but with the R-side type checks dropped, so that
//! benchmarks can compare wrapper layers head-to-head with everything else
//! held constant (same Rust body, same arg type, same return type).

use miniextendr_api::{ExternalPtr, miniextendr};

// region: variants on the i32 identity round-trip

/// Identity (i32) with the standard wrapper. Baseline for comparison.
/// @param x Input value.
/// @export
#[miniextendr]
pub fn fast_i32_default(x: i32) -> i32 {
    x
}

/// Identity (i32) with `no_preconditions` — wrapper drops the R-side type
/// checks. TryFromSexp still raises on bad input, message comes from Rust.
/// @param x Input value.
/// @export
#[miniextendr(no_preconditions)]
pub fn fast_i32_no_preconditions(x: i32) -> i32 {
    x
}

// endregion

// region: multi-arg shape, to validate that preconditions scale by arg count

/// Three-arg numeric sum, standard wrapper.
/// @param a,b,c Numeric scalars.
/// @export
#[miniextendr]
pub fn fast_sum3_default(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

/// Three-arg numeric sum, `no_preconditions` mode.
/// @param a,b,c Numeric scalars.
/// @export
#[miniextendr(no_preconditions)]
pub fn fast_sum3_no_preconditions(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

// endregion

// region: impl-block fixtures
//
// Mirror SimpleCounter from trait_abi_tests.rs but with `no_preconditions` on
// the impl block so every generated method wrapper drops its R-side type
// checks. Used by bench scripts to compare class-system dispatch with and
// without the checks.

#[derive(ExternalPtr)]
pub struct FastCounter {
    value: i32,
}

/// Default-mode counter (R6 wrapper with the full R-side type checks).
#[miniextendr(r6, internal)]
impl FastCounter {
    /// @param initial Initial counter value.
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    /// Get current value.
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Add `n` and return the new value.
    /// @param n Amount to add.
    pub fn add(&mut self, n: i32) -> i32 {
        self.value += n;
        self.value
    }
}

#[derive(ExternalPtr)]
pub struct FastCounterNoPreconditions {
    value: i32,
}

/// `no_preconditions` counter: every method wrapper drops its R-side type checks.
#[miniextendr(r6, internal, no_preconditions)]
impl FastCounterNoPreconditions {
    /// @param initial Initial counter value.
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    /// Get current value.
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Add `n` and return the new value.
    /// @param n Amount to add.
    pub fn add(&mut self, n: i32) -> i32 {
        self.value += n;
        self.value
    }
}

// endregion
