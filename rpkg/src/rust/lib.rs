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
#[miniextendr(unsafe(main_thread))]
fn extptr_counter_new(initial: i32) -> miniextendr_api::externalptr::ExternalPtr<Counter> {
    miniextendr_api::externalptr::ExternalPtr::new(Counter { value: initial })
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
#[miniextendr(unsafe(main_thread))]
fn extptr_point_new(x: f64, y: f64) -> miniextendr_api::externalptr::ExternalPtr<Point> {
    miniextendr_api::externalptr::ExternalPtr::new(Point { x, y })
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
            Some(_) => Rf_ScalarInteger(1), // Unexpected success
            None => Rf_ScalarInteger(0),    // Expected failure - type mismatch
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
            Some(_) => Rf_ScalarInteger(1), // Unexpected - null pointer should fail
            None => Rf_ScalarInteger(0),    // Expected - null pointer detected
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

// region: Additional ALTREP examples

// =============================================================================
// Example 1: Real ALTREP - Constant value (all elements are PI)
// =============================================================================

use miniextendr_api::altrep_traits::AltReal;

/// A custom ALTREP real class: always returns PI.
#[miniextendr(class = "ConstantReal", pkg = "rpkg", base = "Real")]
pub struct ConstantRealClass;

impl Altrep for ConstantRealClass {
    const HAS_LENGTH: bool = true;
    fn length(_x: SEXP) -> R_xlen_t {
        10
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for ConstantRealClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr(x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        use miniextendr_api::ffi::{
            R_NilValue, R_altrep_data2, R_set_altrep_data2, REAL, Rf_allocVector, Rf_protect,
            Rf_unprotect, SEXPTYPE,
        };
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                let n = Self::length(x);
                let val = Rf_allocVector(SEXPTYPE::REALSXP, n);
                Rf_protect(val);
                let buf = REAL(val);
                for i in 0..n {
                    *buf.offset(i) = Self::elt(x, i);
                }
                R_set_altrep_data2(x, val);
                Rf_unprotect(1);
                buf.cast()
            } else {
                REAL(expanded).cast()
            }
        }
    }

    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        use miniextendr_api::ffi::{R_NilValue, R_altrep_data2, REAL};
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                core::ptr::null()
            } else {
                REAL(expanded).cast()
            }
        }
    }
}

impl AltReal for ConstantRealClass {
    const HAS_ELT: bool = true;
    fn elt(_x: SEXP, _i: R_xlen_t) -> f64 {
        std::f64::consts::PI
    }

    // Optimized sum: n * PI
    const HAS_SUM: bool = true;
    fn sum(x: SEXP, _narm: bool) -> SEXP {
        let n = Self::length(x) as f64;
        unsafe { miniextendr_api::ffi::Rf_ScalarReal(n * std::f64::consts::PI) }
    }
}

/// Create a ConstantReal ALTREP instance (all elements are PI, length 10).
///
/// # Safety
/// Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_constant_real() -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::ffi::altrep::R_new_altrep;
    let cls = ConstantRealClass::get_or_init_class();
    unsafe { R_new_altrep(cls, R_NilValue, R_NilValue) }
}

// =============================================================================
// Example 2: Real ALTREP - Arithmetic sequence (like R's seq())
// =============================================================================

/// Stores (start, step) for arithmetic sequence
#[derive(DeriveExternalPtr)]
struct ArithSeqData {
    start: f64,
    step: f64,
    len: i64,
}

