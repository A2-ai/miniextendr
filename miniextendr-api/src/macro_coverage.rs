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

#[miniextendr]
#[unsafe(no_mangle)]
pub(crate) extern "C" fn C_coverage_direct() -> ffi::SEXP {
    unsafe { ffi::R_NilValue }
}

#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub(crate) extern "C" fn C_coverage_indirect() -> ffi::SEXP {
    unsafe { ffi::R_NilValue }
}

miniextendr_module! {
    mod macro_coverage;
    use nested;

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
    fn coverage_named_dots;
    fn coverage_unused_named_dots;
    fn coverage_unnamed_dots;
    fn coverage_argument_plus_dots;
    fn coverage_argument_plus_unused_dots;
    fn coverage_argument_plus_unnamed_dots;
    fn coverage_invisible_option;
    fn coverage_invisible_result;
    fn coverage_panic_path;

    extern "C" fn C_coverage_direct;
    extern fn C_coverage_indirect;

    struct ShowcaseStruct;
}

mod nested {
    use crate::miniextendr_module;

    miniextendr_module! {
        mod nested;
    }
}
