use miniextendr_api::ffi::{R_NilValue, Rf_error, SEXP};
use miniextendr_api::{miniextendr, miniextendr_module};

use miniextendr_api::unwind_protect::with_r_unwind_protect;

// region

#[derive(Debug)]
struct MsgOnDrop;

impl Drop for MsgOnDrop {
    fn drop(&mut self) {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "[Rust] Dropped `MsgOnDrop`!").unwrap();
        stdout.flush().unwrap();
    }
}

#[miniextendr]
fn drop_message_on_success() -> i32 {
    let _a = MsgOnDrop;
    42
}

#[miniextendr]
fn drop_on_panic() {
    let _a = MsgOnDrop;
    panic!()
}

#[miniextendr]
fn drop_on_panic_with_move() {
    let _a = MsgOnDrop;
    unsafe {
        Rf_error(c"%s".as_ptr(), c"an r error occurred".as_ptr());
    }
}

// endregion

// region: panics, (), and Result
#[miniextendr]
#[allow(clippy::unused_unit)]
fn take_and_return_nothing() -> () {}

#[miniextendr]
fn add(left: i32, right: i32) -> i32 {
    left + right
}

#[miniextendr]
fn add2(left: i32, right: i32, _dummy: ()) -> i32 {
    left + right
}

#[miniextendr]
fn add3(left: i32, right: i32, _dummy: ()) -> Result<i32, ()> {
    left.checked_add(right).ok_or(())
}

#[miniextendr]
fn add4(left: i32, right: i32) -> Result<i32, &'static str> {
    left.checked_div(right).ok_or("don't divide by zero dude")
}

#[miniextendr]
fn add_panic(_left: i32, _right: i32) -> i32 {
    let _a = MsgOnDrop;
    panic!("we cannot add right now! ");
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

#[miniextendr]
fn add_r_error(_left: i32, _right: i32) -> i32 {
    let _a = MsgOnDrop;
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"r error in `add_r_error`".as_ptr())
    };
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

#[miniextendr]
fn add_panic_heap(_left: i32, _right: i32) -> i32 {
    let _a = Box::new(MsgOnDrop);
    panic!("we cannot add right now! ")
}

#[miniextendr]
fn add_r_error_heap(_left: i32, _right: i32) -> i32 {
    let _a = Box::new(MsgOnDrop);
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"r error in `add_r_error`".as_ptr())
    }
}

// endregion

// region: with_r_unwind_protect tests

/// Simple RAII type that prints when dropped (without using with_r to avoid deadlocks)
struct SimpleDropMsg(&'static str);
impl Drop for SimpleDropMsg {
    fn drop(&mut self) {
        eprintln!("[Rust] Dropped: {}", self.0);
    }
}

/// Test that with_r_unwind_protect works for normal (non-error) path.
/// Destructors should run normally when the closure completes successfully.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C-unwind" fn C_unwind_protect_normal() -> SEXP {
    with_r_unwind_protect(
        || {
            let _a = SimpleDropMsg("stack resource");
            let _b = Box::new(SimpleDropMsg("heap resource"));
            unsafe { ::miniextendr_api::ffi::Rf_ScalarInteger(42) }
        },
        None,
    )
}

/// Test that with_r_unwind_protect cleans up on R error.
/// Resources captured by the closure ARE dropped when an R error occurs.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C-unwind" fn C_unwind_protect_r_error() -> SEXP {
    // Create resources BEFORE the protected region
    let a = SimpleDropMsg("captured resource 1");
    let b = Box::new(SimpleDropMsg("captured resource 2 (boxed)"));

    with_r_unwind_protect(
        move || {
            // Access resources without moving them out of closure's captured state
            eprintln!("[Rust] Inside closure, using captured resources");
            eprintln!("[Rust] a.0 = {}", a.0);
            eprintln!("[Rust] b.0 = {}", b.0);

            // Now trigger R error - cleanup should drop a and b
            unsafe {
                ::miniextendr_api::ffi::Rf_error(
                    c"%s".as_ptr(),
                    c"intentional R error for testing".as_ptr(),
                )
            };
            #[allow(unreachable_code)]
            unsafe {
                // This is never reached, but we need to "use" a and b
                // to prevent the compiler from moving them earlier
                drop(a);
                drop(b);
                ::miniextendr_api::ffi::R_NilValue
            }
        },
        None,
    )
}