/// ALTREP class for arithmetic sequences: start, start+step, start+2*step, ...
#[miniextendr(class = "ArithSeq", pkg = "rpkg", base = "Real")]
pub struct ArithSeqClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for ArithSeqClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for ArithSeqClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltReal for ArithSeqClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> f64 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => data.start + (i as f64) * data.step,
            None => f64::NAN,
        }
    }

    // Optimized sum using arithmetic series formula: n/2 * (first + last)
    const HAS_SUM: bool = true;
    fn sum(x: SEXP, _narm: bool) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => {
                let n = data.len as f64;
                let first = data.start;
                let last = data.start + (n - 1.0) * data.step;
                let sum = n / 2.0 * (first + last);
                unsafe { miniextendr_api::ffi::Rf_ScalarReal(sum) }
            }
            None => unsafe { miniextendr_api::ffi::Rf_ScalarReal(f64::NAN) },
        }
    }

    // Optimized min/max for monotonic sequences
    const HAS_MIN: bool = true;
    fn min(x: SEXP, _narm: bool) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => {
                let min = if data.step >= 0.0 {
                    data.start
                } else {
                    data.start + ((data.len - 1) as f64) * data.step
                };
                unsafe { miniextendr_api::ffi::Rf_ScalarReal(min) }
            }
            None => unsafe { miniextendr_api::ffi::Rf_ScalarReal(f64::NAN) },
        }
    }

    const HAS_MAX: bool = true;
    fn max(x: SEXP, _narm: bool) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => {
                let max = if data.step >= 0.0 {
                    data.start + ((data.len - 1) as f64) * data.step
                } else {
                    data.start
                };
                unsafe { miniextendr_api::ffi::Rf_ScalarReal(max) }
            }
            None => unsafe { miniextendr_api::ffi::Rf_ScalarReal(f64::NAN) },
        }
    }

    const HAS_IS_SORTED: bool = true;
    fn is_sorted(x: SEXP) -> i32 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => {
                if data.step > 0.0 {
                    1 // SORTED_INCR
                } else if data.step < 0.0 {
                    -1 // SORTED_DECR
                } else {
                    1 // All same value = sorted
                }
            }
            None => 0, // UNKNOWN
        }
    }

    const HAS_NO_NA: bool = true;
    fn no_na(_x: SEXP) -> i32 {
        1 // Arithmetic sequences never have NA
    }
}

/// Create an ArithSeq ALTREP instance: seq(from, to, length.out)
#[miniextendr]
fn arith_seq(from: f64, to: f64, length_out: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let len = length_out as i64;
    let step = if len > 1 {
        (to - from) / (len - 1) as f64
    } else {
        0.0
    };

    let ext_ptr = ExternalPtr::new(ArithSeqData {
        start: from,
        step,
        len,
    });

    let cls = ArithSeqClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 3: Logical ALTREP - All TRUE or all FALSE
// =============================================================================

use miniextendr_api::altrep_traits::AltLogical;

/// Stores the constant value and length
#[derive(DeriveExternalPtr)]
struct ConstantLogicalData {
    value: i32, // TRUE=1, FALSE=0, NA=i32::MIN
    len: i64,
}

/// ALTREP class for constant logical vectors
#[miniextendr(class = "ConstantLogical", pkg = "rpkg", base = "Logical")]
pub struct ConstantLogicalClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for ConstantLogicalClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ConstantLogicalData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for ConstantLogicalClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltLogical for ConstantLogicalClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, _i: R_xlen_t) -> i32 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ConstantLogicalData>(x) } {
            Some(data) => data.value,
            None => i32::MIN, // NA_LOGICAL
        }
    }

    // Optimized sum: n * value (for sum(TRUE_vec) = n, sum(FALSE_vec) = 0)
    const HAS_SUM: bool = true;
    fn sum(x: SEXP, narm: bool) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ConstantLogicalData>(x) } {
            Some(data) => {
                if data.value == i32::MIN {
                    // NA
                    if narm {
                        unsafe { miniextendr_api::ffi::Rf_ScalarInteger(0) }
                    } else {
                        unsafe { miniextendr_api::ffi::Rf_ScalarInteger(i32::MIN) }
                    }
                } else {
                    let sum = data.len as i32 * data.value;
                    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(sum) }
                }
            }
            None => unsafe { miniextendr_api::ffi::Rf_ScalarInteger(i32::MIN) },
        }
    }

    const HAS_NO_NA: bool = true;
    fn no_na(x: SEXP) -> i32 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ConstantLogicalData>(x) } {
            Some(data) => {
                if data.value == i32::MIN {
                    0
                } else {
                    1
                }
            }
            None => 0,
        }
    }
}

