//! Callback validation, explicit ownership, and plain validator messages.
mod r_test_utils;

use miniextendr_api::{
    IntoR, ProtectScope, RFunction, SEXP, SexpError, TryFromRoot, TryFromSexp,
    expression::r_eval_str,
};

#[test]
fn accepts_all_r_function_kinds_and_only_null_is_optional() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        for source in ["function(x) x", "sum", "quote"] {
            let root = scope.protect(r_eval_str(source, miniextendr_api::sys::R_BaseEnv).unwrap());
            let function = RFunction::try_from_root(root).unwrap();
            assert_eq!(function.as_sexp(), root.get());
            assert_eq!(function.into_sexp(), root.get());
            assert!(
                Option::<RFunction<'_>>::try_from_root(root)
                    .unwrap()
                    .is_some()
            );
        }
        for source in ["1L", "'sum'", "quote(sum)", "list()", "NA", "NULL"] {
            let root = scope.protect(r_eval_str(source, miniextendr_api::sys::R_BaseEnv).unwrap());
            assert!(RFunction::try_from_root(root).is_err(), "{source}");
            if source == "NULL" {
                assert!(
                    Option::<RFunction<'_>>::try_from_root(root)
                        .unwrap()
                        .is_none()
                );
            } else {
                assert!(
                    Option::<RFunction<'_>>::try_from_root(root).is_err(),
                    "{source}"
                );
            }
        }
    });
}

#[test]
fn call_builder_retains_the_function_after_its_borrow_ends() {
    r_test_utils::with_r_thread(|| unsafe {
        let call = {
            let scope = ProtectScope::new();
            let root = scope.protect(
                r_eval_str(
                    "local({ offset <- 10L; function(x) x + offset })",
                    miniextendr_api::sys::R_BaseEnv,
                )
                .unwrap(),
            );
            RFunction::try_from_root(root)
                .unwrap()
                .call()
                .arg_value(2_i32)
        };
        miniextendr_api::sys::R_gc();
        assert_eq!(call.eval_base_as::<i32>().unwrap(), 12);
    });
}

fn positive_str(value: SEXP) -> Result<(), &'static str> {
    if i32::try_from_sexp(value).is_ok_and(|x| x > 0) {
        Ok(())
    } else {
        Err("must be positive")
    }
}

fn positive_string(value: SEXP) -> Result<(), String> {
    positive_str(value).map_err(str::to_owned)
}

#[derive(TryFromSexp)]
#[try_from_sexp(validate = positive_str)]
struct PositiveStr(i32);

#[derive(TryFromSexp)]
#[try_from_sexp(validate = positive_string)]
struct PositiveString(i32);

#[test]
fn plain_messages_work_in_derived_validators_and_own_their_text() {
    let error = {
        let message = String::from("borrowed validator text");
        SexpError::from(message.as_str())
    };
    assert!(matches!(error, SexpError::Condition(_)));
    assert_eq!(error.to_string(), "borrowed validator text");
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let good = scope.protect(2_i32.into_sexp());
        assert_eq!(PositiveStr::try_from_sexp(good.get()).unwrap().0, 2);
        assert_eq!(PositiveString::try_from_sexp(good.get()).unwrap().0, 2);
        let bad = scope.protect((-1_i32).into_sexp());
        let left = PositiveStr::try_from_sexp(bad.get()).err().unwrap();
        let right = PositiveString::try_from_sexp(bad.get()).err().unwrap();
        assert!(matches!(left, SexpError::Condition(_)));
        assert!(matches!(right, SexpError::Condition(_)));
        assert_eq!(left.to_string(), "must be positive");
        assert_eq!(right.to_string(), "must be positive");
    });
}
