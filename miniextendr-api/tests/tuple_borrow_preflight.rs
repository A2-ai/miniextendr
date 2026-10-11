//! Tuple alias checks run before creating any Rust references (#2014).
mod r_test_utils;

use miniextendr_api::condition::RCondition;
use miniextendr_api::from_r::borrow::assert_no_aliases;
use miniextendr_api::from_r::{NativeBorrow, TryFromSexp};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::ListBuilder;
use miniextendr_api::unwind_protect::with_r_unwind_protect;
use miniextendr_api::{IntoR, SEXP};

fn list(scope: &ProtectScope, elements: &[SEXP]) -> SEXP {
    let builder = unsafe { ListBuilder::new(scope, elements.len()) };
    for (index, &element) in elements.iter().enumerate() {
        unsafe { builder.set(isize::try_from(index).unwrap(), element) };
    }
    builder.into_sexp()
}

fn rejects<const N: usize>(args: [(SEXP, &str); N], borrows: [Option<NativeBorrow>; N]) -> bool {
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
        RCondition::from_tagged_sexp(result.get()).is_some()
    }
}

#[test]
fn tuple_preflight_finds_internal_nested_and_cross_argument_aliases() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let other = scope.protect(8i32.into_sexp()).get();
        let pair = list(&scope, &[v, v]);
        let distinct = list(&scope, &[v, other]);
        type Pair = (&'static mut i32, &'static i32);
        assert!(rejects([(pair, "pair")], [Pair::NATIVE_BORROW]));
        assert!(!rejects([(distinct, "pair")], [Pair::NATIVE_BORROW]));

        let nested = list(&scope, &[pair]);
        assert!(rejects([(nested, "nested")], [<(Pair,)>::NATIVE_BORROW]));
        let tag = scope.protect("tag".into_sexp()).get();
        let tagged = list(&scope, &[v, tag]);
        assert!(rejects(
            [(tagged, "pair"), (v, "other")],
            [
                <(&mut [i32], String)>::NATIVE_BORROW,
                <&[i32]>::NATIVE_BORROW
            ],
        ));
        assert!(rejects(
            [(tagged, "left"), (tagged, "right")],
            [
                <(&mut i32, String)>::NATIVE_BORROW,
                <(&i32, String)>::NATIVE_BORROW
            ],
        ));
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