/// Create a constant logical ALTREP: rep(value, n)
#[miniextendr]
fn constant_logical(value: i32, n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(ConstantLogicalData {
        value,
        len: n as i64,
    });

    let cls = ConstantLogicalClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 4: String ALTREP - Lazy-generated strings
// =============================================================================

use miniextendr_api::altrep_traits::AltString;

/// Generates strings like "item_0", "item_1", etc. on demand
#[derive(DeriveExternalPtr)]
struct LazyStringData {
    prefix: String,
    len: i64,
}

/// ALTREP class for lazily-generated strings
#[miniextendr(class = "LazyString", pkg = "rpkg", base = "String")]
pub struct LazyStringClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for LazyStringClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<LazyStringData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for LazyStringClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltString for LazyStringClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        use miniextendr_api::ffi::{Rf_mkCharLenCE, cetype_t};
        match unsafe { altrep_data1_as::<LazyStringData>(x) } {
            Some(data) => {
                let s = format!("{}_{}", data.prefix, i);
                unsafe { Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, cetype_t::CE_UTF8) }
            }
            None => unsafe { miniextendr_api::ffi::R_NaString },
        }
    }

    const HAS_NO_NA: bool = true;
    fn no_na(_x: SEXP) -> i32 {
        1 // Generated strings are never NA
    }
}

/// Create a LazyString ALTREP: generates "prefix_0", "prefix_1", ... on demand
#[miniextendr]
fn lazy_string(prefix: &str, n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(LazyStringData {
        prefix: prefix.to_string(),
        len: n as i64,
    });

    let cls = LazyStringClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 5: Raw ALTREP - Repeating byte pattern
// =============================================================================

use miniextendr_api::altrep_traits::AltRaw;

/// Repeating pattern of bytes
#[derive(DeriveExternalPtr)]
struct RepeatingRawData {
    pattern: Vec<u8>,
    total_len: i64,
}

/// ALTREP class for repeating raw byte patterns
#[miniextendr(class = "RepeatingRaw", pkg = "rpkg", base = "Raw")]
pub struct RepeatingRawClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for RepeatingRawClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<RepeatingRawData>(x) } {
            Some(data) => data.total_len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for RepeatingRawClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltRaw for RepeatingRawClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> u8 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<RepeatingRawData>(x) } {
            Some(data) => {
                if data.pattern.is_empty() {
                    0
                } else {
                    data.pattern[i as usize % data.pattern.len()]
                }
            }
            None => 0,
        }
    }
}

