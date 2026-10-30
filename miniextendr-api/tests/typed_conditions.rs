//! Typed payload derivation feeds the existing condition transport.
mod r_test_utils;

use miniextendr_api::condition::ConditionCall;
use miniextendr_api::{RCondition, RValue};

#[derive(RCondition)]
#[condition(kind = warning, class = "pkg_warning", message = "dropped {n} rows", call = none)]
struct DroppedRows {
    n: i32,
}

#[derive(RCondition)]
#[condition(kind = warning, class = "pkg_diagnostic")]
enum Diagnostic {
    #[condition(message = "dropped {n} rows")]
    Dropped { n: i32 },
    #[condition(kind = message, message = "finished {_0} rows")]
    Finished(#[condition(rename = "n")] i32),
    #[condition(kind = error, message = "empty input", call = none)]
    Empty,
    #[condition(kind = condition, message = "progress")]
    Progress,
}

#[derive(RCondition)]
#[condition(message = "tick")]
struct Tick;

/// A struct's own class vector, a fixed `data` entry, and an optional field
/// left out when it is `None`.
#[derive(RCondition)]
#[condition(
    kind = warning,
    class("pkg_warning_argument_ignored", "pkg_warning"),
    message = "`{argument}` is ignored",
    data(param = "unit_table")
)]
struct ArgumentIgnored {
    #[condition(skip)]
    argument: &'static str,
    #[condition(skip_none)]
    hint: Option<String>,
}

/// A borrowed payload: the lifetime reaches both impls, and the data entries
/// are owned copies. A variant's class list replaces its member; the type's
/// `data(...)` entry follows every variant's own.
#[derive(RCondition)]
#[condition(kind = warning, class = "pkg_warning", data(source = "engine"))]
enum EngineNote<'a> {
    #[condition(
        class("pkg_warning_masked", "pkg_engine_warning"),
        message = "rule `{rule}` is masked"
    )]
    Masked { rule: &'a str },
    #[condition(message = "plain", data(step = 2i32))]
    Plain,
}

fn character(value: &RValue) -> Vec<Option<String>> {
    match value {
        RValue::Character(values) => values.clone(),
        _ => panic!("not a character value"),
    }
}

#[test]
fn warning_struct_keeps_classes_fields_and_call_policy() {
    let condition = RCondition::from(DroppedRows { n: 2 });
    let (kind, parts) = condition.into_parts();
    assert_eq!(kind, "warning");
    assert_eq!(parts.message, "dropped 2 rows");
    assert_eq!(parts.class, ["pkg_warning"]);
    assert_eq!(parts.call, ConditionCall::None);
    let fields = parts.data.unwrap();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "n");
    assert!(matches!(&fields[0].1, RValue::Integer(values) if values == &[Some(2)]));
}

#[test]
fn enum_variants_inherit_or_override_the_signal_kind() {
    let (kind, parts) = RCondition::from(Diagnostic::Dropped { n: 3 }).into_parts();
    assert_eq!(kind, "warning");
    assert_eq!(parts.class, ["pkg_diagnostic_dropped", "pkg_diagnostic"]);
    assert_eq!(parts.call, ConditionCall::Inherit);

    let (kind, parts) = RCondition::from(Diagnostic::Finished(8)).into_parts();
    assert_eq!(kind, "message");
    assert_eq!(parts.message, "finished 8 rows");
    assert_eq!(parts.data.unwrap()[0].0, "n");

    let (kind, parts) = RCondition::from(Diagnostic::Empty).into_parts();
    assert_eq!(kind, "error");
    assert_eq!(parts.message, "empty input");
    assert_eq!(parts.call, ConditionCall::None);
    assert!(parts.data.is_none());

    assert_eq!(
        RCondition::from(Diagnostic::Progress).into_parts().0,
        "condition"
    );
    // No `kind` anywhere: an error.
    assert_eq!(RCondition::from(Tick).into_parts().0, "error");
}

#[test]
fn raise_uses_the_recognized_payload_and_unwinds_rust_values() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct Dropped(Arc<AtomicBool>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Relaxed);
        }
    }
    let dropped = Arc::new(AtomicBool::new(false));
    let observed = dropped.clone();
    let payload = std::panic::catch_unwind(move || {
        let _guard = Dropped(observed);
        RCondition::from(DroppedRows { n: 4 }).raise();
    })
    .unwrap_err();
    assert!(dropped.load(Ordering::Relaxed));
    let condition = *payload
        .downcast::<RCondition>()
        .expect("framework condition payload");
    assert_eq!(condition.into_parts().0, "warning");
}

