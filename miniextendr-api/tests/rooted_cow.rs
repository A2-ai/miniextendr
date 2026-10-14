//! Cow borrows need roots; raw conversions and trait results must own data.
mod r_test_utils;

use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope};
use miniextendr_api::{ExternalPtr, IntoR, SEXP, SexpExt, TryFromRoot, TryFromSexp, miniextendr};
use std::borrow::Cow;

#[test]
fn native_cow_borrows_the_root_and_detaches_before_mutation() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(vec![1i32, 2, 3].into_sexp());
        let mut cow: Cow<'_, [i32]> = Cow::try_from_root(root).unwrap();
        assert!(matches!(cow, Cow::Borrowed(_)));
        assert_eq!(cow.as_ptr(), root.get().as_slice::<i32>().as_ptr());
        cow.to_mut()[0] = 9;
        assert!(matches!(cow, Cow::Owned(_)));
        assert_eq!(root.get().as_slice::<i32>(), &[1, 2, 3]);
        assert_eq!(cow.as_ref(), &[9, 2, 3]);
    });
}

#[test]
fn raw_cow_can_outlive_its_source_and_has_no_native_borrow() {
    r_test_utils::with_r_thread(|| unsafe {
        let cow: Cow<'static, [i32]> = {
            let source = OwnedProtect::new(vec![4i32, 5, 6].into_sexp());
            Cow::try_from_sexp(source.get()).unwrap()
        };
        assert!(matches!(cow, Cow::Owned(_)));
        miniextendr_api::sys::R_gc();
        assert_eq!(cow.as_ref(), &[4, 5, 6]);
        assert_eq!(<Cow<'_, [i32]> as TryFromSexp>::NATIVE_BORROW, None);
        assert_eq!(
            <Cow<'_, [i32]> as TryFromRoot<'_>>::NATIVE_BORROW,
            <&[i32] as TryFromSexp>::NATIVE_BORROW,
        );
    });
}

#[test]
fn rooted_and_owned_character_cows_keep_the_same_na_semantics() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let source = scope.protect(vec![Some("hello"), None, Some("")].into_sexp());
        let borrowed: Vec<Cow<'_, str>> = Vec::try_from_root(source).unwrap();
        assert!(borrowed.iter().all(|v| matches!(v, Cow::Borrowed(_))));
        assert_eq!(borrowed, ["hello", "", ""]);
        let optional: Vec<Option<Cow<'_, str>>> = Vec::try_from_root(source).unwrap();
        assert!(optional[1].is_none());
        assert!(matches!(optional[0], Some(Cow::Borrowed("hello"))));
        let owned: Vec<Option<Cow<'static, str>>> =
            TryFromSexp::try_from_sexp(source.get()).unwrap();
        assert_eq!(owned, optional);
        assert!(owned.iter().flatten().all(|v| matches!(v, Cow::Owned(_))));

        let scalar = scope.protect("scalar".into_sexp());
        let borrowed: Cow<'_, str> = Cow::try_from_root(scalar).unwrap();
        let owned: Cow<'static, str> = Cow::try_from_sexp(scalar.get()).unwrap();
        assert!(matches!(borrowed, Cow::Borrowed("scalar")));
        assert!(matches!(owned, Cow::Owned(_)));
        assert_eq!(owned, borrowed);

        let wrong_type = scope.protect(1i32.into_sexp());
        assert!(<Cow<'_, str>>::try_from_root(wrong_type).is_err());
        assert!(<Vec<Cow<'_, str>>>::try_from_root(wrong_type).is_err());
        assert!(<Vec<Option<Cow<'_, str>>>>::try_from_root(wrong_type).is_err());
    });
}

#[miniextendr]
pub trait CowResult {
    fn label(&self) -> Cow<'static, str>;
}

#[derive(ExternalPtr)]
pub struct Label;

#[miniextendr(env, noexport)]
impl CowResult for Label {
    fn label(&self) -> Cow<'static, str> {
        Cow::Owned("trait result".to_owned())
    }
}

#[test]
fn trait_view_returns_owned_cow_after_releasing_the_result_root() {
    r_test_utils::with_r_thread(|| unsafe {
        let value = {
            let owner: SEXP = miniextendr_api::mx_abi::mx_wrap(__mx_wrap_Label(Label));
            let view = CowResultView::from_sexp(owner);
            view.label()
        };
        assert!(matches!(value, Cow::Owned(_)));
        miniextendr_api::sys::R_gc();
        assert_eq!(value, "trait result");
    });
}
