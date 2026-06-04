//! Compile-pass test: `#[derive(TryFromSexp)]` / `#[derive(IntoR)]` forward
//! R↔Rust conversions from a newtype to its inner type, and the container
//! blankets in `miniextendr_api::newtype` light up `Vec` / `Option` /
//! `Vec<Option>` automatically (issue #844).
//!
//! Direction is chosen by *which* derive is listed — there are no attributes.
//! The inner type is `f64` so the test needs no optional crate features. The
//! static `assert_from` / `assert_into` helpers force each generated impl's
//! trait-bound to resolve at compile time without touching the R runtime.

#![allow(dead_code)]

use miniextendr_api::{IntoR, TryFromSexp};

/// Tuple newtype — both directions.
#[derive(TryFromSexp, IntoR)]
struct Meters(f64);

/// Named-field newtype — both directions.
#[derive(TryFromSexp, IntoR)]
struct Celsius {
    degrees: f64,
}

/// Read-only newtype — derive only `TryFromSexp`.
#[derive(TryFromSexp)]
struct ReadOnly(f64);

/// Write-only newtype — derive only `IntoR`.
#[derive(IntoR)]
struct WriteOnly(f64);

fn assert_from<T: TryFromSexp>() {}
fn assert_into<T: IntoR>() {}

fn _check() {
    // Tuple newtype — both directions, scalar.
    assert_from::<Meters>();
    assert_into::<Meters>();

    // Named-field newtype — both directions, scalar.
    assert_from::<Celsius>();
    assert_into::<Celsius>();

    // Container blankets light up from the scalar derives (issue #844):
    // Vec / Option read; Vec / Vec<Option> write. (`Option<T>` -> R is the one
    // shape deliberately not granted — it collides with `IntoR for Option<&T>`.)
    assert_from::<Vec<Meters>>();
    assert_from::<Option<Meters>>();
    assert_from::<Vec<Option<Meters>>>();
    assert_into::<Vec<Meters>>();
    assert_into::<Vec<Option<Meters>>>();

    // Single-direction derives: each exposes only its own family.
    assert_from::<ReadOnly>();
    assert_from::<Vec<ReadOnly>>();
    assert_into::<WriteOnly>();
    assert_into::<Vec<WriteOnly>>();
}

fn main() {}
