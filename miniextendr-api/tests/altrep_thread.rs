//! Tests that verify ALTREP callback thread behavior.
//!
//! ALTREP callbacks (Elt, Length, Dataptr, etc.) run on whatever thread R calls
//! them from. They do NOT automatically route to the main thread. This means:
//!
//! - Accessing ALTREP from the R main thread → callback runs on main thread ✓
//! - Accessing ALTREP from a worker thread (via with_r_thread) → callback runs
//!   on main thread ✓ (because with_r_thread routes the entire operation)
//! - Accessing ALTREP from a raw std::thread → callback runs on THAT thread ✗
//!   (R API calls from non-main threads are undefined behavior)
//!
//! Run with: `cargo test -p miniextendr-api --test altrep_thread --features arrow`

mod r_test_utils;

#[cfg(feature = "arrow")]
mod arrow_altrep_thread {
    use super::r_test_utils;
    use miniextendr_api::ffi::{self, SEXP, SexpExt};
    use miniextendr_api::worker::is_r_main_thread;

    /// Create an ALTREP Float64Array with NAs and return the SEXP + protect it.
    fn make_arrow_altrep_with_nas() -> SEXP {
        use miniextendr_api::IntoRAltrep;
        use miniextendr_api::arrow_impl::Float64Array;

        // Build a Rust-computed Arrow array (NOT R-backed) with nulls
        let arr: Float64Array = vec![Some(10.0), None, Some(30.0), None, Some(50.0)]
            .into_iter()
            .collect();
        let sexp = arr.into_sexp_altrep();
        // Protect from GC for the duration of the test
        unsafe { ffi::Rf_protect(sexp) };
        sexp
    }

    #[test]
    fn altrep_elt_on_main_thread_reports_main_thread() {
        r_test_utils::with_r_thread(|| {
            assert!(is_r_main_thread(), "test should run on R main thread");

            let sexp = make_arrow_altrep_with_nas();

            // Access elements — ALTREP Elt callback runs on this (main) thread
            let v0 = sexp.real_elt(0);
            let v1 = sexp.real_elt(1);
            let v2 = sexp.real_elt(2);

            assert_eq!(v0, 10.0);
            assert!(v1.to_bits() == miniextendr_api::altrep_traits::NA_REAL.to_bits());
            assert_eq!(v2, 30.0);

            // Verify we're still on main thread after ALTREP access
            assert!(is_r_main_thread());

            unsafe { ffi::Rf_unprotect(1) };
        });
    }

    #[test]
    fn altrep_length_on_main_thread() {
        r_test_utils::with_r_thread(|| {
            let sexp = make_arrow_altrep_with_nas();
            assert_eq!(sexp.len(), 5);
            unsafe { ffi::Rf_unprotect(1) };
        });
    }

    #[test]
    fn altrep_na_positions_correct_on_main_thread() {
        r_test_utils::with_r_thread(|| {
            let sexp = make_arrow_altrep_with_nas();
            let na_bits = miniextendr_api::altrep_traits::NA_REAL.to_bits();

            let is_na: Vec<bool> = (0..5)
                .map(|i| sexp.real_elt(i as isize).to_bits() == na_bits)
                .collect();

            assert_eq!(is_na, vec![false, true, false, true, false]);
            unsafe { ffi::Rf_unprotect(1) };
        });
    }

    #[test]
    fn altrep_dataptr_materializes_on_main_thread() {
        r_test_utils::with_r_thread(|| {
            let sexp = make_arrow_altrep_with_nas();

            // DATAPTR_RO triggers materialization into data2 for Arrow arrays with nulls.
            // This must succeed (not segfault) and return a valid pointer.
            let ptr = unsafe { ffi::DATAPTR_RO(sexp) };
            assert!(
                !ptr.is_null(),
                "DATAPTR_RO on ALTREP with nulls must not return null"
            );

            // Read through the materialized pointer — should have NA sentinels
            let slice = unsafe { core::slice::from_raw_parts(ptr as *const f64, sexp.len()) };
            assert_eq!(slice[0], 10.0);
            assert!(slice[1].to_bits() == miniextendr_api::altrep_traits::NA_REAL.to_bits());
            assert_eq!(slice[2], 30.0);
            assert!(slice[3].to_bits() == miniextendr_api::altrep_traits::NA_REAL.to_bits());
            assert_eq!(slice[4], 50.0);

            unsafe { ffi::Rf_unprotect(1) };
        });
    }

