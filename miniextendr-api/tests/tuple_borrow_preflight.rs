//! Tuple alias checks run before creating any Rust references (#2014).
mod r_test_utils;

use miniextendr_api::condition::RCondition;
use miniextendr_api::from_r::borrow::assert_no_aliases;
use miniextendr_api::from_r::{NativeBorrow, TryFromSexp};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::ListBuilder;
use miniextendr_api::unwind_protect::with_r_unwind_protect;
use miniextendr_api::{IntoR, SEXP};

type Pair = (&'static mut i32, &'static i32);

fn list(scope: &ProtectScope, elements: &[SEXP]) -> SEXP {
    let builder = unsafe { ListBuilder::new(scope, elements.len()) };
    for (index, &element) in elements.iter().enumerate() {
        unsafe { builder.set(isize::try_from(index).unwrap(), element) };
    }
    builder.into_sexp()
}

/// The message `assert_no_aliases` raises, or `None` when it accepts.
fn alias_error<const N: usize>(
    args: [(SEXP, &str); N],
    borrows: [Option<NativeBorrow>; N],
) -> Option<String> {
    let result = with_r_unwind_protect(
        || {
            unsafe { assert_no_aliases(args, borrows) };
            SEXP::nil()
        },
        None,
    );
    unsafe {
        let scope = ProtectScope::new();
        let result = scope.protect(result);
        RCondition::from_tagged_sexp(result.get()).map(|condition| condition.message().to_owned())
    }
}

fn rejects<const N: usize>(args: [(SEXP, &str); N], borrows: [Option<NativeBorrow>; N]) -> bool {
    alias_error(args, borrows).is_some()
}

#[track_caller]
fn assert_duplicate(message: Option<String>, param: &str) {
    let message = message.expect("expected an alias error");
    let expected = format!("aliasing native arguments: parameter `{param}` contains duplicate");
    assert!(message.contains(&expected), "{message}");
}

#[track_caller]
fn assert_pair(message: Option<String>, a: &str, b: &str) {
    let message = message.expect("expected an alias error");
    let expected = format!("parameters `{a}` and `{b}` borrow the same non-empty R object");
    assert!(message.contains(&expected), "{message}");
}

#[test]
fn tuple_fields_aliasing_each_other_are_rejected() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let other = scope.protect(8i32.into_sexp()).get();
        let pair = list(&scope, &[v, v]);
        assert_duplicate(alias_error([(pair, "pair")], [Pair::NATIVE_BORROW]), "pair");
        // The shared field may come first.
        assert_duplicate(
            alias_error(
                [(pair, "pair")],
                [<(&'static i32, &'static mut i32)>::NATIVE_BORROW],
            ),
            "pair",
        );
        let distinct = list(&scope, &[v, other]);
        assert!(!rejects([(distinct, "pair")], [Pair::NATIVE_BORROW]));
    });
}

#[test]
fn nested_tuple_and_list_fields_are_walked() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let owned = scope.protect(1i32.into_sexp()).get();
        let pair = list(&scope, &[v, v]);
        let nested = list(&scope, &[pair]);
        assert_duplicate(
            alias_error([(nested, "nested")], [<(Pair,)>::NATIVE_BORROW]),
            "nested",
        );
        // Aliases across sibling tuples: `((i32, &mut i32), (&i32,))`.
        let first = list(&scope, &[owned, v]);
        let second = list(&scope, &[v]);
        let siblings = list(&scope, &[first, second]);
        type Siblings = ((i32, &'static mut i32), (&'static i32,));
        assert_duplicate(
            alias_error([(siblings, "nested")], [Siblings::NATIVE_BORROW]),
            "nested",
        );
        // Duplicate leaves of a list field inside a tuple.
        let leaves = list(&scope, &[v, v]);
        let with_list = list(&scope, &[leaves]);
        assert_duplicate(
            alias_error(
                [(with_list, "pair")],
                [<(Vec<&'static mut i32>,)>::NATIVE_BORROW],
            ),
            "pair",
        );
        // Tuple rows repeat the tuple per element; NULL rows borrow nothing.
        let other = scope.protect(8i32.into_sexp()).get();
        let distinct = list(&scope, &[v, other]);
        let rows = list(&scope, &[distinct, distinct]);
        assert_duplicate(
            alias_error([(rows, "rows")], [<Vec<Pair>>::NATIVE_BORROW]),
            "rows",
        );
        let optional_rows = list(&scope, &[SEXP::nil(), distinct, distinct]);
        assert_duplicate(
            alias_error(
                [(optional_rows, "rows")],
                [<Vec<Option<Pair>>>::NATIVE_BORROW],
            ),
            "rows",
        );
        let one_row = list(&scope, &[SEXP::nil(), distinct]);
        assert!(!rejects(
            [(one_row, "rows")],
            [<Vec<Option<Pair>>>::NATIVE_BORROW]
        ));
        // So does a map whose values are tuples.
        let first = list(&scope, &[v, owned]);
        let second = list(&scope, &[v, owned]);
        let map = list(&scope, &[first, second]);
        type Map = std::collections::HashMap<String, (&'static mut i32, i32)>;
        assert_duplicate(alias_error([(map, "map")], [Map::NATIVE_BORROW]), "map");
    });
}

