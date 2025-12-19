#![allow(dead_code)]

//! Internal module that instantiates every macro path so `cargo expand`
//! can be used as a living catalog of what gets generated.

extern crate self as miniextendr_api;
use crate::ffi;
use crate::{miniextendr, miniextendr_module};

#[derive(Debug)]
struct DropTracer(&'static str);

impl Drop for DropTracer {
    fn drop(&mut self) {
        let _ = self.0;
    }
}

#[miniextendr]
pub(crate) fn coverage_no_return() {
    let _guard = DropTracer("no_return");
}

#[miniextendr]
#[allow(clippy::unused_unit)]
pub(crate) fn coverage_unit_return() -> () {}

#[miniextendr]
pub(crate) fn coverage_option_unit_selector(flag: i32) -> Option<()> {
    if flag % 2 == 0 { Some(()) } else { None }
}

#[miniextendr]
pub(crate) fn coverage_option_value(flag: i32, _unit: ()) -> Option<i32> {
    Some(flag.abs())
}

#[miniextendr]
pub(crate) fn coverage_result_unit(flag: i32) -> Result<(), &'static str> {
    if flag < 0 {
        Err("flag must be non-negative")
    } else {
        Ok(())
    }
}

#[miniextendr]
pub(crate) fn coverage_result_value(left: i32, right: i32) -> Result<i32, ()> {
    left.checked_add(right).ok_or(())
}

#[miniextendr]
pub(crate) fn coverage_plain_value(mut base: i32, increment: i32) -> i32 {
    base += increment;
    base
}

#[miniextendr]
pub(crate) fn coverage_mut_argument(mut counter: i32) -> i32 {
    counter += 1;
    counter
}

#[miniextendr]
pub(crate) fn coverage_mut_and_const(mut value: i32, delta: i32) -> i32 {
    value += delta;
    value
}

#[miniextendr]
pub(crate) fn coverage_reads_unit_argument(_unit: ()) -> i32 {
    7
}

#[miniextendr]
pub(crate) fn coverage_leading_underscore_arg(_hidden: i32) -> i32 {
    _hidden
}

#[miniextendr]
pub(crate) fn coverage_named_dots(dots: ...) {
    let _ = dots.inner;
}

#[miniextendr]
pub(crate) fn coverage_unused_named_dots(_dots: ...) {}

#[miniextendr]
pub(crate) fn coverage_unnamed_dots(...) {}

#[miniextendr]
pub(crate) fn coverage_argument_plus_dots(_count: i32, dots: ...) {
    let _ = dots.inner;
}

#[miniextendr]
pub(crate) fn coverage_argument_plus_unused_dots(_count: i32, _dots: ...) {}

#[miniextendr]
pub(crate) fn coverage_argument_plus_unnamed_dots(_count: i32, ...) {}

#[miniextendr]
pub(crate) fn coverage_invisible_option() -> Option<()> {
    Some(())
}

#[miniextendr]
pub(crate) fn coverage_invisible_result() -> Result<(), ()> {
    Ok(())
}

#[miniextendr]
pub(crate) fn coverage_panic_path() -> i32 {
    let _ = DropTracer("panic");
    panic!("macro coverage panic branch");
}

// =============================================================================
// Attribute option coverage
// =============================================================================

/// Coverage: `#[miniextendr(invisible)]` forces invisible return
#[miniextendr(invisible)]
pub(crate) fn coverage_attr_invisible() -> i32 {
    42
}

/// Coverage: `#[miniextendr(visible)]` forces visible return (overrides default invisible for unit)
#[miniextendr(visible)]
pub(crate) fn coverage_attr_visible() {}

/// Coverage: `#[miniextendr(check_interrupt)]` inserts R_CheckUserInterrupt before call
#[miniextendr(check_interrupt)]
pub(crate) fn coverage_attr_check_interrupt(n: i32) -> i32 {
    n * 2
}

/// Coverage: `#[miniextendr(unsafe(main_thread))]` forces main thread execution
#[miniextendr(unsafe(main_thread))]
pub(crate) fn coverage_attr_main_thread() -> i32 {
    1
}

/// Coverage: multiple attributes combined
#[miniextendr(invisible, check_interrupt)]
pub(crate) fn coverage_attr_combined() -> i32 {
    99
}

// =============================================================================
// Coercion coverage - scalar types
// =============================================================================

/// Coverage: `#[miniextendr(coerce)]` global - u16 from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_u16(x: u16) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - i16 from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_i16(x: i16) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - i8 from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_i8(x: i8) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - u32 from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_u32(x: u32) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - u64 from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_u64(x: u64) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - i64 from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_i64(x: i64) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - isize from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_isize(x: isize) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - usize from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_usize(x: usize) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - bool from i32
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_bool(x: bool) -> i32 {
    x as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - f32 from f64
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_scalar_f32(x: f32) -> f64 {
    x as f64
}

// =============================================================================
// Coercion coverage - Vec types
// =============================================================================

/// Coverage: `#[miniextendr(coerce)]` global - Vec<u16> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_u16(x: Vec<u16>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<i16> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_i16(x: Vec<i16>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<i8> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_i8(x: Vec<i8>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<u32> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_u32(x: Vec<u32>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<u64> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_u64(x: Vec<u64>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<i64> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_i64(x: Vec<i64>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<isize> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_isize(x: Vec<isize>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<usize> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_usize(x: Vec<usize>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<bool> from &[i32]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_bool(x: Vec<bool>) -> i32 {
    x.len() as i32
}

