// Round-trip test for `from_tagged_sexp` slot [4] (structured condition data).
//
// These tests exercise the full Rust→SEXP→Rust path for structured condition
// data through `make_rust_condition_value_with_data` + `from_tagged_sexp`.
// They require an R runtime and are guarded by `r_test_utils::with_r_thread`.

mod r_test_utils;

/// The marker type of a crate that declares no `serde_error_shape!`, for the
/// `Err` probe's marker argument (resolved only under `serde`).
#[cfg(feature = "serde")]
enum NoShapes {}

#[test]
fn condition_data_roundtrip_scalars_and_vecs() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{ConditionData, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        // Build a ConditionData with each supported scalar type.
        let fields: ConditionData = vec![
            ("field_a".to_string(), RValue::Integer(vec![Some(42)])),
            (
                "field_b".to_string(),
                RValue::Character(vec![Some("hello".to_string())]),
            ),
            ("field_c".to_string(), RValue::Double(vec![1.23])),
            ("field_d".to_string(), RValue::Logical(vec![Some(true)])),
            (
                "field_e".to_string(),
                RValue::Integer(vec![Some(1), Some(2), Some(3)]),
            ),
            ("field_f".to_string(), RValue::Double(vec![0.5, 1.5])),
            (
                "field_g".to_string(),
                RValue::Character(vec![Some("x".to_string()), Some("y".to_string())]),
            ),
        ];

        let sexp = make_rust_condition_value_with_data(
            "test message",
            miniextendr_api::error_value::kind::ERROR,
            &["my_test_class".to_string()],
            None,
            Some(fields),
            miniextendr_api::condition::ConditionCall::Inherit,
        );

        let cond = RCondition::from_tagged_sexp(sexp)
            .expect("from_tagged_sexp should return Some for a valid tagged SEXP");

        match cond {
            RCondition::Error {
                message,
                class,
                data,
                call: miniextendr_api::condition::ConditionCall::Inherit,
            } => {
                assert_eq!(message, "test message");
                assert_eq!(class, vec!["my_test_class"]);
                let data = data.expect("data must be Some after round-trip");
                assert_eq!(data.len(), 7);
                assert_eq!(data[0].0, "field_a");
                assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(42)]));
                assert_eq!(data[1].0, "field_b");
                assert!(
                    matches!(&data[1].1, RValue::Character(v) if v == &[Some("hello".to_string())])
                );
                assert_eq!(data[2].0, "field_c");
                // f64 comparison via debug repr (RValue has no PartialEq for f64)
                assert!(
                    matches!(&data[2].1, RValue::Double(v) if v.len() == 1 && (v[0] - 1.23_f64).abs() < 1e-10),
                    "expected Double([1.23]), got {:?}",
                    data[2].1
                );
                assert_eq!(data[3].0, "field_d");
                assert!(matches!(&data[3].1, RValue::Logical(v) if v == &[Some(true)]));
                assert_eq!(data[4].0, "field_e");
                assert!(
                    matches!(&data[4].1, RValue::Integer(v) if v == &[Some(1), Some(2), Some(3)])
                );
                assert_eq!(data[5].0, "field_f");
                assert!(
                    matches!(&data[5].1, RValue::Double(v) if v.len() == 2),
                    "expected Double of len 2, got {:?}",
                    data[5].1
                );
                assert_eq!(data[6].0, "field_g");
                assert!(
                    matches!(&data[6].1, RValue::Character(v) if v[0].as_deref() == Some("x") && v[1].as_deref() == Some("y"))
                );
            }
            other => panic!("wrong RCondition variant: {other:?}"),
        }
    });
}