/// Minimal test using low-level with_unwind_protect
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C-unwind" fn C_unwind_protect_lowlevel_test() -> SEXP {
    eprintln!("[Rust] Starting low-level unwind protect test");
    unsafe {
        with_r_unwind_protect(
            || {
                eprintln!("[Rust] Inside protected function, about to trigger R error");
                ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"test R error".as_ptr());
                #[allow(unreachable_code)]
                ::miniextendr_api::ffi::R_NilValue
            },
            None,
        )
    }
}

// endregion

// region: `mut` checks

#[miniextendr]
fn add_left_mut(mut left: i32, right: i32) -> i32 {
    let left = &mut left;
    *left + right
}

#[miniextendr]
fn add_right_mut(left: i32, right: i32) -> i32 {
    left + right
}

#[miniextendr]
fn add_left_right_mut(left: i32, right: i32) -> i32 {
    left + right
}

// endregion

// region: panic printing

#[unsafe(no_mangle)]
#[miniextendr]
extern "C-unwind" fn C_just_panic() -> ::miniextendr_api::ffi::SEXP {
    panic!("just panic, no capture");
}

/// If you call a miniextendr function that panics, and then `C_panic_catch`,
/// you'll see that the panic hook was not reset.
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_panic_and_catch() -> ::miniextendr_api::ffi::SEXP {
    let result = std::panic::catch_unwind(|| panic!("just panic, no capture"));
    let _ = dbg!(result);
    unsafe { ::miniextendr_api::ffi::R_NilValue }
}

#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_r_error() -> ::miniextendr_api::ffi::SEXP {
    // Use unchecked - this is testing raw R error behavior
    unsafe { miniextendr_api::ffi::Rf_error_unchecked(c"arg1".as_ptr()) }
}

#[miniextendr]
#[allow(non_snake_case)]
#[allow(clippy::diverging_sub_expression)]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_r_error_in_catch() -> ::miniextendr_api::ffi::SEXP {
    unsafe {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Use unchecked - this is testing raw R error behavior
            miniextendr_api::ffi::Rf_error_unchecked(c"arg1".as_ptr())
        }))
        .unwrap();
        miniextendr_api::ffi::R_NilValue
    }
}

/// This crashes immediately. R is simply not present on the spawned thread, hence the present segfault.
/// With the checked `Rf_error`, this would panic instead (which is the correct behavior).
#[miniextendr]
#[allow(non_snake_case)]
#[allow(clippy::diverging_sub_expression)]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_r_error_in_thread() -> ::miniextendr_api::ffi::SEXP {
    // Use checked Rf_error - will panic with clear message about wrong thread
    std::thread::spawn(|| unsafe {
        miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"arg1".as_ptr())
    })
    .join()
    .unwrap();
    unsafe { miniextendr_api::ffi::R_NilValue }
}

/// This will segfault, as R is not present on the spawned thread.
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_r_print_in_thread() -> ::miniextendr_api::ffi::SEXP {
    std::thread::spawn(|| unsafe { miniextendr_api::ffi::Rprintf_unchecked(c"arg1".as_ptr()) })
        .join()
        .unwrap();
    unsafe { miniextendr_api::ffi::R_NilValue }
}

// endregion

// region: dots

#[miniextendr]
fn greetings_with_named_dots(dots: ...) {
    let _ = dots;
}

#[miniextendr]
fn greetings_with_named_and_unused_dots(_dots: ...) {}

#[miniextendr]
fn greetings_with_nameless_dots(...) {}

// LIMITATION: Good!
// #[miniextendr]
// fn greetings_with_dots_then_arg(dots: ..., exclamations: i32) {}

#[miniextendr]
fn greetings_last_as_named_and_unused_dots(_exclamations: i32, _dots: ...) {}

#[miniextendr]
fn greetings_last_as_named_dots(_exclamations: i32, dots: ...) {
    let _ = dots;
}

#[miniextendr]
fn greetings_last_as_nameless_dots(_exclamations: i32, ...) {}