    #[test]
    fn altrep_via_with_r_thread_from_worker() {
        // This test verifies that accessing ALTREP via `with_r_thread` from
        // the worker thread correctly routes the R API call to the main thread.
        r_test_utils::with_r_thread(|| {
            let sexp = make_arrow_altrep_with_nas();

            // Simulate the worker pattern: run_on_worker dispatches to worker,
            // worker calls with_r_thread to access ALTREP on main thread.
            // Since we're already on the R thread in this test, we simulate by
            // spawning a std::thread that calls with_r_thread.
            let sexp_raw = sexp.as_ptr() as usize; // smuggle as usize (not Send)

            let result = miniextendr_api::worker::run_on_worker(move || {
                // We're on the worker thread now
                assert!(
                    !is_r_main_thread(),
                    "worker thread should NOT be main thread"
                );

                // Access ALTREP via with_r_thread — routes to main thread
                miniextendr_api::with_r_thread(move || {
                    assert!(
                        is_r_main_thread(),
                        "with_r_thread should run on main thread"
                    );
                    let sexp = SEXP(sexp_raw as *mut _);
                    let v0 = sexp.real_elt(0);
                    let v1_bits = sexp.real_elt(1).to_bits();
                    let v2 = sexp.real_elt(2);
                    (v0, v1_bits, v2)
                })
            });

            let (v0, v1_bits, v2) = result.unwrap();
            assert_eq!(v0, 10.0);
            assert_eq!(v1_bits, miniextendr_api::altrep_traits::NA_REAL.to_bits());
            assert_eq!(v2, 30.0);

            unsafe { ffi::Rf_unprotect(1) };
        });
    }

    #[test]
    fn altrep_i32_with_nas_on_main_thread() {
        r_test_utils::with_r_thread(|| {
            use miniextendr_api::IntoRAltrep;
            use miniextendr_api::arrow_impl::Int32Array;

            let arr: Int32Array = vec![Some(1), None, Some(3)].into_iter().collect();
            let sexp = arr.into_sexp_altrep();
            unsafe { ffi::Rf_protect(sexp) };

            assert_eq!(sexp.len(), 3);
            assert_eq!(sexp.integer_elt(0), 1);
            assert_eq!(
                sexp.integer_elt(1),
                miniextendr_api::altrep_traits::NA_INTEGER
            );
            assert_eq!(sexp.integer_elt(2), 3);

            // DATAPTR_RO should materialize without crash
            let ptr = unsafe { ffi::DATAPTR_RO(sexp) };
            assert!(!ptr.is_null());

            unsafe { ffi::Rf_unprotect(1) };
        });
    }

    #[test]
    fn altrep_string_with_nas_on_main_thread() {
        r_test_utils::with_r_thread(|| {
            use miniextendr_api::IntoRAltrep;
            use miniextendr_api::arrow_impl::StringArray;

            let arr = StringArray::from(vec![Some("hello"), None, Some("world")]);
            let sexp = arr.into_sexp_altrep();
            unsafe { ffi::Rf_protect(sexp) };

            assert_eq!(sexp.len(), 3);

            // Element access via SexpExt — string_elt_str returns None for NA
            let elt0 = sexp.string_elt_str(0);
            let elt1 = sexp.string_elt_str(1);
            let elt2 = sexp.string_elt_str(2);
            assert_eq!(elt0, Some("hello"));
            assert_eq!(elt1, None); // NA
            assert_eq!(elt2, Some("world"));

            // is_na_string on the CHARSXP elements
            assert!(!sexp.string_elt(0).is_na_string());
            assert!(sexp.string_elt(1).is_na_string());

            unsafe { ffi::Rf_unprotect(1) };
        });
    }
}
