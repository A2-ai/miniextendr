//! Error classification must be independent of methods in the consumer module.

use miniextendr_api::condition::{ConditionCall, ConditionData, ErrParts, RConditionData};
use miniextendr_api::from_r::{
    SexpClassError, SexpError, SexpLengthError, SexpNaError, SexpTypeError,
};
use miniextendr_api::{MatchArgError, SEXPTYPE};
use std::cell::Cell;

#[allow(dead_code)]
trait ConsumerMethods: Sized {
    fn __mx_classed_parts(&self) -> Option<ErrParts> {
        panic!("consumer classed")
    }
    fn __mx_fallback_parts(&self, _: &str) -> ErrParts {
        panic!("consumer fallback")
    }
    fn __mx_conversion_parts(&self, _: bool) -> ErrParts {
        panic!("consumer conversion")
    }
    fn __mx_conversion_expectation(&self) -> Option<String> {
        panic!("consumer expectation")
    }
    fn __mx_conversion_arg_message(&self, _: &str) -> Option<String> {
        panic!("consumer argument")
    }
    // Also shadow methods on the selected framework strategy.
    fn classed<T>(self, _: T) {}
    fn fallback<T>(self, _: T, _: &str) {}
    fn conversion<T>(self, _: T, _: bool) {}
    fn expectation<T>(self, _: T) {}
    fn argument_message<T>(self, _: T, _: &str) {}
    fn index<T>(self, _: T) {}
}
impl<T> ConsumerMethods for T {}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
struct Classed<'a> {
    reason: &'a str,
}

impl std::fmt::Display for Classed<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("display fallback")
    }
}
impl RConditionData for Classed<'_> {
    fn message(&self) -> String {
        self.reason.into()
    }
    fn class(&self) -> Vec<String> {
        vec!["custom_class".into()]
    }
    fn data(&self) -> Option<ConditionData> {
        Some(vec![("code".into(), 7.into())])
    }
    fn call(&self) -> ConditionCall {
        ConditionCall::None
    }
    fn argument_message(&self) -> Option<String> {
        Some("whole argument message".into())
    }
}

// An inherent same-named method used to silently override classification.
#[allow(dead_code)]
impl Classed<'_> {
    fn __mx_classed_parts(&self) -> Option<ErrParts> {
        panic!("inherent classed")
    }
    fn __mx_conversion_parts(&self, _: bool) -> ErrParts {
        panic!("inherent conversion")
    }
    fn __mx_conversion_arg_message(&self, _: &str) -> Option<String> {
        panic!("inherent argument")
    }
}

#[test]
fn classed_errors_keep_all_parts_and_borrowed_data() {
    let reason = String::from("classed reason");
    let err = Classed { reason: &reason };
    for parts in [
        miniextendr_api::__mx_result_err_parts!(err, "pkg"),
        miniextendr_api::__mx_conversion_err_parts!(err, true),
    ] {
        assert_eq!(parts.message, reason);
        assert_eq!(parts.class, ["custom_class"]);
        assert_eq!(parts.data.unwrap()[0].0, "code");
        assert_eq!(parts.call, ConditionCall::None);
    }
    assert_eq!(miniextendr_api::__mx_conversion_expectation!(err), None);
    assert_eq!(
        miniextendr_api::__mx_conversion_arg_message!(err, "x"),
        Some("whole argument message".into())
    );
    assert_eq!(err.reason, reason); // Probing must not move the error.
}

#[test]
fn builtin_reasons_keep_r_facing_wording() {
    let type_error = SexpTypeError {
        expected: SEXPTYPE::INTSXP,
        actual: SEXPTYPE::STRSXP,
    };
    let class_error = SexpClassError {
        expected: "a data frame",
        actual: SEXPTYPE::VECSXP,
    };
    let length_error = SexpLengthError {
        expected: 1,
        actual: 2,
    };
    let na_error = SexpNaError {
        sexp_type: SEXPTYPE::INTSXP,
    };
    assert_eq!(
        miniextendr_api::__mx_conversion_err_parts!(type_error, true).message,
        "got character"
    );
    assert_eq!(
        miniextendr_api::__mx_conversion_err_parts!(length_error, true).message,
        "got length 2"
    );
    assert_eq!(
        miniextendr_api::__mx_conversion_err_parts!(na_error).message,
        "NA is not allowed"
    );
    let wrapped = SexpError::Class(class_error);
    assert_eq!(
        miniextendr_api::__mx_conversion_expectation!(wrapped),
        Some("a data frame".into())
    );
    assert_eq!(
        miniextendr_api::__mx_conversion_expectation!(class_error),
        None
    );
    assert_eq!(
        miniextendr_api::__mx_conversion_arg_message!(wrapped, "x"),
        None
    );
    assert_eq!(
        miniextendr_api::__mx_conversion_arg_message!(na_error, "x"),
        None
    );
}

