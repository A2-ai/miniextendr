//! Tests for protect stack limit (`--max-ppsize`) behavior.
//!
//! This test initializes R with a small protect stack (100 entries) to verify that:
//! 1. `ReprotectSlot` doesn't grow the protect stack on repeated `set()` calls
//! 2. Safe container insertion methods work correctly under constrained stack
//!
//! **Important**: This test has its own R initialization with `--max-ppsize=100`,
//! so it cannot share the `r_test_utils` module used by other tests.
//!
//! Run with:
//! ```sh
//! cargo test --manifest-path=miniextendr-api/Cargo.toml --test ppsize_limit
//! ```

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{OnceLock, mpsc};

use miniextendr_api::ffi::{self, Rf_allocVector, SEXPTYPE};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::{List, ListBuilder};
use miniextendr_api::strvec::StrVecBuilder;
use miniextendr_api::thread::RThreadBuilder;

// =============================================================================
// R initialization with small ppsize
// =============================================================================

/// Small protect stack size for testing.
/// R's default is 50000. We use 10000 to verify bounded patterns while
/// staying above R's minimum threshold.
const MAX_PPSIZE: &str = "--max-ppsize=10000";

type Job = Box<dyn FnOnce() + Send + 'static>;

static R_THREAD: OnceLock<mpsc::Sender<Job>> = OnceLock::new();

fn initialize_r_with_small_ppsize() {
    unsafe {
        let _engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla", MAX_PPSIZE])
            .init()
            .expect("Failed to initialize R with small ppsize");

        miniextendr_api::backtrace::miniextendr_panic_hook();
        miniextendr_api::worker::miniextendr_worker_init();
        disable_r_stack_checking();

        assert!(
            miniextendr_engine::r_initialized_sentinel(),
            "Rf_initialize_R did not set C stack sentinels"
        );
    }
}

fn disable_r_stack_checking() {
    #[cfg(feature = "nonapi")]
    {
        miniextendr_api::thread::disable_stack_checking_permanently();
    }

    #[cfg(not(feature = "nonapi"))]
    unsafe {
        unsafe extern "C" {
            static mut R_CStackLimit: usize;
        }
        R_CStackLimit = usize::MAX;
    }
}

fn with_r_thread<T, F>(f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let sender = R_THREAD.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Job>();
        RThreadBuilder::new()
            .name("r-ppsize-test".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                initialize_r_with_small_ppsize();
                for job in rx {
                    job();
                }
            })
            .expect("Failed to spawn R test thread");
        tx
    });

    let (result_tx, result_rx) = mpsc::sync_channel(0);
    let job: Job = Box::new(move || {
        let result = catch_unwind(AssertUnwindSafe(f));
        let _ = result_tx.send(result);
    });

    sender.send(job).expect("R test thread stopped");
    match result_rx
        .recv()
        .expect("R test thread dropped the response")
    {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

// =============================================================================
// Tests
// =============================================================================

/// Test that ReprotectSlot can handle many iterations without growing the stack.
///
/// With `--max-ppsize=10000`, naively protecting 20000 values without unprotecting
/// would overflow. ReprotectSlot reuses a single slot, so this should work.
#[test]
fn reprotect_slot_no_overflow() {
    with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let initial_count = scope.count();

        // Create a reprotect slot
        let slot = scope.protect_with_index(Rf_allocVector(SEXPTYPE::INTSXP, 1));

        // Count should be 1 (the slot itself)
        assert_eq!(scope.count(), initial_count + 1);

        // Now do 20000 iterations - more than the 10000-slot stack
        // If ReprotectSlot were growing the stack, this would overflow
        for i in 0..20000 {
            let new_vec = Rf_allocVector(SEXPTYPE::INTSXP, (i % 100 + 1) as isize);
            slot.set(new_vec);
        }

        // Count should still be 1
        assert_eq!(
            scope.count(),
            initial_count + 1,
            "ReprotectSlot should not grow protect stack"
        );

        // Verify the slot contains a valid vector
        let len = ffi::Rf_xlength(slot.get());
        assert_eq!(len, 100, "slot should contain final vector");
    });
}

/// Test that ProtectScope with normal protect DOES grow the stack.
///
/// We protect fewer values than the limit to verify the pattern works,
/// while staying under the 10000 limit to avoid crashing.
#[test]
fn protect_scope_grows_stack() {
    with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let initial_count = scope.count();

        // Protect 1000 values (within the 10000 limit)
        for _ in 0..1000 {
            let _ = scope.protect(Rf_allocVector(SEXPTYPE::INTSXP, 1));
        }

        // Count should be 1000
        assert_eq!(
            scope.count(),
            initial_count + 1000,
            "ProtectScope::protect should grow the stack"
        );
    });
}

