//! Integration tests for rayon bridge functionality.
//!
//! Tests `with_r_vec` and parallel collection with an embedded R runtime.
//!
//! All tests run in a single test function to avoid R state isolation issues
//! between test threads (similar to allocator.rs).

#![cfg(feature = "rayon")]

mod r_test_utils;

use miniextendr_api::ffi::{REAL, Rf_xlength, SEXP};
use miniextendr_api::rayon_bridge::{
    RVec, new_r_array, new_r_matrix, with_r_array, with_r_array_slabs, with_r_matrix,
    with_r_matrix_cols, with_r_vec,
};
use rayon::prelude::*;

/// Helper to read f64 values from an R REALSXP vector.
unsafe fn read_real_vec(sexp: SEXP) -> Vec<f64> {
    unsafe {
        let len = Rf_xlength(sexp) as usize;
        if len == 0 {
            return Vec::new();
        }
        let ptr = REAL(sexp);
        std::slice::from_raw_parts(ptr, len).to_vec()
    }
}

#[test]
fn rayon_suite() {
    r_test_utils::with_r_thread(|| {
        test_with_r_vec_basic();
        test_with_r_vec_parallel_write();
        test_with_r_vec_i32();
        test_with_r_vec_empty();
        test_with_r_vec_large();
        test_rvec_parallel_collect();
        test_rvec_into_sexp();
        test_with_r_matrix_basic();
        test_with_r_matrix_parallel();
        test_with_r_array_basic();
        test_with_r_array_parallel();
        test_new_r_matrix();
        test_new_r_array();
        test_with_r_matrix_cols();
        test_with_r_array_slabs();
    });
}

fn test_with_r_vec_basic() {
    let sexp = with_r_vec::<f64, _>(10, |slice| {
        for (i, v) in slice.iter_mut().enumerate() {
            *v = i as f64;
        }
    });

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(
        result,
        vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]
    );
}

fn test_with_r_vec_parallel_write() {
    let sexp = with_r_vec::<f64, _>(1000, |slice| {
        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = (i * i) as f64;
        });
    });

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 1000);
    for (i, &v) in result.iter().enumerate() {
        assert_eq!(v, (i * i) as f64, "mismatch at index {}", i);
    }
}

fn test_with_r_vec_i32() {
    let sexp = with_r_vec::<i32, _>(100, |slice| {
        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = i as i32 * 2;
        });
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 100);

    let ptr = unsafe { miniextendr_api::ffi::INTEGER(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    for (i, &v) in slice.iter().enumerate() {
        assert_eq!(v, i as i32 * 2, "mismatch at index {}", i);
    }
}

fn test_rvec_parallel_collect() {
    let result: RVec<f64> = (0..1000).into_par_iter().map(|i| i as f64 * 0.5).collect();

    assert_eq!(result.len(), 1000);
    for (i, &v) in result.as_slice().iter().enumerate() {
        assert_eq!(v, i as f64 * 0.5, "mismatch at index {}", i);
    }
}

fn test_rvec_into_sexp() {
    let rvec: RVec<f64> = (0..100).into_par_iter().map(|i| i as f64).collect();
    let sexp = miniextendr_api::into_r::IntoR::into_sexp(rvec);

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 100);
    for (i, &v) in result.iter().enumerate() {
        assert_eq!(v, i as f64);
    }
}

fn test_with_r_vec_empty() {
    let sexp = with_r_vec::<f64, _>(0, |slice| {
        assert!(slice.is_empty());
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 0);
}

fn test_with_r_vec_large() {
    const SIZE: usize = 100_000;
    let sexp = with_r_vec::<f64, _>(SIZE, |slice| {
        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = (i % 1000) as f64;
        });
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, SIZE);

    // Spot check some values
    let ptr = unsafe { REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, SIZE) };
    assert_eq!(slice[0], 0.0);
    assert_eq!(slice[999], 999.0);
    assert_eq!(slice[1000], 0.0);
    assert_eq!(slice[50_000], 0.0);
}

// =============================================================================
// Matrix tests
// =============================================================================

