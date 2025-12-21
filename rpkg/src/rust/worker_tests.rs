//! Tests for worker thread (run_on_worker) and with_r_thread functionality.

use miniextendr_api::ffi::{R_NilValue, SEXP};
use miniextendr_api::worker::{run_on_worker, with_r_thread};
use miniextendr_api::{miniextendr, miniextendr_module};

use crate::externalptr_tests::{Counter, Point};
use crate::panic_tests::add;
use crate::unwind_protect_tests::SimpleDropMsg;

/// Test that drops run on the worker thread during normal completion.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_worker_drop_on_success() -> SEXP {
    let result = run_on_worker(|| {
        let _a = SimpleDropMsg("worker: stack resource");
        let _b = Box::new(SimpleDropMsg("worker: heap resource"));
        42
    });
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Test that drops run when Rust code panics on the worker thread.
/// Panic is caught by catch_unwind, converted to R error after unwind.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_worker_drop_on_panic() -> SEXP {
    run_on_worker::<_, ()>(|| {
        let _a = SimpleDropMsg("worker: resource before panic");
        let _b = Box::new(SimpleDropMsg("worker: boxed resource before panic"));

        eprintln!("[Rust] Worker: about to panic");
        panic!("intentional panic from worker");
    });
    // Never reached - panic converts to R error
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

// =============================================================================
// Comprehensive worker/with_r_thread tests
// =============================================================================

// -----------------------------------------------------------------------------
// Test 1: Simple worker execution - no R API calls
// -----------------------------------------------------------------------------

/// Worker executes pure Rust code, returns result.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_simple() -> SEXP {
    let result = run_on_worker(|| {
        let a = 10;
        let b = 32;
        a + b
    });
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

// -----------------------------------------------------------------------------
// Test 2: Worker with with_r_thread - call R API from worker
// -----------------------------------------------------------------------------

