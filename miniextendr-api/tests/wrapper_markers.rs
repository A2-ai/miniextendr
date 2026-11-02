//! The markers a generated wrapper fills or reads, through the generated code
//! and as plain values:
//!
//! - `NArgs` (#1860): no R formal; the wrapper passes `nargs()`, which the C
//!   wrapper converts as a non-negative whole number.
//! - `LazyDots` (#1892): R's `...` left unforced, read from the wrapper's
//!   frame.
//! - `WrapAsS3` (#1521): the R wrapper puts the value in its S3 class; the
//!   Rust side converts the payload unchanged.

mod r_test_utils;

use miniextendr_api::condition::RCondition;
use miniextendr_api::expression::{RCall, REnv, r_eval_str_global};
use miniextendr_api::externalptr::ExternalPtr;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope};
use miniextendr_api::{
    IntoR, LazyDots, NArgs, Preserved, SEXP, SEXPTYPE, SexpExt, WrapAsS3, miniextendr, sys,
};

// Defines this crate's marker type, `__MiniextendrInitMarker`, for the
// generated `Err` arms.
miniextendr_api::miniextendr_init!(wrapper_markers);

// region: generated entry points

/// The registered `.Call` entry whose name ends with `suffix`, with its arity
/// checked.
fn entry(suffix: &str, arity: i32) -> unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void {
    let entry = miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(suffix.as_bytes())
        })
        .unwrap_or_else(|| panic!("`{suffix}` must be registered"));
    assert_eq!(entry.numArgs, arity);
    entry.fun.unwrap()
}

/// Calls the two-argument C wrapper `suffix` (the call slot, then `arg`).
unsafe fn call1(suffix: &str, arg: SEXP) -> SEXP {
    let call = unsafe {
        std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP,
        >(entry(suffix, 2))
    };
    unsafe { call(SEXP::nil(), arg) }
}

/// The generated R wrapper of the function `name`.
fn r_wrapper(name: &str) -> &'static str {
    let head = format!("{name} <- function(");
    miniextendr_api::registry::MX_R_WRAPPERS
        .iter()
        .map(|entry| entry.content)
        .find(|content| content.contains(&head))
        .unwrap_or_else(|| panic!("`{name}` must have an R wrapper"))
}

/// R's value for `src`, kept rooted.
fn eval(src: &str) -> OwnedProtect {
    unsafe { OwnedProtect::new(r_eval_str_global(src).expect("R source should evaluate")) }
}

fn identical(value: SEXP, expected: &str) -> bool {
    let expected = Preserved::new(unsafe { r_eval_str_global(expected).unwrap() });
    bool::from(unsafe { sys::R_compute_identical(value, expected.as_sexp(), 0) })
}

// endregion

// region: NArgs

/// The count of a function's own call.
#[miniextendr]
pub fn marker_arg_count(nargs: NArgs) -> i32 {
    i32::try_from(nargs.get()).unwrap()
}

#[test]
fn nargs_is_a_plain_count() {
    let nargs = NArgs::new(3);
    assert_eq!(nargs.get(), 3);
    assert_eq!(usize::from(nargs), 3);
    assert_eq!(nargs.to_string(), "3");
    assert!(NArgs::new(2) < nargs);
    assert_eq!(nargs, NArgs::new(3));
}

#[test]
fn nargs_converts_what_nargs_returns() {
    r_test_utils::with_r_thread(|| {
        // `nargs()` is an integer scalar; a whole double reads the same.
        for (src, count) in [("0L", 0), ("4L", 4), ("2", 2)] {
            let value = eval(src);
            assert_eq!(
                NArgs::try_from_sexp(value.get()).unwrap(),
                NArgs::new(count)
            );
            let value = unsafe { NArgs::try_from_sexp_unchecked(value.get()) }.unwrap();
            assert_eq!(value, NArgs::new(count));
        }
        // Not a count: negative, NA, fractional, not a scalar, not a number.
        for src in ["-1L", "NA_integer_", "2.5", "c(1L, 2L)", "integer()", "'2'"] {
            let value = eval(src);
            let result: Result<NArgs, SexpError> = NArgs::try_from_sexp(value.get());
            assert!(result.is_err(), "{src} must be refused, got {result:?}");
        }
        // Back to R as an integer, for a trait view's vtable call.
        let back = unsafe { OwnedProtect::new(NArgs::new(5).into_sexp()) };
        assert!(identical(back.get(), "5L"));
    });
}

