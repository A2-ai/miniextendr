//! Trait conversion failures unwind Rust resources and return tagged conditions.
mod r_test_utils;

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

use miniextendr_api::condition::RCondition;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::trait_abi::{extract_arg, from_sexp};
use miniextendr_api::unwind_protect::with_r_unwind_protect_shim;
use miniextendr_api::{IntoR, SEXP, TryFromSexp};

static ERROR_DROPPED: AtomicBool = AtomicBool::new(false);

struct Refusal;

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("conversion refused")
    }
}

impl Drop for Refusal {
    fn drop(&mut self) {
        ERROR_DROPPED.store(true, Ordering::SeqCst);
    }
}

struct RefusingValue;

impl TryFromSexp for RefusingValue {
    type Error = Refusal;

    fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
        Err(Refusal)
    }
}

struct DropFlag<'a>(&'a Cell<bool>);

impl Drop for DropFlag<'_> {
    fn drop(&mut self) {
        self.0.set(true);
    }
}

fn assert_panic(result: SEXP, expected: &str) {
    let (_, parts) = unsafe { RCondition::from_tagged_sexp(result) }
        .expect("Rust conversion failure must return a tagged condition")
        .into_parts();
    assert!(parts.message.contains(expected), "{}", parts.message);
}

#[test]
fn conversion_failure_drops_the_error_and_previously_converted_arguments() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let dropped = Cell::new(false);
        ERROR_DROPPED.store(false, Ordering::SeqCst);
        let args = [SEXP::nil()];
        let result = scope.protect(with_r_unwind_protect_shim(|| {
            let _previous_arg = DropFlag(&dropped);
            let _: RefusingValue = extract_arg(1, args.as_ptr(), 0, "value");
            SEXP::nil()
        }));
        assert_panic(result.get(), "conversion refused");
        assert!(dropped.get(), "earlier arguments must be dropped");
        assert!(
            ERROR_DROPPED.load(Ordering::SeqCst),
            "error must be dropped"
        );
    });
}

#[test]
fn invalid_indices_panic_before_reading_argv_without_narrowing_usize() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let args = [SEXP::nil()];
        for (argc, index) in [(0, 0), (-1, 0), (1, 1), (1, usize::MAX)] {
            let dropped = Cell::new(false);
            let result = scope.protect(with_r_unwind_protect_shim(|| {
                let _previous_arg = DropFlag(&dropped);
                let _: i32 = extract_arg(argc, args.as_ptr(), index, "value");
                SEXP::nil()
            }));
            assert_panic(result.get(), "missing argument: value");
            assert!(dropped.get());
        }
        let value = scope.protect(42i32.into_sexp()).get();
        assert_eq!(from_sexp::<i32>(value), 42);
        assert_eq!(extract_arg::<i32>(1, [value].as_ptr(), 0, "value"), 42);
    });
}
