//! Missing/NULL wrappers preserve rooted borrows instead of fabricating lifetimes.
mod r_test_utils;

use miniextendr_api::{IntoR, Missing, ProtectScope, RCow, SEXP, StrVec, TryFromRoot};
use std::borrow::Cow;

#[test]
fn absence_null_and_conversion_errors_remain_distinct() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let absent = scope.protect(SEXP::missing_arg());
        let null = scope.protect(SEXP::nil());
        let wrong = scope.protect(1_i32.into_sexp());
        let text = scope.protect(vec![Some("value"), None].into_sexp());
        assert!(matches!(
            Missing::<StrVec<'_>>::try_from_root(absent),
            Ok(Missing::Absent)
        ));
        assert!(Missing::<StrVec<'_>>::try_from_root(null).is_err());
        assert!(matches!(
            <Result<StrVec<'_>, ()>>::try_from_root(null),
            Ok(Err(()))
        ));
        for root in [wrong, absent] {
            assert!(<Result<StrVec<'_>, ()>>::try_from_root(root).is_err());
        }
        let Missing::Present(strings) = Missing::<StrVec<'_>>::try_from_root(text).unwrap() else {
            panic!("the argument was supplied");
        };
        assert_eq!(strings.get_str(0), Some("value"));
        assert_eq!(strings.get_str(1), None);
        let strings = <Result<StrVec<'_>, ()>>::try_from_root(text)
            .unwrap()
            .unwrap();
        assert_eq!(strings.get_str(0), Some("value"));
    });
}

#[test]
fn nested_wrappers_keep_storage_identity_and_alias_metadata() {
    type Wrapped<'a> = Missing<Result<RCow<'a, i32>, ()>>;
    assert_eq!(
        <Wrapped<'_> as TryFromRoot<'_>>::NATIVE_BORROW,
        <&[i32] as TryFromRoot<'_>>::NATIVE_BORROW,
    );
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let values = scope.protect(vec![1_i32, 2, 3].into_sexp());
        let direct = RCow::<i32>::try_from_root(values).unwrap();
        let Missing::Present(Ok(wrapped)) = Wrapped::try_from_root(values).unwrap() else {
            panic!("present non-NULL input");
        };
        assert_eq!(wrapped.as_ptr(), direct.as_ptr());
        assert_eq!(wrapped.as_ref(), &[1, 2, 3]);
        let cow = <Result<Cow<'_, [i32]>, ()>>::try_from_root(values)
            .unwrap()
            .unwrap();
        assert!(matches!(cow, Cow::Borrowed(_)));
        assert_eq!(cow.as_ptr(), direct.as_ptr());
        assert!(matches!(
            Wrapped::try_from_root(scope.protect(SEXP::missing_arg())),
            Ok(Missing::Absent)
        ));
        assert!(matches!(
            Wrapped::try_from_root(scope.protect(SEXP::nil())),
            Ok(Missing::Present(Err(())))
        ));
    });
}
