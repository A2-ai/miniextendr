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
//
// The new approach requires:
// 1. A data type that implements high-level data traits (AltrepLen, AltIntegerData, etc.)
// 2. Low-level trait impls generated via impl_alt*_from_data! macro
// 3. A 1-field wrapper struct with #[miniextendr] macro

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};

// -----------------------------------------------------------------------------
// ConstantInt: An ALTREP integer that always returns the same value
// -----------------------------------------------------------------------------

/// Data type that stores a constant value and length
#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantIntData {
    value: i32,
    len: usize,
}

// Implement high-level data traits
impl AltrepLen for ConstantIntData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for ConstantIntData {
    fn elt(&self, _i: usize) -> i32 {
        self.value
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.value != i32::MIN) // NA is i32::MIN
    }
}

// Generate low-level traits from data traits
miniextendr_api::impl_altinteger_from_data!(ConstantIntData);
miniextendr_api::impl_inferbase_integer!(ConstantIntData);

/// ALTREP wrapper for ConstantIntData
#[miniextendr(class = "ConstantInt", pkg = "rpkg")]
pub struct ConstantIntClass(ConstantIntData);

/// Create a ConstantInt ALTREP instance (all elements are 42, length 10).
///
/// # Safety
///
/// Must be called from R main thread with R properly initialized.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_constant_int() -> SEXP {
    let data = ConstantIntData { value: 42, len: 10 };
    unsafe { ConstantIntClass::into_altrep(data) }
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

// region: Additional ALTREP examples - using new 1-field struct pattern
//
// The new ALTREP API requires:
// 1. A data type that implements high-level data traits (AltrepLen, Alt*Data)
// 2. Low-level trait impls generated via impl_alt*_from_data! macro
// 3. A 1-field wrapper struct with #[miniextendr] macro
//
// For custom behavior that can't be expressed through the data traits,
// manually implement the low-level traits on the data type.

use miniextendr_api::altrep_data::{AltRealData, AltLogicalData, AltRawData, AltStringData, Logical};

// -----------------------------------------------------------------------------
// ConstantReal: All elements are PI
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantRealData {
    value: f64,
    len: usize,
}

impl AltrepLen for ConstantRealData {
    fn len(&self) -> usize { self.len }
}

impl AltRealData for ConstantRealData {
    fn elt(&self, _i: usize) -> f64 { self.value }
    fn no_na(&self) -> Option<bool> { Some(!self.value.is_nan()) }
}

miniextendr_api::impl_altreal_from_data!(ConstantRealData);
miniextendr_api::impl_inferbase_real!(ConstantRealData);

