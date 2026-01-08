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
