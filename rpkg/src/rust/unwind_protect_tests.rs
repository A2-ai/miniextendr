//! Tests for `with_r_unwind_protect_or_raise` mechanism.

use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::unwind_protect::with_r_unwind_protect_or_raise;

/// Simple RAII type that prints when dropped (without using with_r to avoid deadlocks).
/// This is used across multiple test modules.
pub(crate) struct SimpleDropMsg(pub &'static str);

impl Drop for SimpleDropMsg {
    fn drop(&mut self) {
        eprintln!("[Rust] Dropped: {}", self.0);
    }
}

/// Test with_r_unwind_protect_or_raise normal execution with RAII resource cleanup.
/// @name rpkg_unwind_protect
/// @examples
/// \dontrun{
/// unsafe_C_unwind_protect_normal()
/// unsafe_C_unwind_protect_r_error()
/// unsafe_C_unwind_protect_lowlevel_test()
/// }
/// @aliases unsafe_C_unwind_protect_normal unsafe_C_unwind_protect_r_error
///   unsafe_C_unwind_protect_lowlevel_test
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_unwind_protect_normal() -> SEXP {
    with_r_unwind_protect_or_raise(
        || {
            let _a = SimpleDropMsg("stack resource");
            let _b = Box::new(SimpleDropMsg("heap resource"));
            ::miniextendr_api::SEXP::scalar_integer(42)
        },
        None,
    )
}

/// Test with_r_unwind_protect_or_raise cleanup when an R error is triggered.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_unwind_protect_r_error() -> SEXP {
    // Create resources BEFORE the protected region
    let a = SimpleDropMsg("captured resource 1");
    let b = Box::new(SimpleDropMsg("captured resource 2 (boxed)"));

    with_r_unwind_protect_or_raise(
        move || {
            // Access resources without moving them out of closure's captured state
            eprintln!("[Rust] Inside closure, using captured resources");
            eprintln!("[Rust] a.0 = {}", a.0);
            eprintln!("[Rust] b.0 = {}", b.0);

            // Now trigger R error - cleanup should drop a and b.
            // a and b are captured by the `move` closure, so they remain
            // alive at this point. Rf_error diverges (returns !).
            unsafe {
                // mxl::allow(MXL300)
                ::miniextendr_api::sys::Rf_error(
                    c"%s".as_ptr(),
                    c"intentional R error for testing".as_ptr(),
                )
            }
        },
        None,
    )
}

/// Test low-level with_r_unwind_protect_or_raise that triggers an R error directly.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_unwind_protect_lowlevel_test() -> SEXP {
    eprintln!("[Rust] Starting low-level unwind protect test");
    unsafe {
        with_r_unwind_protect_or_raise(
            || {
                eprintln!("[Rust] Inside protected function, about to trigger R error");
                // mxl::allow(MXL300)
                ::miniextendr_api::sys::Rf_error(c"%s".as_ptr(), c"test R error".as_ptr())
            },
            None,
        )
    }
}

// region: One condition whichever transport raises it (#1768, #1315)

/// Raise the condition `what` names: `"error"` (`error!` with a class and a
/// data field), `"arg"` (`arg_error!`), `"panic"`, `"warning"` (`warning!`),
/// or `"reserved"`, a hand-built payload whose data fields are named `kind`
/// and `message`. The macros reject those names, but a payload built by hand
/// reaches the transports with them, and both must splice them the same way
/// (#1315).
fn raise_transport_condition(what: &str) -> i32 {
    match what {
        "error" => miniextendr_api::rust_error!(
            class = "pkg_transport",
            data = { step = 1 },
            "transport error"
        ),
        "arg" => miniextendr_api::arg_error!(param = "what", "'what' is not a choice"),
        "panic" => panic!("transport panic"),
        "warning" => miniextendr_api::warning!("transport warning"),
        "reserved" => std::panic::panic_any(miniextendr_api::RCondition::Error {
            message: "base message".to_string(),
            class: vec!["pkg_transport".to_string()],
            data: Some(vec![
                ("kind".to_string(), "user kind".into()),
                ("message".to_string(), "user message".into()),
                ("step".to_string(), 2i32.into()),
            ]),
            call: miniextendr_api::condition::ConditionCall::Inherit,
        }),
        _ => 0,
    }
}

/// Raise the condition `what` names either as the generated wrapper does,
/// from the tagged value the body returns, or through
/// `with_r_unwind_protect_or_raise`, the raising guard ALTREP `RUnwind`
/// callbacks use, given the wrapper's call. Both give one condition (#1768).
/// @param what Which condition: `"error"` (a classed error with a data field),
///   `"arg"` (an argument error), `"panic"`, `"warning"`, or `"reserved"` (data
///   fields named `kind` and `message`); anything else returns `0L`.
/// @param guard Whether to raise through the raising guard.
/// @return `0L` when `what` names no condition.
/// @export
#[miniextendr]
pub fn transport_condition(what: &str, guard: bool, call: miniextendr_api::Call) -> i32 {
    if guard {
        with_r_unwind_protect_or_raise(|| raise_transport_condition(what), Some(call.sexp()))
    } else {
        raise_transport_condition(what)
    }
}

// endregion