#[test]
fn condition_data_roundtrip_no_data() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::RCondition;
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        let sexp = make_rust_condition_value_with_data(
            "plain error",
            miniextendr_api::error_value::kind::ERROR,
            &[],
            None,
            None, // no data
            miniextendr_api::condition::ConditionCall::Inherit,
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Error {
                message,
                class,
                data,
                call: miniextendr_api::condition::ConditionCall::Inherit,
            } => {
                assert_eq!(message, "plain error");
                assert!(class.is_empty());
                assert!(data.is_none(), "data should be None when not provided");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

#[test]
fn condition_data_roundtrip_na_field_survives() {
    // RValue is NA-aware: a field whose element is NA round-trips faithfully as
    // `Integer([None])` rather than being dropped (the pre-#1050 behavior). This
    // pins the NA fidelity that folding in `RValue` delivers.
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{ConditionData, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        // Build a ConditionData with one NA-bearing field alongside a valid one.
        let fields: ConditionData = vec![
            ("good_field".to_string(), RValue::Integer(vec![Some(7)])),
            // NA_integer_ — materialised as integer(1) containing NA, decoded as None.
            ("na_field".to_string(), RValue::Integer(vec![None])),
        ];

        let sexp = make_rust_condition_value_with_data(
            "error with na field",
            miniextendr_api::error_value::kind::ERROR,
            &[],
            None,
            Some(fields),
            miniextendr_api::condition::ConditionCall::Inherit,
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Error { data, .. } => {
                let data = data.expect("data must be Some");
                assert_eq!(
                    data.len(),
                    2,
                    "both fields survive (NA-aware); got: {data:?}"
                );
                assert_eq!(data[0].0, "good_field");
                assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(7)]));
                assert_eq!(data[1].0, "na_field");
                assert!(matches!(&data[1].1, RValue::Integer(v) if v == &[None]));
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

#[test]
fn condition_data_roundtrip_warning_carries_data() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{ConditionData, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        let fields: ConditionData = vec![
            ("dropped".to_string(), RValue::Integer(vec![Some(3)])),
            (
                "reason".to_string(),
                RValue::Character(vec![Some("truncated".to_string())]),
            ),
        ];

        let sexp = make_rust_condition_value_with_data(
            "truncated 3 rows",
            miniextendr_api::error_value::kind::WARNING,
            &["truncation_warning".to_string()],
            None,
            Some(fields),
            miniextendr_api::condition::ConditionCall::Inherit,
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Warning {
                message,
                class,
                data,
                call: miniextendr_api::condition::ConditionCall::Inherit,
            } => {
                assert_eq!(message, "truncated 3 rows");
                assert_eq!(class, vec!["truncation_warning"]);
                let data = data.expect("data must survive for Warning too");
                assert_eq!(data.len(), 2);
                assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(3)]));
                assert!(
                    matches!(&data[1].1, RValue::Character(v) if v == &[Some("truncated".to_string())])
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

/// Class vectors (#1435): every element of the class slot survives the
/// SEXP round trip, in order (member first, family second).
#[test]
fn condition_class_vector_roundtrip() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::RCondition;
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        let classes = vec![
            "pkg_error_missing_field".to_string(),
            "pkg_error".to_string(),
        ];
        let sexp = make_rust_condition_value_with_data(
            "layered",
            miniextendr_api::error_value::kind::ERROR,
            &classes,
            None,
            None,
            miniextendr_api::condition::ConditionCall::Inherit,
        );
        match RCondition::from_tagged_sexp(sexp).expect("tagged") {
            RCondition::Error {
                message,
                class,
                data,
                call: miniextendr_api::condition::ConditionCall::Inherit,
            } => {
                assert_eq!(message, "layered");
                assert_eq!(class, classes);
                assert!(data.is_none());
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

/// `result_err_condition_value` (#1434): the probed parts of a classed error
/// land in the tagged value with `kind = "result_err"`, class and data intact.
#[test]
fn result_err_parts_roundtrip() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{RCondition, RError};
        use miniextendr_api::error_value::result_err_condition_value;

        let err = RError::new("too large")
            .class(["pkg_too_large", "pkg_error"])
            .data("rule", 3)
            .data("max", 100.0);
        let parts = miniextendr_api::__mx_result_err_parts!(err, "pkg_error", NoShapes);
        let sexp = result_err_condition_value(parts, None);
        match RCondition::from_tagged_sexp(sexp).expect("tagged") {
            RCondition::Error {
                message,
                class,
                data,
                call: miniextendr_api::condition::ConditionCall::Inherit,
            } => {
                assert_eq!(message, "too large");
                assert_eq!(class, vec!["pkg_too_large", "pkg_error"]);
                let data = data.expect("data");
                assert_eq!(data[0].0, "rule");
                assert!(matches!(&data[0].1, RValue::Integer(v) if v == &[Some(3)]));
                assert_eq!(data[1].0, "max");
            }
            other => panic!("wrong variant: {other:?}"),
        }

        // A plain String error: under the `serde` feature it takes the serde
        // arm (family class, text unquoted); otherwise the Debug fallback keeps
        // the historical rendering.
        let plain = String::from("boom");
        let parts = miniextendr_api::__mx_result_err_parts!(plain, "pkg_error", NoShapes);
        if cfg!(feature = "serde") {
            assert_eq!(parts.message, "boom");
            assert_eq!(parts.class, ["pkg_error"]);
        } else {
            assert_eq!(parts.message, "\"boom\"");
            assert!(parts.class.is_empty());
        }
        assert!(parts.data.is_none());
    });
}

/// `conversion_condition_value`: the probed parts of a classed conversion
/// error, the crate classes, `param` and `rust_type` land in the tagged
/// value; the
/// trait-ABI re-panic path (`from_tagged_sexp`) keeps the conversion kind,
/// class, data and the message without an unknown-kind prefix.
#[test]
fn conversion_parts_roundtrip() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::RValue;
        use miniextendr_api::condition::{RCondition, RError};
        use miniextendr_api::error_value::conversion_condition_value;

        let err = RError::new("must be positive")
            .class(["pkg_error_negative", "pkg_error"])
            .data("value", -1);
        let parts = miniextendr_api::__mx_conversion_err_parts!(err);
        let sexp = conversion_condition_value(
            "invalid 'n' argument",
            "n",
            Some("Count"),
            &["pkg_error_argument", "pkg_error"],
            parts,
            None,
            None,
        );
        match RCondition::from_tagged_sexp(sexp).expect("tagged") {
            RCondition::Conversion {
                message,
                class,
                data,
                call: miniextendr_api::condition::ConditionCall::Inherit,
            } => {
                assert_eq!(message, "invalid 'n' argument: must be positive");
                assert_eq!(
                    class,
                    vec!["pkg_error_negative", "pkg_error", "pkg_error_argument"]
                );
                let data = data.expect("data");
                assert_eq!(data[0].0, "param");
                assert!(
                    matches!(&data[0].1, RValue::Character(v) if v == &[Some("n".to_string())])
                );
                assert_eq!(data[1].0, "rust_type");
                assert!(
                    matches!(&data[1].1, RValue::Character(v) if v == &[Some("Count".to_string())])
                );
                assert_eq!(data[2].0, "value");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

/// The "no call" marker (#1725): `FALSE` in the call slot. It survives the
/// trait-ABI re-panic (`from_tagged_sexp`), which keeps the rest of the
/// condition as is; `NULL` reads back as `Inherit`.
#[test]
fn no_call_marker_roundtrip() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::{ConditionCall, RCondition, RError};
        use miniextendr_api::error_value::{
            make_rust_condition_value_with_data, result_err_condition_value, rust_condition_value,
        };
        use miniextendr_api::{SEXPTYPE, SexpExt};

        let sexp = make_rust_condition_value_with_data(
            "plain",
            miniextendr_api::error_value::kind::WARNING,
            &["pkg_override".to_string()],
            None,
            None,
            ConditionCall::None,
        );
        let call_slot = sexp.vector_elt(3);
        assert_eq!(call_slot.type_of(), SEXPTYPE::LGLSXP);
        assert_eq!(call_slot.len(), 1);
        assert_eq!(call_slot.logical_elt(0), 0);
        match RCondition::from_tagged_sexp(sexp).expect("tagged") {
            RCondition::Warning {
                message,
                class,
                data,
                call,
            } => {
                assert_eq!(message, "plain");
                assert_eq!(class, vec!["pkg_override"]);
                assert!(data.is_none());
                assert!(matches!(call, ConditionCall::None));
            }
            other => panic!("wrong variant: {other:?}"),
        }

        // The marker wins over a captured call; without it the slot keeps the
        // call.
        let call = miniextendr_api::OwnedProtect::new(miniextendr_api::sys::Rf_lang1(
            miniextendr_api::sys::Rf_install(c"f".as_ptr()),
        ));
        let callless = rust_condition_value(
            RCondition::Error {
                message: "e".into(),
                class: Vec::new(),
                data: None,
                call: ConditionCall::None,
            },
            Some(call.get()),
        );
        assert!(callless.vector_elt(3).is_logical());
        assert!(matches!(
            RCondition::from_tagged_sexp(callless)
                .expect("tagged")
                .call(),
            ConditionCall::None
        ));
        let with_call = rust_condition_value(
            RCondition::Error {
                message: "e".into(),
                class: Vec::new(),
                data: None,
                call: ConditionCall::Inherit,
            },
            Some(call.get()),
        );
        assert_eq!(with_call.vector_elt(3), call.get());
        let without_call = rust_condition_value(
            RCondition::Error {
                message: "e".into(),
                class: Vec::new(),
                data: None,
                call: ConditionCall::Inherit,
            },
            None,
        );
        assert!(without_call.vector_elt(3).is_nil());
        assert!(matches!(
            RCondition::from_tagged_sexp(without_call)
                .expect("tagged")
                .call(),
            ConditionCall::Inherit
        ));

        // A `Result` error built with `without_call()` carries the marker.
        let err = RError::new("data problem").without_call();
        let parts = miniextendr_api::__mx_result_err_parts!(err, "pkg_error", NoShapes);
        let sexp = result_err_condition_value(parts, Some(call.get()));
        assert!(sexp.vector_elt(3).is_logical());
        assert!(matches!(
            RCondition::from_tagged_sexp(sexp).expect("tagged").call(),
            ConditionCall::None
        ));
    });
}

/// A given call (#1852): built into the call slot in place of the captured
/// one, read back by the trait-ABI re-panic (`from_tagged_sexp`) as the same
/// call. A trait shim's guard captures no call, so a call in the slot is a
/// given one.
#[test]
fn given_call_roundtrip() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::{ConditionCall, RCondition, RError};
        use miniextendr_api::error_value::{result_err_condition_value, rust_condition_value};
        use miniextendr_api::expression::r_eval_str_global;
        use miniextendr_api::{IntoR, OwnedProtect, RCallPart, RCallValue, SexpExt, TryFromSexp};

        // 16: `identical()`'s defaults.
        let identical = |a: miniextendr_api::SEXP, b: miniextendr_api::SEXP| -> bool {
            miniextendr_api::sys::R_compute_identical(a, b, 16).into()
        };
        let quoted = |code: &str| OwnedProtect::new(r_eval_str_global(code).expect("parses"));

        // `x$table <- value`
        let assign = RCallValue::new("<-")
            .arg(
                RCallValue::new("$")
                    .arg(RCallPart::symbol("x"))
                    .arg(RCallPart::symbol("table")),
            )
            .arg(RCallPart::symbol("value"));
        let expected = quoted("quote(x$table <- value)");
        let captured = OwnedProtect::new(miniextendr_api::sys::Rf_lang1(
            miniextendr_api::sys::Rf_install(c"f".as_ptr()),
        ));
        let sexp = OwnedProtect::new(rust_condition_value(
            RCondition::Warning {
                message: "w".into(),
                class: Vec::new(),
                data: None,
                call: ConditionCall::from(assign.clone()),
            },
            Some(captured.get()),
        ));
        let slot = sexp.get().vector_elt(3);
        assert!(
            identical(slot, expected.get()),
            "the given call replaces the captured one"
        );
        match RCondition::from_tagged_sexp(sexp.get()).expect("tagged") {
            RCondition::Warning {
                call: ConditionCall::Given(call),
                ..
            } => {
                let back = OwnedProtect::new((*call).into_sexp());
                assert!(identical(back.get(), expected.get()));
            }
            other => panic!("wrong variant: {other:?}"),
        }

        // Every part: a namespaced head, a named constant, an empty argument
        // and a nested call; a value read back converts to the same call.
        let expected = quoted(r#"quote(pkg::f(x[1, ], n = 2L, "s", g(TRUE)))"#);
        let built = RCallValue::with_function(
            RCallValue::new("::")
                .arg(RCallPart::symbol("pkg"))
                .arg(RCallPart::symbol("f"))
                .into(),
        )
        .arg(
            RCallValue::new("[")
                .arg(RCallPart::symbol("x"))
                .arg(1.0)
                .arg(RCallPart::symbol("")),
        )
        .named_arg("n", 2)
        .arg("s")
        .arg(RCallValue::new("g").arg(true));
        let built = OwnedProtect::new(built.into_sexp());
        assert!(identical(built.get(), expected.get()));
        let read = RCallValue::try_from_sexp(expected.get()).expect("a call RCallValue reads");
        let again = OwnedProtect::new(read.into_sexp());
        assert!(identical(again.get(), expected.get()));

        // A name R refuses: the conversion is an error, and the condition
        // keeps the captured call instead of raising R's error.
        let too_long = RCallValue::new("x".repeat(10_001));
        let err = too_long
            .clone()
            .try_into_sexp()
            .expect_err("a name over the limit");
        assert!(miniextendr_api::condition::RConditionData::message(&err).contains("10001 bytes"));
        let sexp = result_err_condition_value(
            miniextendr_api::__mx_result_err_parts!(
                RError::new("e").with_call(too_long),
                "pkg_error",
                NoShapes
            ),
            Some(captured.get()),
        );
        assert_eq!(sexp.vector_elt(3), captured.get());

        // A call RCallValue can't read (a function object as the head, as
        // `do.call()` builds) reads back as `Inherit`.
        let closure_call = quoted("as.call(list(sum, 1))");
        let sexp = make_tagged_with_call_slot(closure_call.get());
        assert!(matches!(
            RCondition::from_tagged_sexp(sexp.get())
                .expect("tagged")
                .call(),
            ConditionCall::Inherit
        ));
        assert!(RCallValue::try_from_sexp(closure_call.get()).is_err());
    });
}