/// Create a RepeatingRaw ALTREP: repeats pattern to fill n bytes
#[miniextendr]
fn repeating_raw(pattern: &[u8], n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(RepeatingRawData {
        pattern: pattern.to_vec(),
        total_len: n as i64,
    });

    let cls = RepeatingRawClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 6: List ALTREP - Lazy list of numbered lists
// =============================================================================

use miniextendr_api::altrep_traits::AltList;

/// Generates list elements on demand
#[derive(DeriveExternalPtr)]
struct LazyListData {
    len: i64,
}

/// ALTREP class for lazily-generated lists
#[miniextendr(class = "LazyList", pkg = "rpkg", base = "List")]
pub struct LazyListClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for LazyListClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<LazyListData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for LazyListClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltList for LazyListClass {
    const HAS_ELT: bool = true;
    fn elt(_x: SEXP, i: R_xlen_t) -> SEXP {
        // Return a list with index info: list(index = i, squared = i*i)
        use miniextendr_api::ffi::{
            R_NamesSymbol, Rf_allocVector, Rf_protect, Rf_setAttrib, Rf_unprotect, SET_VECTOR_ELT,
            SEXPTYPE,
        };
        unsafe {
            let result = Rf_allocVector(SEXPTYPE::VECSXP, 2);
            Rf_protect(result);

            SET_VECTOR_ELT(result, 0, miniextendr_api::ffi::Rf_ScalarInteger(i as i32));
            SET_VECTOR_ELT(
                result,
                1,
                miniextendr_api::ffi::Rf_ScalarInteger((i * i) as i32),
            );

            // Set names
            let names = Rf_allocVector(SEXPTYPE::STRSXP, 2);
            Rf_protect(names);
            miniextendr_api::ffi::SET_STRING_ELT(
                names,
                0,
                miniextendr_api::ffi::Rf_mkChar(c"index".as_ptr()),
            );
            miniextendr_api::ffi::SET_STRING_ELT(
                names,
                1,
                miniextendr_api::ffi::Rf_mkChar(c"squared".as_ptr()),
            );
            Rf_setAttrib(result, R_NamesSymbol, names);

            Rf_unprotect(2);
            result
        }
    }
}

/// Create a LazyList ALTREP: each element is list(index=i, squared=i*i)
#[miniextendr]
fn lazy_list(n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(LazyListData { len: n as i64 });

    let cls = LazyListClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 7: Integer ALTREP - Fibonacci sequence with memoization
// =============================================================================

use std::cell::RefCell;

/// Fibonacci data with memoization cache
#[derive(DeriveExternalPtr)]
struct FibonacciData {
    len: i64,
    cache: RefCell<Vec<Option<i32>>>,
}

/// ALTREP class for Fibonacci sequence with memoization
#[miniextendr(class = "Fibonacci", pkg = "rpkg", base = "Int")]
pub struct FibonacciClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for FibonacciClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<FibonacciData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for FibonacciClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltInteger for FibonacciClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        use miniextendr_api::altrep_data1_as;

        fn fib(n: usize, cache: &RefCell<Vec<Option<i32>>>) -> i32 {
            if n <= 1 {
                return n as i32;
            }

            // Check cache
            {
                let c = cache.borrow();
                if let Some(&Some(v)) = c.get(n) {
                    return v;
                }
            }

            // Compute iteratively to avoid stack overflow
            let mut c = cache.borrow_mut();
            while c.len() <= n {
                c.push(None);
            }

            if c[0].is_none() {
                c[0] = Some(0);
            }
            if n >= 1 && c[1].is_none() {
                c[1] = Some(1);
            }

            for idx in 2..=n {
                if c[idx].is_none() {
                    let a = c[idx - 1].unwrap_or(0);
                    let b = c[idx - 2].unwrap_or(0);
                    c[idx] = Some(a.saturating_add(b));
                }
            }

            c[n].unwrap_or(0)
        }

        match unsafe { altrep_data1_as::<FibonacciData>(x) } {
            Some(data) => fib(i as usize, &data.cache),
            None => i32::MIN,
        }
    }

    const HAS_NO_NA: bool = true;
    fn no_na(_x: SEXP) -> i32 {
        1 // Fibonacci values are never NA
    }
}