/// Coverage: `#[miniextendr(coerce)]` global - Vec<f32> from &[f64]
#[miniextendr(coerce)]
pub(crate) fn coverage_coerce_vec_f32(x: Vec<f32>) -> i32 {
    x.len() as i32
}

// =============================================================================
// Per-parameter coercion coverage
// =============================================================================

/// Coverage: per-param `#[miniextendr(coerce)]` - only first param coerced
#[miniextendr]
pub(crate) fn coverage_coerce_per_param(#[miniextendr(coerce)] x: u16, y: i32) -> i32 {
    x as i32 + y
}

/// Coverage: per-param coerce on multiple params
#[miniextendr]
pub(crate) fn coverage_coerce_per_param_multiple(
    #[miniextendr(coerce)] a: u16,
    b: i32,
    #[miniextendr(coerce)] c: bool,
) -> i32 {
    a as i32 + b + c as i32
}

// =============================================================================
// Wildcard parameter coverage
// =============================================================================

/// Coverage: wildcard `_` parameter gets renamed to `__unused0`
#[miniextendr]
pub(crate) fn coverage_wildcard_single(_: i32) -> i32 {
    1
}

/// Coverage: multiple wildcards get sequential names
#[miniextendr]
pub(crate) fn coverage_wildcard_multiple(_: i32, _: f64) -> i32 {
    2
}

/// Coverage: wildcard with coerce attribute
#[miniextendr]
pub(crate) fn coverage_wildcard_with_coerce(#[miniextendr(coerce)] _: u16) -> i32 {
    3
}

#[miniextendr]
#[unsafe(no_mangle)]
pub(crate) extern "C-unwind" fn C_coverage_direct() -> ffi::SEXP {
    unsafe { ffi::R_NilValue }
}

#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) extern "C-unwind" fn C_coverage_indirect() -> ffi::SEXP {
    unsafe { ffi::R_NilValue }
}

miniextendr_module! {
    mod macro_coverage;
    use nested;

    // Basic return types
    fn coverage_no_return;
    fn coverage_unit_return;
    fn coverage_option_unit_selector;
    fn coverage_option_value;
    fn coverage_result_unit;
    fn coverage_result_value;
    fn coverage_plain_value;
    fn coverage_mut_argument;
    fn coverage_mut_and_const;
    fn coverage_reads_unit_argument;
    fn coverage_leading_underscore_arg;

    // Dots/variadic
    fn coverage_named_dots;
    fn coverage_unused_named_dots;
    fn coverage_unnamed_dots;
    fn coverage_argument_plus_dots;
    fn coverage_argument_plus_unused_dots;
    fn coverage_argument_plus_unnamed_dots;

    // Invisible return inference
    fn coverage_invisible_option;
    fn coverage_invisible_result;
    fn coverage_panic_path;

    // Attribute options
    fn coverage_attr_invisible;
    fn coverage_attr_visible;
    fn coverage_attr_check_interrupt;
    fn coverage_attr_main_thread;
    fn coverage_attr_combined;

    // Coercion - scalars
    fn coverage_coerce_scalar_u16;
    fn coverage_coerce_scalar_i16;
    fn coverage_coerce_scalar_i8;
    fn coverage_coerce_scalar_u32;
    fn coverage_coerce_scalar_u64;
    fn coverage_coerce_scalar_i64;
    fn coverage_coerce_scalar_isize;
    fn coverage_coerce_scalar_usize;
    fn coverage_coerce_scalar_bool;
    fn coverage_coerce_scalar_f32;

    // Coercion - vecs
    fn coverage_coerce_vec_u16;
    fn coverage_coerce_vec_i16;
    fn coverage_coerce_vec_i8;
    fn coverage_coerce_vec_u32;
    fn coverage_coerce_vec_u64;
    fn coverage_coerce_vec_i64;
    fn coverage_coerce_vec_isize;
    fn coverage_coerce_vec_usize;
    fn coverage_coerce_vec_bool;
    fn coverage_coerce_vec_f32;

    // Per-parameter coercion
    fn coverage_coerce_per_param;
    fn coverage_coerce_per_param_multiple;

    // Wildcard parameters
    fn coverage_wildcard_single;
    fn coverage_wildcard_multiple;
    fn coverage_wildcard_with_coerce;

    // Extern C functions
    extern "C-unwind" fn C_coverage_direct;
    extern fn C_coverage_indirect;

    // struct ShowcaseStruct; // ALTREP showcase intentionally omitted in coverage
}

mod nested {
    use crate::miniextendr_module;

    miniextendr_module! {
        mod nested;
    }
}
