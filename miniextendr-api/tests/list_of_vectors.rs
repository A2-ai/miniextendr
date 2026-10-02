//! Integration tests for the list-of-vectors `IntoR` shapes (`Vec<Vec<T>>`,
//! `Vec<&[T]>`, `Vec<Box<[T]>>`, `Vec<[T; N]>`, `Vec<Option<Vec<T>>>`,
//! `Vec<Option<&[T]>>`, `Option<Vec<T>>`) and their strict counterparts.
//!
//! Every row must land exactly as the matching flat `Vec<T>` would: same R
//! type, same smart INTSXP-or-REALSXP choice for the wide integers.

mod r_test_utils;

use miniextendr_api::altrep_traits::NA_INTEGER;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::{R_xlen_t, SEXPTYPE, strict};

/// `3_000_000_000` does not fit an R integer but is exact as a double.
const BIG: usize = 3_000_000_000;

#[test]
fn list_of_vectors_suite() {
    r_test_utils::with_r_thread(|| {
        test_vec_vec_usize_small_rows_are_integer();
        test_vec_vec_usize_widens_per_row();
        test_vec_vec_element_types_match_flat_vec();
        test_vec_vec_empty_rows_keep_their_type();
        test_vec_vec_unchecked_and_try_paths_agree();
        test_borrowed_boxed_and_array_rows();
        test_option_rows_and_option_vec();
        test_strict_vec_vec_usize_in_range();
        test_strict_vec_option_vec_in_range();
    });
}

/// Row `i` of a list, its R type and its length.
fn row(list: SEXP, i: isize) -> (SEXP, SEXPTYPE, R_xlen_t) {
    let elt = list.vector_elt(i);
    (elt, elt.type_of(), elt.xlength())
}

fn int_row(list: SEXP, i: isize) -> Vec<i32> {
    let (elt, ty, len) = row(list, i);
    assert_eq!(ty, SEXPTYPE::INTSXP, "row {i} should be integer");
    (0..len).map(|j| elt.integer_elt(j)).collect()
}

fn real_row(list: SEXP, i: isize) -> Vec<f64> {
    let (elt, ty, len) = row(list, i);
    assert_eq!(ty, SEXPTYPE::REALSXP, "row {i} should be double");
    (0..len).map(|j| elt.real_elt(j)).collect()
}

fn test_vec_vec_usize_small_rows_are_integer() {
    let scope = unsafe { ProtectScope::new() };
    let rows: Vec<Vec<usize>> = vec![vec![1, 5, 9], vec![], vec![42]];
    let list = unsafe { scope.protect_raw(rows.into_sexp()) };
    assert_eq!(list.type_of(), SEXPTYPE::VECSXP);
    assert_eq!(list.xlength(), 3);
    assert_eq!(int_row(list, 0), [1, 5, 9]);
    assert_eq!(int_row(list, 1), Vec::<i32>::new());
    assert_eq!(int_row(list, 2), [42]);
}

/// The INTSXP-or-REALSXP choice is made per row, as `Vec<usize>` makes it:
/// one row with a value past `i32::MAX` becomes double, the others stay integer.
fn test_vec_vec_usize_widens_per_row() {
    let scope = unsafe { ProtectScope::new() };
    let rows: Vec<Vec<usize>> = vec![vec![1, 2], vec![BIG, 3], vec![]];
    let list = unsafe { scope.protect_raw(rows.into_sexp()) };
    assert_eq!(int_row(list, 0), [1, 2]);
    assert_eq!(real_row(list, 1), [BIG as f64, 3.0]);
    assert_eq!(int_row(list, 2), Vec::<i32>::new());

    // Identical to the flat conversion of the same row.
    let flat = unsafe { scope.protect_raw(vec![BIG, 3usize].into_sexp()) };
    assert_eq!(flat.type_of(), SEXPTYPE::REALSXP);

    // `i64` / `u64` / `isize` / `u32` follow the same rule.
    let i64_rows = unsafe { scope.protect_raw(vec![vec![-1i64], vec![1i64 << 40]].into_sexp()) };
    assert_eq!(int_row(i64_rows, 0), [-1]);
    assert_eq!(real_row(i64_rows, 1), [(1i64 << 40) as f64]);
    let u64_rows = unsafe { scope.protect_raw(vec![vec![7u64], vec![u64::MAX]].into_sexp()) };
    assert_eq!(int_row(u64_rows, 0), [7]);
    assert_eq!(real_row(u64_rows, 1), [u64::MAX as f64]);
    let isize_rows = unsafe { scope.protect_raw(vec![vec![-3isize]].into_sexp()) };
    assert_eq!(int_row(isize_rows, 0), [-3]);
    let u32_rows = unsafe { scope.protect_raw(vec![vec![4u32], vec![u32::MAX]].into_sexp()) };
    assert_eq!(int_row(u32_rows, 0), [4]);
    assert_eq!(real_row(u32_rows, 1), [f64::from(u32::MAX)]);
}

