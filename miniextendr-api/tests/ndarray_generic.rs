mod r_test_utils;

#[cfg(feature = "ndarray")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "ndarray")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "ndarray")]
use miniextendr_api::{Array0, Array1, Array2, Array3, ArrayD};

#[cfg(feature = "ndarray")]
#[test]
fn array1_i32_blanket_impl() {
    // Verify blanket impl works for i32
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 4));
            let slice: &mut [i32] = sexp.as_mut_slice();
            slice[0] = 1;
            slice[1] = 2;
            slice[2] = 3;
            slice[3] = 4;

            let arr: Array1<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], 1);
            assert_eq!(arr[1], 2);
            assert_eq!(arr[2], 3);
            assert_eq!(arr[3], 4);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array1_u8_blanket_impl() {
    // Verify blanket impl works for u8 (raw)
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::RAWSXP, 3));
            let slice: &mut [u8] = sexp.as_mut_slice();
            slice[0] = 10;
            slice[1] = 20;
            slice[2] = 30;

            let arr: Array1<u8> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], 10);
            assert_eq!(arr[1], 20);
            assert_eq!(arr[2], 30);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array2_i32_blanket_impl() {
    // Verify blanket impl works for i32 matrices
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 2x3 matrix
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::INTSXP, 2, 3));
            let slice: &mut [i32] = sexp.as_mut_slice();
            // Column-major layout
            for (i, slot) in slice.iter_mut().enumerate() {
                *slot = i as i32 + 1;
            }

            let arr: Array2<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(arr.nrows(), 2);
            assert_eq!(arr.ncols(), 3);
            // Verify column-major layout preserved
            assert_eq!(arr[[0, 0]], 1);
            assert_eq!(arr[[1, 0]], 2);
            assert_eq!(arr[[0, 1]], 3);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array3_i32_blanket_impl() {
    // Verify blanket impl works for 3D arrays
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 2x3x2 = 12 element 3D array
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 12));
            let data: &mut [i32] = sexp.as_mut_slice();
            for (i, slot) in data.iter_mut().enumerate() {
                *slot = i as i32 + 1;
            }

            // Set dimensions
            let dims = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 3));
            let dims_data: &mut [i32] = dims.as_mut_slice();
            dims_data[0] = 2;
            dims_data[1] = 3;
            dims_data[2] = 2;
            sexp.set_dim(dims);

            let arr: Array3<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(arr.shape(), &[2, 3, 2]);
            // Column-major: arr[[row, col, slice]]
            assert_eq!(arr[[0, 0, 0]], 1);
            assert_eq!(arr[[1, 0, 0]], 2);
            assert_eq!(arr[[0, 1, 0]], 3);

            Rf_unprotect(2);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array0_scalar_blanket_impl() {
    // Verify Array0 (scalar) blanket impl works
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::SEXP;

        let sexp = SEXP::scalar_integer(42);
        let arr: Array0<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(arr[()], 42);
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array1_f64_roundtrip() {
    r_test_utils::with_r_thread(|| {
        use ndarray::Array1;

        let arr = Array1::from_vec(vec![1.5, 2.5, 3.5]);
        let sexp = arr.clone().into_sexp();
        let back: Array1<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back[0], 1.5);
        assert_eq!(back[1], 2.5);
        assert_eq!(back[2], 3.5);
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn arrayd_dynamic_dims() {
    // Test ArrayD with dynamic number of dimensions
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 3D array: 2x3x2
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 12));
            let data: &mut [i32] = sexp.as_mut_slice();
            for (i, slot) in data.iter_mut().enumerate() {
                *slot = i as i32;
            }

            // Set dimensions
            let dims = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 3));
            let dims_data: &mut [i32] = dims.as_mut_slice();
            dims_data[0] = 2; // dim[0]
            dims_data[1] = 3; // dim[1]
            dims_data[2] = 2; // dim[2]

            sexp.set_dim(dims);

            let arr: ArrayD<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(arr.ndim(), 3);
            assert_eq!(arr.shape(), &[2, 3, 2]);
            assert_eq!(arr[[0, 0, 0]], 0);
            assert_eq!(arr[[1, 0, 0]], 1);

            Rf_unprotect(2);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array_blanket_coverage_all_rnative_types() {
    // Verify blanket impl works for all RNativeType: i32, f64, u8, RLogical
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{RLogical, Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE};
        use ndarray::Array1;

        unsafe {
            // i32
            let sexp_int = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 2));
            let arr_i32: Array1<i32> = TryFromSexp::try_from_sexp(sexp_int).unwrap();
            assert_eq!(arr_i32.len(), 2);

            // f64
            let sexp_real = Rf_protect(Rf_allocVector(SEXPTYPE::REALSXP, 2));
            let arr_f64: Array1<f64> = TryFromSexp::try_from_sexp(sexp_real).unwrap();
            assert_eq!(arr_f64.len(), 2);

            // u8
            let sexp_raw = Rf_protect(Rf_allocVector(SEXPTYPE::RAWSXP, 2));
            let arr_u8: Array1<u8> = TryFromSexp::try_from_sexp(sexp_raw).unwrap();
            assert_eq!(arr_u8.len(), 2);

            // RLogical
            let sexp_lgl = Rf_protect(Rf_allocVector(SEXPTYPE::LGLSXP, 2));
            let arr_lgl: Array1<RLogical> = TryFromSexp::try_from_sexp(sexp_lgl).unwrap();
            assert_eq!(arr_lgl.len(), 2);

            Rf_unprotect(4);
        }
    });
}