#[test]
fn choice_errors_keep_expectation_and_whole_message() {
    let err = MatchArgError::NoMatch {
        aliases: &[],
        input: "bad".into(),
        choices: &["fast", "slow"],
    };
    let expected = Some("one of \"fast\", \"slow\"".into());
    let message = Some("'mode' should be one of \"fast\", \"slow\"".into());
    assert_eq!(miniextendr_api::__mx_conversion_expectation!(err), expected);
    assert_eq!(
        miniextendr_api::__mx_conversion_arg_message!(err, "mode"),
        message
    );
    assert_eq!(
        miniextendr_api::__mx_conversion_err_parts!(err, true).message,
        "got \"bad\""
    );
    let wrapped = SexpError::MatchArg(err);
    assert_eq!(
        miniextendr_api::__mx_conversion_expectation!(wrapped),
        expected
    );
    assert_eq!(
        miniextendr_api::__mx_conversion_arg_message!(wrapped, "mode"),
        message
    );
}

#[test]
fn plain_errors_keep_debug_and_display_fallbacks() {
    #[derive(Debug)]
    struct Plain;
    impl std::fmt::Display for Plain {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("plain display")
        }
    }
    assert_eq!(
        miniextendr_api::__mx_result_err_parts!(Plain, "pkg").message,
        "Plain"
    );
    let parts = miniextendr_api::__mx_conversion_err_parts!(Plain);
    assert_eq!(parts.message, "plain display");
    assert!(parts.class.is_empty());
    assert!(parts.data.is_none());
    assert_eq!(parts.call, ConditionCall::Inherit);
    assert_eq!(miniextendr_api::__mx_conversion_expectation!(Plain), None);
    assert_eq!(
        miniextendr_api::__mx_conversion_arg_message!(Plain, "x"),
        None
    );
}

#[test]
fn serde_priority_is_feature_dependent() {
    let err = String::from("plain text");
    let parts = miniextendr_api::__mx_result_err_parts!(err, "pkg");
    if cfg!(feature = "serde") {
        assert_eq!(parts.message, err);
        assert_eq!(parts.class, ["pkg"]);
    } else {
        assert_eq!(parts.message, "\"plain text\"");
        assert!(parts.class.is_empty());
    }
    // Conversion errors never take the serde path.
    assert!(
        miniextendr_api::__mx_conversion_err_parts!(err)
            .class
            .is_empty()
    );
}

#[test]
fn expressions_are_evaluated_once_and_unused_prefix_is_lazy() {
    let count = Cell::new(0);
    let parts = miniextendr_api::__mx_result_err_parts!(
        {
            count.set(count.get() + 1);
            Classed { reason: "once" }
        },
        {
            count.set(count.get() + 100);
            "pkg"
        }
    );
    assert_eq!(count.get(), 1);
    assert_eq!(parts.message, "once");
    let parts = miniextendr_api::__mx_conversion_err_parts!(
        {
            count.set(count.get() + 1);
            "display"
        },
        {
            count.set(count.get() + 10);
            true
        }
    );
    assert_eq!(count.get(), 12);
    assert_eq!(parts.message, "display");
    let _ = miniextendr_api::__mx_conversion_arg_message!(
        {
            count.set(count.get() + 1);
            "display"
        },
        {
            count.set(count.get() + 10);
            "x"
        }
    );
    assert_eq!(count.get(), 23);
    assert_eq!(
        miniextendr_api::__mx_conversion_expectation!({
            count.set(count.get() + 1);
            "display"
        }),
        None
    );
    assert_eq!(count.get(), 24);
}

#[cfg(feature = "serde")]
#[test]
fn reporting_failures_keep_the_macro_callers_location() {
    #[derive(Debug)]
    struct CannotSerialize;
    impl std::fmt::Display for CannotSerialize {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("original failure")
        }
    }
    impl serde::Serialize for CannotSerialize {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("reporting failure"))
        }
    }

    let location = std::sync::Arc::new(std::sync::Mutex::new(None));
    let captured = location.clone();
    let thread = std::thread::current().id();
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if std::thread::current().id() == thread {
            *captured.lock().unwrap() = info.location().map(|l| (l.file().to_string(), l.line()));
        }
    }));
    let expected_line = Cell::new(0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        expected_line.set(line!() + 1);
        miniextendr_api::__mx_result_err_parts!(CannotSerialize, "pkg")
    }));
    std::panic::set_hook(previous_hook);
    let panic = result.expect_err("serialization must fail");
    assert!(
        panic
            .downcast_ref::<String>()
            .unwrap()
            .contains("original failure")
    );
    assert_eq!(
        *location.lock().unwrap(),
        Some((file!().into(), expected_line.get()))
    );
}

#[test]
fn classed_conversions_need_neither_display_nor_debug() {
    struct Opaque;
    impl RConditionData for Opaque {
        fn message(&self) -> String {
            "classed opaque".into()
        }
    }
    assert_eq!(
        miniextendr_api::__mx_conversion_err_parts!(Opaque).message,
        "classed opaque"
    );
}

#[test]
fn debug_result_errors_need_no_display() {
    #[derive(Debug)]
    struct DebugOnly;
    assert_eq!(
        miniextendr_api::__mx_result_err_parts!(DebugOnly, "pkg").message,
        "DebugOnly"
    );
}