/// Each element type keeps the R type its flat `Vec<T>` produces.
fn test_vec_vec_element_types_match_flat_vec() {
    let scope = unsafe { ProtectScope::new() };

    let ints = unsafe { scope.protect_raw(vec![vec![1i32, 2]].into_sexp()) };
    assert_eq!(int_row(ints, 0), [1, 2]);

    let doubles = unsafe { scope.protect_raw(vec![vec![0.5f64]].into_sexp()) };
    assert_eq!(real_row(doubles, 0), [0.5]);

    let narrow = unsafe { scope.protect_raw(vec![vec![-8i8, 9]].into_sexp()) };
    assert_eq!(int_row(narrow, 0), [-8, 9]);

    let floats = unsafe { scope.protect_raw(vec![vec![1.5f32]].into_sexp()) };
    assert_eq!(real_row(floats, 0), [1.5]);

    let raw = unsafe { scope.protect_raw(vec![vec![0xffu8]].into_sexp()) };
    assert_eq!(row(raw, 0).1, SEXPTYPE::RAWSXP);

    let logicals = unsafe { scope.protect_raw(vec![vec![true, false]].into_sexp()) };
    let (elt, ty, len) = row(logicals, 0);
    assert_eq!(ty, SEXPTYPE::LGLSXP);
    assert_eq!(len, 2);
    assert_eq!((elt.logical_elt(0), elt.logical_elt(1)), (1, 0));

    let strings =
        unsafe { scope.protect_raw(vec![vec!["a".to_string(), "b".to_string()]].into_sexp()) };
    let (elt, ty, _) = row(strings, 0);
    assert_eq!(ty, SEXPTYPE::STRSXP);
    assert_eq!(elt.string_elt_str(1), Some("b"));

    let strs = unsafe { scope.protect_raw(vec![vec!["x"]].into_sexp()) };
    assert_eq!(row(strs, 0).1, SEXPTYPE::STRSXP);

    let with_na = unsafe { scope.protect_raw(vec![vec![Some(1i32), None]].into_sexp()) };
    assert_eq!(int_row(with_na, 0), [1, NA_INTEGER]);

    let opt_wide = unsafe { scope.protect_raw(vec![vec![Some(2usize), None]].into_sexp()) };
    assert_eq!(int_row(opt_wide, 0), [2, NA_INTEGER]);

    // Nested one level deeper: a list of lists of integer vectors.
    let deeper = unsafe { scope.protect_raw(vec![vec![vec![1usize]]].into_sexp()) };
    let (inner, ty, _) = row(deeper, 0);
    assert_eq!(ty, SEXPTYPE::VECSXP);
    assert_eq!(int_row(inner, 0), [1]);
}

