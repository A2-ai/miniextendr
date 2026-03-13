//! Integration tests for coercion traits.

use miniextendr_api::coerce::{Coerce, CoerceError, TryCoerce};

#[test]
fn coerce_suite() {
    // Infallible coercions
    assert_eq!(Coerce::<i32>::coerce(5i8), 5i32);
    assert_eq!(Coerce::<f64>::coerce(true), 1.0f64);
    assert_eq!(Coerce::<i32>::coerce(false), 0i32);

    // Fallible coercions
    assert_eq!(TryCoerce::<i32>::try_coerce(123u16).unwrap(), 123i32);
    assert!(matches!(
        TryCoerce::<i32>::try_coerce(u32::MAX),
        Err(CoerceError::Overflow)
    ));

    assert!(matches!(
        TryCoerce::<i32>::try_coerce(1.5f64),
        Err(CoerceError::PrecisionLoss)
    ));

    assert!(matches!(
        TryCoerce::<i32>::try_coerce(f64::NAN),
        Err(CoerceError::NaN)
    ));
}

// region: Tests for marker trait blanket implementations

#[test]
fn widen_to_i32_marker_i8() {
    // Verify WidensToI32 marker trait works via blanket impl
    assert_eq!(Coerce::<i32>::coerce(5i8), 5i32);
    assert_eq!(Coerce::<i32>::coerce(-128i8), -128i32);
    assert_eq!(Coerce::<i32>::coerce(127i8), 127i32);
}

#[test]
fn widen_to_i32_marker_i16() {
    // Verify WidensToI32 marker trait works via blanket impl
    assert_eq!(Coerce::<i32>::coerce(1000i16), 1000i32);
    assert_eq!(Coerce::<i32>::coerce(-32768i16), -32768i32);
    assert_eq!(Coerce::<i32>::coerce(32767i16), 32767i32);
}

#[test]
fn widen_to_i32_marker_u8() {
    // Verify WidensToI32 marker trait works via blanket impl
    assert_eq!(Coerce::<i32>::coerce(0u8), 0i32);
    assert_eq!(Coerce::<i32>::coerce(255u8), 255i32);
}

#[test]
fn widen_to_i32_marker_u16() {
    // Verify WidensToI32 marker trait works via blanket impl
    assert_eq!(Coerce::<i32>::coerce(0u16), 0i32);
    assert_eq!(Coerce::<i32>::coerce(65535u16), 65535i32);
}

#[test]
fn widen_to_f64_marker_f32() {
    // Verify WidensToF64 marker trait works via blanket impl
    assert_eq!(Coerce::<f64>::coerce(1.5f32), 1.5f64);
    assert_eq!(Coerce::<f64>::coerce(0.0f32), 0.0f64);
}

#[test]
fn widen_to_f64_marker_i32() {
    // Verify WidensToF64 marker trait works via blanket impl
    assert_eq!(Coerce::<f64>::coerce(42i32), 42.0f64);
    assert_eq!(Coerce::<f64>::coerce(-100i32), -100.0f64);
}

#[test]
fn widen_to_f64_marker_all_integer_types() {
    // Verify all integer types that widen to f64 work via marker
    assert_eq!(Coerce::<f64>::coerce(1i8), 1.0f64);
    assert_eq!(Coerce::<f64>::coerce(2i16), 2.0f64);
    assert_eq!(Coerce::<f64>::coerce(3i32), 3.0f64);
    assert_eq!(Coerce::<f64>::coerce(4u8), 4.0f64);
    assert_eq!(Coerce::<f64>::coerce(5u16), 5.0f64);
    assert_eq!(Coerce::<f64>::coerce(6u32), 6.0f64);
}
// endregion

// region: Tests for container element-wise coercion

#[test]
fn vec_element_coerce_i8_to_i32() {
    let v: Vec<i8> = vec![1, 2, 3, 127, -128];
    let coerced: Vec<i32> = v.coerce();
    assert_eq!(coerced, vec![1i32, 2, 3, 127, -128]);
}

#[test]
fn vec_element_coerce_u8_to_f64() {
    let v: Vec<u8> = vec![0, 100, 255];
    let coerced: Vec<f64> = v.coerce();
    assert_eq!(coerced, vec![0.0, 100.0, 255.0]);
}

#[test]
fn slice_element_coerce_i16_to_i32() {
    let arr: [i16; 4] = [10, 20, 30, 40];
    let coerced: Vec<i32> = arr.as_slice().coerce();
    assert_eq!(coerced, vec![10i32, 20, 30, 40]);
}

use std::collections::VecDeque;

#[test]
fn vecdeque_element_coerce() {
    let mut deque = VecDeque::new();
    deque.push_back(1i8);
    deque.push_back(2i8);
    deque.push_back(3i8);

    let coerced: VecDeque<i32> = deque.coerce();
    assert_eq!(coerced.front(), Some(&1i32));
    assert_eq!(coerced.get(1), Some(&2i32));
    assert_eq!(coerced.get(2), Some(&3i32));
}

#[cfg(feature = "tinyvec")]
use miniextendr_api::{ArrayVec, TinyVec};

#[cfg(feature = "tinyvec")]
#[test]
fn tinyvec_element_coerce_i8_to_i32() {
    let mut tv: TinyVec<[i8; 8]> = TinyVec::new();
    tv.push(1);
    tv.push(2);
    tv.push(127);

    let coerced: TinyVec<[i32; 8]> = tv.coerce();
    assert_eq!(coerced.len(), 3);
    assert_eq!(coerced[0], 1i32);
    assert_eq!(coerced[1], 2i32);
    assert_eq!(coerced[2], 127i32);
}

#[cfg(feature = "tinyvec")]
#[test]
fn arrayvec_element_coerce_u8_to_i32() {
    let mut av: ArrayVec<[u8; 8]> = ArrayVec::new();
    av.push(10);
    av.push(20);
    av.push(255);

    let coerced: ArrayVec<[i32; 8]> = av.coerce();
    assert_eq!(coerced.len(), 3);
    assert_eq!(coerced[0], 10i32);
    assert_eq!(coerced[1], 20i32);
    assert_eq!(coerced[2], 255i32);
}
// endregion
