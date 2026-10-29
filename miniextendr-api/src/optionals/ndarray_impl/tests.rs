use super::*;

#[test]
fn array1_can_be_created() {
    let arr = Array1::from_vec(vec![1.0, 2.0, 3.0]);
    assert_eq!(arr.len(), 3);
}

#[test]
fn array2_can_be_created() {
    let arr = Array2::from_shape_vec((2, 3), vec![1, 2, 3, 4, 5, 6]).unwrap();
    assert_eq!(arr.dim(), (2, 3));
}

#[test]
fn array3_can_be_created() {
    // 2x3x4 = 24 elements
    let data: Vec<f64> = (0..24).map(|x| x as f64).collect();
    let arr = Array3::from_shape_vec((2, 3, 4), data).unwrap();
    assert_eq!(arr.dim(), (2, 3, 4));
    assert_eq!(arr[[0, 0, 0]], 0.0);
    assert_eq!(arr[[1, 2, 3]], 23.0);
}

#[test]
fn array3_fortran_order() {
    // Test that Fortran order works correctly
    let data: Vec<i32> = (0..24).collect();
    let arr = Array3::from_shape_vec((2, 3, 4).f(), data).unwrap();
    assert_eq!(arr.dim(), (2, 3, 4));
    // In Fortran order, first dimension varies fastest
    // So arr[0,0,0]=0, arr[1,0,0]=1, arr[0,1,0]=2, ...
    assert_eq!(arr[[0, 0, 0]], 0);
    assert_eq!(arr[[1, 0, 0]], 1);
    assert_eq!(arr[[0, 1, 0]], 2);
    assert_eq!(arr[[1, 1, 0]], 3);
}

#[test]
fn arrayd_can_be_created() {
    // Create a 4D array: 2x3x4x5 = 120 elements
    let data: Vec<f64> = (0..120).map(|x| x as f64).collect();
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3, 4, 5]), data).unwrap();
    assert_eq!(arr.shape(), &[2, 3, 4, 5]);
    assert_eq!(arr.ndim(), 4);
}

#[test]
fn arrayd_fortran_order() {
    // Test dynamic dimension array in Fortran order
    let data: Vec<i32> = (0..12).collect();
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 2, 3]).f(), data).unwrap();
    assert_eq!(arr.shape(), &[2, 2, 3]);
    // In Fortran order, first dimension varies fastest
    assert_eq!(arr[IxDyn(&[0, 0, 0])], 0);
    assert_eq!(arr[IxDyn(&[1, 0, 0])], 1);
    assert_eq!(arr[IxDyn(&[0, 1, 0])], 2);
}

#[test]
fn fortran_order_iter_generates_correct_indices() {
    let mut indices = Vec::new();
    fortran_order_iter(&[2, 3], |idx| {
        indices.push(idx);
    });
    // Fortran order: first index varies fastest
    assert_eq!(
        indices,
        vec![
            vec![0, 0],
            vec![1, 0],
            vec![0, 1],
            vec![1, 1],
            vec![0, 2],
            vec![1, 2],
        ]
    );
}

#[test]
fn fortran_order_iter_3d() {
    let mut indices = Vec::new();
    fortran_order_iter(&[2, 2, 2], |idx| {
        indices.push(idx);
    });
    // Should be: [0,0,0], [1,0,0], [0,1,0], [1,1,0], [0,0,1], [1,0,1], [0,1,1], [1,1,1]
    assert_eq!(indices.len(), 8);
    assert_eq!(indices[0], vec![0, 0, 0]);
    assert_eq!(indices[1], vec![1, 0, 0]);
    assert_eq!(indices[4], vec![0, 0, 1]);
    assert_eq!(indices[7], vec![1, 1, 1]);
}

// RNdArrayOps tests
#[test]
fn rndarrayops_array1_basic() {
    let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(RNdArrayOps::len(&arr), 5);
    assert!(!RNdArrayOps::is_empty(&arr));
    assert_eq!(RNdArrayOps::ndim(&arr), 1);
    assert_eq!(RNdArrayOps::shape(&arr), vec![5]);
}