#[test]
fn nargs_is_filled_by_the_r_wrapper() {
    let wrapper = r_wrapper("marker_arg_count");
    // No formal: the count is passed in its place.
    assert!(
        wrapper.contains("marker_arg_count <- function()"),
        "{wrapper}"
    );
    assert!(wrapper.contains("nargs()"), "{wrapper}");

    r_test_utils::with_r_thread(|| unsafe {
        let three = OwnedProtect::new(3_i32.into_sexp());
        let value = call1("_marker_arg_count", three.get());
        assert_eq!(i32::try_from_sexp(value).unwrap(), 3);

        // A value `nargs()` never gives is the C wrapper's argument error.
        let negative = OwnedProtect::new((-1_i32).into_sexp());
        let value = OwnedProtect::new(call1("_marker_arg_count", negative.get()));
        match RCondition::from_tagged_sexp(value.get()) {
            Some(RCondition::Conversion { data, .. }) => {
                let data = data.expect("the parameter as a field");
                assert!(data.iter().any(|(name, _)| name == "param"), "{data:?}");
            }
            other => panic!("expected an argument error, got {other:?}"),
        }
    });
}

// endregion

// region: LazyDots

/// `(function(...) environment())(...)`: a frame whose `...` holds the dots
/// as R matched them, as a generated wrapper's frame does, plus a counter
/// of how many times the promise `bump()` ran.
const FRAME: &str = "local({
    state <- new.env()
    state$forced <- 0L
    bump <- function() { state$forced <- state$forced + 1L; 2 }
    list(
        frame = (function(...) environment())(1, , b = bump(), c = stop('boom')),
        state = state
    )
})";

/// How many times `bump()` ran.
fn forced(state: SEXP) -> i32 {
    let value = unsafe { REnv::from_sexp(state).get("forced") }.expect("`forced` is bound");
    i32::try_from_sexp(value).unwrap()
}

#[test]
fn lazy_dots_count_name_and_read_without_forcing() {
    r_test_utils::with_r_thread(|| {
        let parts = eval(FRAME);
        let frame = parts.get().vector_elt(0);
        let state = parts.get().vector_elt(1);
        let dots = unsafe { LazyDots::from_wrapper_arg(&frame) };

        assert_eq!(dots.len(), 4);
        assert!(!dots.is_empty());
        assert_eq!(dots.names(), [None, None, Some("b"), Some("c")]);
        assert_eq!(
            (0..4).map(|i| dots.is_missing_arg(i)).collect::<Vec<_>>(),
            [false, true, false, false]
        );
        // A constant is itself, a promise its expression.
        assert_eq!(f64::try_from_sexp(dots.expr(0)).unwrap(), 1.0);
        assert_eq!(dots.expr(1), SEXP::missing_arg());
        assert_eq!(dots.expr(2).type_of(), SEXPTYPE::LANGSXP);
        assert!(identical(dots.expr(2), "quote(bump())"));
        // Nothing above forced anything.
        assert_eq!(forced(state), 0);
        assert_eq!(format!("{dots:?}"), "LazyDots { len: Some(4), .. }");
    });
}

#[test]
fn lazy_dots_force_each_element_once() {
    r_test_utils::with_r_thread(|| {
        let parts = eval(FRAME);
        let frame = parts.get().vector_elt(0);
        let state = parts.get().vector_elt(1);
        let dots = unsafe { LazyDots::from_wrapper_arg(&frame) };

        assert_eq!(f64::try_from_sexp(dots.force(0)).unwrap(), 1.0);
        let value = dots.try_force(2).expect("`bump()` evaluates");
        assert_eq!(f64::try_from_sexp(value).unwrap(), 2.0);
        assert_eq!(forced(state), 1);
        // R keeps the promise's value: forcing again evaluates nothing.
        assert_eq!(f64::try_from_sexp(dots.force(2)).unwrap(), 2.0);
        assert_eq!(forced(state), 1);

        // R's errors come back as `Err`: an empty element, a failing one.
        let empty = dots.try_force(1).expect_err("an empty element");
        assert!(
            empty.message().contains("\"..2\" is missing"),
            "{}",
            empty.message()
        );
        let boom = dots.try_force(3).expect_err("`stop('boom')`");
        assert_eq!(boom.message(), "boom");
    });
}

#[test]
fn lazy_dots_index_past_the_end_panics_before_reading() {
    r_test_utils::with_r_thread(|| {
        let parts = eval(FRAME);
        let frame = parts.get().vector_elt(0);
        let dots = unsafe { LazyDots::from_wrapper_arg(&frame) };
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dots.expr(4)))
            .expect_err("index 4 is past the end");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .unwrap_or_default();
        assert_eq!(
            message,
            "index out of bounds: `...` has 4 elements but the index is 4"
        );
    });
}

