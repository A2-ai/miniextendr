//! Tests for `impl Trait` in `#[miniextendr]` return position.
//!
//! Return position works when the trait bound includes `IntoR`, since the
//! C wrapper only sees the opaque trait bound and needs `into_sexp()`.
//!
//! Argument position is not supported — Rust's type inference cannot resolve
//! the concrete type from `TryFromSexp + Trait` across a `let` binding.

use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::*;

/// `-> impl IntoR` with String behind it.
#[miniextendr]
pub fn impl_return_string() -> impl IntoR {
    String::from("hello from impl IntoR")
}

/// `-> impl IntoR` with Vec<i32> behind it.
#[miniextendr]
pub fn impl_return_vec() -> impl IntoR {
    vec![10, 20, 30]
}