#[test]
fn rndarrayops_array1_stats() {
    let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert!((RNdArrayOps::sum(&arr) - 15.0).abs() < 1e-10);
    assert!((RNdArrayOps::mean(&arr) - 3.0).abs() < 1e-10);
    assert!((RNdArrayOps::min(&arr) - 1.0).abs() < 1e-10);
    assert!((RNdArrayOps::max(&arr) - 5.0).abs() < 1e-10);
    assert!((RNdArrayOps::product(&arr) - 120.0).abs() < 1e-10);
}

#[test]
fn rndarrayops_array1_var_std() {
    let arr = Array1::from_vec(vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    // Mean = 5.0, variance = 4.0, std = 2.0
    assert!((RNdArrayOps::mean(&arr) - 5.0).abs() < 1e-10);
    assert!((RNdArrayOps::var(&arr) - 4.0).abs() < 1e-10);
    assert!((RNdArrayOps::std(&arr) - 2.0).abs() < 1e-10);
}

#[test]
fn rndarrayops_array2_basic() {
    let arr = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
    assert_eq!(RNdArrayOps::len(&arr), 6);
    assert_eq!(RNdArrayOps::ndim(&arr), 2);
    assert_eq!(RNdArrayOps::shape(&arr), vec![2, 3]);
    assert!((RNdArrayOps::sum(&arr) - 21.0).abs() < 1e-10);
    assert!((RNdArrayOps::mean(&arr) - 3.5).abs() < 1e-10);
}

#[test]
fn rndarrayops_arrayd_basic() {
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 2, 2]), vec![1.0; 8]).unwrap();
    assert_eq!(RNdArrayOps::len(&arr), 8);
    assert_eq!(RNdArrayOps::ndim(&arr), 3);
    assert_eq!(RNdArrayOps::shape(&arr), vec![2, 2, 2]);
    assert!((RNdArrayOps::sum(&arr) - 8.0).abs() < 1e-10);
    assert!((RNdArrayOps::mean(&arr) - 1.0).abs() < 1e-10);
}

#[test]
fn rndarrayops_empty() {
    let arr: Array1<f64> = Array1::from_vec(vec![]);
    assert_eq!(RNdArrayOps::len(&arr), 0);
    assert!(RNdArrayOps::is_empty(&arr));
    assert!(RNdArrayOps::mean(&arr).is_nan());
    assert!(RNdArrayOps::var(&arr).is_nan());
}

// RNdSlice tests
#[test]
fn rndslice_get() {
    let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(RNdSlice::get(&arr, 0), Some(1.0));
    assert_eq!(RNdSlice::get(&arr, 2), Some(3.0));
    assert_eq!(RNdSlice::get(&arr, 4), Some(5.0));
    assert_eq!(RNdSlice::get(&arr, 5), None);
    assert_eq!(RNdSlice::get(&arr, -1), None);
}

#[test]
fn rndslice_first_last() {
    let arr = Array1::from_vec(vec![10.0, 20.0, 30.0]);
    assert_eq!(RNdSlice::first(&arr), Some(10.0));
    assert_eq!(RNdSlice::last(&arr), Some(30.0));

    let empty: Array1<f64> = Array1::from_vec(vec![]);
    assert_eq!(RNdSlice::first(&empty), None);
    assert_eq!(RNdSlice::last(&empty), None);
}

#[test]
fn rndslice_slice_1d() {
    let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(RNdSlice::slice_1d(&arr, 1, 4), vec![2.0, 3.0, 4.0]);
    assert_eq!(RNdSlice::slice_1d(&arr, 0, 2), vec![1.0, 2.0]);
    assert_eq!(RNdSlice::slice_1d(&arr, 3, 10), vec![4.0, 5.0]); // Clamped
    assert_eq!(RNdSlice::slice_1d(&arr, -5, 2), vec![1.0, 2.0]); // Negative start clamped
    assert_eq!(RNdSlice::slice_1d(&arr, 3, 2), Vec::<f64>::new()); // Empty range
}

#[test]
fn rndslice_get_many() {
    let arr = Array1::from_vec(vec![10, 20, 30, 40, 50]);
    let results = RNdSlice::get_many(&arr, vec![0, 2, 4, 10]);
    assert_eq!(results, vec![Some(10), Some(30), Some(50), None]);
}

