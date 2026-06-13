// Round-trip test for `from_tagged_sexp` slot [4] (structured condition data).
//
// These tests exercise the full Rust→SEXP→Rust path for structured condition
// data through `make_rust_condition_value_with_data` + `from_tagged_sexp`.
// They require an R runtime and are guarded by `r_test_utils::with_r_thread`.

mod r_test_utils;

#[test]
fn condition_data_roundtrip_scalars_and_vecs() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::{ConditionData, ConditionDataValue, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        // Build a ConditionData with each supported v1 scalar type.
        let fields: ConditionData = vec![
            ("field_a".to_string(), ConditionDataValue::Int(42)),
            (
                "field_b".to_string(),
                ConditionDataValue::Str("hello".to_string()),
            ),
            ("field_c".to_string(), ConditionDataValue::Real(1.23)),
            ("field_d".to_string(), ConditionDataValue::Bool(true)),
            (
                "field_e".to_string(),
                ConditionDataValue::IntVec(vec![1, 2, 3]),
            ),
            (
                "field_f".to_string(),
                ConditionDataValue::RealVec(vec![0.5, 1.5]),
            ),
            (
                "field_g".to_string(),
                ConditionDataValue::StrVec(vec!["x".to_string(), "y".to_string()]),
            ),
        ];

        let sexp = make_rust_condition_value_with_data(
            "test message",
            miniextendr_api::error_value::kind::ERROR,
            Some("my_test_class"),
            None,
            Some(fields),
        );

        let cond = RCondition::from_tagged_sexp(sexp)
            .expect("from_tagged_sexp should return Some for a valid tagged SEXP");

        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "test message");
                assert_eq!(class.as_deref(), Some("my_test_class"));
                let data = data.expect("data must be Some after round-trip");
                assert_eq!(data.len(), 7);
                assert_eq!(data[0].0, "field_a");
                assert!(matches!(data[0].1, ConditionDataValue::Int(42)));
                assert_eq!(data[1].0, "field_b");
                assert!(matches!(&data[1].1, ConditionDataValue::Str(s) if s == "hello"));
                assert_eq!(data[2].0, "field_c");
                // f64 comparison via debug repr (ConditionDataValue has no PartialEq for f64)
                assert!(
                    matches!(&data[2].1, ConditionDataValue::Real(v) if (*v - 1.23_f64).abs() < 1e-10),
                    "expected Real(1.23), got {:?}",
                    data[2].1
                );
                assert_eq!(data[3].0, "field_d");
                assert!(matches!(data[3].1, ConditionDataValue::Bool(true)));
                assert_eq!(data[4].0, "field_e");
                assert!(matches!(&data[4].1, ConditionDataValue::IntVec(v) if v == &[1, 2, 3]));
                assert_eq!(data[5].0, "field_f");
                assert!(
                    matches!(&data[5].1, ConditionDataValue::RealVec(v) if v.len() == 2),
                    "expected RealVec of len 2, got {:?}",
                    data[5].1
                );
                assert_eq!(data[6].0, "field_g");
                assert!(
                    matches!(&data[6].1, ConditionDataValue::StrVec(v) if v[0] == "x" && v[1] == "y")
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
            None,
            None,
            None, // no data
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "plain error");
                assert!(class.is_none());
                assert!(data.is_none(), "data should be None when not provided");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

#[test]
fn condition_data_roundtrip_na_field_is_dropped() {
    // v1 behavior: fields whose sole element is NA are dropped rather than
    // emitted as bogus Rust values. Full NA fidelity is deferred to the
    // Option-variant PR. This test pins the documented drop behavior.
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::{ConditionData, ConditionDataValue, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        // Build a ConditionData with one NA-bearing field (NA_integer_ = i32::MIN)
        // alongside a valid field. The NA field must be dropped on round-trip.
        let fields: ConditionData = vec![
            ("good_field".to_string(), ConditionDataValue::Int(7)),
            // NA_integer_ is i32::MIN — materialised as integer(1) containing NA.
            // from_tagged_sexp must drop this field, not emit Int(i32::MIN).
            ("na_field".to_string(), ConditionDataValue::Int(i32::MIN)),
        ];

        let sexp = make_rust_condition_value_with_data(
            "error with na field",
            miniextendr_api::error_value::kind::ERROR,
            None,
            None,
            Some(fields),
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Error { data, .. } => {
                let data = data.expect("data must be Some (good_field survives)");
                assert_eq!(
                    data.len(),
                    1,
                    "only good_field should survive; na_field must be dropped (v1 NA-drop behavior). \
                     Full NA fidelity is deferred to the Option-variant PR. got: {:?}",
                    data
                );
                assert_eq!(data[0].0, "good_field");
                assert!(matches!(data[0].1, ConditionDataValue::Int(7)));
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}

#[test]
fn condition_data_roundtrip_warning_carries_data() {
    r_test_utils::with_r_thread(|| unsafe {
        use miniextendr_api::condition::{ConditionData, ConditionDataValue, RCondition};
        use miniextendr_api::error_value::make_rust_condition_value_with_data;

        let fields: ConditionData = vec![
            ("dropped".to_string(), ConditionDataValue::Int(3)),
            (
                "reason".to_string(),
                ConditionDataValue::Str("truncated".to_string()),
            ),
        ];

        let sexp = make_rust_condition_value_with_data(
            "truncated 3 rows",
            miniextendr_api::error_value::kind::WARNING,
            Some("truncation_warning"),
            None,
            Some(fields),
        );

        let cond = RCondition::from_tagged_sexp(sexp).expect("must parse");
        match cond {
            RCondition::Warning {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "truncated 3 rows");
                assert_eq!(class.as_deref(), Some("truncation_warning"));
                let data = data.expect("data must survive for Warning too");
                assert_eq!(data.len(), 2);
                assert!(matches!(data[0].1, ConditionDataValue::Int(3)));
                assert!(matches!(&data[1].1, ConditionDataValue::Str(s) if s == "truncated"));
            }
            other => panic!("wrong variant: {other:?}"),
        }
    });
}