// endregion

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C-unwind" fn C_check_interupt_after() -> SEXP {
    use miniextendr_api::ffi::R_CheckUserInterrupt;

    std::thread::sleep(std::time::Duration::from_secs(2));

    unsafe {
        R_CheckUserInterrupt();
        R_NilValue
    }
}

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
extern "C-unwind" fn C_check_interupt_unwind() -> SEXP {
    use miniextendr_api::ffi::R_CheckUserInterrupt;

    std::thread::sleep(std::time::Duration::from_secs(2));

    unsafe {
        with_r_unwind_protect(
            || {
                R_CheckUserInterrupt();
                R_NilValue
            },
            None,
        );
        R_NilValue
    }
}

// region: scalar conversion tests

// i32 tests
#[miniextendr]
fn test_i32_identity(x: i32) -> i32 {
    x
}

#[miniextendr]
fn test_i32_add_one(x: i32) -> i32 {
    x + 1
}

#[miniextendr]
fn test_i32_sum(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

// f64 tests
#[miniextendr]
fn test_f64_identity(x: f64) -> f64 {
    x
}

#[miniextendr]
fn test_f64_add_one(x: f64) -> f64 {
    x + 1.0
}

#[miniextendr]
fn test_f64_multiply(a: f64, b: f64) -> f64 {
    a * b
}

// u8 (raw) tests
#[miniextendr]
fn test_u8_identity(x: u8) -> u8 {
    x
}

#[miniextendr]
fn test_u8_add_one(x: u8) -> u8 {
    x.wrapping_add(1)
}

// Rboolean tests
#[miniextendr]
fn test_logical_identity(x: miniextendr_api::ffi::Rboolean) -> miniextendr_api::ffi::Rboolean {
    x
}

#[miniextendr]
fn test_logical_not(x: miniextendr_api::ffi::Rboolean) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    match x {
        Rboolean::TRUE => Rboolean::FALSE,
        _ => Rboolean::TRUE,
    }
}

#[miniextendr]
fn test_logical_and(
    a: miniextendr_api::ffi::Rboolean,
    b: miniextendr_api::ffi::Rboolean,
) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    match (a, b) {
        (Rboolean::TRUE, Rboolean::TRUE) => Rboolean::TRUE,
        _ => Rboolean::FALSE,
    }
}

// Mixed type tests
#[miniextendr]
fn test_i32_to_f64(x: i32) -> f64 {
    x as f64
}

#[miniextendr]
fn test_f64_to_i32(x: f64) -> i32 {
    x as i32
}

// Slice tests - i32
#[miniextendr]
fn test_i32_slice_len(x: &'static [i32]) -> i32 {
    x.len() as i32
}

#[miniextendr]
fn test_i32_slice_sum(x: &'static [i32]) -> i32 {
    x.iter().sum()
}

#[miniextendr]
fn test_i32_slice_first(x: &'static [i32]) -> i32 {
    x.first().copied().unwrap_or(0)
}

#[miniextendr]
fn test_i32_slice_last(x: &'static [i32]) -> i32 {
    x.last().copied().unwrap_or(0)
}

// Slice tests - f64
#[miniextendr]
fn test_f64_slice_len(x: &'static [f64]) -> i32 {
    x.len() as i32
}

#[miniextendr]
fn test_f64_slice_sum(x: &'static [f64]) -> f64 {
    x.iter().sum()
}

#[miniextendr]
fn test_f64_slice_mean(x: &'static [f64]) -> f64 {
    if x.is_empty() {
        0.0
    } else {
        x.iter().sum::<f64>() / x.len() as f64
    }
}

// Slice tests - u8 (raw)
#[miniextendr]
fn test_u8_slice_len(x: &'static [u8]) -> i32 {
    x.len() as i32
}

#[miniextendr]
fn test_u8_slice_sum(x: &'static [u8]) -> i32 {
    x.iter().map(|&b| b as i32).sum()
}

// Slice tests - logical
#[miniextendr]
fn test_logical_slice_len(x: &'static [miniextendr_api::ffi::Rboolean]) -> i32 {
    x.len() as i32
}

#[miniextendr]
fn test_logical_slice_any_true(
    x: &'static [miniextendr_api::ffi::Rboolean],
) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    if x.contains(&Rboolean::TRUE) {
        Rboolean::TRUE
    } else {
        Rboolean::FALSE
    }
}

