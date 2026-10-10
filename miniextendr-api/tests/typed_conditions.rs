//! Typed payload derivation feeds the existing condition transport.
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
    assert_eq!(RCondition::from(Tick).into_parts().0, "condition");
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

#[test]
fn deferred_mixed_enum_keeps_kinds_and_queue_order() {
    use miniextendr_api::deferred_condition::{mark, take_pending};

    let start = mark();
    RCondition::from(Diagnostic::Dropped { n: 2 }).defer();
    RCondition::from(Diagnostic::Finished(10)).defer();
    RCondition::from(Diagnostic::Progress).defer();
    let queued = take_pending(start);
    let kinds: Vec<_> = queued
        .into_iter()
        .map(|condition| condition.into_parts().0)
        .collect();
    assert_eq!(kinds, ["warning", "message", "condition"]);
}
