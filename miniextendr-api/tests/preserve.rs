//! Integration tests for preserve list utilities.

use miniextendr_api::ffi::{R_NilValue, Rf_ScalarInteger, Rf_ScalarReal};
use miniextendr_api::preserve;
use std::sync::Once;

static INIT: Once = Once::new();

fn initialize_r() {
    INIT.call_once(|| unsafe {
        let engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .init()
            .expect("Failed to initialize R");
        // Initialize in same order as rpkg/src/entrypoint.c.in
        miniextendr_api::backtrace::miniextendr_panic_hook();
        miniextendr_api::worker::miniextendr_worker_init();
        assert!(
            miniextendr_engine::r_initialized_sentinel(),
            "Rf_initialize_R did not set C stack sentinels"
        );
        std::mem::forget(engine);
    });
}

#[test]
fn preserve_suite() {
    initialize_r();

    unsafe {
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
    }
}
