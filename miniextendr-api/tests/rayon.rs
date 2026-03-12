//! Integration tests for rayon bridge functionality.
//!
//! Tests `with_r_vec`, `with_r_matrix`, `with_r_array` chunk-based APIs
//! with an embedded R runtime.
//!
//! All tests run in a single test function to avoid R state isolation issues
//! between test threads (similar to allocator.rs).

#![cfg(feature = "rayon")]

mod r_test_utils;

use miniextendr_api::ffi::{REAL, Rf_xlength, SEXP};
use miniextendr_api::rayon_bridge::{
    new_r_array, new_r_matrix, par_map, par_map2, par_map3, with_r_array, with_r_matrix,
    with_r_vec, with_r_vec_map,
};

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
        test_with_r_vec_map();
        test_par_map();
        test_par_map2();
        test_par_map3();
        test_par_map_empty();
        test_vec_parallel_collect();
        test_with_r_matrix_basic();
        test_with_r_matrix_parallel();
        test_with_r_array_basic();
        test_with_r_array_parallel();
        test_new_r_matrix();
        test_new_r_array();
    });
}

fn test_with_r_vec_basic() {
    let sexp = with_r_vec::<f64, _>(10, |chunk, offset| {
        for (i, v) in chunk.iter_mut().enumerate() {
            *v = (offset + i) as f64;
        }
    });

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(
        result,
        vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]
    );
}

fn test_with_r_vec_parallel_write() {
    let sexp = with_r_vec::<f64, _>(1000, |chunk, offset| {
        for (i, v) in chunk.iter_mut().enumerate() {
            let idx = offset + i;
            *v = (idx * idx) as f64;
        }
    });

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 1000);
    for (i, &v) in result.iter().enumerate() {
        assert_eq!(v, (i * i) as f64, "mismatch at index {}", i);
    }
}

