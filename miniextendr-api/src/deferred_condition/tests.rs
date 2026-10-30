//! Queue mechanics on the test's own thread (no R runtime). Each test runs on
//! a fresh thread, so it starts with an empty queue. `defer` sends from that
//! thread (it is not R's main thread); the typed payloads, the `defer_*!`
//! macros and the channel from other threads are covered on R's main thread
//! by `tests/deferred_conditions.rs`.

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

/// `capture` refuses a thread that isn't R's main thread before running the
/// block. A spawned thread never is, whichever test recorded one.
#[test]
fn capture_off_the_main_thread_panics() {
    let (message, ran) = std::thread::spawn(|| {
        let ran = std::cell::Cell::new(false);
        let payload =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| capture(|| ran.set(true))))
                .expect_err("capture off R's main thread must panic");
        (
            crate::unwind_protect::panic_payload_to_string(payload.as_ref()).into_owned(),
            ran.get(),
        )
    })
    .join()
    .unwrap();
    assert!(message.contains("off R's main thread"), "{message}");
    assert!(message.contains("around the"), "{message}");
    assert!(!ran);
}
