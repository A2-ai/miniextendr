//! Tests for worker thread (run_on_worker) and with_r_thread functionality.

use miniextendr_api::ffi::{R_NilValue, SEXP};
use miniextendr_api::miniextendr;
use miniextendr_api::worker::{panic_message_to_r_error, run_on_worker, with_r_thread};

use crate::externalptr_tests::{Counter, Point};
use crate::panic_tests::add;
use crate::unwind_protect_tests::SimpleDropMsg;

/// Convenience: run on worker, converting panics to R errors (diverges on panic).
fn run_on_worker_or_error<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    match run_on_worker(f) {
        Ok(val) => val,
        Err(msg) => panic_message_to_r_error(msg, None),
    }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_worker_drop_on_success() -> SEXP {
    let result = run_on_worker_or_error(|| {
        let _a = SimpleDropMsg("worker: stack resource");
        let _b = Box::new(SimpleDropMsg("worker: heap resource"));
        42
    });
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_worker_drop_on_panic() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        let _a = SimpleDropMsg("worker: resource before panic");
        let _b = Box::new(SimpleDropMsg("worker: boxed resource before panic"));

        eprintln!("[Rust] Worker: about to panic");
        panic!("intentional panic from worker");
    });
    unsafe { R_NilValue }
}

// region: Comprehensive worker/with_r_thread tests
// endregion

// region: Test 1: Simple worker execution - no R API calls

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_simple() -> SEXP {
    let result = run_on_worker_or_error(|| {
        let a = 10;
        let b = 32;
        a + b
    });
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}
// endregion

// region: Test 2: Worker with with_r_thread - call R API from worker

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_with_r_thread() -> SEXP {
    let result = run_on_worker_or_error(|| {
        let value = 123;
        // Call R API on main thread, return i32 (Send)
        with_r_thread(move || {
            let sexp = miniextendr_api::ffi::SEXP::scalar_integer(value);
            unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
        })
    });
    // Convert to SEXP on main thread after run_on_worker returns
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_multiple_r_calls() -> SEXP {
    let values = run_on_worker_or_error(|| {
        // First R call: get some value
        let v1 = with_r_thread(|| 10i32);

        // Second R call: compute something
        let v2 = with_r_thread(move || v1 + 20);

        // Third R call: final computation
        let v3 = with_r_thread(move || v2 + 30);

        // Return tuple of values (all Send)
        (v1, v2, v3)
    });

    // Create the SEXP vector on main thread
    unsafe {
        let vec = miniextendr_api::ffi::Rf_allocVector(miniextendr_api::ffi::SEXPTYPE::INTSXP, 3);
        miniextendr_api::ffi::Rf_protect(vec);
        let ptr = miniextendr_api::ffi::INTEGER(vec);
        *ptr.offset(0) = values.0;
        *ptr.offset(1) = values.1;
        *ptr.offset(2) = values.2;
        miniextendr_api::ffi::Rf_unprotect(1);
        vec
    }
}
// endregion

// region: Test 3: Panic scenarios

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_panic_simple() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        panic!("simple panic on worker");
    });
    unsafe { R_NilValue }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_panic_with_drops() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        let _resource1 = SimpleDropMsg("test_panic_drops: resource1");
        let _resource2 = Box::new(SimpleDropMsg("test_panic_drops: resource2 (boxed)"));
        panic!("panic after creating resources");
    });
    unsafe { R_NilValue }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_panic_in_r_thread() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        with_r_thread::<_, ()>(|| {
            panic!("panic inside with_r_thread callback");
        });
    });
    unsafe { R_NilValue }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_panic_in_r_thread_with_drops() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        let _worker_resource = SimpleDropMsg("test: worker resource before with_r_thread");

        with_r_thread::<_, ()>(|| {
            let _main_resource = SimpleDropMsg("test: main thread resource before panic");
            panic!("panic in with_r_thread with resources");
        });
    });
    unsafe { R_NilValue }
}
// endregion

// region: Test 4: R error scenarios (via with_r_thread)

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_r_error_in_r_thread() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        with_r_thread::<_, ()>(|| unsafe {
            miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"R error in with_r_thread".as_ptr()); // mxl::allow(MXL300)
        });
    });
    unsafe { R_NilValue }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_r_error_with_drops() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        let _worker_resource = SimpleDropMsg("r_error_drops: worker resource");

        with_r_thread::<_, ()>(|| {
            let _main_resource = SimpleDropMsg("r_error_drops: main thread resource");
            unsafe {
                miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"R error with drops test".as_ptr()); // mxl::allow(MXL300)
            }
        });
    });
    unsafe { R_NilValue }
}
// endregion

