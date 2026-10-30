use super::*;

#[test]
fn test_connections_version() {
    // This just verifies the constant is defined correctly
    assert_eq!(R_CONNECTIONS_VERSION, 1);
    assert_eq!(EXPECTED_CONNECTIONS_VERSION, 1);
}

#[test]
fn test_rconn_struct_size() {
    // Sanity check that the struct is reasonably sized
    // The actual size depends on platform alignment
    let size = std::mem::size_of::<Rconn>();
    // Should be at least several hundred bytes
    assert!(size > 200, "Rconn struct seems too small: {} bytes", size);
    // Should be less than 2KB
    assert!(size < 2048, "Rconn struct seems too large: {} bytes", size);
}

#[test]
fn test_catch_connection_panic_ok() {
    let result = catch_connection_panic(42, || 7);
    assert_eq!(result, 7);
}

#[test]
fn test_catch_connection_panic_catches_panic() {
    let result = catch_connection_panic(42, || {
        panic!("test panic");
    });
    assert_eq!(result, 42);
}

#[test]
fn test_catch_connection_panic_catches_string_panic() {
    let result = catch_connection_panic(-1, || {
        let msg = "custom message".to_string();
        std::panic::panic_any(msg);
    });
    assert_eq!(result, -1);
}

#[test]
fn test_catch_connection_panic_catches_non_string_panic() {
    let result = catch_connection_panic(0usize, || {
        std::panic::panic_any(123i32);
    });
    assert_eq!(result, 0);
}

#[test]
fn test_checked_mul_overflow_returns_zero() {
    // Simulate what read/write trampolines do: checked_mul on extreme values
    let size: usize = usize::MAX;
    let nitems: usize = 2;
    let total = size.checked_mul(nitems);
    assert!(total.is_none(), "usize::MAX * 2 should overflow");
}

#[test]
fn test_connection_capabilities_default() {
    // Verify the struct can be constructed with all fields
    let caps = ConnectionCapabilities {
        can_read: true,
        can_write: false,
        can_seek: false,
        is_text: true,
        is_open: true,
        is_blocking: Some(true),
    };
    assert!(caps.can_read);
    assert!(!caps.can_write);
    assert!(!caps.can_seek);
    assert!(caps.is_text);
    assert!(caps.is_open);
    assert_eq!(caps.is_blocking, Some(true));
}

#[test]
fn test_connection_capabilities_clone_debug() {
    let caps = ConnectionCapabilities {
        can_read: true,
        can_write: true,
        can_seek: false,
        is_text: false,
        is_open: true,
        is_blocking: Some(false),
    };
    let cloned = caps.clone();
    assert_eq!(cloned.can_read, caps.can_read);
    assert_eq!(cloned.can_write, caps.can_write);
    assert_eq!(cloned.is_open, caps.is_open);

    // Verify Debug is implemented
    let debug_str = format!("{:?}", caps);
    assert!(debug_str.contains("ConnectionCapabilities"));
}

// region: RStdin line buffer — issue #1912

/// A `StdinLine::fill_with` source that appends `text`.
fn line_source(text: &'static [u8]) -> impl FnOnce(&mut Vec<u8>) -> std::io::Result<()> {
    move |buf| {
        buf.extend_from_slice(text);
        Ok(())
    }
}

#[test]
fn stdin_line_refills_only_when_used_up() {
    let mut line = StdinLine::new();
    assert_eq!(line.fill_with(line_source(b"hello\n")).unwrap(), b"hello\n");
    line.consume(2);
    let rest = line
        .fill_with(|_| panic!("refilled before the line was used up"))
        .unwrap();
    assert_eq!(rest, b"llo\n");
    line.consume(4);
    assert_eq!(line.fill_with(line_source(b"next\n")).unwrap(), b"next\n");
}

#[test]
fn stdin_line_is_empty_at_end_of_input() {
    let mut line = StdinLine::new();
    assert!(line.fill_with(line_source(b"")).unwrap().is_empty());
    // The end of the input doesn't stick: the next fill asks again.
    assert_eq!(line.fill_with(line_source(b"more\n")).unwrap(), b"more\n");
}

#[test]
fn stdin_line_consume_stops_at_the_end_of_the_line() {
    let mut line = StdinLine::new();
    line.fill_with(line_source(b"ab\n")).unwrap();
    line.consume(usize::MAX);
    assert_eq!(line.fill_with(line_source(b"c\n")).unwrap(), b"c\n");
}

#[test]
fn stdin_line_passes_on_the_source_error() {
    let mut line = StdinLine::new();
    let err = line
        .fill_with(|_| Err(std::io::Error::other("boom")))
        .unwrap_err();
    assert_eq!(err.to_string(), "boom");
}

#[test]
fn rstdin_off_main_thread_is_an_error() {
    use std::io::Read;
    // No R runtime in unit tests, so this thread is not R's main thread.
    let mut buf = [0u8; 8];
    let err = RStdin.read(&mut buf).unwrap_err();
    assert_eq!(
        err.to_string(),
        "RStdin can only be read on R's main thread"
    );
    // An empty buffer reads nothing and needs no line.
    assert_eq!(RStdin.read(&mut []).unwrap(), 0);
}

#[test]
fn rstdin_handles_share_one_buffer() {
    use std::io::Read;
    // Put a line in this thread's buffer, as a refill on R's main thread would.
    let mut lock = RStdin.lock();
    lock.line.bytes.extend_from_slice(b"abc\n");
    drop(lock);

    let mut buf = [0u8; 2];
    assert_eq!(RStdin.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"ab");
    let mut rest = String::new();
    assert_eq!(RStdin.read_line(&mut rest).unwrap(), 2);
    assert_eq!(rest, "c\n");
    // The line is used up, and a new one needs R's main thread.
    assert!(RStdin.read(&mut buf).is_err());
}

#[test]
#[should_panic(expected = "RStdin is already locked on this thread")]
fn rstdin_lock_twice_on_one_thread_panics() {
    let _first = RStdin.lock();
    let _second = RStdin.lock();
}

// endregion