#[miniextendr]
fn test_logical_slice_all_true(
    x: &'static [miniextendr_api::ffi::Rboolean],
) -> miniextendr_api::ffi::Rboolean {
    use miniextendr_api::ffi::Rboolean;
    if x.iter().all(|&b| b == Rboolean::TRUE) {
        Rboolean::TRUE
    } else {
        Rboolean::FALSE
    }
}

// endregion

// region: miniextendr_module! tests

mod altrep;

// region: proc-macro ALTREP test
// This tests the #[miniextendr] on struct path for custom ALTREP classes.

use miniextendr_api::altrep_traits::{AltInteger, AltVec, Altrep};
use miniextendr_api::ffi::R_xlen_t;

/// A simple custom ALTREP integer class: always returns the constant 42.
#[miniextendr(class = "ConstantInt", pkg = "rpkg", base = "Int")]
pub struct ConstantIntClass;

// Implement the required traits for ConstantIntClass
impl Altrep for ConstantIntClass {
    const HAS_LENGTH: bool = true;
    fn length(_x: SEXP) -> R_xlen_t {
        // Always length 10
        10
    }
}

// Safety: ALTREP callbacks are only called by R with valid SEXP arguments
#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for ConstantIntClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        use miniextendr_api::ffi::{
            INTEGER, R_NilValue, R_altrep_data2, R_set_altrep_data2, Rf_allocVector, Rf_protect,
            Rf_unprotect, SEXPTYPE,
        };
        // Materialize the data if not already expanded
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                let n = Self::length(x);
                let val = Rf_allocVector(SEXPTYPE::INTSXP, n);
                Rf_protect(val);
                let buf = INTEGER(val);
                for i in 0..n {
                    *buf.offset(i) = Self::elt(x, i);
                }
                R_set_altrep_data2(x, val);
                Rf_unprotect(1);
                buf.cast()
            } else {
                INTEGER(expanded).cast()
            }
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        use miniextendr_api::ffi::{INTEGER, R_NilValue, R_altrep_data2};
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                core::ptr::null()
            } else {
                INTEGER(expanded).cast()
            }
        }
    }
}

impl AltInteger for ConstantIntClass {
    const HAS_ELT: bool = true;
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        // Every element is 42
        42
    }
}

/// Create a ConstantInt ALTREP instance (all elements are 42, length 10).
///
/// # Safety
///
/// Must be called from R main thread with R properly initialized.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_constant_int() -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::ffi::altrep::R_new_altrep;
    // Get the (already registered) class and create an instance
    let cls = ConstantIntClass::get_or_init_class();
    unsafe { R_new_altrep(cls, R_NilValue, R_NilValue) }
}

// endregion

// ALTREP .Call wrappers (delegating to miniextendr_api)
// Named with rpkg_ prefix to avoid symbol collision with miniextendr_api exports.

/// Create a compact integer sequence ALTREP.
///
/// # Safety
///
/// All SEXP arguments must be valid. Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_compact_int(n: SEXP, start: SEXP, step: SEXP) -> SEXP {
    unsafe { miniextendr_api::altrep::C_altrep_compact_int(n, start, step) }
}

/// Create an ALTREP from doubles.
///
/// # Safety
///
/// `x` must be a valid REALSXP. Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_doubles(x: SEXP) -> SEXP {
    unsafe { miniextendr_api::altrep::C_altrep_from_doubles(x) }
}

/// Create an ALTREP from strings.
///
/// # Safety
///
/// `x` must be a valid STRSXP. Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_strings(x: SEXP) -> SEXP {
    unsafe { miniextendr_api::altrep::C_altrep_from_strings(x) }
}

/// Create an ALTREP from logicals.
///
/// # Safety
///
/// `x` must be a valid LGLSXP. Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_logicals(x: SEXP) -> SEXP {
    unsafe { miniextendr_api::altrep::C_altrep_from_logicals(x) }
}

/// Create an ALTREP from raw bytes.
///
/// # Safety
///
/// `x` must be a valid RAWSXP. Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_raw(x: SEXP) -> SEXP {
    unsafe { miniextendr_api::altrep::C_altrep_from_raw(x) }
}

/// Create an ALTREP from a list.
///
/// # Safety
///
/// `x` must be a valid VECSXP. Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_list(x: SEXP) -> SEXP {
    unsafe { miniextendr_api::altrep::C_altrep_from_list(x) }
}