#[test]
fn rndslice_is_valid_index() {
    let arr = Array1::from_vec(vec![1.0, 2.0, 3.0]);
    assert!(RNdSlice::is_valid_index(&arr, 0));
    assert!(RNdSlice::is_valid_index(&arr, 2));
    assert!(!RNdSlice::is_valid_index(&arr, 3));
    assert!(!RNdSlice::is_valid_index(&arr, -1));
}

// RNdSlice2D tests
#[test]
fn rndslice2d_get_2d() {
    let arr = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
    assert_eq!(RNdSlice2D::get_2d(&arr, 0, 0), Some(1.0));
    assert_eq!(RNdSlice2D::get_2d(&arr, 0, 2), Some(3.0));
    assert_eq!(RNdSlice2D::get_2d(&arr, 1, 1), Some(5.0));
    assert_eq!(RNdSlice2D::get_2d(&arr, 2, 0), None);
    assert_eq!(RNdSlice2D::get_2d(&arr, -1, 0), None);
}

#[test]
fn rndslice2d_row_col() {
    let arr = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
    assert_eq!(RNdSlice2D::row(&arr, 0), vec![1.0, 2.0, 3.0]);
    assert_eq!(RNdSlice2D::row(&arr, 1), vec![4.0, 5.0, 6.0]);
    assert_eq!(RNdSlice2D::row(&arr, 2), Vec::<f64>::new()); // Out of bounds
    assert_eq!(RNdSlice2D::col(&arr, 0), vec![1.0, 4.0]);
    assert_eq!(RNdSlice2D::col(&arr, 1), vec![2.0, 5.0]);
    assert_eq!(RNdSlice2D::col(&arr, 3), Vec::<f64>::new()); // Out of bounds
}

#[test]
fn rndslice2d_diag() {
    let arr =
        Array2::from_shape_vec((3, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]).unwrap();
    assert_eq!(RNdSlice2D::diag(&arr), vec![1.0, 5.0, 9.0]);
}

#[test]
fn rndslice2d_nrows_ncols() {
    let arr = Array2::from_shape_vec((2, 4), vec![1.0; 8]).unwrap();
    assert_eq!(RNdSlice2D::nrows(&arr), 2);
    assert_eq!(RNdSlice2D::ncols(&arr), 4);
}

// RNdIndex tests
#[test]
fn rndindex_get_nd() {
    // 2x3x2 array with values 1..12
    let data: Vec<f64> = (1..=12).map(|x| x as f64).collect();
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3, 2]), data).unwrap();

    // Test valid indices
    assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0, 0]), Some(1.0));
    assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0, 1]), Some(2.0));
    assert_eq!(RNdIndex::get_nd(&arr, vec![1, 2, 1]), Some(12.0));

    // Test out of bounds
    assert_eq!(RNdIndex::get_nd(&arr, vec![2, 0, 0]), None);
    assert_eq!(RNdIndex::get_nd(&arr, vec![0, 3, 0]), None);

    // Test wrong number of indices
    assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0]), None);
    assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0, 0, 0]), None);

    // Test negative indices
    assert_eq!(RNdIndex::get_nd(&arr, vec![-1, 0, 0]), None);
}

#[test]
fn rndindex_shape_ndim_len() {
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3, 4]), vec![0.0; 24]).unwrap();

    assert_eq!(RNdIndex::shape_nd(&arr), vec![2, 3, 4]);
    assert_eq!(RNdIndex::ndim(&arr), 3);
    assert_eq!(RNdIndex::len_nd(&arr), 24);
}

#[test]
fn rndindex_slice_nd() {
    // 3x3 array
    let data: Vec<f64> = (1..=9).map(|x| x as f64).collect();
    let arr = ArrayD::from_shape_vec(IxDyn(&[3, 3]), data).unwrap();

    // Slice [0:2, 0:2] - 2x2 subarray
    let slice = RNdIndex::slice_nd(&arr, vec![0, 0], vec![2, 2]);
    assert!(slice.is_some());
    let slice = slice.unwrap();
    // In Fortran order (column-major): [0,0]=1, [1,0]=4, [0,1]=2, [1,1]=5
    assert_eq!(slice, vec![1.0, 4.0, 2.0, 5.0]);

    // Invalid slice (wrong dimensions)
    assert_eq!(RNdIndex::slice_nd(&arr, vec![0], vec![2, 2]), None);

    // Empty slice (start >= end)
    let empty = RNdIndex::slice_nd(&arr, vec![2, 2], vec![1, 1]);
    assert!(empty.is_some());
    assert!(empty.unwrap().is_empty());
}

