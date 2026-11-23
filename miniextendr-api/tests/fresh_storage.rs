//! A freshly allocated R vector handed to a caller's closure starts as `NA`.
//!
//! `Rf_allocVector` leaves a numeric payload uninitialized, and a closure may
//! read its slice before it writes it, so every API that hands out a fresh
//! payload as `&mut [T]` gives each element a value first.

mod r_test_utils;

use miniextendr_api::expression::r_eval_str_global;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{IntoR, RArray, RMatrix, RNativeType, SexpExt};

#[test]
fn rarray_new_starts_as_na_and_keeps_its_dims_under_gctorture() {
    r_test_utils::with_r_thread(|| unsafe {
        r_eval_str_global("gctorture(TRUE)").unwrap();
        let mut seen = Vec::new();
        let matrix = RMatrix::<f64>::new([3, 4], |slice| {
            seen.extend(slice.iter().map(|v| v.to_bits()));
            slice[5] = 1.5;
        });
        let matrix = OwnedProtect::new(matrix.as_sexp());
        let array = RArray::<i32, 3>::new([2, 3, 4], |_| {});
        let array = OwnedProtect::new(array.as_sexp());
        r_eval_str_global("gctorture(FALSE)").unwrap();

        assert_eq!(seen, vec![f64::R_NA.to_bits(); 12]);
        let matrix = RMatrix::<f64>::from_sexp(matrix.get()).unwrap();
        assert_eq!(matrix.dims(), [3, 4]);
        let values = matrix.as_slice();
        assert_eq!(values[5], 1.5);
        assert!(
            values
                .iter()
                .enumerate()
                .all(|(i, v)| i == 5 || v.to_bits() == f64::R_NA.to_bits())
        );

        let array = RArray::<i32, 3>::from_sexp(array.get()).unwrap();
        assert_eq!(array.dims(), [2, 3, 4]);
        assert_eq!(array.as_slice(), &[i32::R_NA; 24]);
    });
}

#[test]
fn rarray_zeros_writes_zeros() {
    r_test_utils::with_r_thread(|| unsafe {
        let raw = RArray::<u8, 2>::zeros([2, 2]);
        assert_eq!(raw.as_slice(), &[0u8; 4]);
        let real = RArray::<f64, 1>::zeros([3]);
        assert_eq!(real.as_slice(), &[0.0; 3]);
    });
}

#[test]
fn a_builder_column_the_closure_leaves_alone_is_na() {
    r_test_utils::with_r_thread(|| unsafe {
        let frame = miniextendr_api::DataFrame::builder(5)
            .column::<f64>("x", |_, _| {})
            .column::<i32>("y", |chunk, offset| {
                for (i, v) in chunk.iter_mut().enumerate() {
                    assert_eq!(*v, i32::R_NA, "a fresh chunk starts as NA");
                    if (offset + i) % 2 == 0 {
                        *v = i32::try_from(offset + i).unwrap();
                    }
                }
            })
            .build();
        let frame = OwnedProtect::new(frame.into_sexp());
        let x = frame.get().vector_elt(0);
        assert!(
            x.as_slice::<f64>()
                .iter()
                .all(|v| v.to_bits() == f64::R_NA.to_bits())
        );
        let y = frame.get().vector_elt(1);
        assert_eq!(y.as_slice::<i32>(), &[0, i32::R_NA, 2, i32::R_NA, 4]);
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn ndarray_owned_vectors_start_as_na() {
    use miniextendr_api::optionals::{RndMat, RndVec};
    r_test_utils::with_r_thread(|| unsafe {
        let v = RndVec::<i32>::new(4, |slice| assert_eq!(slice, &[i32::R_NA; 4]));
        assert_eq!(v.view().to_vec(), vec![i32::R_NA; 4]);
        let m = RndMat::<f64>::new(2, 3, |slice| slice[0] = 7.0);
        let values: Vec<u64> = m.view().t().iter().map(|v| v.to_bits()).collect();
        assert_eq!(values[0], 7.0f64.to_bits());
        assert!(values[1..].iter().all(|&b| b == f64::R_NA.to_bits()));
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn nalgebra_storage_starts_as_na() {
    use miniextendr_api::optionals::RVecStorage;
    use nalgebra::{Dyn, U1};
    r_test_utils::with_r_thread(|| unsafe {
        let v = RVecStorage::<u8, Dyn, U1>::new_vector(3, |slice| assert_eq!(slice, &[0u8; 3]));
        assert_eq!(v.as_sexp().as_slice::<u8>(), &[0u8; 3]);
        let m = RVecStorage::<i32, Dyn, Dyn>::new_matrix(2, 2, |slice| slice[3] = 9);
        assert_eq!(
            m.as_sexp().as_slice::<i32>(),
            &[i32::R_NA, i32::R_NA, i32::R_NA, 9]
        );
    });
}

#[cfg(feature = "arrow")]
#[test]
fn an_r_backed_arrow_buffer_starts_as_na() {
    use miniextendr_api::optionals::arrow_impl::alloc_r_backed_buffer;
    r_test_utils::with_r_thread(|| unsafe {
        let (buffer, sexp) = alloc_r_backed_buffer::<i32>(3);
        let _root = OwnedProtect::new(sexp);
        assert_eq!(buffer.typed_data::<i32>(), &[i32::R_NA; 3]);
        assert_eq!(sexp.as_slice::<i32>(), &[i32::R_NA; 3]);
    });
}