// region: ExternalPtr tests

use miniextendr_api::externalptr::ErasedExternalPtr;
// Note: ExternalPtr type is accessed via full path to avoid conflict with derive macro
use miniextendr_api::ExternalPtr as DeriveExternalPtr;

/// A simple test struct for ExternalPtr
#[derive(DeriveExternalPtr, Debug)]
struct Counter {
    value: i32,
}

/// Another test struct to verify type safety
#[derive(DeriveExternalPtr, Debug)]
struct Point {
    x: f64,
    y: f64,
}

/// Create a new Counter wrapped in an ExternalPtr
#[miniextendr]
fn extptr_counter_new(initial: i32) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    let counter = Counter { value: initial };
    let ptr = ExternalPtr::new(counter);
    ptr.as_sexp()
}

/// Get the current value from a Counter ExternalPtr
///
/// # Safety
///
/// `ptr` must be a valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_counter_get(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        match ExternalPtr::<Counter>::try_from_sexp(ptr) {
            Some(ext) => Rf_ScalarInteger(ext.value),
            None => Rf_ScalarInteger(i32::MIN), // NA_INTEGER equivalent
        }
    }
}

/// Increment the counter and return the new value
///
/// # Safety
///
/// `ptr` must be a valid SEXP pointing to a Counter ExternalPtr.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_counter_increment(ptr: SEXP) -> SEXP {
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        // Get mutable access via downcast_mut on erased pointer
        let mut erased = ErasedExternalPtr::from_sexp(ptr);
        if let Some(counter) = erased.downcast_mut::<Counter>() {
            counter.value += 1;
            return Rf_ScalarInteger(counter.value);
        }
        Rf_ScalarInteger(i32::MIN) // NA_INTEGER equivalent
    }
}

/// Create a new Point wrapped in an ExternalPtr
#[miniextendr]
fn extptr_point_new(x: f64, y: f64) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    let point = Point { x, y };
    let ptr = ExternalPtr::new(point);
    ptr.as_sexp()
}

/// Get the x coordinate from a Point ExternalPtr
///
/// # Safety
///
/// `ptr` must be a valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_point_get_x(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarReal;
    unsafe {
        match ExternalPtr::<Point>::try_from_sexp(ptr) {
            Some(ext) => Rf_ScalarReal(ext.x),
            None => Rf_ScalarReal(f64::NAN),
        }
    }
}

/// Get the y coordinate from a Point ExternalPtr
///
/// # Safety
///
/// `ptr` must be a valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_point_get_y(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarReal;
    unsafe {
        match ExternalPtr::<Point>::try_from_sexp(ptr) {
            Some(ext) => Rf_ScalarReal(ext.y),
            None => Rf_ScalarReal(f64::NAN),
        }
    }
}

/// Test type safety: try to get a Counter from a Point (should fail)
///
/// # Safety
///
/// `ptr` must be a valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_type_mismatch_test(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        // Try to interpret a Point as a Counter - should return None
        match ExternalPtr::<Counter>::try_from_sexp(ptr) {
            Some(_) => Rf_ScalarInteger(1),  // Unexpected success
            None => Rf_ScalarInteger(0),     // Expected failure - type mismatch
        }
    }
}

/// Test with R's `new("externalptr")` - a null external pointer
///
/// # Safety
///
/// `ptr` must be a valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_null_test(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        // R's new("externalptr") creates a null external pointer
        // Our try_from_sexp should return None for it
        match ExternalPtr::<Counter>::try_from_sexp(ptr) {
            Some(_) => Rf_ScalarInteger(1),  // Unexpected - null pointer should fail
            None => Rf_ScalarInteger(0),     // Expected - null pointer detected
        }
    }
}

/// Check if an external pointer is a Counter using ErasedExternalPtr
///
/// # Safety
///
/// `ptr` must be a valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_is_counter(ptr: SEXP) -> SEXP {
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        let erased = ErasedExternalPtr::from_sexp(ptr);
        if erased.is::<Counter>() {
            Rf_ScalarInteger(1)
        } else {
            Rf_ScalarInteger(0)
        }
    }
}