fn test_with_r_vec_i32() {
    let sexp = with_r_vec::<i32, _>(100, |chunk, offset| {
        for (i, v) in chunk.iter_mut().enumerate() {
            *v = (offset + i) as i32 * 2;
        }
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 100);

    let ptr = unsafe { miniextendr_api::ffi::INTEGER(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    for (i, &v) in slice.iter().enumerate() {
        assert_eq!(v, i as i32 * 2, "mismatch at index {}", i);
    }
}

fn test_vec_parallel_collect() {
    use rayon::prelude::*;
    let result: Vec<f64> = (0..1000).into_par_iter().map(|i| i as f64 * 0.5).collect();

    assert_eq!(result.len(), 1000);
    for (i, &v) in result.iter().enumerate() {
        assert_eq!(v, i as f64 * 0.5, "mismatch at index {}", i);
    }
}

fn test_with_r_vec_empty() {
    let sexp = with_r_vec::<f64, _>(0, |_chunk, _offset| {
        panic!("should not be called for empty vec");
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 0);
}

fn test_with_r_vec_large() {
    const SIZE: usize = 100_000;
    let sexp = with_r_vec::<f64, _>(SIZE, |chunk, offset| {
        for (i, v) in chunk.iter_mut().enumerate() {
            *v = ((offset + i) % 1000) as f64;
        }
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

fn test_with_r_vec_map() {
    let sexp = with_r_vec_map(100, |i: usize| (i as f64).sqrt());

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 100);
    for (i, &v) in result.iter().enumerate() {
        assert_eq!(v, (i as f64).sqrt(), "mismatch at index {}", i);
    }
}

// region: par_map tests

fn test_par_map() {
    let input: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    let sexp = par_map(&input, |&v| v.sqrt());

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 1000);
    for (i, &v) in result.iter().enumerate() {
        assert_eq!(v, (i as f64).sqrt(), "mismatch at index {}", i);
    }
}

fn test_par_map2() {
    let a: Vec<f64> = (0..500).map(|i| i as f64).collect();
    let b: Vec<f64> = (0..500).map(|i| (i * 2) as f64).collect();
    let sexp = par_map2(&a, &b, |&x, &y| x + y);

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 500);
    for (i, &v) in result.iter().enumerate() {
        let expected = i as f64 + (i * 2) as f64;
        assert_eq!(v, expected, "mismatch at index {}", i);
    }
}

fn test_par_map3() {
    let a: Vec<f64> = (0..200).map(|i| i as f64).collect();
    let b: Vec<f64> = (0..200).map(|i| (i + 1) as f64).collect();
    let c: Vec<f64> = (0..200).map(|i| (i + 2) as f64).collect();
    // fused multiply-add: a * b + c
    let sexp = par_map3(&a, &b, &c, |&x, &y, &z| x * y + z);

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 200);
    for (i, &v) in result.iter().enumerate() {
        let expected = (i as f64) * ((i + 1) as f64) + (i + 2) as f64;
        assert_eq!(v, expected, "mismatch at index {}", i);
    }
}

fn test_par_map_empty() {
    let input: Vec<f64> = Vec::new();
    let sexp = par_map(&input, |&v: &f64| v.sqrt());
    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 0);

    // par_map2 with empty slices
    let a: Vec<f64> = Vec::new();
    let b: Vec<f64> = Vec::new();
    let sexp = par_map2(&a, &b, |&x: &f64, &y: &f64| x + y);
    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 0);
}
// endregion

// region: Matrix tests

fn test_with_r_matrix_basic() {
    // Create a 3x4 matrix (12 elements total)
    // Closure receives one column at a time
    let sexp = with_r_matrix::<f64, _>(3, 4, |col, col_idx| {
        assert_eq!(col.len(), 3);
        for (row, val) in col.iter_mut().enumerate() {
            *val = (row * 10 + col_idx) as f64;
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
    // Create a larger matrix, fill by column
    let nrow = 100;
    let ncol = 50;
    let sexp = with_r_matrix::<f64, _>(nrow, ncol, |col, col_idx| {
        assert_eq!(col.len(), nrow);
        for (row, slot) in col.iter_mut().enumerate() {
            *slot = (row + col_idx * 1000) as f64;
        }
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
// endregion

// region: Array tests

fn test_with_r_array_basic() {
    // Create a 2x3x4 array (4 slabs of 6 elements each)
    let sexp = with_r_array::<f64, 3, _>([2, 3, 4], |slab, slab_idx| {
        assert_eq!(slab.len(), 6); // 2*3 = 6 per slab
        for (i, v) in slab.iter_mut().enumerate() {
            *v = (slab_idx * 6 + i) as f64;
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
    let slab_size = 10 * 20; // = 200
    let total: usize = dims.iter().product();

    let sexp = with_r_array::<f64, 3, _>(dims, |slab, slab_idx| {
        for (i, v) in slab.iter_mut().enumerate() {
            *v = (slab_idx * slab_size + i) as f64 * 2.0;
        }
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
// endregion

// region: Typed wrapper tests (new_r_matrix, new_r_array)

fn test_new_r_matrix() {
    let matrix = new_r_matrix::<f64, _>(5, 3, |col, col_idx| {
        for (row, slot) in col.iter_mut().enumerate() {
            *slot = (col_idx * 5 + row) as f64 * 0.1;
        }
    });

    // RMatrix provides typed access (unsafe because it calls R APIs)
    unsafe {
        assert_eq!(matrix.nrow(), 5);
        assert_eq!(matrix.ncol(), 3);
    }
    assert_eq!(matrix.len(), 15);
}

fn test_new_r_array() {
    let array = new_r_array::<f64, 3, _>([4, 5, 6], |slab, slab_idx| {
        for (i, slot) in slab.iter_mut().enumerate() {
            *slot = (slab_idx * 20 + i) as f64;
        }
    });

    // RArray provides typed access (unsafe because it calls R APIs)
    unsafe {
        assert_eq!(array.dims(), [4, 5, 6]);
    }
    assert_eq!(array.len(), 120);
}
// endregion
