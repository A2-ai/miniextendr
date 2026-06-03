//! Compile-pass test: `#[derive(RConvert)]` forwards R↔Rust conversions from a
//! newtype to its inner type.
//!
//! PR for sprint #835 (PR-3). Verifies that the derive emits well-formed scalar
//! `TryFromSexp` and `IntoR` impls for tuple and named newtypes, and that the
//! `into = false` / `from = false` direction flags suppress the right one. The
//! derive is scalar-only by necessity — the orphan rule forbids
//! `impl TryFromSexp for Vec<LocalNewtype>` in a downstream crate.
//!
//! The inner type is `f64` so the test needs no optional crate features. The
//! static `assert_from` / `assert_into` helpers force each generated impl's
//! trait-bound to resolve at compile time without touching the R runtime.

#![allow(dead_code)]

use miniextendr_api::{IntoR, RConvert, TryFromSexp};

/// Tuple newtype — both directions.
#[derive(RConvert)]
struct Meters(f64);

/// Named-field newtype — both directions.
#[derive(RConvert)]
struct Celsius {
    degrees: f64,
}

/// Explicit `forward` keyword is accepted as a no-op spelling of the default.
#[derive(RConvert)]
#[rconvert(forward)]
struct Explicit(f64);

/// `into = false` — emit only the `TryFromSexp` family.
#[derive(RConvert)]
#[rconvert(into = false)]
struct ReadOnly(f64);

/// `from = false` — emit only the `IntoR` family.
#[derive(RConvert)]
#[rconvert(from = false)]
struct WriteOnly(f64);

fn assert_from<T: TryFromSexp>() {}
fn assert_into<T: IntoR>() {}

fn _check() {
    // Tuple newtype — both directions.
    assert_from::<Meters>();
    assert_into::<Meters>();

    // Named-field newtype — both directions.
    assert_from::<Celsius>();
    assert_into::<Celsius>();

    // Explicit `forward`.
    assert_from::<Explicit>();
    assert_into::<Explicit>();

    // `into = false`: TryFromSexp present, IntoR absent.
    assert_from::<ReadOnly>();

    // `from = false`: IntoR present, TryFromSexp absent.
    assert_into::<WriteOnly>();
}

fn main() {}
