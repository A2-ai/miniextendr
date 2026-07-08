//! Fixtures exercising the `#[miniextendr(no_preconditions)]`,
//! `no_call_attribution`, and `fast` bundle options.
//!
//! These mirror the canonical identity-style `conv_i32_arg` / `conv_i32_ret`
//! fns from `conversions.rs` but with the fast-path knobs flipped, so that
//! benchmarks can compare wrapper layers head-to-head with everything else
//! held constant (same Rust body, same arg type, same return type).
//!
//! See `analysis/scaffolding-deep-findings-2026-05-20.md` for the cost map.

use miniextendr_api::{ExternalPtr, miniextendr};

// ---------------------------------------------------------------------------
// Variants on the i32 identity round-trip.
// ---------------------------------------------------------------------------

/// Identity (i32) with the standard wrapper. Baseline for comparison.
/// @param x Input value.
/// @export
#[miniextendr]
pub fn fast_i32_default(x: i32) -> i32 {
    x
}

/// Identity (i32) with `no_preconditions` — wrapper drops the `stopifnot(...)`
/// block. TryFromSexp still raises on bad input, message comes from Rust.
/// @param x Input value.
/// @export
#[miniextendr(no_preconditions)]
pub fn fast_i32_no_preconditions(x: i32) -> i32 {
    x
}

/// Identity (i32) with `no_call_attribution` — wrapper emits `.call = NULL`
/// instead of `match.call()`. Error UX falls back to `sys.call()`.
/// @param x Input value.
/// @export
#[miniextendr(no_call_attribution)]
pub fn fast_i32_no_call_attribution(x: i32) -> i32 {
    x
}

/// Identity (i32) with `fast` — bundle of `no_preconditions` +
/// `no_call_attribution`. Largest single-fn perf win.
/// @param x Input value.
/// @export
#[miniextendr(fast)]
pub fn fast_i32_fast(x: i32) -> i32 {
    x
}

// ---------------------------------------------------------------------------
// Multi-arg shape — to validate that preconditions scale by arg count.
// ---------------------------------------------------------------------------

/// Three-arg numeric sum, standard wrapper.
/// @param a,b,c Numeric scalars.
/// @export
#[miniextendr]
pub fn fast_sum3_default(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

/// Three-arg numeric sum, `fast` mode.
/// @param a,b,c Numeric scalars.
/// @export
#[miniextendr(fast)]
pub fn fast_sum3_fast(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

// ---------------------------------------------------------------------------
// Impl-block fast-path fixtures.
//
// Mirror SimpleCounter from trait_abi_tests.rs but with `fast` on the impl
// block so every generated method wrapper inherits no_preconditions +
// no_call_attribution. Used by bench scripts to compare class-system
// dispatch with and without the fast bundle.
// ---------------------------------------------------------------------------

#[derive(ExternalPtr)]
pub struct FastCounter {
    value: i32,
}

/// Default-mode counter (R6 wrapper with full stopifnot + match.call).
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
pub struct FastCounterFast {
    value: i32,
}

/// Fast-mode counter: every method wrapper drops stopifnot + match.call.
#[miniextendr(r6, internal, fast)]
impl FastCounterFast {
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