/// A tagged error value whose call slot is `call`, as a producer's guard
/// would write it.
unsafe fn make_tagged_with_call_slot(call: miniextendr_api::SEXP) -> miniextendr_api::OwnedProtect {
    use miniextendr_api::SexpExt;
    use miniextendr_api::condition::ConditionCall;
    unsafe {
        let sexp = miniextendr_api::OwnedProtect::new(
            miniextendr_api::error_value::make_rust_condition_value_with_data(
                "e",
                miniextendr_api::error_value::kind::ERROR,
                &[],
                None,
                None,
                ConditionCall::Inherit,
            ),
        );
        sexp.get().set_vector_elt(3, call);
        sexp
    }
}

/// `arg_error!` (#1740): `kind = "conversion"`, the crate-class marker in the
/// class slot, `param` as data and the call choice in the call slot; across
/// the trait ABI (`from_tagged_sexp`) all of it survives, so the consumer's
/// wrapper resolves the marker.
#[test]
fn arg_error_roundtrip() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::{
            CONVERSION_ERROR_CLASS_MARKER, ConditionCall, RCondition,
        };
        use miniextendr_api::error_value::{kind, rust_condition_value};
        use miniextendr_api::{OwnedProtect, RValue, SexpExt};

        for call_none in [false, true] {
            let call = if call_none {
                ConditionCall::None
            } else {
                ConditionCall::Inherit
            };
            let cond = RCondition::arg_error("mode", "'mode' is bad".to_string(), call);
            let sexp = OwnedProtect::new(rust_condition_value(cond, None));
            let sexp = sexp.get();
            assert_eq!(sexp.vector_elt(1).string_elt_str(0), Some(kind::CONVERSION));
            assert_eq!(
                sexp.vector_elt(2).string_elt_str(0),
                Some(CONVERSION_ERROR_CLASS_MARKER)
            );
            match RCondition::from_tagged_sexp(sexp).expect("tagged") {
                RCondition::Conversion {
                    message,
                    class,
                    data,
                    call: got,
                } => {
                    assert_eq!(message, "'mode' is bad");
                    assert_eq!(class, [CONVERSION_ERROR_CLASS_MARKER]);
                    let data = data.expect("data");
                    assert_eq!(data.len(), 1);
                    assert_eq!(data[0].0, "param");
                    assert!(
                        matches!(&data[0].1, RValue::Character(v) if v == &[Some("mode".to_string())])
                    );
                    assert_eq!(matches!(got, ConditionCall::None), call_none);
                }
                other => panic!("wrong variant: {other:?}"),
            }
        }
    });
}
