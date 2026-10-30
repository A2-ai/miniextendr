//! The hand-over bookkeeping, without R: the SEXPs here are never read. The
//! round trips through R are in `tests/serde_exact.rs`.

use super::{buffer, buffered, forget_written, is_written, reading, with_buffer, written};
use crate::SEXP;

fn fake(n: usize) -> SEXP {
    SEXP(std::ptr::without_provenance_mut(n * 16))
}

#[test]
fn a_token_finds_its_object_only_while_its_decoder_runs() {
    assert_eq!(buffer(fake(1)), None, "no decoder runs");
    let token = with_buffer(|| {
        let first = buffer(fake(1)).expect("a decoder runs");
        let second = buffer(fake(2)).expect("a decoder runs");
        assert_eq!(buffered(&first), Some(fake(1)));
        assert_eq!(buffered(&second), Some(fake(2)));
        // A string that is not a token, or names no buffered object.
        assert_eq!(buffered("1"), None);
        assert_eq!(buffered("$miniextendr::RObject#7"), None);
        // A nested decoder has its own buffer, and the outer one is back
        // after it.
        with_buffer(|| assert_eq!(buffered(&first), None));
        assert_eq!(buffered(&first), Some(fake(1)));
        first
    });
    assert_eq!(buffered(&token), None, "the decoder ended");
}

#[test]
fn the_buffer_ends_with_its_decoder_on_a_panic_too() {
    let result = std::panic::catch_unwind(|| {
        with_buffer(|| {
            buffer(fake(3));
            panic!("decoding failed");
        })
    });
    assert!(result.is_err());
    assert_eq!(buffer(fake(3)), None);
}

#[test]
fn the_reading_object_is_taken_once_and_cleared_after_the_visit() {
    reading(fake(4), || {
        assert_eq!(super::READING.take(), Some(fake(4)));
        assert_eq!(super::READING.take(), None);
    });
    // A visit that does not take it leaves nothing behind.
    reading(fake(5), || ());
    assert_eq!(super::READING.take(), None);
}

#[test]
fn a_written_object_is_handed_over_once() {
    assert_eq!(written(), None, "no RObject is serializing");
    super::WRITING.set(Some(fake(6)));
    assert_eq!(written(), Some(fake(6)));
    assert!(is_written(fake(6)));
    assert!(!is_written(fake(7)));
    assert_eq!(written(), None);
    forget_written();
    assert!(!is_written(fake(6)));
}
