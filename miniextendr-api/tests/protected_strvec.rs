//! Owned character-vector roots survive source scopes and arbitrary drop order.
mod r_test_utils;

use miniextendr_api::{IntoR, OwnedProtect, ProtectedStrVec, TryFromSexp};

#[test]
fn converted_string_owners_outlive_source_roots_and_drop_in_any_order() {
    r_test_utils::with_r_thread(|| unsafe {
        let first = {
            let source = OwnedProtect::new(vec!["first", "héllo"].into_sexp());
            ProtectedStrVec::try_from_sexp(source.get()).unwrap()
        };
        let second = {
            let source = OwnedProtect::new(vec![Some("second"), None].into_sexp());
            ProtectedStrVec::try_from_sexp(source.get()).unwrap()
        };
        // An earlier owner dropping must not pop the later owner's root.
        drop(first);
        miniextendr_api::sys::R_gc();
        assert_eq!(second.iter().collect::<Vec<_>>(), [Some("second"), None]);
        assert_eq!(second.as_strvec().get_str(0), Some("second"));
        let result = OwnedProtect::new(second.into_sexp());
        miniextendr_api::sys::R_gc();
        assert_eq!(
            Vec::<Option<String>>::try_from_sexp(result.get()).unwrap(),
            [Some("second".into()), None]
        );
    });
}

#[test]
fn borrowed_views_use_scope_roots_and_keep_rcow_round_trip_identity() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = miniextendr_api::ProtectScope::new();
        let chars = scope.protect(vec![Some("rooted"), None].into_sexp());
        let strings = miniextendr_api::StrVec::try_from_root(chars).unwrap();
        let nums = scope.protect(vec![1_i32, 2, 3].into_sexp());
        let values = miniextendr_api::RCow::<i32>::try_from_root(nums).unwrap();
        miniextendr_api::sys::R_gc();
        assert_eq!(strings.iter().collect::<Vec<_>>(), [Some("rooted"), None]);
        assert_eq!(&*values, &[1, 2, 3]);
        assert_eq!(values.into_sexp(), nums.get());
        assert!(miniextendr_api::StrVec::try_from_root(nums).is_err());
        assert!(miniextendr_api::RCow::<i32>::try_from_root(chars).is_err());
    });
}
