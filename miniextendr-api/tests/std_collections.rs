mod r_test_utils;

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::into_r::IntoR;
use std::borrow::Cow;
use std::collections::BinaryHeap;

// region: BinaryHeap tests

#[test]
fn binaryheap_i32_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let mut heap = BinaryHeap::new();
        heap.push(3);
        heap.push(1);
        heap.push(4);
        heap.push(2);

        let sexp = heap.clone().into_sexp();
        let back: BinaryHeap<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();

        // BinaryHeap doesn't preserve order, but should have same elements
        assert_eq!(back.len(), 4);
        let back_sorted: Vec<i32> = back.into_sorted_vec();
        assert_eq!(back_sorted, vec![1, 2, 3, 4]);
    });
}

#[test]
fn binaryheap_u8_max_heap() {
    // Note: BinaryHeap requires Ord, so f64 doesn't work (NaN issues)
    // Use u8 instead
    r_test_utils::with_r_thread(|| {
        let mut heap = BinaryHeap::new();
        heap.push(10u8);
        heap.push(30u8);
        heap.push(20u8);

        let sexp = heap.into_sexp();
        let mut back: BinaryHeap<u8> = TryFromSexp::try_from_sexp(sexp).unwrap();

        // Verify it's a max heap
        assert_eq!(back.pop(), Some(30));
        assert_eq!(back.pop(), Some(20));
        assert_eq!(back.pop(), Some(10));
    });
}

#[test]
fn binaryheap_empty() {
    r_test_utils::with_r_thread(|| {
        let heap: BinaryHeap<i32> = BinaryHeap::new();
        let sexp = heap.into_sexp();
        let back: BinaryHeap<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_empty());
    });
}
// endregion

// region: Cow tests

#[test]
fn cow_slice_borrowed() {
    r_test_utils::with_r_thread(|| {
        let data = vec![1i32, 2, 3];
        let cow: Cow<[i32]> = Cow::Borrowed(&data);

        let sexp = cow.into_sexp();
        let back: Vec<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, vec![1, 2, 3]);
    });
}

#[test]
fn cow_slice_owned() {
    r_test_utils::with_r_thread(|| {
        let cow: Cow<[i32]> = Cow::Owned(vec![10, 20, 30]);

        let sexp = cow.into_sexp();
        let back: Vec<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, vec![10, 20, 30]);
    });
}

#[test]
fn cow_slice_from_r() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{INTEGER, Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE};

        unsafe {
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 3));
            let ptr = INTEGER(sexp);
            *ptr.add(0) = 100;
            *ptr.add(1) = 200;
            *ptr.add(2) = 300;

            let cow: Cow<'static, [i32]> = TryFromSexp::try_from_sexp(sexp).unwrap();

            // Zero-copy: borrows directly from R's SEXP data
            assert!(matches!(cow, Cow::Borrowed(_)));
            assert_eq!(cow.as_ref(), &[100, 200, 300]);

            Rf_unprotect(1);
        }
    });
}

#[test]
fn cow_str_borrowed() {
    r_test_utils::with_r_thread(|| {
        let s = "hello world";
        let cow: Cow<str> = Cow::Borrowed(s);

        let sexp = cow.into_sexp();
        let back: String = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, "hello world");
    });
}

#[test]
fn cow_str_owned() {
    r_test_utils::with_r_thread(|| {
        let cow: Cow<str> = Cow::Owned("hello rust".to_string());

        let sexp = cow.into_sexp();
        let back: String = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, "hello rust");
    });
}

#[test]
fn cow_str_from_r() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::Rf_mkString;
        use std::ffi::CString;

        unsafe {
            let c_str = CString::new("test string").unwrap();
            let sexp = Rf_mkString(c_str.as_ptr());

            let cow: Cow<'static, str> = TryFromSexp::try_from_sexp(sexp).unwrap();

            // Zero-copy: borrows directly from R's CHARSXP data
            assert!(matches!(cow, Cow::Borrowed(_)));
            assert_eq!(cow.as_ref(), "test string");
        }
    });
}

#[test]
fn cow_slice_f64() {
    r_test_utils::with_r_thread(|| {
        let data = vec![1.5, 2.5, 3.5];
        let cow: Cow<[f64]> = Cow::Borrowed(&data);

        let sexp = cow.into_sexp();
        let back: Vec<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, vec![1.5, 2.5, 3.5]);
    });
}
// endregion