fn test_with_r_matrix_basic() {
    // Create a 3x4 matrix (12 elements total)
    let sexp = with_r_matrix::<f64, _>(3, 4, |slice, nrow, ncol| {
        assert_eq!(nrow, 3);
        assert_eq!(ncol, 4);
        assert_eq!(slice.len(), 12);

        // Fill with row * 10 + col (column-major order)
        for col in 0..ncol {
            for row in 0..nrow {
                let idx = col * nrow + row;
                slice[idx] = (row * 10 + col) as f64;
            }
        }
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 12);

    // Verify the dim attribute
    let dim =
        unsafe { miniextendr_api::ffi::Rf_getAttrib(sexp, miniextendr_api::ffi::R_DimSymbol) };
    let dim_ptr = unsafe { miniextendr_api::ffi::INTEGER(dim) };
    let dim_slice = unsafe { std::slice::from_raw_parts(dim_ptr, 2) };
    assert_eq!(dim_slice[0], 3); // nrow
    assert_eq!(dim_slice[1], 4); // ncol

    // Verify values (column-major)
    let ptr = unsafe { REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, 12) };
    // Column 0: rows 0,1,2 -> values 0,10,20
    assert_eq!(slice[0], 0.0);
    assert_eq!(slice[1], 10.0);
    assert_eq!(slice[2], 20.0);
    // Column 1: rows 0,1,2 -> values 1,11,21
    assert_eq!(slice[3], 1.0);
    assert_eq!(slice[4], 11.0);
    assert_eq!(slice[5], 21.0);
}

fn test_with_r_matrix_parallel() {
    // Create a larger matrix with parallel fill
    let nrow = 100;
    let ncol = 50;
    let sexp = with_r_matrix::<f64, _>(nrow, ncol, |slice, nr, nc| {
        assert_eq!(nr, nrow);
        assert_eq!(nc, ncol);

        slice.par_iter_mut().enumerate().for_each(|(idx, v)| {
            let row = idx % nrow;
            let col = idx / nrow;
            *v = (row + col * 1000) as f64;
        });
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, nrow * ncol);

    // Spot check
    let ptr = unsafe { REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    // [0,0] = 0+0 = 0
    assert_eq!(slice[0], 0.0);
    // [1,0] = 1+0 = 1
    assert_eq!(slice[1], 1.0);
    // [0,1] = 0+1000 = 1000
    assert_eq!(slice[nrow], 1000.0);
}

// =============================================================================
// Array tests
// =============================================================================

fn test_with_r_array_basic() {
    // Create a 2x3x4 array (24 elements total)
    let sexp = with_r_array::<f64, 3, _>([2, 3, 4], |slice, dims| {
        assert_eq!(dims, [2, 3, 4]);
        assert_eq!(slice.len(), 24);

        // Fill with index value
        for (i, v) in slice.iter_mut().enumerate() {
            *v = i as f64;
        }
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 24);

    // Verify the dim attribute
    let dim =
        unsafe { miniextendr_api::ffi::Rf_getAttrib(sexp, miniextendr_api::ffi::R_DimSymbol) };
    let dim_ptr = unsafe { miniextendr_api::ffi::INTEGER(dim) };
    let dim_slice = unsafe { std::slice::from_raw_parts(dim_ptr, 3) };
    assert_eq!(dim_slice[0], 2);
    assert_eq!(dim_slice[1], 3);
    assert_eq!(dim_slice[2], 4);

    // Verify values
    let ptr = unsafe { REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, 24) };
    for (i, &v) in slice.iter().enumerate() {
        assert_eq!(v, i as f64);
    }
}

fn test_with_r_array_parallel() {
    // Create a 10x20x30 array with parallel fill
    let dims = [10, 20, 30];
    let total: usize = dims.iter().product();

    let sexp = with_r_array::<f64, 3, _>(dims, |slice, d| {
        assert_eq!(d, dims);

        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = (i * 2) as f64;
        });
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, total);

    // Spot check
    let ptr = unsafe { REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, total) };
    assert_eq!(slice[0], 0.0);
    assert_eq!(slice[1], 2.0);
    assert_eq!(slice[100], 200.0);
}

// =============================================================================
// Typed wrapper tests (new_r_matrix, new_r_array)
// =============================================================================

fn test_new_r_matrix() {
    let matrix = new_r_matrix::<f64, _>(5, 3, |slice, nrow, ncol| {
        assert_eq!(nrow, 5);
        assert_eq!(ncol, 3);
        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = i as f64 * 0.1;
        });
    });

    // RMatrix provides typed access (unsafe because it calls R APIs)
    unsafe {
        assert_eq!(matrix.nrow(), 5);
        assert_eq!(matrix.ncol(), 3);
    }
    assert_eq!(matrix.len(), 15);
}