#[test]
fn lazy_dots_empty_and_forwarded() {
    r_test_utils::with_r_thread(|| {
        let frame = eval("(function(...) environment())()");
        let empty = frame.get();
        let dots = unsafe { LazyDots::from_wrapper_arg(&empty) };
        assert_eq!(dots.len(), 0);
        assert!(dots.is_empty());
        assert!(dots.names().is_empty());

        // `c(...)` in the frame gets each element as the promise R matched.
        let frame = eval("(function(...) environment())(1, b = 2)");
        let two = frame.get();
        let dots = unsafe { LazyDots::from_wrapper_arg(&two) };
        assert_eq!(dots.names(), [None, Some("b")]);
        let scope = unsafe { ProtectScope::new() };
        let joined = dots
            .try_forward(unsafe { RCall::new("c") }.arg_value(0.5))
            .expect("`c(0.5, ...)` evaluates");
        let joined = unsafe { scope.protect(joined) }.get();
        assert!(identical(joined, "c(0.5, 1, b = 2)"), "c(0.5, ...)");

        let frame = eval("(function(...) environment())(stop('nope'))");
        let failing = frame.get();
        let dots = unsafe { LazyDots::from_wrapper_arg(&failing) };
        let error = dots
            .try_forward(unsafe { RCall::new("c") })
            .expect_err("forcing `stop('nope')` fails");
        assert_eq!(error.message(), "nope");
    });
}

// endregion

// region: WrapAsS3

/// A meter reading, an external pointer with no impl block of its own.
#[derive(miniextendr_api::ExternalPtr)]
pub struct Meter {
    reading: f64,
}

/// A meter returned in its S3 class.
#[miniextendr]
pub fn marker_meter(reading: f64) -> WrapAsS3<Meter> {
    WrapAsS3(Meter { reading })
}

/// A meter for a non-negative reading, else `NULL`.
#[miniextendr]
pub fn marker_meter_or_null(reading: f64) -> Option<WrapAsS3<Meter>> {
    (reading >= 0.0).then(|| WrapAsS3::new(Meter { reading }))
}

/// A value whose conversion fails above 10.
struct Capped(i32);

impl IntoR for Capped {
    type Error = String;

    fn try_into_sexp(self) -> Result<SEXP, String> {
        if self.0 > 10 {
            Err(format!("{} is over 10", self.0))
        } else {
            Ok(self.0.into_sexp())
        }
    }
}

#[test]
fn wrap_as_s3_is_a_transparent_value() {
    let mut wrapped = WrapAsS3::new(vec![1, 2]);
    wrapped.push(3);
    assert_eq!(wrapped.len(), 3);
    assert_eq!(*wrapped, [1, 2, 3]);
    assert_eq!(WrapAsS3::from(vec![1, 2, 3]), wrapped);
    assert_eq!(wrapped.into_inner(), [1, 2, 3]);
    assert_eq!(size_of::<WrapAsS3<u64>>(), size_of::<u64>());
}

#[test]
fn wrap_as_s3_converts_the_payload_unchanged() {
    r_test_utils::with_r_thread(|| {
        // No class from Rust: the R wrapper adds it.
        let value = unsafe { OwnedProtect::new(WrapAsS3(vec![1, 2]).into_sexp()) };
        assert!(identical(value.get(), "1:2"));
        // The payload's error is the marker's.
        let error: Result<SEXP, String> = WrapAsS3(Capped(11)).try_into_sexp();
        assert_eq!(error.unwrap_err(), "11 is over 10");
        let ok = unsafe { OwnedProtect::new(WrapAsS3(Capped(4)).try_into_sexp().unwrap()) };
        assert!(identical(ok.get(), "4L"));
    });
}

#[test]
fn wrap_as_s3_return_is_classed_by_the_r_wrapper() {
    let wrapper = r_wrapper("marker_meter");
    // The class is the S3 one, named through the write-time class registry.
    assert!(
        wrapper.contains("structure(") && wrapper.contains(".__MX_CLASS_REF_OR_BARE_Meter__"),
        "{wrapper}"
    );
    let optional = r_wrapper("marker_meter_or_null");
    assert!(optional.contains("if (is.null("), "{optional}");

    r_test_utils::with_r_thread(|| unsafe {
        let reading = OwnedProtect::new(2.5_f64.into_sexp());
        let value = OwnedProtect::new(call1("_marker_meter", reading.get()));
        assert_eq!(value.get().type_of(), SEXPTYPE::EXTPTRSXP);
        let meter = ExternalPtr::<Meter>::wrap_sexp(value.get()).expect("a `Meter`");
        assert_eq!(meter.borrow().reading, 2.5);

        let negative = OwnedProtect::new((-1.0_f64).into_sexp());
        assert!(call1("_marker_meter_or_null", negative.get()).is_nil());
        let value = OwnedProtect::new(call1("_marker_meter_or_null", reading.get()));
        let meter = ExternalPtr::<Meter>::wrap_sexp(value.get()).expect("a `Meter`");
        assert_eq!(meter.borrow().reading, 2.5);
    });
}

// endregion
