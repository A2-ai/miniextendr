#![allow(
    dead_code,
    unused_variables,
    clippy::unused_unit,
    unused_mut,
    varargs_without_pattern,
    varargs_without_pattern
)]

//! Function-level macro option coverage.
//!
//! Every `MiniextendrFnAttrs` option has at least one atomic fixture,
//! and common combinations are also covered.

use crate::ffi::{self, SEXP};
use crate::miniextendr;

#[derive(Debug)]
struct DropTracer(&'static str);

impl Drop for DropTracer {
    fn drop(&mut self) {
        let _ = self.0;
    }
}

// region: Basic return type variants

#[miniextendr]
pub(crate) fn cov_fn_no_return() {
    let _guard = DropTracer("no_return");
}

#[miniextendr]
pub(crate) fn cov_fn_unit_return() -> () {}

#[miniextendr]
pub(crate) fn cov_fn_option_unit(flag: i32) -> Option<()> {
    if flag % 2 == 0 { Some(()) } else { None }
}

#[miniextendr]
pub(crate) fn cov_fn_option_value(flag: i32, _unit: ()) -> Option<i32> {
    Some(flag.abs())
}

#[miniextendr]
pub(crate) fn cov_fn_result_unit(flag: i32) -> Result<(), &'static str> {
    if flag < 0 {
        Err("flag must be non-negative")
    } else {
        Ok(())
    }
}

#[miniextendr]
pub(crate) fn cov_fn_result_value(left: i32, right: i32) -> Result<i32, ()> {
    left.checked_add(right).ok_or(())
}

#[miniextendr]
pub(crate) fn cov_fn_plain_value(mut base: i32, increment: i32) -> i32 {
    base += increment;
    base
}

#[miniextendr]
pub(crate) fn cov_fn_mut_argument(mut counter: i32) -> i32 {
    counter += 1;
    counter
}

#[miniextendr]
pub(crate) fn cov_fn_reads_unit(_unit: ()) -> i32 {
    7
}

#[miniextendr]
pub(crate) fn cov_fn_leading_underscore(_hidden: i32) -> i32 {
    _hidden
}

#[miniextendr]
pub(crate) fn cov_fn_panic_path() -> i32 {
    let _ = DropTracer("panic");
    panic!("macro coverage panic branch");
}
// endregion

// region: Dots / variadic parameter coverage

#[miniextendr]
pub(crate) fn cov_fn_named_dots(dots: ...) {
    let _ = dots.inner;
}

#[miniextendr]
pub(crate) fn cov_fn_unused_named_dots(_dots: ...) {}

#[miniextendr]
pub(crate) fn cov_fn_unnamed_dots(_dots: ...) {}

#[miniextendr]
pub(crate) fn cov_fn_arg_plus_dots(_count: i32, dots: ...) {
    let _ = dots.inner;
}

#[miniextendr]
pub(crate) fn cov_fn_arg_plus_unnamed_dots(_count: i32, _dots: ...) {}
// endregion

// region: Invisible / visible return

#[miniextendr]
pub(crate) fn cov_fn_invisible_option() -> Option<()> {
    Some(())
}

#[miniextendr]
pub(crate) fn cov_fn_invisible_result() -> Result<(), ()> {
    Ok(())
}
// endregion

// region: Attribute option: atomic coverage (one per option)

#[miniextendr(invisible)]
pub(crate) fn cov_fn_attr_invisible() -> i32 {
    42
}

#[miniextendr(visible)]
pub(crate) fn cov_fn_attr_visible() {}

#[miniextendr(check_interrupt)]
pub(crate) fn cov_fn_attr_check_interrupt(n: i32) -> i32 {
    n * 2
}

#[miniextendr(unsafe(main_thread))]
pub(crate) fn cov_fn_attr_main_thread() -> i32 {
    1
}

#[miniextendr(worker)]
pub(crate) fn cov_fn_attr_worker(x: i32) -> i32 {
    x
}

#[miniextendr(rng)]
pub(crate) fn cov_fn_attr_rng(x: i32) -> i32 {
    x
}

#[miniextendr(unwrap_in_r, no_error_in_r)]
pub(crate) fn cov_fn_attr_unwrap_in_r(x: i32) -> Result<i32, &'static str> {
    Ok(x)
}