#[test]
fn tuple_fields_aliasing_other_parameters_are_rejected() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(vec![1i32, 2].into_sexp()).get();
        let tag = scope.protect("tag".into_sexp()).get();
        let tagged = list(&scope, &[v, tag]);
        assert_pair(
            alias_error(
                [(tagged, "pair"), (v, "other")],
                [
                    <(&mut [i32], String)>::NATIVE_BORROW,
                    <&[i32]>::NATIVE_BORROW,
                ],
            ),
            "pair",
            "other",
        );
        // Shared tuple field, mutable direct parameter.
        assert_pair(
            alias_error(
                [(v, "first"), (tagged, "pair")],
                [
                    <&mut [i32]>::NATIVE_BORROW,
                    <(&[i32], String)>::NATIVE_BORROW,
                ],
            ),
            "first",
            "pair",
        );
        let scalar = scope.protect(3i32.into_sexp()).get();
        let left = list(&scope, &[scalar, tag]);
        let right = list(&scope, &[scalar, tag]);
        assert_pair(
            alias_error(
                [(left, "left"), (right, "right")],
                [
                    <(&mut i32, String)>::NATIVE_BORROW,
                    <(&i32, String)>::NATIVE_BORROW,
                ],
            ),
            "left",
            "right",
        );
    });
}

#[test]
fn tuple_preflight_preserves_owned_shared_empty_and_invalid_shape_cases() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let pair = list(&scope, &[v, v]);
        assert!(!rejects(
            [(pair, "pair")],
            [<(i32, &mut i32)>::NATIVE_BORROW]
        ));
        assert!(!rejects([(pair, "pair")], [<(&i32, &i32)>::NATIVE_BORROW]));
        // One field will fail type conversion; it does not borrow integer storage.
        assert!(!rejects(
            [(pair, "pair")],
            [<(&mut i32, &f64)>::NATIVE_BORROW]
        ));
        // Exact tuple length is checked before any field conversion.
        assert!(!rejects(
            [(pair, "triple")],
            [<(&mut i32, &i32, i32)>::NATIVE_BORROW]
        ));
        assert!(!rejects(
            [(v, "not_a_list")],
            [<(&mut i32, &i32)>::NATIVE_BORROW]
        ));
        let empty = scope.protect(Vec::<i32>::new().into_sexp()).get();
        let empties = list(&scope, &[empty, empty]);
        assert!(!rejects(
            [(empties, "pair")],
            [<(&mut [i32], &[i32])>::NATIVE_BORROW]
        ));
        // A mutable integer elsewhere in the product must not make two shared
        // double fields conflict with each other.
        let real = scope.protect(1.0f64.into_sexp()).get();
        let mixed = list(&scope, &[v, real, real]);
        assert!(!rejects(
            [(mixed, "mixed")],
            [<(&mut i32, &f64, &f64)>::NATIVE_BORROW]
        ));
    });
}