/// An empty row is a zero-length vector of the element's R type.
fn test_vec_vec_empty_rows_keep_their_type() {
    let scope = unsafe { ProtectScope::new() };
    let cases: [(SEXP, SEXPTYPE); 5] = unsafe {
        [
            (
                scope.protect_raw(vec![Vec::<usize>::new()].into_sexp()),
                SEXPTYPE::INTSXP,
            ),
            (
                scope.protect_raw(vec![Vec::<i64>::new()].into_sexp()),
                SEXPTYPE::INTSXP,
            ),
            (
                scope.protect_raw(vec![Vec::<f64>::new()].into_sexp()),
                SEXPTYPE::REALSXP,
            ),
            (
                scope.protect_raw(vec![Vec::<String>::new()].into_sexp()),
                SEXPTYPE::STRSXP,
            ),
            (
                scope.protect_raw(vec![Vec::<bool>::new()].into_sexp()),
                SEXPTYPE::LGLSXP,
            ),
        ]
    };
    for (list, expected) in cases {
        let (_, ty, len) = row(list, 0);
        assert_eq!((ty, len), (expected, 0));
    }

    let none = unsafe { scope.protect_raw(Vec::<Vec<usize>>::new().into_sexp()) };
    assert_eq!((none.type_of(), none.xlength()), (SEXPTYPE::VECSXP, 0));
}

/// `try_into_sexp` and the `_unchecked` paths build the same list.
fn test_vec_vec_unchecked_and_try_paths_agree() {
    let scope = unsafe { ProtectScope::new() };
    let rows = || -> Vec<Vec<usize>> { vec![vec![1, BIG], vec![2]] };

    let checked = unsafe { scope.protect_raw(rows().try_into_sexp().unwrap()) };
    let unchecked = unsafe { scope.protect_raw(rows().into_sexp_unchecked()) };
    let try_unchecked = unsafe { scope.protect_raw(rows().try_into_sexp_unchecked().unwrap()) };
    for list in [checked, unchecked, try_unchecked] {
        assert_eq!(real_row(list, 0), [1.0, BIG as f64]);
        assert_eq!(int_row(list, 1), [2]);
    }
}

fn test_borrowed_boxed_and_array_rows() {
    let scope = unsafe { ProtectScope::new() };
    let a = [1usize, 2];
    let b: [usize; 0] = [];
    let c = [BIG];

    let slices = unsafe { scope.protect_raw(vec![&a[..], &b[..], &c[..]].into_sexp()) };
    assert_eq!(int_row(slices, 0), [1, 2]);
    assert_eq!(int_row(slices, 1), Vec::<i32>::new());
    assert_eq!(real_row(slices, 2), [BIG as f64]);

    // A flat slice converts like the vector.
    let flat = unsafe { scope.protect_raw(a.as_slice().into_sexp()) };
    assert_eq!(flat.type_of(), SEXPTYPE::INTSXP);

    let boxed: Vec<Box<[usize]>> = vec![vec![3usize].into_boxed_slice(), Box::new([])];
    let boxed = unsafe { scope.protect_raw(boxed.into_sexp()) };
    assert_eq!(int_row(boxed, 0), [3]);
    assert_eq!(int_row(boxed, 1), Vec::<i32>::new());

    let arrays = unsafe { scope.protect_raw(vec![[4usize, 5], [6, BIG]].into_sexp()) };
    assert_eq!(int_row(arrays, 0), [4, 5]);
    assert_eq!(real_row(arrays, 1), [6.0, BIG as f64]);

    let strings = ["s".to_string()];
    let string_slices = unsafe { scope.protect_raw(vec![&strings[..]].into_sexp()) };
    assert_eq!(row(string_slices, 0).1, SEXPTYPE::STRSXP);
}