/// Check if an external pointer is a Point using ErasedExternalPtr
///
/// # Safety
///
/// `ptr` must be a valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_is_point(ptr: SEXP) -> SEXP {
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        let erased = ErasedExternalPtr::from_sexp(ptr);
        if erased.is::<Point>() {
            Rf_ScalarInteger(1)
        } else {
            Rf_ScalarInteger(0)
        }
    }
}

// endregion

// region: ALTREP with ExternalPtr backend

/// An ALTREP integer class that stores its data in an ExternalPtr
#[derive(DeriveExternalPtr)]
struct VecIntData {
    data: Vec<i32>,
}

/// ALTREP class using ExternalPtr for storage
#[miniextendr(class = "VecIntAltrep", pkg = "rpkg", base = "Int")]
pub struct VecIntAltrepClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for VecIntAltrepClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::externalptr::ExternalPtr;
        use miniextendr_api::ffi::R_altrep_data1;
        unsafe {
            let data1 = R_altrep_data1(x);
            match ExternalPtr::<VecIntData>::try_from_sexp(data1) {
                Some(ext) => ext.data.len() as R_xlen_t,
                None => 0,
            }
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for VecIntAltrepClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr(x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        use miniextendr_api::ffi::R_altrep_data1;
        unsafe {
            let data1 = R_altrep_data1(x);
            // Use ErasedExternalPtr for mutable access
            let mut erased = ErasedExternalPtr::from_sexp(data1);
            if let Some(vec_data) = erased.downcast_mut::<VecIntData>() {
                return vec_data.data.as_mut_ptr().cast();
            }
            core::ptr::null_mut()
        }
    }

    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        use miniextendr_api::externalptr::ExternalPtr;
        use miniextendr_api::ffi::R_altrep_data1;
        unsafe {
            let data1 = R_altrep_data1(x);
            match ExternalPtr::<VecIntData>::try_from_sexp(data1) {
                Some(ext) => ext.data.as_ptr().cast(),
                None => core::ptr::null(),
            }
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltInteger for VecIntAltrepClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        use miniextendr_api::externalptr::ExternalPtr;
        use miniextendr_api::ffi::R_altrep_data1;
        unsafe {
            let data1 = R_altrep_data1(x);
            match ExternalPtr::<VecIntData>::try_from_sexp(data1) {
                Some(ext) => ext.data.get(i as usize).copied().unwrap_or(i32::MIN),
                None => i32::MIN,
            }
        }
    }
}