fn test_new_r_array() {
    let array = new_r_array::<f64, 3, _>([4, 5, 6], |slice, dims| {
        assert_eq!(dims, [4, 5, 6]);
        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = i as f64;
        });
    });

    // RArray provides typed access (unsafe because it calls R APIs)
    unsafe {
        assert_eq!(array.dims(), [4, 5, 6]);
    }
    assert_eq!(array.len(), 120);
}

// =============================================================================
// Column/slab access tests
// =============================================================================

fn test_with_r_matrix_cols() {
    // Create a 3x4 matrix with column-wise parallel access
    let sexp = with_r_matrix_cols::<f64, _>(3, 4, |cols| {
        // Process columns in parallel
        cols.into_iter().enumerate().for_each(|(col_idx, col)| {
            assert_eq!(col.len(), 3, "column {} should have 3 rows", col_idx);
            for (row_idx, val) in col.iter_mut().enumerate() {
                // Value = row + col * 10 (e.g., [0,10,20,30], [1,11,21,31], [2,12,22,32])
                *val = (row_idx + col_idx * 10) as f64;
            }
        });
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 12);

    // Verify the dim attribute
    let dim =
        unsafe { miniextendr_api::ffi::Rf_getAttrib(sexp, miniextendr_api::ffi::R_DimSymbol) };
    let dim_ptr = unsafe { miniextendr_api::ffi::INTEGER(dim) };
    let dim_slice = unsafe { std::slice::from_raw_parts(dim_ptr, 2) };
    assert_eq!(dim_slice[0], 3); // nrow
    assert_eq!(dim_slice[1], 4); // ncol

    // Verify values (column-major)
    let ptr = unsafe { REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, 12) };
    // Column 0: rows 0,1,2 -> values 0,1,2
    assert_eq!(slice[0], 0.0);
    assert_eq!(slice[1], 1.0);
    assert_eq!(slice[2], 2.0);
    // Column 1: rows 0,1,2 -> values 10,11,12
    assert_eq!(slice[3], 10.0);
    assert_eq!(slice[4], 11.0);
    assert_eq!(slice[5], 12.0);
    // Column 3: rows 0,1,2 -> values 30,31,32
    assert_eq!(slice[9], 30.0);
    assert_eq!(slice[10], 31.0);
    assert_eq!(slice[11], 32.0);
}

fn test_with_r_array_slabs() {
    // Create a 2x3x4 array (4 slabs of 6 elements each)
    let sexp = with_r_array_slabs::<f64, 3, _>([2, 3, 4], |slabs, dims| {
        assert_eq!(dims, [2, 3, 4]);

        // Process slabs in parallel
        slabs.into_iter().enumerate().for_each(|(slab_idx, slab)| {
            // Each slab has 2*3=6 elements
            assert_eq!(slab.len(), 6, "slab {} should have 6 elements", slab_idx);
            for (i, val) in slab.iter_mut().enumerate() {
                // Value = slab_idx * 100 + i
                *val = (slab_idx * 100 + i) as f64;
            }
        });
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 24);

    // Verify the dim attribute
    let dim =
        unsafe { miniextendr_api::ffi::Rf_getAttrib(sexp, miniextendr_api::ffi::R_DimSymbol) };
    let dim_ptr = unsafe { miniextendr_api::ffi::INTEGER(dim) };
    let dim_slice = unsafe { std::slice::from_raw_parts(dim_ptr, 3) };
    assert_eq!(dim_slice[0], 2);
    assert_eq!(dim_slice[1], 3);
    assert_eq!(dim_slice[2], 4);

    // Verify values
    let ptr = unsafe { REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, 24) };
    // Slab 0: elements 0-5 -> values 0,1,2,3,4,5
    for (i, val) in slice.iter().take(6).enumerate() {
        assert_eq!(*val, i as f64, "slab 0, element {}", i);
    }
    // Slab 1: elements 6-11 -> values 100,101,102,103,104,105
    for (i, val) in slice[6..12].iter().enumerate() {
        assert_eq!(*val, (100 + i) as f64, "slab 1, element {}", i);
    }
    // Slab 3: elements 18-23 -> values 300,301,302,303,304,305
    for (i, val) in slice[18..24].iter().enumerate() {
        assert_eq!(*val, (300 + i) as f64, "slab 3, element {}", i);
    }
}
