//! Queue mechanics on the test's own thread (no R runtime). Each test runs on
//! a fresh thread, so it starts with an empty queue. `defer` itself refuses
//! that thread (it is not R's main thread); the typed payloads and the
//! `defer_*!` macros are covered on R's main thread by
//! `tests/deferred_conditions.rs`.

use super::*;
use crate::condition::ConditionCall;

fn warning(message: &str) -> RCondition {
    RCondition::Warning {
        message: message.to_string(),
        class: Vec::new(),
        data: None,
        call: ConditionCall::Inherit,
    }
}

fn messages(queued: Vec<RCondition>) -> Vec<String> {
    queued
        .into_iter()
        .map(|condition| condition.message().to_string())
        .collect()
}

#[test]
fn nested_marks_take_only_their_own_entries() {
    let outer = mark();
    push(warning("outer"));
    let inner = mark();
    push(warning("inner"));
    assert_eq!(messages(take_pending(inner)), ["inner"]);
    assert_eq!(messages(take_pending(outer)), ["outer"]);
}

#[test]
fn finalizer_suppression_nests_and_preserves_the_enclosing_queue() {
    push(warning("outer"));
    {
        let _outer = Suppress::new();
        push(warning("finalizer"));
        {
            let _inner = Suppress::new();
            push(warning("nested finalizer"));
        }
        // A nested guard inside the finalizer neither drains nor discards
        // the enclosing call's entries.
        assert!(take_pending(0).is_empty());
        discard(0);
        assert_eq!(mark(), 1);
        push(warning("still suppressed"));
    }
    push(warning("after"));
    assert_eq!(messages(take_pending(0)), ["outer", "after"]);
}

#[test]
fn abandoned_inner_queue_does_not_discard_outer_conditions() {
    push(warning("outer"));
    let inner = mark();
    push(warning("abandoned"));
    discard(inner);
    assert_eq!(messages(take_pending(0)), ["outer"]);
}

#[test]
fn take_pending_past_the_end_is_empty() {
    assert!(take_pending(usize::MAX).is_empty());
    assert!(take_pending(mark() + 1).is_empty());
}

/// A spawned thread is never R's main thread, whichever test recorded one.
#[test]
fn defer_off_the_main_thread_panics_without_queueing() {
    let (payload, queued) = std::thread::spawn(|| {
        let payload = std::panic::catch_unwind(|| crate::defer_warning!("from a Rayon task"))
            .expect_err("defer_warning! off R's main thread must panic");
        (
            crate::unwind_protect::panic_payload_to_string(payload.as_ref()).into_owned(),
            mark(),
        )
    })
    .join()
    .unwrap();
    assert!(payload.contains("off R's main thread"), "{payload}");
    assert!(payload.contains("queue the conditions"), "{payload}");
    assert_eq!(queued, 0);
}
