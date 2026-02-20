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
        is_blocking: true,
    };
    assert!(caps.can_read);
    assert!(!caps.can_write);
    assert!(!caps.can_seek);
    assert!(caps.is_text);
    assert!(caps.is_open);
    assert!(caps.is_blocking);
}

#[test]
fn test_connection_capabilities_clone_debug() {
    let caps = ConnectionCapabilities {
        can_read: true,
        can_write: true,
        can_seek: false,
        is_text: false,
        is_open: true,
        is_blocking: false,
    };
    let cloned = caps.clone();
    assert_eq!(cloned.can_read, caps.can_read);
    assert_eq!(cloned.can_write, caps.can_write);
    assert_eq!(cloned.is_open, caps.is_open);

    // Verify Debug is implemented
    let debug_str = format!("{:?}", caps);
    assert!(debug_str.contains("ConnectionCapabilities"));
}