#[miniextendr(lifecycle = "deprecated")]
pub(crate) fn cov_fn_lifecycle_simple(x: i32) -> i32 {
    x
}

#[miniextendr(lifecycle(stage = "deprecated", when = "0.9.0", with = "cov_fn_attr_worker()"))]
pub(crate) fn cov_fn_lifecycle_full(x: i32) -> i32 {
    x
}

#[miniextendr]
pub(crate) fn cov_fn_param_default(#[miniextendr(default = "1L")] x: i32) -> i32 {
    x
}
// endregion

// region: Attribute option: combinations

#[miniextendr(worker, invisible)]
pub(crate) fn cov_combo_worker_invisible(x: i32) -> i32 {
    x
}

#[miniextendr(worker, visible)]
pub(crate) fn cov_combo_worker_visible() {}

#[miniextendr(worker, coerce)]
pub(crate) fn cov_combo_worker_coerce(x: u16) -> i32 {
    x as i32
}

#[miniextendr(worker, rng)]
pub(crate) fn cov_combo_worker_rng(x: i32) -> i32 {
    x
}

#[miniextendr(worker, unwrap_in_r, no_error_in_r)]
pub(crate) fn cov_combo_worker_unwrap(x: i32) -> Result<i32, &'static str> {
    Ok(x)
}

#[miniextendr(unsafe(main_thread), check_interrupt)]
pub(crate) fn cov_combo_mainthread_interrupt(x: i32) -> i32 {
    x
}

#[miniextendr(unsafe(main_thread), visible, check_interrupt)]
pub(crate) fn cov_combo_mainthread_visible_interrupt() {}

#[miniextendr(invisible, check_interrupt)]
pub(crate) fn cov_combo_invisible_interrupt() -> i32 {
    99
}
// endregion

// region: Coercion coverage - scalars

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_u16(x: u16) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_i16(x: i16) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_i8(x: i8) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_u32(x: u32) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_u64(x: u64) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_i64(x: i64) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_isize(x: isize) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_usize(x: usize) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_bool(x: bool) -> i32 {
    x as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_f32(x: f32) -> f64 {
    x as f64
}
// endregion

// region: Coercion coverage - Vec types

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_vec_u16(x: Vec<u16>) -> i32 {
    x.len() as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_vec_bool(x: Vec<bool>) -> i32 {
    x.len() as i32
}

#[miniextendr(coerce)]
pub(crate) fn cov_coerce_vec_f32(x: Vec<f32>) -> i32 {
    x.len() as i32
}
// endregion

// region: Per-parameter coercion

#[miniextendr]
pub(crate) fn cov_coerce_per_param(#[miniextendr(coerce)] x: u16, y: i32) -> i32 {
    x as i32 + y
}

#[miniextendr]
pub(crate) fn cov_coerce_per_param_multiple(
    #[miniextendr(coerce)] a: u16,
    b: i32,
    #[miniextendr(coerce)] c: bool,
) -> i32 {
    a as i32 + b + c as i32
}
// endregion

// region: Wildcard parameters

#[miniextendr]
pub(crate) fn cov_wildcard_single(_: i32) -> i32 {
    1
}

#[miniextendr]
pub(crate) fn cov_wildcard_multiple(_: i32, _: f64) -> i32 {
    2
}

#[miniextendr]
pub(crate) fn cov_wildcard_with_coerce(#[miniextendr(coerce)] _: u16) -> i32 {
    3
}
// endregion

// region: Inline attribute preservation

#[miniextendr]
#[inline(always)]
pub fn cov_explicit_inline_always() -> i32 {
    42
}

#[miniextendr]
#[inline]
pub fn cov_explicit_inline() -> i32 {
    43
}
// endregion

// region: Extern C function variants

#[miniextendr]
#[unsafe(no_mangle)]
pub(crate) extern "C-unwind" fn C_cov_direct() -> ffi::SEXP {
    SEXP::null()
}

#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) extern "C-unwind" fn C_cov_indirect() -> ffi::SEXP {
    SEXP::null()
}
// endregion

// region: S3 method coverage

#[miniextendr(s3(generic = "format", class = "cov_s3_type"))]
pub(crate) fn cov_fn_s3_method(x: i32) -> String {
    format!("cov_s3: {x}")
}
// endregion
