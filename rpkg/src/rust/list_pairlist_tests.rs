//! A `List` argument given a pairlist (#1866), driven by
//! `tests/testthat/test-list-pairlist.R`.
//!
//! `List`, `Option<List>` and `NamedList` refuse a pairlist with a type
//! error. They used to coerce it to a list, a new object that nothing rooted,
//! so the first allocation in the function could free it. Each function here
//! allocates lists of the argument's length before it reads the argument: under
//! `gctorture(TRUE)` a list cell that nothing roots is freed and reused by one
//! of them, and the read sees the reused cell's values.

use miniextendr_api::from_r::{SexpError, SexpTypeError};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::NamedList;
use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::{List, TryFromSexp, miniextendr};

/// Allocate 64 lists of `len` integer scalars (negative, so none equals an
/// argument's value), rooted in `scope`. A freed list cell of that length is
/// the first candidate for one of them.
fn churn_lists(scope: &ProtectScope, len: isize) {
    for round in 0..64i32 {
        let list = unsafe { scope.alloc_vector(miniextendr_api::SEXPTYPE::VECSXP, len) };
        for i in 0..len {
            let value = -(round * 1000 + i32::try_from(i).expect("a small index") + 1);
            // No allocation between the scalar and storing it in the rooted list.
            list.get().set_vector_elt(i, SEXP::scalar_integer(value));
        }
    }
}

/// The integer elements of `x`, read after the body allocates lists of its
/// length.
/// @param x A list of integer scalars.
#[miniextendr(noexport)]
pub fn list_read_after_alloc(x: List) -> Vec<Option<i32>> {
    let scope = unsafe { ProtectScope::new() };
    churn_lists(&scope, x.len());
    (0..x.len()).map(|i| x.get_index::<i32>(i)).collect()
}

/// As [`list_read_after_alloc`], for an `Option<List>`: `NULL` reads as an
/// empty vector.
/// @param x A list of integer scalars, or `NULL`.
#[miniextendr(noexport)]
pub fn option_list_read_after_alloc(x: Option<List>) -> Vec<Option<i32>> {
    let Some(x) = x else {
        return Vec::new();
    };
    let scope = unsafe { ProtectScope::new() };
    churn_lists(&scope, x.len());
    (0..x.len()).map(|i| x.get_index::<i32>(i)).collect()
}

/// As [`list_read_after_alloc`], for a `NamedList`.
/// @param x A named list of integer scalars.
#[miniextendr(noexport)]
pub fn named_list_read_after_alloc(x: NamedList) -> Vec<Option<i32>> {
    let scope = unsafe { ProtectScope::new() };
    churn_lists(&scope, x.len());
    (0..x.len()).map(|i| x.get_index::<i32>(i)).collect()
}

/// Assert that `err` is the type error of a pairlist given for a list.
fn assert_pairlist_refused(err: SexpTypeError, what: &str) {
    use miniextendr_api::SEXPTYPE::{LISTSXP, VECSXP};
    assert_eq!((err.expected, err.actual), (VECSXP, LISTSXP), "{what}");
}

/// Convert a pairlist and a list of the same named values under GC pressure.
///
/// The pairlist is refused by `List`, `Option<List>` and `NamedList`, so no
/// view of an unrooted copy exists. The list converts to all three; the body
/// then allocates lists of the same length and reads every value back
/// through each view, which the argument list still roots.
///
/// No arguments: picked up by the `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_list_pairlist() {
    let scope = unsafe { ProtectScope::new() };
    let values = vec![("a", 1i32), ("b", 2), ("c", 3)];
    let n = isize::try_from(values.len()).expect("a small length");
    let list = unsafe { scope.protect(List::from_pairs(values).as_sexp()) };
    let pairlist = unsafe { scope.protect(list.get().coerce(miniextendr_api::SEXPTYPE::LISTSXP)) };

    assert_pairlist_refused(
        List::try_from_sexp(pairlist.get()).expect_err("List refuses a pairlist"),
        "List",
    );
    match <Option<List>>::try_from_sexp(pairlist.get()) {
        Err(SexpError::Type(e)) => assert_pairlist_refused(e, "Option<List>"),
        other => panic!("Option<List> refuses a pairlist, got {other:?}"),
    }
    match NamedList::try_from_sexp(pairlist.get()) {
        Err(SexpError::Type(e)) => assert_pairlist_refused(e, "NamedList"),
        Err(other) => panic!("NamedList refuses a pairlist with a type error, got {other:?}"),
        Ok(_) => panic!("NamedList refuses a pairlist"),
    }

    let plain = List::try_from_sexp(list.get()).expect("a list converts");
    let optional = <Option<List>>::try_from_sexp(list.get())
        .expect("a list converts")
        .expect("a list is not NULL");
    let named = NamedList::try_from_sexp(list.get()).expect("a named list converts");

    churn_lists(&scope, n);

    for (i, expected) in (1..=3).enumerate() {
        let i = isize::try_from(i).expect("a small index");
        assert_eq!(
            plain.get_index::<i32>(i),
            Some(expected),
            "List element {i}"
        );
        assert_eq!(
            optional.get_index::<i32>(i),
            Some(expected),
            "Option<List> element {i}"
        );
        assert_eq!(
            named.get_index::<i32>(i),
            Some(expected),
            "NamedList element {i}"
        );
    }
    assert_eq!(named.get::<i32>("b"), Some(2));
}
