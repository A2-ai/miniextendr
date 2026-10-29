use super::*;
use crate::condition::{ConditionCall, ConditionData, RError};
use crate::error_value::kind;

/// `(name, i32)` view of a data payload (`RValue` has no `PartialEq`).
fn ints(data: &Option<ConditionData>) -> Vec<(&str, Option<i32>)> {
    data.as_ref()
        .map(|fields| {
            fields
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_i32()))
                .collect()
        })
        .unwrap_or_default()
}

/// The queue is process-wide; serialise the tests that touch it.
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn with_empty_queue<T>(f: impl FnOnce() -> T) -> T {
    let _lock = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    take_pending(0);
    f()
}

#[test]
fn typed_payloads_keep_kind_class_and_data() {
    with_empty_queue(|| {
        let m = mark();
        defer_warning(RError::new("w").class(["a", "b"]).data("n", 1));
        defer_message(RError::new("m").class("c"));
        defer_condition(RError::new("c"));
        let parts: Vec<_> = take_pending(m)
            .into_iter()
            .map(RCondition::into_parts)
            .collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].0, kind::WARNING);
        assert_eq!(parts[0].1.message, "w");
        assert_eq!(parts[0].1.class, vec!["a", "b"]);
        assert_eq!(ints(&parts[0].1.data), vec![("n", Some(1))]);
        assert_eq!(parts[1].0, kind::MESSAGE);
        assert_eq!(parts[1].1.class, vec!["c"]);
        assert!(parts[1].1.data.is_none());
        assert_eq!(parts[2].0, kind::CONDITION);
        assert!(parts[2].1.class.is_empty());
        assert!(
            parts
                .iter()
                .all(|(_, parts)| parts.call == ConditionCall::Inherit)
        );
        assert!(take_pending(m).is_empty());
    });
}

#[test]
fn nested_marks_take_only_their_own_entries() {
    with_empty_queue(|| {
        let outer = mark();
        crate::defer_warning!("outer");
        let inner = mark();
        crate::defer_warning!(class = "t", data = ("n", 2), "inner {}", 2);
        let inner_taken = take_pending(inner);
        assert_eq!(inner_taken.len(), 1);
        let (kind, parts) = inner_taken.into_iter().next().unwrap().into_parts();
        assert_eq!((kind, parts.message.as_str()), (kind::WARNING, "inner 2"));
        assert_eq!(parts.class, vec!["t"]);
        assert_eq!(ints(&parts.data), vec![("n", Some(2))]);
        let outer_taken = take_pending(outer);
        assert_eq!(outer_taken.len(), 1);
        assert_eq!(outer_taken.into_iter().next().unwrap().message(), "outer");
    });
}

#[test]
fn capture_takes_only_the_block_s_conditions() {
    with_empty_queue(|| {
        let outer = mark();
        crate::defer_warning!("before");
        let (value, captured) = capture(|| {
            crate::defer_warning!(class = "replayed", "inside {}", 1);
            let ((), nested) = capture(|| crate::defer_message!("nested"));
            assert_eq!(nested.len(), 1);
            assert_eq!(nested[0].message(), "nested");
            crate::defer_condition!("last");
            7
        });
        assert_eq!(value, 7);
        let parts: Vec<_> = captured.into_iter().map(RCondition::into_parts).collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].0, kind::WARNING);
        assert_eq!(parts[0].1.message, "inside 1");
        assert_eq!(parts[0].1.class, vec!["replayed"]);
        assert_eq!(parts[1].0, kind::CONDITION);
        let remaining = take_pending(outer);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].message(), "before");
    });
}

#[test]
fn capture_leaves_the_queue_on_unwind() {
    with_empty_queue(|| {
        let start = mark();
        let caught = crate::condition::catch_condition(|| {
            capture(|| {
                crate::defer_warning!("queued before the error");
                crate::error!("failed");
            })
        });
        assert_eq!(caught.unwrap_err().message(), "failed");
        let remaining = take_pending(start);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].message(), "queued before the error");

        // Catching inside the capture keeps the block's conditions with it.
        let (outcome, captured) = capture(|| {
            crate::condition::catch_condition(|| {
                crate::defer_warning!("kept");
                crate::error!("failed");
            })
        });
        assert_eq!(outcome.unwrap_err().message(), "failed");
        assert_eq!(captured.len(), 1);
        assert!(take_pending(start).is_empty());
    });
}

