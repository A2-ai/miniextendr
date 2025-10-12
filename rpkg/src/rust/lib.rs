use miniextendr_api::{miniextendr, miniextendr_module};

#[derive(Debug)]
struct PanicRError;
thread_local! { static CONT: std::cell::RefCell<::miniextendr_api::ffi::SEXP> = std::cell::RefCell::new(std::ptr::null_mut()); }

#[inline(always)]
fn cont_set() {
    CONT.with(|t| unsafe { t.replace(::miniextendr_api::ffi::R_MakeUnwindCont()) });
}
#[inline(always)]
fn cont_get() -> ::miniextendr_api::ffi::SEXP {
    CONT.with(|t| *t.borrow())
}
#[inline(always)]
fn cont_take() -> ::miniextendr_api::ffi::SEXP {
    CONT.with(|t| t.replace(std::ptr::null_mut()))
}

#[inline]
fn r_error_from_panic(payload: Box<dyn std::any::Any + Send>) -> ! {
    let panic_kind = if let Some(&panic_message) = payload.downcast_ref::<&'static str>() {
        panic_message
    } else if let Some(panic_message) = payload.downcast_ref::<String>() {
        panic_message.as_str()
    } else {
        // TODO: document that this is a totally unusual panic from rust
        // as it is not in panic!("") or panic!("".to_string())
        "rust panic"
    };
    let c = std::ffi::CString::new(panic_kind).unwrap();
    unsafe { ::miniextendr_api::ffi::Rf_error(c.as_ptr()) }
}

// inner: catch Rust panics before they hit C; map to Rf_error (longjmp)
#[inline]
pub unsafe extern "C" fn tramp_mut<F>(p: *mut std::ffi::c_void) -> ::miniextendr_api::ffi::SEXP
where
    F: FnMut() -> ::miniextendr_api::ffi::SEXP,
{
    let f: &mut F = unsafe { p.cast::<F>().as_mut().unwrap() };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || f())) {
        Ok(v) => v,
        Err(e) => r_error_from_panic(e),
    }
}

// cleanfun: always drop boxed closure; clear token only on normal return
pub unsafe extern "C" fn clean_drop_and_mark<F>(
    p: *mut std::ffi::c_void,
    jump: ::miniextendr_api::ffi::Rboolean,
) where
    F: FnMut() -> ::miniextendr_api::ffi::SEXP,
{
    if !p.is_null() {
        drop(unsafe { Box::<F>::from_raw(p.cast()) });
    }
    if jump == ::miniextendr_api::ffi::Rboolean::FALSE {
        CONT.with(|t| {
            t.replace(std::ptr::null_mut());
        });
    }
}

// outer: perform local Rust unwind if R longjmp happened, then resume R
pub fn with_r_unwind<F>(f: F) -> ::miniextendr_api::ffi::SEXP
where
    F: FnMut() -> ::miniextendr_api::ffi::SEXP + 'static,
{
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || unsafe {
        let f = f;
        let data = Box::into_raw(Box::new(f)).cast();
        cont_set();
        let result = ::miniextendr_api::ffi::R_UnwindProtect(
            Some(tramp_mut::<F>),
            data,
            Some(clean_drop_and_mark::<F>),
            data,
            cont_get(),
        );
        if !cont_get().is_null() {
            std::panic::panic_any(PanicRError); // local unwind for outer Rust frames
        }
        result
    }));

    match result {
        Ok(v) => v,
        Err(payload) => {
            // don't need to downcast, as the panic doesn't hold useful information
            if payload.is::<PanicRError>() {
                unsafe { ::miniextendr_api::ffi::R_ContinueUnwind(cont_take()) }; // never returns
            }
            r_error_from_panic(payload) // unexpected outer panic -> Rf_error
        }
    }
}

// region: panics, (), and Result
#[miniextendr]
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
    left.checked_add(right).ok_or_else(|| ())
}

#[miniextendr]
fn add4(left: i32, right: i32) -> Result<i32, &'static str> {
    Ok(left
        .checked_div(right)
        .ok_or_else(|| "don't divide by zero dude")?)
}

#[miniextendr]
fn add_panic(_left: i32, _right: i32) -> i32 {
    panic!("we cannot add right now! ");
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

#[miniextendr]
fn add_r_error(_left: i32, _right: i32) -> i32 {
    unsafe { ::miniextendr_api::ffi::Rf_error(c"r error in `add_r_error`".as_ptr()) };
    #[allow(unreachable_code)]
    {
        _left + _right
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
fn add_right_mut(left: i32, mut right: i32) -> i32 {
    left + *&mut right
}

#[miniextendr]
fn add_left_right_mut(mut left: i32, mut right: i32) -> i32 {
    *&mut left + *&mut right
}

// endregion

// region: panic printing

#[miniextendr]
#[unsafe(no_mangle)]
extern "C" fn C_just_panic() -> ::miniextendr_api::ffi::SEXP {
    panic!("just panic, no capture");
}

/// If you call a miniextendr function that panics, and then `C_panic_catch`,
/// you'll see that the panic hook was not reset.
#[miniextendr]
#[unsafe(no_mangle)]
extern "C" fn C_panic_and_catch() -> ::miniextendr_api::ffi::SEXP {
    let result = std::panic::catch_unwind(|| panic!("just panic, no capture"));
    let _ = dbg!(result);
    unsafe { ::miniextendr_api::ffi::R_NilValue }
}

// endregion

// region: dots

#[miniextendr]
fn greetings_with_named_dots(_dots: ...) {}

#[miniextendr]
fn greetings_with_nameless_dots(...) {}

// LIMITATION: Good!
// #[miniextendr]
// fn greetings_with_dots_then_arg(dots: ..., exclamations: i32) {}

#[miniextendr]
fn greetings_last_as_named_dots(_exclamations: i32, _dots: ...) {}

#[miniextendr]
fn greetings_last_as_nameless_dots(_exclamations: i32, ...) {}

// endregion

// region: miniextendr_module! tests

miniextendr_module! {
   mod rpkg1;
}

miniextendr_module! {
   mod rpkg2;
   fn add2;
}

miniextendr_module! {
   mod rpkg3;
   fn add2;
   fn add3;
}

mod altrep {
    miniextendr_api::miniextendr_module! {
        mod altrep;
    }
}

miniextendr_module! {
   mod rpkg4;
   use altrep;
}

miniextendr_module! {
    mod rpkg;
    use altrep;

    fn add;
    fn add2;
    fn add3;
    fn add4;
    fn add_panic;
    fn add_r_error;

    fn add_left_mut;
    fn add_right_mut;
    fn add_left_right_mut;

    // TODO: make r wrapper, no C wrapper!
    extern "C" fn C_just_panic;
    extern "C" fn C_panic_and_catch;

    // TODO: adjust the R wrapper to include list(...) in the arg that is ...
    fn greetings_with_named_dots;
    fn greetings_with_nameless_dots;
    fn greetings_last_as_named_dots;
    fn greetings_last_as_nameless_dots;
}

// endregion

// region: r-wrappers return invisibly

#[miniextendr]
fn invisibly_return_no_arrow() {}

#[miniextendr]
fn invisibly_return_arrow() -> () {}

// TODO:
#[miniextendr]
fn invisibly_option_return_none() -> Option<()> {
    None
}

#[miniextendr]
fn invisibly_option_return_some() -> Option<()> {
    Some(())
}

#[miniextendr]
fn invisibly_result_return_ok() -> Result<(), ()> {
    Ok(())
}

// endregion
