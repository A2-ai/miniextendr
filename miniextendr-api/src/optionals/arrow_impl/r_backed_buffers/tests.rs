use super::*;
use arrow_buffer::Buffer;

/// A stand-in address; the registry never dereferences it.
const VECTOR: usize = 0x1000;

fn key_of(buffer: &Buffer) -> BufferKey {
    (buffer.as_ptr() as usize, buffer.len())
}

#[test]
fn unregistered_exact_capacity_buffer_misses() {
    // The DataFusion one-row shape: offset 0, capacity exactly the data.
    let buffer = Buffer::from(vec![0u8; 8]);
    assert_eq!(buffer.ptr_offset(), 0);
    assert_eq!(buffer.capacity(), 8);
    assert!(
        RBackedBuffers::default()
            .lookup::<f64>(&buffer, 1)
            .is_none()
    );
}

#[test]
fn entry_lives_while_any_guard_does() {
    let buffer = Buffer::from(vec![0u8; 16]);
    let mut map = RBackedBuffers::default();
    map.remember(key_of(&buffer), VECTOR, SEXPTYPE::REALSXP);
    map.remember(key_of(&buffer), VECTOR, SEXPTYPE::REALSXP);
    assert_eq!(map.lookup::<f64>(&buffer.clone(), 2), Some(VECTOR));
    map.forget(key_of(&buffer));
    assert_eq!(map.lookup::<f64>(&buffer, 2), Some(VECTOR));
    map.forget(key_of(&buffer));
    assert!(map.lookup::<f64>(&buffer, 2).is_none());
    // A stray forget leaves the map consistent.
    map.forget(key_of(&buffer));
    assert!(map.entries.is_empty());
}

#[test]
fn type_or_length_mismatch_misses() {
    let buffer = Buffer::from(vec![0u8; 16]);
    let mut map = RBackedBuffers::default();
    map.remember(key_of(&buffer), VECTOR, SEXPTYPE::REALSXP);
    // Same 16 bytes read as four i32 values: key hit, type miss.
    assert!(map.lookup::<i32>(&buffer, 4).is_none());
    // Two i32 values cover 8 bytes: key miss.
    assert!(map.lookup::<i32>(&buffer, 2).is_none());
    assert!(map.lookup::<f64>(&buffer, 1).is_none());
    assert!(map.lookup::<f64>(&buffer, 0).is_none());
}

#[test]
fn slices_miss() {
    let buffer = Buffer::from(vec![0u8; 16]);
    let mut map = RBackedBuffers::default();
    map.remember(key_of(&buffer), VECTOR, SEXPTYPE::REALSXP);
    assert!(
        map.lookup::<f64>(&buffer.slice_with_length(0, 8), 1)
            .is_none()
    );
    assert!(map.lookup::<f64>(&buffer.slice(8), 1).is_none());
    assert_eq!(map.lookup::<f64>(&buffer.slice(0), 2), Some(VECTOR));
}
