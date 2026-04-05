//! Integration tests for preserve list utilities.

mod r_test_utils;

use miniextendr_api::ffi::{SEXP, SexpExt};
use miniextendr_api::preserve;

#[test]
fn preserve_insert_release() {
    r_test_utils::with_r_thread(|| unsafe {
        let a = SEXP::scalar_integer(1);
        let b = SEXP::scalar_real(2.5);

        let cell_a = preserve::insert(a);
        let cell_b = preserve::insert(b);

        preserve::release(cell_a);
        preserve::release(cell_b);

        // R_NilValue is never collected, so insert returns R_NilValue itself
        let nil_cell = preserve::insert(SEXP::null());
        assert!(nil_cell.is_nil());
    });
}

#[cfg(feature = "debug-preserve")]
#[test]
fn preserve_count_tracking() {
    r_test_utils::with_r_thread(|| unsafe {
        let initial = preserve::count();

        let a = SEXP::scalar_integer(1);
        let b = SEXP::scalar_real(2.5);

        let cell_a = preserve::insert(a);
        let cell_b = preserve::insert(b);

        assert_eq!(preserve::count(), initial + 2);

        preserve::release(cell_a);
        assert_eq!(preserve::count(), initial + 1);

        preserve::release(cell_b);
        assert_eq!(preserve::count(), initial);

        let nil_cell = preserve::insert(SEXP::null());
        assert!(nil_cell.is_nil());
        assert_eq!(preserve::count(), initial);
    });
}
