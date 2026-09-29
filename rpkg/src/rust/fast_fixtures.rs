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

    /// Current value, as the `count` active binding.
    #[miniextendr(r6(active))]
    pub fn count(&self) -> i32 {
        self.value
    }

    /// Set the value through the `count` active binding.
    /// @param value New value.
    #[miniextendr(r6(setter, prop = "count"))]
    pub fn set_count(&mut self, value: i32) {
        self.value = value;
    }
}

// endregion

// region: trait-impl fixtures
//
// The impl block's `no_preconditions` reaches trait-impl wrappers too: an impl
// with method bodies, and an empty-body impl expanded from the trait's
// metadata (TPIE).

/// Scales a counter's value, for the trait-impl `no_preconditions` fixtures.
#[miniextendr]
pub trait FastScale {
    /// The counter's value times `k`.
    fn scaled(&self, k: f64) -> f64;
}

/// Default-mode trait impl: keeps the R-side type checks.
#[miniextendr(r6)]
impl FastScale for FastCounter {
    fn scaled(&self, k: f64) -> f64 {
        f64::from(self.value) * k
    }
}

/// `no_preconditions` trait impl: drops the R-side type checks.
#[miniextendr(r6, no_preconditions)]
impl FastScale for FastCounterNoPreconditions {
    fn scaled(&self, k: f64) -> f64 {
        f64::from(self.value) * k
    }
}

impl std::str::FromStr for FastCounterNoPreconditions {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self { value: s.parse()? })
    }
}

/// Empty-body `no_preconditions` trait impl: `from_str(s)` drops the R-side
/// check on `s`.
#[miniextendr(r6, no_preconditions)]
impl miniextendr_api::adapter_traits::RFromStr for FastCounterNoPreconditions {}

// endregion
