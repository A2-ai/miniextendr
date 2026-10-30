//! The deferred-condition queue on R's main thread: typed payloads, the
//! `defer_*!` macro grammar, `capture`, and conditions queued on other threads.
//! Every test runs on the one recorded R thread, so each starts by clearing
//! what an earlier test may have left queued.

mod r_test_utils;

use miniextendr_api::condition::{ConditionCall, ConditionData, RCondition, RError};
use miniextendr_api::deferred_condition::{capture, mark, take_pending};
use miniextendr_api::error_value::kind;
use miniextendr_api::{defer_condition, defer_message, defer_warning};

/// `(name, i32)` view of a data payload (`RValue` has no `PartialEq`).
fn ints(data: &Option<ConditionData>) -> Vec<(String, Option<i32>)> {
    data.as_ref()
        .map(|fields| {
            fields
                .iter()
                .map(|(name, value)| (name.clone(), value.as_i32()))
                .collect()
        })
        .unwrap_or_default()
}

fn on_main_with_empty_queue<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    r_test_utils::with_r_thread(|| {
        take_pending(0);
        f()
    })
}

#[test]
fn typed_payloads_keep_kind_class_and_data() {
    on_main_with_empty_queue(|| {
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
        assert_eq!(ints(&parts[0].1.data), vec![("n".to_string(), Some(1))]);
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
fn macro_forms_keep_class_data_and_format() {
    on_main_with_empty_queue(|| {
        let m = mark();
        defer_warning!(class = "t", data = ("n", 2), "inner {}", 2);
        defer_message!(data = { step = 1 }, "step {}", 1);
        defer_condition!(class = ["member", "family"], "tick");
        let parts: Vec<_> = take_pending(m)
            .into_iter()
            .map(RCondition::into_parts)
            .collect();
        assert_eq!(parts[0].0, kind::WARNING);
        assert_eq!(parts[0].1.message, "inner 2");
        assert_eq!(parts[0].1.class, vec!["t"]);
        assert_eq!(ints(&parts[0].1.data), vec![("n".to_string(), Some(2))]);
        assert_eq!(parts[1].0, kind::MESSAGE);
        assert_eq!(parts[1].1.message, "step 1");
        assert_eq!(parts[2].0, kind::CONDITION);
        assert_eq!(parts[2].1.class, vec!["member", "family"]);
    });
}

#[test]
fn call_none_is_queued_with_the_payload() {
    let calls = on_main_with_empty_queue(|| {
        let m = mark();
        defer_warning!(call = none, class = "t", data = { n = 3 }, "plain {}", 1);
        defer_condition!(call = none, "plain condition");
        defer_message!(call = none, "accepted, a message has no call anyway");
        defer_warning!("sibling keeps its call");
        defer_warning(RError::new("typed").without_call());
        take_pending(m)
            .into_iter()
            .map(|condition| condition.into_parts().1.call)
            .collect::<Vec<_>>()
    });
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
}

#[test]
fn capture_takes_only_the_block_s_conditions() {
    on_main_with_empty_queue(|| {
        let outer = mark();
        defer_warning!("before");
        let (value, captured) = capture(|| {
            defer_warning!(class = "replayed", "inside {}", 1);
            let ((), nested) = capture(|| defer_message!("nested"));
            assert_eq!(nested.len(), 1);
            assert_eq!(nested[0].message(), "nested");
            defer_condition!("last");
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
    on_main_with_empty_queue(|| {
        let start = mark();
        let caught = miniextendr_api::condition::catch_condition(|| {
            capture(|| {
                defer_warning!("queued before the error");
                miniextendr_api::error!("failed");
            })
        });
        assert_eq!(caught.unwrap_err().message(), "failed");
        let remaining = take_pending(start);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].message(), "queued before the error");

        // Catching inside the capture keeps the block's conditions with it.
        let (outcome, captured) = capture(|| {
            miniextendr_api::condition::catch_condition(|| {
                defer_warning!("kept");
                miniextendr_api::error!("failed");
            })
        });
        assert_eq!(outcome.unwrap_err().message(), "failed");
        assert_eq!(captured.len(), 1);
        assert!(take_pending(start).is_empty());
    });
}

/// A spawned thread's conditions reach R's main thread over the channel: the
/// main thread's next read of the queue takes them, after its own, in the
/// order the thread queued them.
#[test]
fn a_joined_thread_s_conditions_join_the_queue() {
    let queued = on_main_with_empty_queue(|| {
        let start = mark();
        defer_warning!("on main");
        std::thread::spawn(|| {
            defer_warning!(class = "elsewhere", "from a spawned thread");
            defer_message!("and a message");
        })
        .join()
        .unwrap();
        take_pending(start)
            .into_iter()
            .map(RCondition::into_parts)
            .map(|(kind, parts)| (kind, parts.message, parts.class))
            .collect::<Vec<_>>()
    });
    assert_eq!(
        queued,
        [
            (kind::WARNING, "on main".to_string(), Vec::<String>::new()),
            (
                kind::WARNING,
                "from a spawned thread".to_string(),
                vec!["elsewhere".to_string()]
            ),
            (kind::MESSAGE, "and a message".to_string(), Vec::new()),
        ]
    );
}

/// `capture` around a parallel section takes what its tasks queued.
#[test]
fn capture_takes_a_joined_thread_s_conditions() {
    let captured = on_main_with_empty_queue(|| {
        let ((), captured) = capture(|| {
            std::thread::scope(|s| {
                s.spawn(|| defer_warning!("in scope"));
            });
        });
        let messages: Vec<String> = captured
            .iter()
            .map(|condition| condition.message().to_string())
            .collect();
        assert!(take_pending(0).is_empty());
        messages
    });
    assert_eq!(captured, ["in scope"]);
}

/// The channel holds 1024 conditions; the rest are dropped and counted, and
/// the main thread queues one warning saying how many, after the ones kept.
#[test]
fn a_full_channel_drops_the_newest_and_says_how_many() {
    let messages = on_main_with_empty_queue(|| {
        let start = mark();
        std::thread::spawn(|| {
            for i in 0..1030 {
                defer_condition!("tick {i}");
            }
        })
        .join()
        .unwrap();
        take_pending(start)
            .into_iter()
            .map(|condition| condition.message().to_string())
            .collect::<Vec<_>>()
    });
    assert_eq!(messages.len(), 1025);
    assert_eq!(messages[0], "tick 0");
    assert_eq!(messages[1023], "tick 1023");
    assert_eq!(
        messages[1024],
        "6 deferred conditions from other threads dropped due to queue overflow"
    );
}