/// Worker uses with_r_thread to call R API, returns i32 (Send-able).
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_with_r_thread() -> SEXP {
    let result = run_on_worker(|| {
        let value = 123;
        // Call R API on main thread, return i32 (Send)
        with_r_thread(move || {
            let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger(value) };
            unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
        })
    });
    // Convert to SEXP on main thread after run_on_worker returns
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Worker makes multiple with_r_thread calls, each returning Send-able values.
/// Final SEXP creation happens on main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_multiple_r_calls() -> SEXP {
    let values = run_on_worker(|| {
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

// -----------------------------------------------------------------------------
// Test 3: Panic scenarios
// -----------------------------------------------------------------------------

/// Panic on worker thread (no with_r_thread).
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_panic_simple() -> SEXP {
    run_on_worker::<_, ()>(|| {
        panic!("simple panic on worker");
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

/// Panic on worker with RAII resources - drops must run.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_panic_with_drops() -> SEXP {
    run_on_worker::<_, ()>(|| {
        let _resource1 = SimpleDropMsg("test_panic_drops: resource1");
        let _resource2 = Box::new(SimpleDropMsg("test_panic_drops: resource2 (boxed)"));
        panic!("panic after creating resources");
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

/// Panic INSIDE a with_r_thread callback.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_panic_in_r_thread() -> SEXP {
    run_on_worker::<_, ()>(|| {
        with_r_thread::<_, ()>(|| {
            panic!("panic inside with_r_thread callback");
        });
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

/// Panic in with_r_thread with resources - worker resources must still drop.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_panic_in_r_thread_with_drops() -> SEXP {
    run_on_worker::<_, ()>(|| {
        let _worker_resource = SimpleDropMsg("test: worker resource before with_r_thread");

        with_r_thread::<_, ()>(|| {
            let _main_resource = SimpleDropMsg("test: main thread resource before panic");
            panic!("panic in with_r_thread with resources");
        });
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

// -----------------------------------------------------------------------------
// Test 4: R error scenarios (via with_r_thread)
// -----------------------------------------------------------------------------

/// R error inside with_r_thread callback.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_r_error_in_r_thread() -> SEXP {
    run_on_worker::<_, ()>(|| {
        with_r_thread::<_, ()>(|| unsafe {
            miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"R error in with_r_thread".as_ptr());
        });
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

/// R error with RAII resources - both worker and main thread resources must drop.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_r_error_with_drops() -> SEXP {
    run_on_worker::<_, ()>(|| {
        let _worker_resource = SimpleDropMsg("r_error_drops: worker resource");

        with_r_thread::<_, ()>(|| {
            let _main_resource = SimpleDropMsg("r_error_drops: main thread resource");
            unsafe {
                miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"R error with drops test".as_ptr());
            }
        });
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

// -----------------------------------------------------------------------------
// Test 5: Mixed scenarios - some R calls succeed, then error/panic
// -----------------------------------------------------------------------------

/// Multiple R calls, last one errors.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_r_calls_then_error() -> SEXP {
    run_on_worker::<_, ()>(|| {
        // First R call succeeds - return a simple i32 instead of SEXP
        let val1 = with_r_thread(|| 1i32);
        eprintln!("[Rust] First R call succeeded, got {}", val1);

        // Second R call succeeds
        let val2 = with_r_thread(|| 2i32);
        eprintln!("[Rust] Second R call succeeded, got {}", val2);

        // Third R call errors
        with_r_thread::<_, ()>(|| unsafe {
            miniextendr_api::ffi::Rf_error(
                c"%s".as_ptr(),
                c"Error after successful calls".as_ptr(),
            );
        });
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

/// Multiple R calls, then panic in Rust (not in R).
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_worker_r_calls_then_panic() -> SEXP {
    run_on_worker::<_, ()>(|| {
        // Successful R call - return i32 instead of SEXP
        let val = with_r_thread(|| 42i32);
        eprintln!(
            "[Rust] R call succeeded with {}, now panicking in Rust",
            val
        );

        panic!("Rust panic after successful R call");
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

// -----------------------------------------------------------------------------
// Test 6: Return value propagation
// -----------------------------------------------------------------------------

/// Test that i32 return from worker works.
#[miniextendr]
/// @title Worker Thread Tests
/// @name rpkg_worker_tests
/// @keywords internal
/// @description Worker-thread and main-thread helpers
/// @examples
/// test_worker_return_i32()
/// test_worker_return_string()
/// test_worker_return_f64()
/// try(test_main_thread_r_error())
/// \dontrun{
/// unsafe_C_test_worker_simple()
/// }
/// @aliases test_worker_return_i32 test_worker_return_string test_worker_return_f64
/// @aliases test_main_thread_r_api test_main_thread_r_error test_main_thread_r_error_with_drops
/// @aliases unsafe_C_worker_drop_on_success unsafe_C_worker_drop_on_panic
/// @aliases unsafe_C_test_worker_simple unsafe_C_test_worker_with_r_thread
/// @aliases unsafe_C_test_worker_multiple_r_calls unsafe_C_test_worker_panic_simple
/// @aliases unsafe_C_test_worker_panic_with_drops unsafe_C_test_worker_panic_in_r_thread
/// @aliases unsafe_C_test_worker_panic_in_r_thread_with_drops
/// @aliases unsafe_C_test_worker_r_error_in_r_thread unsafe_C_test_worker_r_error_with_drops
/// @aliases unsafe_C_test_worker_r_calls_then_error unsafe_C_test_worker_r_calls_then_panic
/// @aliases unsafe_C_test_extptr_from_worker unsafe_C_test_multiple_extptrs_from_worker
/// @aliases unsafe_C_test_wrong_thread_r_api unsafe_C_test_call_worker_fn_from_main
/// @aliases unsafe_C_test_nested_helper_from_worker unsafe_C_test_nested_multiple_helpers
/// @aliases unsafe_C_test_nested_with_r_thread unsafe_C_test_nested_worker_calls
/// @aliases unsafe_C_test_nested_with_error unsafe_C_test_nested_with_panic
/// @aliases unsafe_C_test_deep_with_r_thread_sequence
pub fn test_worker_return_i32() -> i32 {
    // This uses worker strategy automatically (returns non-SEXP)
    let x = 21;
    x * 2
}

/// Test that String return from worker works.
#[miniextendr]
pub fn test_worker_return_string() -> String {
    // Uses worker strategy
    format!("hello from {}", "worker")
}

/// Test that f64 return from worker works.
#[miniextendr]
pub fn test_worker_return_f64() -> f64 {
    std::f64::consts::PI * 2.0
}

// -----------------------------------------------------------------------------
// Test 7: ExternalPtr creation (must be main thread - ExternalPtr is !Send)
// -----------------------------------------------------------------------------

/// Test ExternalPtr creation with computation done on worker.
/// The computation happens on worker, but ExternalPtr creation happens on main thread
/// since ExternalPtr is !Send. We run_on_worker to get Send-able values, then create SEXP after.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_extptr_from_worker() -> SEXP {
    // Do computation on worker, return Send-able value
    let value = run_on_worker(|| {
        let a = 42;
        let b = 58;
        a + b
    });

    // Create ExternalPtr on main thread (after run_on_worker returns)
    use miniextendr_api::externalptr::ExternalPtr;
    let ptr = ExternalPtr::new(Counter { value });
    ptr.as_sexp()
}

/// Test creating multiple ExternalPtrs with values computed on worker.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_multiple_extptrs_from_worker() -> SEXP {
    // Compute values on worker, return tuple (all Send)
    let (counter_val, point_x, point_y) = run_on_worker(|| {
        let counter_val = 50 + 50;
        let point_x = 0.5 + 1.0;
        let point_y = 1.5 + 1.0;
        (counter_val, point_x, point_y)
    });

    // Create ExternalPtrs on main thread
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::{
        Rf_allocVector, Rf_protect, Rf_unprotect, SET_VECTOR_ELT, SEXPTYPE,
    };

    unsafe {
        // Create a list of 2 elements
        let list = Rf_allocVector(SEXPTYPE::VECSXP, 2);
        Rf_protect(list);

        // Create Counter ExternalPtr
        let counter_ptr = ExternalPtr::new(Counter { value: counter_val });
        SET_VECTOR_ELT(list, 0, counter_ptr.as_sexp());

        // Create Point ExternalPtr
        let point_ptr = ExternalPtr::new(Point {
            x: point_x,
            y: point_y,
        });
        SET_VECTOR_ELT(list, 1, point_ptr.as_sexp());

        Rf_unprotect(1);
        list
    }
}

// -----------------------------------------------------------------------------
// Test 8: Main thread functions (via attribute)
// -----------------------------------------------------------------------------

/// Function that must run on main thread (uses R API directly).
#[miniextendr(unsafe(main_thread))]
pub fn test_main_thread_r_api() -> i32 {
    // This runs on main thread, can call R API directly
    let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) };
    unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
}

/// Main thread function that triggers R error.
#[miniextendr(unsafe(main_thread))]
pub fn test_main_thread_r_error() -> i32 {
    unsafe {
        miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"R error from main_thread fn".as_ptr())
    }
}

/// Main thread function with RAII drops that triggers R error.
#[miniextendr(unsafe(main_thread))]
pub fn test_main_thread_r_error_with_drops() -> i32 {
    let _resource = SimpleDropMsg("main_thread_r_error: resource");
    unsafe {
        miniextendr_api::ffi::Rf_error(
            c"%s".as_ptr(),
            c"R error from main_thread fn with drops".as_ptr(),
        )
    }
}

// -----------------------------------------------------------------------------
// Test 9: Calling checked R APIs from wrong thread (should panic clearly)
// -----------------------------------------------------------------------------

/// This demonstrates what happens if you call a checked R API from worker
/// without using with_r_thread - it should panic with clear message.
/// NOTE: This is an intentional misuse for testing error messages.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_wrong_thread_r_api() -> SEXP {
    run_on_worker::<_, ()>(|| {
        // This should panic because Rf_ScalarInteger is checked
        // and we're not on main thread
        let _ = unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) };
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

// -----------------------------------------------------------------------------
// Test 10: Nested wrappers - calling miniextendr functions from miniextendr functions
// -----------------------------------------------------------------------------

/// Helper that calls with_r_thread and returns a Send-able value.
fn helper_r_call_value(value: i32) -> i32 {
    with_r_thread(move || {
        // Create SEXP on main thread, extract value, return i32
        let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger(value * 2) };
        unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
    })
}

/// Nested: call helper function from worker.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_helper_from_worker() -> SEXP {
    let result = run_on_worker(|| helper_r_call_value(21));
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Nested: multiple helper calls with with_r_thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_multiple_helpers() -> SEXP {
    let result = run_on_worker(|| {
        let v1 = helper_r_call_value(10);
        let v2 = helper_r_call_value(20);
        v1 + v2
    });
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Nested with_r_thread calls - with_r_thread inside with_r_thread.
/// Since with_r_thread checks if already on main thread, this should work.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_with_r_thread() -> SEXP {
    let result = run_on_worker(|| {
        with_r_thread(|| {
            // Already on main thread, nested call runs directly
            with_r_thread(|| 42i32)
        })
    });
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Test calling a miniextendr function that uses worker strategy from main thread.
/// The inner function will use run_on_worker, outer is extern "C-unwind".
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_call_worker_fn_from_main() -> SEXP {
    // Call add() which uses worker strategy internally
    // This should work: we're on main thread, add() spawns worker job
    let result = add(10, 32);
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Call a worker-strategy function from inside run_on_worker.
/// This tests nested run_on_worker - the inner call should detect
/// we're on worker and use with_r_thread's direct execution path.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_worker_calls() -> SEXP {
    let result = run_on_worker(|| {
        // We're on worker thread now
        // Call helper_r_call_value which uses with_r_thread and returns i32 (Send)
        let val = helper_r_call_value(100);

        // Return i32 (Send-able) from run_on_worker
        val * 2
    });
    // Convert to SEXP on main thread
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
}

/// Complex nested scenario: worker -> multiple with_r_thread -> one errors.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_with_error() -> SEXP {
    run_on_worker::<_, ()>(|| {
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
                miniextendr_api::ffi::Rf_error(
                    c"%s".as_ptr(),
                    c"Error in nested with_r_thread".as_ptr(),
                )
            }
        })
    });
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

/// Complex nested scenario: worker -> multiple with_r_thread -> one panics.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_nested_with_panic() -> SEXP {
    run_on_worker::<_, ()>(|| {
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
    #[allow(unreachable_code)]
    unsafe {
        R_NilValue
    }
}

/// Deep nesting: with_r_thread called many times in sequence.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_test_deep_with_r_thread_sequence() -> SEXP {
    let sum = run_on_worker(|| {
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

    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(sum) }
}

miniextendr_module! {
    mod worker_tests;

    // Worker thread tests (basic)
    extern "C-unwind" fn C_worker_drop_on_success;
    extern "C-unwind" fn C_worker_drop_on_panic;

    // Comprehensive worker/with_r_thread tests
    extern "C-unwind" fn C_test_worker_simple;
    extern "C-unwind" fn C_test_worker_with_r_thread;
    extern "C-unwind" fn C_test_worker_multiple_r_calls;
    extern "C-unwind" fn C_test_worker_panic_simple;
    extern "C-unwind" fn C_test_worker_panic_with_drops;
    extern "C-unwind" fn C_test_worker_panic_in_r_thread;
    extern "C-unwind" fn C_test_worker_panic_in_r_thread_with_drops;
    extern "C-unwind" fn C_test_worker_r_error_in_r_thread;
    extern "C-unwind" fn C_test_worker_r_error_with_drops;
    extern "C-unwind" fn C_test_worker_r_calls_then_error;
    extern "C-unwind" fn C_test_worker_r_calls_then_panic;
    fn test_worker_return_i32;
    fn test_worker_return_string;
    fn test_worker_return_f64;
    extern "C-unwind" fn C_test_extptr_from_worker;
    extern "C-unwind" fn C_test_multiple_extptrs_from_worker;
    fn test_main_thread_r_api;
    fn test_main_thread_r_error;
    fn test_main_thread_r_error_with_drops;
    extern "C-unwind" fn C_test_wrong_thread_r_api;

    // Nested wrapper tests
    extern "C-unwind" fn C_test_nested_helper_from_worker;
    extern "C-unwind" fn C_test_nested_multiple_helpers;
    extern "C-unwind" fn C_test_nested_with_r_thread;
    extern "C-unwind" fn C_test_call_worker_fn_from_main;
    extern "C-unwind" fn C_test_nested_worker_calls;
    extern "C-unwind" fn C_test_nested_with_error;
    extern "C-unwind" fn C_test_nested_with_panic;
    extern "C-unwind" fn C_test_deep_with_r_thread_sequence;
}