// region: Test 5: Mixed scenarios - some R calls succeed, then error/panic

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_r_calls_then_error() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        // First R call succeeds - return a simple i32 instead of SEXP
        let val1 = with_r_thread(|| 1i32);
        eprintln!("[Rust] First R call succeeded, got {}", val1);

        // Second R call succeeds
        let val2 = with_r_thread(|| 2i32);
        eprintln!("[Rust] Second R call succeeded, got {}", val2);

        // Third R call errors
        with_r_thread::<_, ()>(|| unsafe {
            // mxl::allow(MXL300)
            miniextendr_api::ffi::Rf_error(
                c"%s".as_ptr(),
                c"Error after successful calls".as_ptr(),
            );
        });
    });
    unsafe { R_NilValue }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_r_calls_then_panic() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        // Successful R call - return i32 instead of SEXP
        let val = with_r_thread(|| 42i32);
        eprintln!(
            "[Rust] R call succeeded with {}, now panicking in Rust",
            val
        );

        panic!("Rust panic after successful R call");
    });
    unsafe { R_NilValue }
}
// endregion

// region: Test 6: Return value propagation

/// @noRd
#[miniextendr]
/// @name rpkg_worker_tests
/// @noRd
/// @examples
/// test_worker_return_i32()
/// test_worker_return_string()
/// test_worker_return_f64()
/// try(test_main_thread_r_error())
/// \dontrun{
/// unsafe_C_test_worker_simple()
/// }
/// @aliases test_worker_return_i32 test_worker_return_string test_worker_return_f64
/// test_main_thread_r_api test_main_thread_r_error test_main_thread_r_error_with_drops
/// unsafe_C_worker_drop_on_success unsafe_C_worker_drop_on_panic
/// unsafe_C_test_worker_simple unsafe_C_test_worker_with_r_thread
/// unsafe_C_test_worker_multiple_r_calls unsafe_C_test_worker_panic_simple
/// unsafe_C_test_worker_panic_with_drops unsafe_C_test_worker_panic_in_r_thread
/// unsafe_C_test_worker_panic_in_r_thread_with_drops
/// unsafe_C_test_worker_r_error_in_r_thread unsafe_C_test_worker_r_error_with_drops
/// unsafe_C_test_worker_r_calls_then_error unsafe_C_test_worker_r_calls_then_panic
/// unsafe_C_test_extptr_from_worker unsafe_C_test_multiple_extptrs_from_worker
/// unsafe_C_test_wrong_thread_r_api unsafe_C_test_call_worker_fn_from_main
/// unsafe_C_test_nested_helper_from_worker unsafe_C_test_nested_multiple_helpers
/// unsafe_C_test_nested_with_r_thread unsafe_C_test_nested_worker_calls
/// unsafe_C_test_nested_with_error unsafe_C_test_nested_with_panic
/// unsafe_C_test_deep_with_r_thread_sequence
pub fn test_worker_return_i32() -> i32 {
    // This uses worker strategy automatically (returns non-SEXP)
    let x = 21;
    x * 2
}

/// @noRd
#[miniextendr]
pub fn test_worker_return_string() -> String {
    // Uses worker strategy
    format!("hello from {}", "worker")
}

/// @noRd
#[miniextendr]
pub fn test_worker_return_f64() -> f64 {
    std::f64::consts::PI * 2.0
}
// endregion

// region: Test 7: ExternalPtr creation (must be main thread - ExternalPtr is !Send)

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_extptr_from_worker() -> SEXP {
    // Do computation on worker, return Send-able value
    let value = run_on_worker_or_error(|| {
        let a = 42;
        let b = 58;
        a + b
    });

    // Create ExternalPtr on main thread (after run_on_worker returns)
    use miniextendr_api::externalptr::ExternalPtr;
    let ptr = ExternalPtr::new(Counter { value });
    ptr.as_sexp()
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_multiple_extptrs_from_worker() -> SEXP {
    // Compute values on worker, return tuple (all Send)
    let (counter_val, point_x, point_y) = run_on_worker_or_error(|| {
        let counter_val = 50 + 50;
        let point_x = 0.5 + 1.0;
        let point_y = 1.5 + 1.0;
        (counter_val, point_x, point_y)
    });

    // Create ExternalPtrs on main thread
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::{
        Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt,
    };

    unsafe {
        // Create a list of 2 elements
        let list = Rf_allocVector(SEXPTYPE::VECSXP, 2);
        Rf_protect(list);

        // Create Counter ExternalPtr
        let counter_ptr = ExternalPtr::new(Counter { value: counter_val });
        list.set_vector_elt(0, counter_ptr.as_sexp());

        // Create Point ExternalPtr
        let point_ptr = ExternalPtr::new(Point {
            x: point_x,
            y: point_y,
        });
        list.set_vector_elt(1, point_ptr.as_sexp());

        Rf_unprotect(1);
        list
    }
}
// endregion

// region: Test 8: Main thread functions (via attribute)

/// @noRd
#[miniextendr(unsafe(main_thread))]
pub fn test_main_thread_r_api() -> i32 {
    // This runs on main thread, can call R API directly
    let sexp = miniextendr_api::ffi::SEXP::scalar_integer(42);
    unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
}

/// @noRd
#[miniextendr(unsafe(main_thread))]
pub fn test_main_thread_r_error() -> i32 {
    unsafe {
        miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"R error from main_thread fn".as_ptr()) // mxl::allow(MXL300)
    }
}