#[test]
fn rndindex_flatten() {
    // 2x3 array: [[1,2,3], [4,5,6]]
    let data: Vec<f64> = (1..=6).map(|x| x as f64).collect();
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3]), data).unwrap();

    // Fortran order (column-major, R-compatible)
    let f_order = RNdIndex::flatten(&arr);
    assert_eq!(f_order, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);

    // C order (row-major)
    let c_order = RNdIndex::flatten_c(&arr);
    assert_eq!(c_order, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn rndindex_is_valid_nd() {
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3]), vec![0.0; 6]).unwrap();

    assert!(RNdIndex::is_valid_nd(&arr, vec![0, 0]));
    assert!(RNdIndex::is_valid_nd(&arr, vec![1, 2]));
    assert!(!RNdIndex::is_valid_nd(&arr, vec![2, 0]));
    assert!(!RNdIndex::is_valid_nd(&arr, vec![0, 3]));
    assert!(!RNdIndex::is_valid_nd(&arr, vec![0])); // Wrong dim count
    assert!(!RNdIndex::is_valid_nd(&arr, vec![-1, 0])); // Negative
}

#[test]
fn rndindex_axis_slice() {
    // 2x3 array: [[1,2,3], [4,5,6]]
    let data: Vec<f64> = (1..=6).map(|x| x as f64).collect();
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3]), data).unwrap();

    // Get row 0 (axis 0, index 0)
    let row0 = RNdIndex::axis_slice(&arr, 0, 0);
    assert_eq!(row0, vec![1.0, 2.0, 3.0]);

    // Get row 1 (axis 0, index 1)
    let row1 = RNdIndex::axis_slice(&arr, 0, 1);
    assert_eq!(row1, vec![4.0, 5.0, 6.0]);

    // Get col 1 (axis 1, index 1)
    let col1 = RNdIndex::axis_slice(&arr, 1, 1);
    assert_eq!(col1, vec![2.0, 5.0]);

    // Invalid axis
    let invalid = RNdIndex::axis_slice(&arr, 2, 0);
    assert!(invalid.is_empty());

    // Invalid index
    let invalid = RNdIndex::axis_slice(&arr, 0, 5);
    assert!(invalid.is_empty());
}

#[test]
fn rndindex_reshape() {
    // 2x3 array flattened and reshaped to 3x2
    let data: Vec<f64> = (1..=6).map(|x| x as f64).collect();
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3]), data).unwrap();

    // Valid reshape
    let reshaped = RNdIndex::reshape(&arr, vec![3, 2]);
    assert!(reshaped.is_some());
    assert_eq!(reshaped.unwrap().len(), 6);

    // Reshape to 1D
    let flat = RNdIndex::reshape(&arr, vec![6]);
    assert!(flat.is_some());
    assert_eq!(flat.unwrap().len(), 6);

    // Invalid reshape (wrong total size)
    let invalid = RNdIndex::reshape(&arr, vec![2, 2]);
    assert!(invalid.is_none());

    // Invalid reshape (zero dimension)
    let invalid = RNdIndex::reshape(&arr, vec![0, 6]);
    assert!(invalid.is_none());

    // Invalid reshape (negative dimension)
    let invalid = RNdIndex::reshape(&arr, vec![-1, 6]);
    assert!(invalid.is_none());
}

#[test]
fn rndindex_i32_impl() {
    // Test i32 implementation
    let arr = ArrayD::from_shape_vec(IxDyn(&[2, 2]), vec![1i32, 2, 3, 4]).unwrap();

    assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0]), Some(1));
    assert_eq!(RNdIndex::get_nd(&arr, vec![1, 1]), Some(4));
    assert_eq!(RNdIndex::shape_nd(&arr), vec![2, 2]);
    assert_eq!(RNdIndex::ndim(&arr), 2);
    assert_eq!(RNdIndex::len_nd(&arr), 4);
}

mod altrep_no_na_tests;