#[miniextendr(class = "ConstantReal", pkg = "rpkg")]
pub struct ConstantRealClass(ConstantRealData);

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_constant_real() -> SEXP {
    let data = ConstantRealData { value: std::f64::consts::PI, len: 10 };
    unsafe { ConstantRealClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// ArithSeq: Arithmetic sequence (like R's seq())
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct ArithSeqData {
    start: f64,
    step: f64,
    len: usize,
}

impl AltrepLen for ArithSeqData {
    fn len(&self) -> usize { self.len }
}

impl AltRealData for ArithSeqData {
    fn elt(&self, i: usize) -> f64 {
        self.start + (i as f64) * self.step
    }
    fn no_na(&self) -> Option<bool> { Some(true) }
}

miniextendr_api::impl_altreal_from_data!(ArithSeqData);
miniextendr_api::impl_inferbase_real!(ArithSeqData);

#[miniextendr(class = "ArithSeq", pkg = "rpkg")]
pub struct ArithSeqClass(ArithSeqData);

#[miniextendr]
fn arith_seq(from: f64, to: f64, length_out: i32) -> SEXP {
    let len = length_out as usize;
    let step = if len > 1 { (to - from) / (len - 1) as f64 } else { 0.0 };
    let data = ArithSeqData { start: from, step, len };
    unsafe { ArithSeqClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// LazyIntSeq: Integer arithmetic sequence with lazy materialization
// This demonstrates the Dataptr lazy materialization pattern:
// - Elements are computed on-demand via Elt/Get_region
// - Full buffer is only allocated when Dataptr is called
// - Dataptr_or_null returns NULL until materialized
// -----------------------------------------------------------------------------

/// Data type for lazy integer sequence with materialization support
#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyIntSeqData {
    start: i32,
    step: i32,
    len: usize,
    /// Lazily-allocated buffer for materialization
    materialized: Option<Vec<i32>>,
}

impl AltrepLen for LazyIntSeqData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for LazyIntSeqData {
    fn elt(&self, i: usize) -> i32 {
        // Compute element on-the-fly (no materialization needed)
        self.start.saturating_add((i as i32).saturating_mul(self.step))
    }

    fn no_na(&self) -> Option<bool> {
        // Check if any element would be NA (i32::MIN)
        // This is a conservative check - we know the formula
        Some(true)
    }

    fn is_sorted(&self) -> Option<miniextendr_api::altrep_data::Sortedness> {
        use miniextendr_api::altrep_data::Sortedness;
        if self.step == 0 {
            Some(Sortedness::Increasing) // All same value
        } else if self.step > 0 {
            Some(Sortedness::StrictlyIncreasing)
        } else {
            Some(Sortedness::StrictlyDecreasing)
        }
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        // Arithmetic sequence sum: n * (first + last) / 2
        let n = self.len as i64;
        let first = self.start as i64;
        let last = first + (self.len.saturating_sub(1) as i64) * (self.step as i64);
        Some(n * (first + last) / 2)
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if self.len == 0 {
            None
        } else if self.step >= 0 {
            Some(self.start)
        } else {
            Some(self.elt(self.len - 1))
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if self.len == 0 {
            None
        } else if self.step >= 0 {
            Some(self.elt(self.len - 1))
        } else {
            Some(self.start)
        }
    }
}

/// Implement AltrepDataptr for lazy materialization
impl miniextendr_api::altrep_data::AltrepDataptr<i32> for LazyIntSeqData {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        // Materialize on first access
        if self.materialized.is_none() {
            eprintln!("[Rust] LazyIntSeq: Materializing {} elements...", self.len);
            let data: Vec<i32> = (0..self.len)
                .map(|i| self.start.saturating_add((i as i32).saturating_mul(self.step)))
                .collect();
            self.materialized = Some(data);
            eprintln!("[Rust] LazyIntSeq: Materialization complete!");
        }
        self.materialized.as_mut().map(|v| v.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        // Only return pointer if already materialized
        // This allows R to use Elt/Get_region for unmaterialized data
        self.materialized.as_ref().map(|v| v.as_ptr())
    }
}

// Implement serialization support
impl miniextendr_api::altrep_data::AltrepSerialize for LazyIntSeqData {
    fn serialized_state(&self) -> SEXP {
        // Store start, step, len in an integer vector
        // Note: We don't serialize the materialized buffer - it will be recomputed on demand
        unsafe {
            use miniextendr_api::ffi::{Rf_allocVector, SET_INTEGER_ELT, SEXPTYPE};
            let state = Rf_allocVector(SEXPTYPE::INTSXP, 3);
            SET_INTEGER_ELT(state, 0, self.start);
            SET_INTEGER_ELT(state, 1, self.step);
            SET_INTEGER_ELT(state, 2, self.len as i32);
            state
        }
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        unsafe {
            use miniextendr_api::ffi::INTEGER_ELT;
            let start = INTEGER_ELT(state, 0);
            let step = INTEGER_ELT(state, 1);
            let len = INTEGER_ELT(state, 2) as usize;
            Some(LazyIntSeqData {
                start,
                step,
                len,
                materialized: None, // Fresh start - not materialized
            })
        }
    }
}

// Use the dataptr + serialize variant to enable both Dataptr and serialization methods
miniextendr_api::impl_altinteger_from_data!(LazyIntSeqData, dataptr, serialize);
miniextendr_api::impl_inferbase_integer!(LazyIntSeqData);

/// ALTREP wrapper for LazyIntSeqData - base type auto-inferred!
#[miniextendr(class = "LazyIntSeq", pkg = "rpkg")]
pub struct LazyIntSeqClass(LazyIntSeqData);

/// Create a lazy integer sequence (similar to R's seq())
/// Elements are computed on-demand; full buffer only allocated on DATAPTR access.
#[miniextendr]
pub fn lazy_int_seq(from: i32, to: i32, by: i32) -> SEXP {
    let len = if by == 0 {
        1
    } else {
        ((to - from) / by + 1).max(0) as usize
    };
    let data = LazyIntSeqData {
        start: from,
        step: by,
        len,
        materialized: None,
    };
    unsafe { LazyIntSeqClass::into_altrep(data) }
}

/// Check if a LazyIntSeq has been materialized
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_lazy_int_seq_is_materialized(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{ALTREP, Rf_ScalarLogical};
    use miniextendr_api::altrep_data1_as;

    // Check if it's an ALTREP object
    if unsafe { ALTREP(x) } == 0 {
        return unsafe { Rf_ScalarLogical(0) }; // Not ALTREP
    }

    // Try to extract the data
    match unsafe { altrep_data1_as::<LazyIntSeqData>(x) } {
        Some(data) => {
            let is_mat = data.materialized.is_some();
            unsafe { Rf_ScalarLogical(if is_mat { 1 } else { 0 }) }
        }
        None => unsafe { Rf_ScalarLogical(0) },
    }
}

// -----------------------------------------------------------------------------
// ConstantLogical: All TRUE or all FALSE
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantLogicalData {
    value: Logical,
    len: usize,
}

impl AltrepLen for ConstantLogicalData {
    fn len(&self) -> usize { self.len }
}

impl AltLogicalData for ConstantLogicalData {
    fn elt(&self, _i: usize) -> Logical { self.value }
    fn no_na(&self) -> Option<bool> {
        Some(!matches!(self.value, Logical::Na))
    }
}

miniextendr_api::impl_altlogical_from_data!(ConstantLogicalData);
miniextendr_api::impl_inferbase_logical!(ConstantLogicalData);

#[miniextendr(class = "ConstantLogical", pkg = "rpkg")]
pub struct ConstantLogicalClass(ConstantLogicalData);

#[miniextendr]
fn constant_logical(value: i32, n: i32) -> SEXP {
    let logical_value = match value {
        0 => Logical::False,
        i if i == i32::MIN => Logical::Na,
        _ => Logical::True,
    };
    let data = ConstantLogicalData { value: logical_value, len: n as usize };
    unsafe { ConstantLogicalClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// LazyString: Lazily-generated strings
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyStringData {
    prefix: String,
    len: usize,
}

impl AltrepLen for LazyStringData {
    fn len(&self) -> usize { self.len }
}

impl AltStringData for LazyStringData {
    fn elt(&self, _i: usize) -> Option<&str> {
        // Note: For a real implementation you'd want to cache generated strings
        // Since we can't return a reference to a newly created String, return None
        // which triggers R's default behavior (NA)
        None
    }
    fn no_na(&self) -> Option<bool> { Some(false) } // We return None which is like NA
}

miniextendr_api::impl_altstring_from_data!(LazyStringData);
miniextendr_api::impl_inferbase_string!(LazyStringData);

#[miniextendr(class = "LazyString", pkg = "rpkg")]
pub struct LazyStringClass(LazyStringData);

#[miniextendr]
fn lazy_string(prefix: &str, n: i32) -> SEXP {
    let data = LazyStringData { prefix: prefix.to_string(), len: n as usize };
    unsafe { LazyStringClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// RepeatingRaw: Repeating byte pattern
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct RepeatingRawData {
    pattern: Vec<u8>,
    total_len: usize,
}

impl AltrepLen for RepeatingRawData {
    fn len(&self) -> usize { self.total_len }
}

impl AltRawData for RepeatingRawData {
    fn elt(&self, i: usize) -> u8 {
        if self.pattern.is_empty() { 0 }
        else { self.pattern[i % self.pattern.len()] }
    }
}

miniextendr_api::impl_altraw_from_data!(RepeatingRawData);
miniextendr_api::impl_inferbase_raw!(RepeatingRawData);

#[miniextendr(class = "RepeatingRaw", pkg = "rpkg")]
pub struct RepeatingRawClass(RepeatingRawData);

#[miniextendr]
fn repeating_raw(pattern: &[u8], n: i32) -> SEXP {
    let data = RepeatingRawData { pattern: pattern.to_vec(), total_len: n as usize };
    unsafe { RepeatingRawClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// UnitCircle: Complex numbers on the unit circle (e^(i*theta))
// This demonstrates ALTREP for complex vectors
// -----------------------------------------------------------------------------

use miniextendr_api::altrep_data::AltComplexData;
use miniextendr_api::ffi::Rcomplex;

#[derive(miniextendr_api::ExternalPtr)]
pub struct UnitCircleData {
    /// Number of points on the unit circle
    n: usize,
}

impl AltrepLen for UnitCircleData {
    fn len(&self) -> usize {
        self.n
    }
}

impl AltComplexData for UnitCircleData {
    fn elt(&self, i: usize) -> Rcomplex {
        // Generate e^(i * 2π * k/n) = cos(2πk/n) + i*sin(2πk/n)
        let theta = 2.0 * std::f64::consts::PI * (i as f64) / (self.n as f64);
        Rcomplex {
            r: theta.cos(),
            i: theta.sin(),
        }
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        let end = (start + len).min(self.n);
        for (buf_i, i) in (start..end).enumerate() {
            buf[buf_i] = self.elt(i);
        }
        end - start
    }
}

miniextendr_api::impl_altcomplex_from_data!(UnitCircleData);
miniextendr_api::impl_inferbase_complex!(UnitCircleData);

/// ALTREP wrapper for UnitCircleData - generates complex numbers on unit circle
#[miniextendr(class = "UnitCircle", pkg = "rpkg")]
pub struct UnitCircleClass(UnitCircleData);

/// Create complex numbers on the unit circle: e^(i * 2π * k/n) for k = 0, 1, ..., n-1
/// These are the n-th roots of unity, evenly spaced around the unit circle.
#[miniextendr]
pub fn unit_circle(n: i32) -> SEXP {
    let data = UnitCircleData { n: n as usize };
    unsafe { UnitCircleClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// SimpleVecInt: Vec<i32> wrapper (simplest example)
// -----------------------------------------------------------------------------

#[miniextendr(class = "SimpleVecInt", pkg = "rpkg")]
pub struct SimpleVecIntClass(Vec<i32>);

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_simple_vec_int(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{INTEGER, Rf_xlength};
    let n = unsafe { Rf_xlength(x) } as usize;
    let src = unsafe { INTEGER(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(unsafe { *src.add(i) });
    }
    unsafe { SimpleVecIntClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// InferredVecReal: Vec<f64> wrapper with base type inferred from inner type
// -----------------------------------------------------------------------------

/// Test case for auto-inferred base type (no explicit `base = "..."` attribute)
#[miniextendr(class = "InferredVecReal", pkg = "rpkg")]
pub struct InferredVecRealClass(Vec<f64>);

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_inferred_vec_real(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{REAL, Rf_xlength};
    let n = unsafe { Rf_xlength(x) } as usize;
    let src = unsafe { REAL(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(unsafe { *src.add(i) });
    }
    unsafe { InferredVecRealClass::into_altrep(data) }
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

    // Lazy materialization ALTREP example
    struct LazyIntSeqClass;
    fn lazy_int_seq;
    extern "C-unwind" fn rpkg_lazy_int_seq_is_materialized;

    // Logical ALTREP
    struct ConstantLogicalClass;
    fn constant_logical;

    // String ALTREP
    struct LazyStringClass;
    fn lazy_string;

    // Raw ALTREP
    struct RepeatingRawClass;
    fn repeating_raw;

    // Complex ALTREP - unit circle (roots of unity)
    struct UnitCircleClass;
    fn unit_circle;

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

    // ALTREP with Vec<i32> backend - simplified API
    struct SimpleVecIntClass;
    extern "C-unwind" fn rpkg_simple_vec_int;

    // ALTREP with Vec<f64> backend - base type auto-inferred
    struct InferredVecRealClass;
    extern "C-unwind" fn rpkg_inferred_vec_real;
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