fn test_option_rows_and_option_vec() {
    let scope = unsafe { ProtectScope::new() };

    let rows: Vec<Option<Vec<usize>>> = vec![Some(vec![1]), None, Some(vec![]), Some(vec![BIG])];
    let list = unsafe { scope.protect_raw(rows.into_sexp()) };
    assert_eq!(int_row(list, 0), [1]);
    assert_eq!(row(list, 1).1, SEXPTYPE::NILSXP);
    assert_eq!(int_row(list, 2), Vec::<i32>::new());
    assert_eq!(real_row(list, 3), [BIG as f64]);

    let a = [7usize];
    let borrowed: Vec<Option<&[usize]>> = vec![Some(&a[..]), None];
    let borrowed = unsafe { scope.protect_raw(borrowed.into_sexp()) };
    assert_eq!(int_row(borrowed, 0), [7]);
    assert_eq!(row(borrowed, 1).1, SEXPTYPE::NILSXP);

    let strings: Vec<Option<Vec<String>>> = vec![Some(vec!["z".into()]), None];
    let strings = unsafe { scope.protect_raw(strings.into_sexp()) };
    assert_eq!(row(strings, 0).1, SEXPTYPE::STRSXP);
    assert_eq!(row(strings, 1).1, SEXPTYPE::NILSXP);

    let boxed: Vec<Option<Box<[usize]>>> =
        vec![Some(vec![BIG].into_boxed_slice()), None, Some(Box::new([]))];
    let boxed = unsafe { scope.protect_raw(boxed.into_sexp()) };
    assert_eq!(real_row(boxed, 0), [BIG as f64]);
    assert_eq!(row(boxed, 1).1, SEXPTYPE::NILSXP);
    assert_eq!(int_row(boxed, 2), Vec::<i32>::new());
    let boxed_try: Vec<Option<Box<[u64]>>> = vec![Some(vec![5u64].into_boxed_slice()), None];
    let boxed_try = unsafe { scope.protect_raw(boxed_try.try_into_sexp().unwrap()) };
    assert_eq!(int_row(boxed_try, 0), [5]);
    assert_eq!(row(boxed_try, 1).1, SEXPTYPE::NILSXP);

    let some_box = unsafe { scope.protect_raw(Some(vec![3usize].into_boxed_slice()).into_sexp()) };
    assert_eq!(some_box.type_of(), SEXPTYPE::INTSXP);
    assert_eq!(
        Option::<Box<[usize]>>::None.into_sexp().type_of(),
        SEXPTYPE::NILSXP
    );

    let some = unsafe { scope.protect_raw(Some(vec![1usize, 2]).into_sexp()) };
    assert_eq!(some.type_of(), SEXPTYPE::INTSXP);
    let wide = unsafe { scope.protect_raw(Some(vec![BIG]).into_sexp()) };
    assert_eq!(wide.type_of(), SEXPTYPE::REALSXP);
    assert_eq!(
        Option::<Vec<usize>>::None.into_sexp().type_of(),
        SEXPTYPE::NILSXP
    );
}

/// The strict helpers build integer rows when every value fits (their panic
/// paths, which run before any R allocation, are unit-tested in `strict.rs`).
fn test_strict_vec_vec_usize_in_range() {
    let scope = unsafe { ProtectScope::new() };
    let rows: Vec<Vec<usize>> = vec![vec![1, 5, 9], vec![], vec![i32::MAX as usize]];
    let list = unsafe { scope.protect_raw(strict::checked_vec_vec_usize_into_sexp(rows)) };
    assert_eq!(list.type_of(), SEXPTYPE::VECSXP);
    assert_eq!(int_row(list, 0), [1, 5, 9]);
    assert_eq!(int_row(list, 1), Vec::<i32>::new());
    assert_eq!(int_row(list, 2), [i32::MAX]);

    let i64_rows = unsafe {
        scope.protect_raw(strict::checked_vec_vec_i64_into_sexp(vec![
            vec![-1, 0],
            vec![],
        ]))
    };
    assert_eq!(int_row(i64_rows, 0), [-1, 0]);
    assert_eq!(int_row(i64_rows, 1), Vec::<i32>::new());
}

fn test_strict_vec_option_vec_in_range() {
    let scope = unsafe { ProtectScope::new() };
    let rows: Vec<Option<Vec<u64>>> = vec![Some(vec![3]), None, Some(vec![])];
    let list = unsafe { scope.protect_raw(strict::checked_vec_option_vec_u64_into_sexp(rows)) };
    assert_eq!(int_row(list, 0), [3]);
    assert_eq!(row(list, 1).1, SEXPTYPE::NILSXP);
    assert_eq!(int_row(list, 2), Vec::<i32>::new());
}