/// Queueing needs R's main thread.
#[test]
fn deferred_mixed_enum_keeps_kinds_and_queue_order() {
    use miniextendr_api::deferred_condition::{mark, take_pending};

    let kinds = r_test_utils::with_r_thread(|| {
        let start = mark();
        RCondition::from(Diagnostic::Dropped { n: 2 }).defer();
        RCondition::from(Diagnostic::Finished(10)).defer();
        RCondition::from(Diagnostic::Progress).defer();
        take_pending(start)
            .into_iter()
            .map(|condition| condition.into_parts().0)
            .collect::<Vec<_>>()
    });
    assert_eq!(kinds, ["warning", "message", "condition"]);
}

#[test]
fn struct_class_vector_fixed_entry_and_skip_none() {
    let (kind, parts) = RCondition::from(ArgumentIgnored {
        argument: "unit_table",
        hint: None,
    })
    .into_parts();
    assert_eq!(kind, "warning");
    assert_eq!(parts.message, "`unit_table` is ignored");
    assert_eq!(parts.class, ["pkg_warning_argument_ignored", "pkg_warning"]);
    let data = parts.data.expect("the fixed entry");
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].0, "param");
    assert_eq!(character(&data[0].1), [Some("unit_table".to_string())]);

    let (_, parts) = RCondition::from(ArgumentIgnored {
        argument: "unit_table",
        hint: Some("give it when grouping".to_string()),
    })
    .into_parts();
    let data = parts.data.expect("both entries");
    let names: Vec<&str> = data.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["hint", "param"]);
    assert_eq!(
        character(&data[0].1),
        [Some("give it when grouping".to_string())]
    );
}

#[test]
fn borrowed_enum_with_variant_class_list_and_data_entries() {
    let rule = String::from("lambda_z");
    let (kind, parts) = RCondition::from(EngineNote::Masked { rule: &rule }).into_parts();
    assert_eq!(kind, "warning");
    assert_eq!(parts.message, "rule `lambda_z` is masked");
    assert_eq!(
        parts.class,
        ["pkg_warning_masked", "pkg_engine_warning", "pkg_warning"]
    );
    let data = parts.data.expect("field and type entries");
    let names: Vec<&str> = data.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["rule", "source"]);
    assert_eq!(character(&data[0].1), [Some("lambda_z".to_string())]);

    let (_, parts) = RCondition::from(EngineNote::Plain).into_parts();
    assert_eq!(parts.class, ["pkg_warning_plain", "pkg_warning"]);
    let data = parts.data.expect("type and variant entries");
    let names: Vec<&str> = data.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["source", "step"]);
    assert!(matches!(&data[1].1, RValue::Integer(values) if values == &[Some(2)]));
}

#[derive(RCondition)]
#[condition(kind = warning, class = "tuple_payload", message = "tuple details")]
struct TuplePayload {
    details: (i32, String, (bool, Option<i32>)),
}

#[test]
fn tuple_condition_fields_are_owned_unnamed_lists() {
    let condition = RCondition::from(TuplePayload {
        details: (7, "seven".into(), (true, None)),
    });
    let (_, parts) = condition.into_parts();
    let data = parts.data.unwrap();
    assert_eq!(data[0].0, "details");
    let RValue::List(fields) = &data[0].1 else {
        panic!("expected a positional list")
    };
    assert_eq!(fields.len(), 3);
    assert!(fields.iter().all(|(name, _)| name.is_none()));
    assert_eq!(fields[0].1.as_i32(), Some(7));
    assert_eq!(fields[1].1.as_str(), Some("seven"));
    let RValue::List(nested) = &fields[2].1 else {
        panic!("expected a nested tuple")
    };
    assert_eq!(nested[0].1.as_bool(), Some(true));
    assert!(matches!(&nested[1].1, RValue::Integer(values) if values == &[None]));
}

#[test]
fn rvalue_tuple_arity_boundaries_keep_each_position() {
    let RValue::List(one) = RValue::from((42i32,)) else {
        panic!("expected a singleton list")
    };
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].1.as_i32(), Some(42));
    let RValue::List(twelve) = RValue::from((1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)) else {
        panic!("expected twelve fields")
    };
    assert_eq!(twelve.len(), 12);
    for (index, (name, value)) in twelve.iter().enumerate() {
        assert!(name.is_none());
        assert_eq!(value.as_i32(), Some(i32::try_from(index + 1).unwrap()));
    }
}
