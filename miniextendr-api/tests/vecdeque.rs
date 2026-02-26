mod r_test_utils;

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::into_r::IntoR;
use std::collections::VecDeque;

#[test]
fn vecdeque_i32_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let mut deque = VecDeque::new();
        deque.push_back(1i32);
        deque.push_back(2);
        deque.push_back(3);

        let sexp = deque.clone().into_sexp();
        let back: VecDeque<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back.front(), Some(&1));
        assert_eq!(back.get(1), Some(&2));
        assert_eq!(back.get(2), Some(&3));
    });
}

#[test]
fn vecdeque_f64_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let mut deque = VecDeque::new();
        deque.push_back(1.5f64);
        deque.push_back(2.5);
        deque.push_back(3.5);

        let sexp = deque.clone().into_sexp();
        let back: VecDeque<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back.front(), Some(&1.5));
        assert_eq!(back.get(1), Some(&2.5));
        assert_eq!(back.get(2), Some(&3.5));
    });
}

#[test]
fn vecdeque_u8_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let mut deque = VecDeque::new();
        deque.push_back(10u8);
        deque.push_back(20);
        deque.push_back(30);

        let sexp = deque.clone().into_sexp();
        let back: VecDeque<u8> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back.front(), Some(&10));
        assert_eq!(back.get(1), Some(&20));
        assert_eq!(back.get(2), Some(&30));
    });
}

#[test]
fn vecdeque_empty() {
    r_test_utils::with_r_thread(|| {
        let deque: VecDeque<i32> = VecDeque::new();
        let sexp = deque.into_sexp();
        let back: VecDeque<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_empty());
    });
}

#[test]
fn vecdeque_preserves_order() {
    r_test_utils::with_r_thread(|| {
        let mut deque = VecDeque::new();
        for i in 1..=10 {
            deque.push_back(i);
        }

        let sexp = deque.clone().into_sexp();
        let back: VecDeque<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();

        // Check order preserved
        let vec1: Vec<i32> = deque.into_iter().collect();
        let vec2: Vec<i32> = back.into_iter().collect();
        assert_eq!(vec1, vec2);
    });
}

#[cfg(feature = "tinyvec")]
use miniextendr_api::coerce::Coerce;

#[cfg(feature = "tinyvec")]
#[test]
fn vecdeque_coerce_i8_to_i32() {
    // Test element-wise coercion for VecDeque
    let mut deque = VecDeque::new();
    deque.push_back(1i8);
    deque.push_back(2i8);
    deque.push_back(127i8);

    let coerced: VecDeque<i32> = deque.coerce();

    assert_eq!(coerced.len(), 3);
    assert_eq!(coerced.front(), Some(&1i32));
    assert_eq!(coerced.get(1), Some(&2i32));
    assert_eq!(coerced.get(2), Some(&127i32));
}