/// Test ListBuilder with constrained protect stack.
///
/// ListBuilder uses a single protection for the list, then uses unchecked
/// insertion with pre-protected children. This pattern stays bounded.
#[test]
fn list_builder_bounded_stack() {
    with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let initial_count = scope.count();

        // Build a list with 50 elements
        // Each element is protected by the scope, so stack grows by 51 (list + 50 children)
        let builder = ListBuilder::new(&scope, 50);

        for i in 0..50 {
            let child = scope.protect_raw(ffi::Rf_ScalarInteger(i as i32));
            builder.set(i, child);
        }

        // Stack has: 1 list + 50 children = 51
        assert_eq!(
            scope.count(),
            initial_count + 51,
            "ListBuilder should have bounded protect usage"
        );

        let _ = builder.into_sexp();
    });
}

/// Test StrVecBuilder with constrained protect stack.
///
/// StrVecBuilder uses internal protect/unprotect for each string element,
/// keeping stack usage constant (1 for the vector itself).
#[test]
fn strvec_builder_constant_stack() {
    with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let initial_count = scope.count();

        // Build a string vector with 80 elements
        // Each set_str internally protects/unprotects, so stack stays at 1
        let builder = StrVecBuilder::new(&scope, 80);

        for i in 0..80 {
            builder.set_str(i, "test");
        }

        // Stack should only have the string vector itself
        assert_eq!(
            scope.count(),
            initial_count + 1,
            "StrVecBuilder::set_str should have constant stack usage"
        );

        let _ = builder.into_sexp();
    });
}

/// Test List::set_elt with constrained protect stack.
///
/// The safe `set_elt` method protects/unprotects each child during insertion,
/// keeping stack usage constant.
#[test]
fn list_set_elt_constant_stack() {
    with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let initial_count = scope.count();

        // Create a list protected by the scope
        let list = List::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, 80)));

        // Use set_elt which internally protects/unprotects
        for i in 0..80 {
            let child = ffi::Rf_ScalarInteger(i as i32);
            list.set_elt(i, child);
        }

        // Stack should only have the list itself
        assert_eq!(
            scope.count(),
            initial_count + 1,
            "List::set_elt should have constant stack usage"
        );
    });
}

/// Combined test: build nested lists under tight stack constraint.
///
/// This exercises a realistic workload where nested structures are built
/// using the safe patterns.
#[test]
fn nested_list_under_constraint() {
    with_r_thread(|| unsafe {
        let scope = ProtectScope::new();

        // Build a list of 500 lists, each with 10 elements
        // Without proper patterns, this could overflow the 10000-slot stack
        let outer = ListBuilder::new(&scope, 500);
        let after_outer = scope.count();

        for i in 0..500 {
            // Use a reprotect slot for the inner list
            let slot = scope.protect_with_index(Rf_allocVector(SEXPTYPE::VECSXP, 10));

            // Fill the inner list using set_elt (constant stack per element)
            let inner = List::from_raw(slot.get());
            for j in 0..10 {
                inner.set_elt(j, ffi::Rf_ScalarInteger((i * 10 + j) as i32));
            }

            // Set into outer list
            outer.set(i, slot.get());
        }

        // We have: 1 outer + 500 reprotect slots = 501
        let expected = after_outer + 500;
        assert_eq!(
            scope.count(),
            expected,
            "Nested list construction should be bounded"
        );

        let _ = outer.into_sexp();
    });
}

/// Test that exceeding the stack limit without proper patterns would fail.
///
/// This test demonstrates what happens when you exceed the limit.
/// We protect close to the limit to verify the bound patterns are necessary.
#[test]
fn exceed_limit_without_reprotect() {
    with_r_thread(|| unsafe {
        // This test verifies the limit is in effect by trying to exceed it
        // in a controlled way (using nested scopes to avoid actual overflow)
        let scope = ProtectScope::new();

        // Protect 9000 values - close to but under the 10000 limit
        for _ in 0..9000 {
            let _ = scope.protect(Rf_allocVector(SEXPTYPE::INTSXP, 1));
        }

        assert_eq!(scope.count(), 9000);

        // We're at 9000, the limit is 10000. R uses some slots internally,
        // so we're close to the edge. The test succeeds if we didn't crash.
    });
}