/// Create a Fibonacci ALTREP: fib(0), fib(1), ..., fib(n-1)
#[miniextendr]
fn fibonacci(n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(FibonacciData {
        len: n as i64,
        cache: RefCell::new(Vec::new()),
    });

    let cls = FibonacciClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 8: Integer ALTREP - Powers of 2
// =============================================================================

/// ALTREP class for powers of 2: 1, 2, 4, 8, 16, ...
#[miniextendr(class = "PowersOf2", pkg = "rpkg", base = "Int")]
pub struct PowersOf2Class;

impl Altrep for PowersOf2Class {
    const HAS_LENGTH: bool = true;
    fn length(_x: SEXP) -> R_xlen_t {
        31 // 2^0 to 2^30 fit in i32
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for PowersOf2Class {}

impl AltInteger for PowersOf2Class {
    const HAS_ELT: bool = true;
    fn elt(_x: SEXP, i: R_xlen_t) -> i32 {
        if i >= 31 {
            i32::MIN // NA for overflow
        } else {
            1 << i
        }
    }

    // Optimized sum: 2^n - 1 (sum of geometric series)
    const HAS_SUM: bool = true;
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        // Sum of 2^0 + 2^1 + ... + 2^30 = 2^31 - 1
        let sum = (1i64 << 31) - 1;
        unsafe { miniextendr_api::ffi::Rf_ScalarReal(sum as f64) }
    }

    const HAS_IS_SORTED: bool = true;
    fn is_sorted(_x: SEXP) -> i32 {
        1 // Always sorted ascending
    }

    const HAS_NO_NA: bool = true;
    fn no_na(_x: SEXP) -> i32 {
        1
    }
}

/// Create a PowersOf2 ALTREP: 1, 2, 4, 8, ..., 2^30
///
/// # Safety
/// Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_powers_of_2() -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::ffi::altrep::R_new_altrep;
    let cls = PowersOf2Class::get_or_init_class();
    unsafe { R_new_altrep(cls, R_NilValue, R_NilValue) }
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
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<VecIntData>(x) } {
            Some(ext) => ext.data.len() as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for VecIntAltrepClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr(x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        use miniextendr_api::altrep_data1_mut;
        match unsafe { altrep_data1_mut::<VecIntData>(x) } {
            Some(vec_data) => vec_data.data.as_mut_ptr().cast(),
            None => core::ptr::null_mut(),
        }
    }

    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<VecIntData>(x) } {
            Some(ext) => ext.data.as_ptr().cast(),
            None => core::ptr::null(),
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltInteger for VecIntAltrepClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<VecIntData>(x) } {
            Some(ext) => ext.data.get(i as usize).copied().unwrap_or(i32::MIN),
            None => i32::MIN,
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
    use miniextendr_api::ffi::{INTEGER, Rf_xlength};

    // Copy data from input SEXP
    let n = unsafe { Rf_xlength(x) } as usize;
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

    // Proc-macro ALTREP test: struct registers the class, fn creates instances
    struct ConstantIntClass;
    extern "C-unwind" fn rpkg_constant_int;

    // Additional ALTREP examples
    // Real ALTREP
    struct ConstantRealClass;
    extern "C-unwind" fn rpkg_constant_real;
    struct ArithSeqClass;
    fn arith_seq;

    // Logical ALTREP
    struct ConstantLogicalClass;
    fn constant_logical;

    // String ALTREP
    struct LazyStringClass;
    fn lazy_string;

    // Raw ALTREP
    struct RepeatingRawClass;
    fn repeating_raw;

    // List ALTREP
    struct LazyListClass;
    fn lazy_list;

    // More Integer ALTREP examples
    struct FibonacciClass;
    fn fibonacci;
    struct PowersOf2Class;
    extern "C-unwind" fn rpkg_powers_of_2;

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

use miniextendr_api::worker::{run_on_worker, with_r_thread};

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
fn test_worker_return_i32() -> i32 {
    // This uses worker strategy automatically (returns non-SEXP)
    let x = 21;
    x * 2
}

/// Test that String return from worker works.
#[miniextendr]
fn test_worker_return_string() -> String {
    // Uses worker strategy
    format!("hello from {}", "worker")
}

/// Test that f64 return from worker works.
#[miniextendr]
fn test_worker_return_f64() -> f64 {
    std::f64::consts::PI * 2.0
}

// -----------------------------------------------------------------------------
// Test 7: ExternalPtr creation (must be main thread - ExternalPtr is !Send)
// -----------------------------------------------------------------------------

/// Test ExternalPtr creation and usage on main thread.
/// Note: ExternalPtr is !Send, so it can only be used on main thread.
#[miniextendr(unsafe(main_thread))]
fn test_extptr_on_main_thread() -> i32 {
    use miniextendr_api::externalptr::ExternalPtr;
    let ptr = ExternalPtr::new(Counter { value: 99 });
    ptr.value
}

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
fn test_main_thread_r_api() -> i32 {
    // This runs on main thread, can call R API directly
    let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger(42) };
    unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
}

/// Main thread function that triggers R error.
#[miniextendr(unsafe(main_thread))]
fn test_main_thread_r_error() -> i32 {
    unsafe {
        miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"R error from main_thread fn".as_ptr())
    }
}

/// Main thread function with RAII drops that triggers R error.
#[miniextendr(unsafe(main_thread))]
fn test_main_thread_r_error_with_drops() -> i32 {
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

// endregion
