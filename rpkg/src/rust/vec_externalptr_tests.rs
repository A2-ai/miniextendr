//! Fixtures for `Vec<ExternalPtr<T>>` across the R boundary (issue #827).
//!
//! Exercises both directions — an R `list()` of external pointers as an
//! argument (`TryFromSexp for Vec<ExternalPtr<T>>`) and as a return value
//! (`IntoR for Vec<ExternalPtr<T>>`) — plus the `Option` variants where a
//! `NULL` list element maps to `None`.
//!
//! ## GC discipline when *building* the returned `Vec`
//!
//! `ExternalPtr::new` does not root its `EXTPTRSXP` (see the no-op `Drop`), so a
//! `Vec<ExternalPtr<T>>` built element-by-element is unsafe under GC: each
//! `ExternalPtr::new` allocates, and that allocation can collect the *earlier*,
//! still-unprotected handles already sitting in the `Vec`. The `IntoR` impl
//! re-roots whatever handles it is handed before allocating the list, but it
//! cannot protect handles that were already collected during construction.
//!
//! These fixtures therefore build the `Vec` under a [`ProtectScope`] that roots
//! each handle as it is created. Making `ExternalPtr` self-rooting so the
//! natural `.map(ExternalPtr::new).collect()` is safe without this dance is
//! tracked in #836.

use miniextendr_api::ProtectScope;
use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::miniextendr;

/// Opaque handle used to exercise list-of-external-pointer conversions.
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct Bag {
    /// The integer payload carried by the handle.
    pub value: i32,
}

/// Build a `Vec<ExternalPtr<Bag>>`, rooting each handle as it is created.
///
/// `f` produces each `Bag` (or `None` to leave a `NULL` slot). The returned
/// `Vec` is GC-safe to construct because the [`ProtectScope`] keeps every
/// already-created handle on R's protect stack across the allocation performed
/// by the next `ExternalPtr::new`. The scope releases everything when it drops;
/// by then the caller's `IntoR` re-roots the handles before the list
/// allocation, and no allocation happens in the hand-off gap.
fn build_rooted<F>(n: i32, mut f: F) -> Vec<Option<ExternalPtr<Bag>>>
where
    F: FnMut(i32) -> Option<Bag>,
{
    // SAFETY: `#[miniextendr]` fixtures run on R's main thread by default.
    let scope = unsafe { ProtectScope::new() };
    (0..n)
        .map(|i| {
            f(i).map(|bag| {
                let p = ExternalPtr::new(bag);
                // SAFETY: main thread; `as_sexp()` is a live EXTPTRSXP.
                unsafe { scope.protect_raw(p.as_sexp()) };
                p
            })
        })
        .collect()
}

/// Build a `list()` of `Bag` handles with values `1..=n`.
///
/// Return side: `Vec<ExternalPtr<Bag>>` becomes an R `list()` of external
/// pointers.
/// @param n Number of handles to create.
#[miniextendr]
pub fn veptr_make_bags(n: i32) -> Vec<ExternalPtr<Bag>> {
    build_rooted(n, |i| Some(Bag { value: i + 1 }))
        .into_iter()
        .flatten()
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
    let next: Vec<i32> = bags.iter().map(|b| b.value + 1).collect();
    build_rooted(next.len() as i32, |i| {
        Some(Bag {
            value: next[i as usize],
        })
    })
    .into_iter()
    .flatten()
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
    build_rooted(n, |i| {
        if i % 2 == 0 {
            None
        } else {
            Some(Bag { value: i })
        }
    })
}

/// Drive both `Vec<ExternalPtr<Bag>>` conversion directions under GC pressure.
///
/// Builds the input list with [`build_rooted`] (GC-safe construction), then
/// drives the `IntoR` list-assembly and the `TryFromSexp` readback — the code
/// this fixture exists to stress. The list is held under `OwnedProtect` across
/// the readback (which allocates). No arguments — suitable for the fast
/// gctorture no-arg sweep (issues #827, #430).
#[miniextendr]
pub fn gc_stress_vec_externalptr() {
    use miniextendr_api::OwnedProtect;
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::into_r::IntoR;

    // Return side: Vec<ExternalPtr<Bag>> -> list(); protect the result so the
    // readback can't collect it mid-fixture.
    let bags: Vec<ExternalPtr<Bag>> = build_rooted(8, |i| Some(Bag { value: i + 1 }))
        .into_iter()
        .flatten()
        .collect();
    // SAFETY: main thread.
    let list = unsafe { OwnedProtect::new(bags.into_sexp()) };

    // Argument side: list() -> Vec<ExternalPtr<Bag>>; force element access.
    let roundtrip: Vec<ExternalPtr<Bag>> = TryFromSexp::try_from_sexp(list.get()).unwrap();
    let sum: i32 = roundtrip.iter().map(|b| b.value).sum();
    assert_eq!(sum, (1..=8).sum::<i32>());

    // Option variant with NULL holes, both directions.
    let opt: Vec<Option<ExternalPtr<Bag>>> =
        build_rooted(8, |i| if i % 2 == 0 { None } else { Some(Bag { value: i }) });
    // SAFETY: main thread.
    let opt_list = unsafe { OwnedProtect::new(opt.into_sexp()) };
    let opt_roundtrip: Vec<Option<ExternalPtr<Bag>>> =
        TryFromSexp::try_from_sexp(opt_list.get()).unwrap();
    assert_eq!(opt_roundtrip.iter().filter(|b| b.is_some()).count(), 4);
}