/// @noRd
#[miniextendr(unsafe(main_thread))]
pub fn test_main_thread_r_error_with_drops() -> i32 {
    let _resource = SimpleDropMsg("main_thread_r_error: resource");
    unsafe {
        // mxl::allow(MXL300)
        miniextendr_api::ffi::Rf_error(
            c"%s".as_ptr(),
            c"R error from main_thread fn with drops".as_ptr(),
        )
    }
}
// endregion

// region: Test 9: Calling checked R APIs from worker thread (routed to main thread)

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_wrong_thread_r_api() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        // With worker-thread: routed to main thread via with_r_thread.
        // Without worker-thread: run_on_worker is a stub, runs inline on main thread.
        // Either way, this should succeed (not panic).
        let _ = miniextendr_api::ffi::SEXP::scalar_integer(42);
    });
    unsafe { R_NilValue }
}
// endregion

// region: Test 10: Nested wrappers - calling miniextendr functions from miniextendr functions

/// Helper that calls with_r_thread and returns a Send-able value.
fn helper_r_call_value(value: i32) -> i32 {
    with_r_thread(move || {
        // Create SEXP on main thread, extract value, return i32
        let sexp = miniextendr_api::ffi::SEXP::scalar_integer(value * 2);
        unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
    })
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_helper_from_worker() -> SEXP {
    let result = run_on_worker_or_error(|| helper_r_call_value(21));
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_multiple_helpers() -> SEXP {
    let result = run_on_worker_or_error(|| {
        let v1 = helper_r_call_value(10);
        let v2 = helper_r_call_value(20);
        v1 + v2
    });
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_with_r_thread() -> SEXP {
    let result = run_on_worker_or_error(|| {
        with_r_thread(|| {
            // Already on main thread, nested call runs directly
            with_r_thread(|| 42i32)
        })
    });
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_call_worker_fn_from_main() -> SEXP {
    // Call add() which uses worker strategy internally
    // This should work: we're on main thread, add() spawns worker job
    let result = add(10, 32);
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_worker_calls() -> SEXP {
    let result = run_on_worker_or_error(|| {
        // We're on worker thread now
        // Call helper_r_call_value which uses with_r_thread and returns i32 (Send)
        let val = helper_r_call_value(100);

        // Return i32 (Send-able) from run_on_worker
        val * 2
    });
    // Convert to SEXP on main thread
    miniextendr_api::ffi::SEXP::scalar_integer(result)
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_with_error() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        let _resource = SimpleDropMsg("nested_error: outer resource");

        // First nested call succeeds
        let val = with_r_thread(|| {
            let _inner_resource = SimpleDropMsg("nested_error: first call resource");
            42i32
        });
        eprintln!("[Rust] First nested call returned: {}", val);

        // Second nested call errors
        with_r_thread::<_, ()>(|| {
            let _inner_resource = SimpleDropMsg("nested_error: second call resource");
            unsafe {
                // mxl::allow(MXL300)
                miniextendr_api::ffi::Rf_error(
                    c"%s".as_ptr(),
                    c"Error in nested with_r_thread".as_ptr(),
                )
            }
        })
    });
    unsafe { R_NilValue }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_with_panic() -> SEXP {
    run_on_worker_or_error::<_, ()>(|| {
        let _resource = SimpleDropMsg("nested_panic: outer resource");

        // First nested call succeeds
        let val = with_r_thread(|| {
            let _inner_resource = SimpleDropMsg("nested_panic: first call resource");
            42i32
        });
        eprintln!("[Rust] First nested call returned: {}", val);

        // Second nested call panics
        with_r_thread::<_, ()>(|| {
            let _inner_resource = SimpleDropMsg("nested_panic: second call resource");
            panic!("Panic in nested with_r_thread");
        })
    });
    unsafe { R_NilValue }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_deep_with_r_thread_sequence() -> SEXP {
    let sum = run_on_worker_or_error(|| {
        let mut sum = 0i32;

        for i in 0..10 {
            let current = sum;
            sum = with_r_thread(move || {
                let new_sum = current + i;
                eprintln!("[Rust] Iteration {}: sum = {}", i, new_sum);
                new_sum
            });
        }

        sum
    });

    miniextendr_api::ffi::SEXP::scalar_integer(sum)
}
// endregion
