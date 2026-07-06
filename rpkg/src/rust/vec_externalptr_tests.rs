//! Fixtures for `Vec<ExternalPtr<T>>` across the R boundary (issue #827).
//!
//! Exercises both directions — an R `list()` of external pointers as an
//! argument (`TryFromSexp for Vec<ExternalPtr<T>>`) and as a return value
//! (`IntoR for Vec<ExternalPtr<T>>`) — plus the `Option` variants where a
//! `NULL` list element maps to `None`.
//!
//! ## GC discipline when *building* the returned `Vec`
//!
//! Owned `ExternalPtr` handles self-root in the process-wide `ProtectPool` for
//! their whole Rust lifetime (#836/#841), so the natural
//! `.map(ExternalPtr::new).collect()` is GC-safe: each new handle is rooted the
//! instant it is created, so the allocation performed by the *next*
//! `ExternalPtr::new` cannot collect the earlier handles already sitting in the
//! `Vec`. Before #841 this pattern failed **40/40** under `gctorture(TRUE)`
//! (handles were unprotected — see
//! `reviews/2026-06-03-vec-externalptr-construction-gc.md`), which is exactly
//! what `gc_stress_vec_externalptr` now guards against.

use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::miniextendr;

/// Opaque handle used to exercise list-of-external-pointer conversions.
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct Bag {
    /// The integer payload carried by the handle.
    pub value: i32,
}

/// Build a `list()` of `Bag` handles with values `1..=n`.
///
/// Return side: `Vec<ExternalPtr<Bag>>` becomes an R `list()` of external
/// pointers.
/// @param n Number of handles to create.
#[miniextendr]
pub fn veptr_make_bags(n: i32) -> Vec<ExternalPtr<Bag>> {
    (0..n)
        .map(|i| ExternalPtr::new(Bag { value: i + 1 }))
        .collect()
}

/// Sum the payloads of a `list()` of `Bag` handles.
///
/// Argument side: an R `list()` of external pointers becomes
/// `Vec<ExternalPtr<Bag>>`.
/// @param bags A `list()` of `Bag` external pointers.
#[miniextendr]
pub fn veptr_sum_bags(bags: Vec<ExternalPtr<Bag>>) -> i32 {
    bags.iter().map(|b| b.value).sum()
}

/// Increment every `Bag` payload, returning a fresh `list()` of handles.
///
/// Round-trips both conversions in one call.
/// @param bags A `list()` of `Bag` external pointers.
#[miniextendr]
pub fn veptr_increment_bags(bags: Vec<ExternalPtr<Bag>>) -> Vec<ExternalPtr<Bag>> {
    bags.iter()
        .map(|b| ExternalPtr::new(Bag { value: b.value + 1 }))
        .collect()
}

/// Count the non-`NULL` handles in a `list()`.
///
/// Argument side with `NULL` holes: `Vec<Option<ExternalPtr<Bag>>>`.
/// @param bags A `list()` of `Bag` external pointers and/or `NULL`s.
#[miniextendr]
pub fn veptr_count_some(bags: Vec<Option<ExternalPtr<Bag>>>) -> i32 {
    bags.iter().filter(|b| b.is_some()).count() as i32
}

/// Build a `list()` of `n` slots where even indices are `NULL`.
///
/// Return side with `NULL` holes: `Vec<Option<ExternalPtr<Bag>>>`.
/// @param n Number of slots to create.
#[miniextendr]
pub fn veptr_make_bags_with_holes(n: i32) -> Vec<Option<ExternalPtr<Bag>>> {
    (0..n)
        .map(|i| {
            if i % 2 == 0 {
                None
            } else {
                Some(ExternalPtr::new(Bag { value: i }))
            }
        })
        .collect()
}

/// Drive both `Vec<ExternalPtr<Bag>>` conversion directions under GC pressure.
///
/// Builds the input list the natural way (`.map(ExternalPtr::new).collect()` —
/// GC-safe now that owned handles self-root, #836/#841), then drives the
/// `IntoR` list-assembly and the `TryFromSexp` readback — the code this fixture
/// exists to stress. The list is held under `OwnedProtect` across the readback
/// (which allocates). No arguments — suitable for the fast gctorture no-arg
/// sweep (issues #827, #430).
#[miniextendr(noexport)]
pub fn gc_stress_vec_externalptr() {
    use miniextendr_api::OwnedProtect;
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;

    // Return side: Vec<ExternalPtr<Bag>> -> list(). The element-by-element build
    // is the exact pattern that failed 40/40 under gctorture before #841.
    let bags: Vec<ExternalPtr<Bag>> = (1..=8)
        .map(|v| ExternalPtr::new(Bag { value: v }))
        .collect();
    // SAFETY: main thread. Protect the result so the readback can't collect it.
    let list = unsafe { OwnedProtect::new(bags.into_sexp()) };

    // Argument side: list() -> Vec<ExternalPtr<Bag>>; force element access.
    let roundtrip: Vec<ExternalPtr<Bag>> = TryFromSexp::try_from_sexp(list.get()).unwrap();
    let sum: i32 = roundtrip.iter().map(|b| b.value).sum();
    assert_eq!(sum, (1..=8).sum::<i32>());

    // Option variant with NULL holes, both directions.
    let opt: Vec<Option<ExternalPtr<Bag>>> = (0..8)
        .map(|i| {
            if i % 2 == 0 {
                None
            } else {
                Some(ExternalPtr::new(Bag { value: i }))
            }
        })
        .collect();
    // SAFETY: main thread.
    let opt_list = unsafe { OwnedProtect::new(opt.into_sexp()) };
    let opt_roundtrip: Vec<Option<ExternalPtr<Bag>>> =
        TryFromSexp::try_from_sexp(opt_list.get()).unwrap();
    assert_eq!(opt_roundtrip.iter().filter(|b| b.is_some()).count(), 4);
}
