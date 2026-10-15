//! Borrowed strings must retain uncached ALTSTRING Elt results.
mod r_test_utils;

use miniextendr_api::altrep_traits::{AltString, AltVec, Altrep};
use miniextendr_api::{IntoR, ProtectScope, ProtectedStrVec, R_xlen_t, SEXP, StrVec, TryFromRoot};
use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(miniextendr_api::AltrepString)]
#[altrep(manual, no_lowlevel, class = "TransientBorrowStrings")]
struct TransientStrings;

impl Altrep for TransientStrings {
    fn length(_: SEXP) -> R_xlen_t {
        1
    }
}
impl AltVec for TransientStrings {}
impl AltString for TransientStrings {
    fn elt(_: SEXP, _: R_xlen_t) -> SEXP {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let next = NEXT.fetch_add(1, Ordering::Relaxed);
        SEXP::charsxp(&format!("borrowed-{next:08}-{}", "x".repeat(300)))
    }
}
miniextendr_api::impl_inferbase_string!(TransientStrings);

fn churn() {
    unsafe { miniextendr_api::sys::R_gc() };
    // Reuse nodes of the same size as the uncached CHARSXP.
    for i in 0..20_000 {
        SEXP::charsxp(&format!("recycled-{i:08}-{}", "z".repeat(300)));
    }
}

#[test]
fn scalar_reference_optional_and_cow_survive_gc() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(TransientStrings.into_sexp());
        let plain = <&str>::try_from_root(root).unwrap();
        let plain_expected = plain.to_owned();
        let optional = Option::<&str>::try_from_root(root).unwrap().unwrap();
        let optional_expected = optional.to_owned();
        let cow = Cow::<str>::try_from_root(root).unwrap();
        let cow_expected = cow.to_string();
        churn();
        assert_eq!(plain, plain_expected);
        assert_eq!(optional, optional_expected);
        assert_eq!(cow, cow_expected);
    });
}

#[test]
fn rooted_view_retains_each_read_including_iterators() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(TransientStrings.into_sexp());
        let view = StrVec::try_from_root(root).unwrap();
        let first = view.get_str(0).unwrap();
        let expected_first = first.to_owned();
        let second = view.iter().next().unwrap().unwrap();
        let expected_second = second.to_owned();
        let third = view.iter_cow().next().unwrap().unwrap();
        let expected_third = third.to_string();
        // Different Elt calls must remain lazy, with distinct returned values.
        assert_ne!(expected_first, expected_second);
        churn();
        assert_eq!(first, expected_first);
        assert_eq!(second, expected_second);
        assert_eq!(third, expected_third);
    });
}

#[test]
fn owned_view_retains_children_without_a_protection_scope() {
    r_test_utils::with_r_thread(|| unsafe {
        let owner = ProtectedStrVec::new(TransientStrings.into_sexp());
        let first = owner.get_str(0).unwrap();
        let expected_first = first.to_owned();
        let second = owner.as_strvec().get_cow(0).unwrap();
        let expected_second = second.to_string();
        let third = owner.iter().next().unwrap().unwrap();
        let expected_third = third.to_owned();
        churn();
        assert_eq!(first, expected_first);
        assert_eq!(second, expected_second);
        assert_eq!(third, expected_third);
    });
}

#[test]
fn scalar_rooted_strings_keep_na_null_and_validation_semantics() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let missing = scope.protect(vec![None::<&str>].into_sexp());
        assert_eq!(<&str>::try_from_root(missing).unwrap(), "");
        assert_eq!(Option::<&str>::try_from_root(missing).unwrap(), None);
        assert_eq!(Cow::<str>::try_from_root(missing).unwrap(), "");
        let null = scope.protect(SEXP::nil());
        assert_eq!(Option::<&str>::try_from_root(null).unwrap(), None);
        assert!(<&str>::try_from_root(null).is_err());
        let empty = scope.protect("".into_sexp());
        assert_eq!(Option::<&str>::try_from_root(empty).unwrap(), Some(""));
        let wrong_length = scope.protect(vec!["one", "two"].into_sexp());
        let wrong_type = scope.protect(1_i32.into_sexp());
        for root in [wrong_length, wrong_type] {
            assert!(<&str>::try_from_root(root).is_err());
            assert!(Option::<&str>::try_from_root(root).is_err());
            assert!(Cow::<str>::try_from_root(root).is_err());
        }
    });
}
