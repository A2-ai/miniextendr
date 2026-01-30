mod r_test_utils;

#[cfg(feature = "tinyvec")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "tinyvec")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "tinyvec")]
use miniextendr_api::{ArrayVec, TinyVec};

#[cfg(feature = "tinyvec")]
#[test]
fn tinyvec_i32_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let mut tv: TinyVec<[i32; 8]> = TinyVec::new();
        tv.push(1);
        tv.push(2);
        tv.push(3);

        let sexp = tv.clone().into_sexp();
        let back: TinyVec<[i32; 8]> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back[0], 1);
        assert_eq!(back[1], 2);
        assert_eq!(back[2], 3);
    });
}

#[cfg(feature = "tinyvec")]
#[test]
fn tinyvec_f64_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let mut tv: TinyVec<[f64; 4]> = TinyVec::new();
        tv.push(1.5);
        tv.push(2.5);

        let sexp = tv.clone().into_sexp();
        let back: TinyVec<[f64; 4]> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 2);
        assert_eq!(back[0], 1.5);
        assert_eq!(back[1], 2.5);
    });
}

#[cfg(feature = "tinyvec")]
#[test]
fn tinyvec_u8_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let mut tv: TinyVec<[u8; 16]> = TinyVec::new();
        tv.push(10);
        tv.push(20);
        tv.push(30);

        let sexp = tv.clone().into_sexp();
        let back: TinyVec<[u8; 16]> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back[0], 10);
        assert_eq!(back[1], 20);
        assert_eq!(back[2], 30);
    });
}

#[cfg(feature = "tinyvec")]
#[test]
fn tinyvec_stays_inline() {
    r_test_utils::with_r_thread(|| {
        // Small vector should stay inline (no heap allocation)
        let mut tv: TinyVec<[i32; 8]> = TinyVec::new();
        tv.push(1);
        tv.push(2);
        tv.push(3);

        assert!(tv.is_inline());

        let sexp = tv.into_sexp();
        let back: TinyVec<[i32; 8]> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert!(back.is_inline());
    });
}

#[cfg(feature = "tinyvec")]
#[test]
fn tinyvec_spills_to_heap() {
    r_test_utils::with_r_thread(|| {
        // Large vector should spill to heap
        let mut tv: TinyVec<[i32; 4]> = TinyVec::new();
        for i in 0..10 {
            tv.push(i);
        }

        assert!(!tv.is_inline()); // Should be on heap now

        let sexp = tv.into_sexp();
        let back: TinyVec<[i32; 4]> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 10);
        assert!(!back.is_inline()); // Should still be on heap
    });
}

#[cfg(feature = "tinyvec")]
#[test]
fn arrayvec_capacity_ok() {
    r_test_utils::with_r_thread(|| {
        let mut av: ArrayVec<[i32; 5]> = ArrayVec::new();
        av.push(1);
        av.push(2);
        av.push(3);

        let sexp = av.into_sexp();
        let back: ArrayVec<[i32; 5]> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back[0], 1);
    });
}

#[cfg(feature = "tinyvec")]
#[test]
fn arrayvec_capacity_error() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, INTEGER, SEXPTYPE};

        // Create R vector with 10 elements
        let sexp = unsafe {
            let s = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 10));
            let ptr = INTEGER(s);
            for i in 0..10 {
                *ptr.add(i) = i as i32;
            }
            Rf_unprotect(1);
            s
        };

        // Try to convert to ArrayVec<[i32; 5]> (capacity = 5)
        let result: Result<ArrayVec<[i32; 5]>, _> = TryFromSexp::try_from_sexp(sexp);

        // Should fail with capacity error
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("exceeds") || msg.contains("capacity"));
    });
}

#[cfg(feature = "tinyvec")]
#[test]
fn tinyvec_empty() {
    r_test_utils::with_r_thread(|| {
        let tv: TinyVec<[i32; 4]> = TinyVec::new();
        let sexp = tv.into_sexp();
        let back: TinyVec<[i32; 4]> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_empty());
    });
}

#[cfg(feature = "tinyvec")]
use miniextendr_api::coerce::Coerce;

#[cfg(feature = "tinyvec")]
#[test]
fn tinyvec_coerce_i8_to_i32() {
    // Test element-wise coercion for TinyVec
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
fn arrayvec_coerce_u8_to_i32() {
    // Test element-wise coercion for ArrayVec
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
