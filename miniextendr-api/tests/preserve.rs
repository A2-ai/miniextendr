//! Integration tests for preserve list utilities.

mod r_test_utils;

use miniextendr_api::ffi::{R_NilValue, Rf_ScalarInteger, Rf_ScalarReal};
use miniextendr_api::preserve;

#[test]
fn preserve_suite() {
    r_test_utils::with_r_thread(|| unsafe {
        let initial = preserve::count();

        let a = Rf_ScalarInteger(1);
        let b = Rf_ScalarReal(2.5);

        let cell_a = preserve::insert(a);
        let cell_b = preserve::insert(b);

        assert_eq!(preserve::count(), initial + 2);

        preserve::release(cell_a);
        assert_eq!(preserve::count(), initial + 1);

        preserve::release(cell_b);
        assert_eq!(preserve::count(), initial);

        let nil_cell = preserve::insert(R_NilValue);
        assert!(std::ptr::addr_eq(nil_cell.0, R_NilValue.0));
        assert_eq!(preserve::count(), initial);
    });
}