/// Create a VecIntAltrep instance from an integer vector
///
/// # Safety
///
/// Must be called from R main thread with valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_vec_int_altrep(x: SEXP) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;
    use miniextendr_api::ffi::{Rf_xlength, INTEGER};

    // Copy data from input SEXP
    let n = unsafe { Rf_xlength(x) as usize };
    let src = unsafe { INTEGER(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(unsafe { *src.add(i) });
    }

    // Wrap in ExternalPtr
    let ext_ptr = ExternalPtr::new(VecIntData { data });

    // Create ALTREP instance
    let cls = VecIntAltrepClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// endregion

miniextendr_module! {
    mod rpkg;
    // ALTREP entrypoints are called directly from R via R/altrep.R

    fn add;
    fn add2;
    fn add3;
    fn add4;
    fn add_panic;
    fn add_r_error;

    fn add_panic_heap;
    fn add_r_error_heap;

    extern "C-unwind" fn C_unwind_protect_normal;
    extern "C-unwind" fn C_unwind_protect_r_error;
    extern "C-unwind" fn C_unwind_protect_lowlevel_test;

    fn add_left_mut;
    fn add_right_mut;
    fn add_left_right_mut;

    fn take_and_return_nothing;

    extern "C-unwind" fn C_just_panic;
    extern "C-unwind" fn C_panic_and_catch;

    fn drop_message_on_success;
    fn drop_on_panic;
    fn drop_on_panic_with_move;

    fn greetings_with_named_dots;
    fn greetings_with_named_and_unused_dots;
    fn greetings_with_nameless_dots;
    fn greetings_last_as_named_dots;
    fn greetings_last_as_named_and_unused_dots;
    fn greetings_last_as_nameless_dots;

    fn invisibly_return_no_arrow;
    fn invisibly_return_arrow;
    fn invisibly_option_return_none;
    fn invisibly_option_return_some;
    fn invisibly_result_return_ok;
    fn force_invisible_i32;
    fn force_visible_unit;
    fn with_interrupt_check;

    extern fn C_r_error;
    extern fn C_r_error_in_catch;
    extern fn C_r_error_in_thread;
    extern fn C_r_print_in_thread;

    extern fn C_check_interupt_after;
    extern fn C_check_interupt_unwind;

    // Worker thread tests
    extern "C-unwind" fn C_worker_drop_on_success;
    extern "C-unwind" fn C_worker_drop_on_panic;

    // Scalar conversion tests
    fn test_i32_identity;
    fn test_i32_add_one;
    fn test_i32_sum;
    fn test_f64_identity;
    fn test_f64_add_one;
    fn test_f64_multiply;
    fn test_u8_identity;
    fn test_u8_add_one;
    fn test_logical_identity;
    fn test_logical_not;
    fn test_logical_and;
    fn test_i32_to_f64;
    fn test_f64_to_i32;

    // Slice conversion tests
    fn test_i32_slice_len;
    fn test_i32_slice_sum;
    fn test_i32_slice_first;
    fn test_i32_slice_last;
    fn test_f64_slice_len;
    fn test_f64_slice_sum;
    fn test_f64_slice_mean;
    fn test_u8_slice_len;
    fn test_u8_slice_sum;
    fn test_logical_slice_len;
    fn test_logical_slice_any_true;
    fn test_logical_slice_all_true;

    // Wildcard parameter test
    fn underscore_it_all;

    // ALTREP .Call entrypoints
    extern "C-unwind" fn rpkg_altrep_compact_int;
    extern "C-unwind" fn rpkg_altrep_from_doubles;
    extern "C-unwind" fn rpkg_altrep_from_strings;
    extern "C-unwind" fn rpkg_altrep_from_logicals;
    extern "C-unwind" fn rpkg_altrep_from_raw;
    extern "C-unwind" fn rpkg_altrep_from_list;

    // Proc-macro ALTREP test: struct registers the class, fn creates instances
    struct ConstantIntClass;
    extern "C-unwind" fn rpkg_constant_int;

    // ExternalPtr tests
    fn extptr_counter_new;
    extern "C-unwind" fn C_extptr_counter_get;
    extern "C-unwind" fn C_extptr_counter_increment;
    fn extptr_point_new;
    extern "C-unwind" fn C_extptr_point_get_x;
    extern "C-unwind" fn C_extptr_point_get_y;
    extern "C-unwind" fn C_extptr_type_mismatch_test;
    extern "C-unwind" fn C_extptr_null_test;
    extern "C-unwind" fn C_extptr_is_counter;
    extern "C-unwind" fn C_extptr_is_point;

    // ALTREP with ExternalPtr backend
    struct VecIntAltrepClass;
    extern "C-unwind" fn rpkg_vec_int_altrep;
}

// endregion

// region: r-wrappers return invisibly

#[miniextendr]
fn invisibly_return_no_arrow() {}

#[miniextendr]
#[allow(clippy::unused_unit)]
fn invisibly_return_arrow() -> () {}

#[miniextendr]
fn invisibly_option_return_none() -> Option<()> {
    None // expectation: error!
}

#[miniextendr]
fn invisibly_option_return_some() -> Option<()> {
    Some(())
}

#[miniextendr]
fn invisibly_result_return_ok() -> Result<(), ()> {
    Ok(())
}

// Test explicit invisible attribute (force i32 return to be invisible)
#[miniextendr(invisible)]
fn force_invisible_i32() -> i32 {
    42
}

// Test explicit visible attribute (force () return to be visible)
#[miniextendr(visible)]
fn force_visible_unit() {}

// Test check_interrupt attribute - checks for Ctrl+C before executing
#[miniextendr(check_interrupt)]
fn with_interrupt_check(x: i32) -> i32 {
    x * 2
}

// endregion

// region: weird

// Test that wildcard `_` parameters work (transformed to synthetic names internally)
#[miniextendr]
fn underscore_it_all(_: i32, _: f64) {}

// endregion

// region: rust worker thread

#[miniextendr]
fn do_nothing() -> SEXP {
    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) }
}

// endregion

// region: worker thread tests

use miniextendr_api::worker::run_on_worker;

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

// endregion
