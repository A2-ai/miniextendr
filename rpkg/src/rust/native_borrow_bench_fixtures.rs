//! Benchmark fixtures: the cost of the native-borrow preflight that runs
//! before argument conversion (#1504). `rpkg/dev/bench-native-borrow/` drives
//! them through their R wrappers; `miniextendr-bench/benches/native_borrow.rs`
//! has the same signatures and calls the C wrappers directly.
//!
//! The cases: a call with no borrowed argument, one and several borrowed
//! native vectors, borrowed strings, and an `Either` argument on each arm,
//! alone and next to a borrowed vector (which makes the wrapper run its alias
//! check). `nbb_either_alias` is called with the same vector twice: its owned
//! left arm wins, so the call is valid, and the check has to find out which
//! arm the conversion takes.

#[cfg(feature = "either")]
use miniextendr_api::either_impl::Either;
use miniextendr_api::miniextendr;

/// No borrowed argument.
/// @param x Integer scalar.
#[miniextendr(noexport)]
pub fn nbb_plain(x: i32) -> i32 {
    x
}

/// One borrowed double vector.
/// @param x Double vector.
#[miniextendr(noexport)]
pub fn nbb_slice(x: &[f64]) -> i32 {
    i32::try_from(x.len()).unwrap_or(i32::MAX)
}

/// Three borrowed vectors, one of them mutable, so the alias check runs.
/// @param a Double vector.
/// @param b Double vector, borrowed mutably.
/// @param c Integer vector.
#[miniextendr(noexport)]
pub fn nbb_slices(a: &[f64], b: &mut [f64], c: &[i32]) -> i32 {
    i32::try_from(a.len() + b.len() + c.len()).unwrap_or(i32::MAX)
}

/// One borrowed string.
/// @param s Character scalar.
#[miniextendr(noexport)]
pub fn nbb_str(s: &str) -> i32 {
    i32::try_from(s.len()).unwrap_or(i32::MAX)
}

/// Two borrowed strings and a vector of borrowed strings.
/// @param a Character scalar.
/// @param b Character scalar.
/// @param c Character vector.
#[miniextendr(noexport)]
pub fn nbb_strs(a: &str, b: &str, c: Vec<&str>) -> i32 {
    i32::try_from(a.len() + b.len() + c.len()).unwrap_or(i32::MAX)
}

/// An `Either` whose right arm borrows mutably, next to a borrowed vector.
/// @param a Integer scalar (left) or double vector (right).
/// @param b Double vector.
#[cfg(feature = "either")]
#[miniextendr(noexport)]
pub fn nbb_either(a: Either<i32, &mut [f64]>, b: &[f64]) -> i32 {
    let a = match a {
        Either::Left(n) => n,
        Either::Right(v) => i32::try_from(v.len()).unwrap_or(i32::MAX),
    };
    a.saturating_add(i32::try_from(b.len()).unwrap_or(i32::MAX))
}

/// An `Either` whose right arm borrows, alone: no alias check is needed.
/// @param a Integer scalar (left) or double vector (right).
#[cfg(feature = "either")]
#[miniextendr(noexport)]
pub fn nbb_either_alone(a: Either<i32, &[f64]>) -> i32 {
    match a {
        Either::Left(n) => n,
        Either::Right(v) => i32::try_from(v.len()).unwrap_or(i32::MAX),
    }
}

/// Called with one double scalar twice: the owned left arm takes it, so the
/// mutable right arm never borrows and the call is valid.
/// @param a Double scalar (left) or double vector (right).
/// @param b Double vector.
#[cfg(feature = "either")]
#[miniextendr(noexport)]
pub fn nbb_either_alias(a: Either<f64, &mut [f64]>, b: &[f64]) -> f64 {
    let len = |n: usize| u32::try_from(n).map_or(f64::INFINITY, f64::from);
    let a = match a {
        Either::Left(x) => x,
        Either::Right(v) => len(v.len()),
    };
    a + len(b.len())
}
