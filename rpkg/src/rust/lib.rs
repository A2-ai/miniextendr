use miniextendr_api::{
    ffi::{R_NilValue, Rprintf, SEXP},
    miniextendr, miniextendr_module,
};

struct Payload<F, C, D> {
    op: Option<F>,
    on_jump: Option<C>,
    finally: Option<D>,
}
#[inline]
unsafe extern "C" fn tramp_once<F, C, D>(p: *mut std::ffi::c_void) -> SEXP
where
    F: FnOnce() -> SEXP + std::panic::UnwindSafe,
    C: FnOnce(),
    D: FnOnce(),
{
    let pl = unsafe { p.cast::<Payload<F, C, D>>().as_mut().unwrap() };
    let op = pl.op.take().expect("op already taken");

    match std::panic::catch_unwind(op) {
        Ok(val) => val,
        Err(payload) => {
            // Extract message safely
            let msg = if let Some(&s) = payload.downcast_ref::<&'static str>() {
                s
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.as_str()
            } else {
                "unusual rust panic occurred; please report this"
            };
            let cmsg = std::ffi::CString::new(msg)
                .expect("the panic message wrapping failed; this is very problematic");
            drop(payload);
            // Never returns; triggers R's longjmp, then cleanfn runs.
            unsafe { ::miniextendr_api::ffi::Rf_error(cmsg.as_ptr()) }
        }
    }
}

unsafe extern "C" fn clean<F, C, D>(
    p: *mut std::ffi::c_void,
    jump: ::miniextendr_api::ffi::Rboolean,
) where
    F: FnOnce() -> SEXP + std::panic::UnwindSafe,
    C: FnOnce(),
    D: FnOnce(),
{
    dbg!("just tell me");
    if p.is_null() {
        return;
    }
    let mut boxed: Box<Payload<F, C, D>> = unsafe { Box::from_raw(p.cast()) };

    if jump != ::miniextendr_api::ffi::Rboolean::FALSE {
        if let Some(on_jump) = boxed.on_jump.take() {
            on_jump(); // do not call R APIs that may longjmp here
        }
    }

    if let Some(finally) = boxed.finally.take() {
        finally(); // always runs
    }
    // Box drops here in both paths, releasing any captured inputs.
}

/// Run `op` under R_UnwindProtect. If a Rust panic or R error occurs,
/// the on-jump hook runs in the cleanup path.
pub fn with_r_unwind<F, C, D>(op: F, on_jump: C, finally: D) -> SEXP
where
    F: FnOnce() -> SEXP + std::panic::UnwindSafe,
    C: FnOnce(),
    D: FnOnce(),
{
    let payload: *mut std::ffi::c_void = Box::into_raw(Box::new(Payload {
        op: Some(op),
        on_jump: Some(on_jump),
        finally: Some(finally),
    }))
    .cast();

    unsafe {
        ::miniextendr_api::ffi::R_UnwindProtect(
            Some(tramp_once::<F, C, D>),
            payload,
            Some(clean::<F, C, D>),
            payload,
            core::ptr::null_mut(), // no continuation token
        )
    }
}
// region

#[derive(Debug)]
struct MsgOnDrop;

impl Drop for MsgOnDrop {
    fn drop(&mut self) {
        unsafe { Rprintf(c"Dropped `MsgOnDrop`!\n\n".as_ptr()) };
    }
}

#[miniextendr]
#[unsafe(no_mangle)]
extern "C" fn drop_on_panic() -> SEXP {
    let a = MsgOnDrop;
    // fail
    with_r_unwind(|| panic!(), || {}, || {})
}

#[miniextendr]
#[unsafe(no_mangle)]
extern "C" fn drop_on_panic_with_move() -> SEXP {
    let a = MsgOnDrop;
    with_r_unwind(
        move || {
            let _a = &a;
            // works!
            panic!();
        },
        || {},
        || {},
    )
}

// endregion

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
    let a = MsgOnDrop;
    panic!("we cannot add right now! ");
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

#[miniextendr]
fn add_r_error(_left: i32, _right: i32) -> i32 {
    let a = MsgOnDrop;
    // WARNING: doesn't drop
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

#[unsafe(no_mangle)]
#[miniextendr]
extern "C" fn C_just_panic() -> ::miniextendr_api::ffi::SEXP {
    panic!("just panic, no capture");
}

/// If you call a miniextendr function that panics, and then `C_panic_catch`,
/// you'll see that the panic hook was not reset.
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn C_panic_and_catch() -> ::miniextendr_api::ffi::SEXP {
    let result = std::panic::catch_unwind(|| panic!("just panic, no capture"));
    let _ = dbg!(result);
    unsafe { ::miniextendr_api::ffi::R_NilValue }
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

    fn drop_on_panic;
    fn drop_on_panic_with_move;

    // TODO: adjust the R wrapper to include list(...) in the arg that is ...
    fn greetings_with_named_dots;
    fn greetings_with_named_and_unused_dots;
    fn greetings_with_nameless_dots;
    fn greetings_last_as_named_dots;
    fn greetings_last_as_named_and_unused_dots;
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

// region: weird

// FIXME: should compile...
// #[miniextendr]
// fn underscore_it_all(_: i32, _: f64) {}

// endregion