#[test]
fn macro_forms_cover_message_and_condition() {
    with_empty_queue(|| {
        let m = mark();
        crate::defer_message!(data = { step = 1 }, "step {}", 1);
        crate::defer_condition!(class = ["member", "family"], "tick");
        let parts: Vec<_> = take_pending(m)
            .into_iter()
            .map(RCondition::into_parts)
            .collect();
        assert_eq!(parts[0].0, kind::MESSAGE);
        assert_eq!(parts[0].1.message, "step 1");
        assert_eq!(parts[1].0, kind::CONDITION);
        assert_eq!(parts[1].1.class, vec!["member", "family"]);
    });
}

#[test]
fn call_none_is_queued_with_the_payload() {
    with_empty_queue(|| {
        let m = mark();
        crate::defer_warning!(call = none, class = "t", data = { n = 3 }, "plain {}", 1);
        crate::defer_condition!(call = none, "plain condition");
        crate::defer_message!(call = none, "accepted, a message has no call anyway");
        crate::defer_warning!("sibling keeps its call");
        defer_warning(RError::new("typed").without_call());
        let parts: Vec<_> = take_pending(m)
            .into_iter()
            .map(RCondition::into_parts)
            .collect();
        let calls: Vec<_> = parts.iter().map(|(_, parts)| parts.call).collect();
        assert_eq!(
            calls,
            [
                ConditionCall::None,
                ConditionCall::None,
                ConditionCall::None,
                ConditionCall::Inherit,
                ConditionCall::None,
            ]
        );
        assert_eq!(parts[0].0, kind::WARNING);
        assert_eq!(parts[0].1.message, "plain 1");
        assert_eq!(parts[0].1.class, vec!["t"]);
        assert_eq!(ints(&parts[0].1.data), vec![("n", Some(3))]);
        assert_eq!(parts[1].0, kind::CONDITION);
        assert_eq!(parts[2].0, kind::MESSAGE);
    });
}

#[test]
fn finalizer_suppression_nests_and_preserves_the_enclosing_queue() {
    with_empty_queue(|| {
        crate::defer_warning!("outer");
        {
            let _outer = Suppress::new();
            crate::defer_warning!("finalizer");
            {
                let _inner = Suppress::new();
                crate::defer_message!("nested finalizer");
            }
            crate::defer_condition!("still suppressed");
        }
        crate::defer_message!("after");
        let messages: Vec<_> = take_pending(0)
            .into_iter()
            .map(|condition| condition.message().to_string())
            .collect();
        assert_eq!(messages, ["outer", "after"]);
    });
}

#[test]
fn finalizer_suppression_does_not_drop_another_threads_conditions() {
    with_empty_queue(|| {
        {
            let _suppressed = Suppress::new();
            std::thread::spawn(|| crate::defer_warning!("worker"))
                .join()
                .unwrap();
            crate::defer_warning!("finalizer");
            assert!(take_pending(0).is_empty());
            discard(0);
            assert_eq!(mark(), 1);
        }
        let remaining = take_pending(0);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining.into_iter().next().unwrap().message(), "worker");
    });
}

#[test]
fn abandoned_inner_queue_does_not_discard_outer_conditions() {
    with_empty_queue(|| {
        crate::defer_warning!("outer");
        let inner = mark();
        crate::defer_warning!("abandoned");
        discard(inner);
        let remaining = take_pending(0);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining.into_iter().next().unwrap().message(), "outer");
    });
}

#[test]
fn take_pending_past_the_end_is_empty() {
    with_empty_queue(|| {
        assert!(take_pending(usize::MAX).is_empty());
        assert!(take_pending(mark() + 1).is_empty());
    });
}
