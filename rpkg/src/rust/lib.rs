use miniextendr_api::{
    ffi::{R_NilValue, SEXP},
    miniextendr, miniextendr_module,
};

use crate::unwind_protect::with_unwind_protect;

mod unwind_protect {

    struct Ctx<F, C> {
        fun: Option<F>,
        clean: Option<C>,
    }

    unsafe extern "C" fn fun_tramp<F, C>(data: *mut std::ffi::c_void) -> miniextendr_api::ffi::SEXP
    where
        F: FnOnce() -> miniextendr_api::ffi::SEXP,
    {
        let ctx = unsafe { data.cast::<Ctx<F, C>>().as_mut().unwrap() };
        let f = ctx.fun.take().unwrap();
        f()
    }

    unsafe extern "C" fn clean_tramp<F, C>(
        data: *mut std::ffi::c_void,
        jump: miniextendr_api::ffi::Rboolean,
    ) where
        C: FnOnce(bool),
    {
        // let ctx = unsafe { data.cast::<Ctx<F, C>>().as_mut().unwrap() };
        let mut ctx = unsafe { Box::from_raw(data.cast::<Ctx<F, C>>()) };
        if let Some(c) = ctx.as_mut().clean.take() {
            c(jump != miniextendr_api::ffi::Rboolean::FALSE);
        }
        drop(ctx);
    }

    /// Wrap a Rust closure with R_UnwindProtect.
    /// `clean` sees `true` if a non-local jump happened, `false` on normal return.
    pub unsafe fn with_unwind_protect<F, C>(fun: F, clean: C) -> miniextendr_api::ffi::SEXP
    where
        F: FnOnce() -> miniextendr_api::ffi::SEXP,
        C: FnOnce(bool),
    {
        let data = Box::into_raw(Box::new(Ctx {
            fun: Some(fun),
            clean: Some(clean),
        }));

        unsafe {
            miniextendr_api::ffi::R_UnwindProtect(
                Some(fun_tramp::<F, C>),
                data.cast(),
                Some(clean_tramp::<F, C>),
                data.cast(),
                std::ptr::null_mut(),
            )
        }
    }
}

// region

#[derive(Debug)]
struct MsgOnDrop;

impl Drop for MsgOnDrop {
    fn drop(&mut self) {
        // FiXME: use thread-local for Rprintf, and make Rprintf private!
        // put an alias on the macro that uses the thread-local buffer to Rprintf!

        unsafe { miniextendr_api::ffi::Rprintf(c"Dropped `MsgOnDrop`!\n\n".as_ptr()) };
    }
}

#[miniextendr]
#[unsafe(no_mangle)]
extern "C" fn drop_on_panic() -> miniextendr_api::ffi::SEXP {
    let _a = MsgOnDrop;
    // fail
    panic!()
}

#[miniextendr]
#[unsafe(no_mangle)]
extern "C" fn drop_on_panic_with_move() -> miniextendr_api::ffi::SEXP {
    let a = MsgOnDrop;
    panic!();
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
    // WARNING: doesn't drop
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"r error in `add_r_error`".as_ptr())
    };
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

#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn C_r_error() -> ::miniextendr_api::ffi::SEXP {
    unsafe { miniextendr_api::ffi::Rf_error(c"arg1".as_ptr()) }
}

#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn C_r_error_in_catch() -> ::miniextendr_api::ffi::SEXP {
    unsafe {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            miniextendr_api::ffi::Rf_error(c"arg1".as_ptr())
        }))
        .unwrap();
        miniextendr_api::ffi::R_NilValue
    }
}

/// This crashes immediately. R is simply not present on the spawned thread, hence the present segfault.
///
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn C_r_error_in_thread() -> ::miniextendr_api::ffi::SEXP {
    let _ = std::thread::spawn(|| unsafe { miniextendr_api::ffi::Rf_error(c"arg1".as_ptr()) })
        .join()
        .unwrap();
    unsafe { miniextendr_api::ffi::R_NilValue }
}

/// This will segfault, as R is not present on the spawned thread.
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn C_r_print_in_thread() -> ::miniextendr_api::ffi::SEXP {
    let _ = std::thread::spawn(|| unsafe { miniextendr_api::ffi::Rprintf(c"arg1".as_ptr()) })
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
extern "C" fn C_check_interupt_after() -> SEXP {
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
extern "C" fn C_check_interupt_unwind() -> SEXP {
    use miniextendr_api::ffi::R_CheckUserInterrupt;

    std::thread::sleep(std::time::Duration::from_secs(2));

    unsafe {
        // todo!();
        with_unwind_protect(
            || {
                R_CheckUserInterrupt();
                R_NilValue
            },
            |jump| {
                if jump {
                    println!("jump occurred, i.e. an interupt!")
                }
            },
        );
        R_NilValue
    }
}

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

    extern "C" fn C_just_panic;
    extern "C" fn C_panic_and_catch;

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

    // experimental unwinding support
    extern fn C_rust_worker1;
    extern fn C_rust_worker2;

    extern fn C_r_error;
    extern fn C_r_error_in_catch;
    extern fn C_r_error_in_thread;
    extern fn C_r_print_in_thread;

    extern fn C_check_interupt_after;
    extern fn C_check_interupt_unwind;

}

// endregion

// region: r-wrappers return invisibly

#[miniextendr]
fn invisibly_return_no_arrow() {}

#[miniextendr]
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

// endregion

// region: weird

// FIXME: should compile...
// #[miniextendr]
// fn underscore_it_all(_: i32, _: f64) {}

// endregion

// region: rust runtime!

// ---------- messages ----------
//TODO: Shouldn't this stuff just be FnOnce?
// Also not all of R api return SEXP. We may need to extract other results,
// outside of this mechanism.

// TODO: Wrap all R api code that `Rf_error/error`s with this mechanism,
// but also **don't** wrap R api code that do no `Rf_error/error`s in the mechanism.

pub fn with_rust_worker<F>(f: F) -> miniextendr_api::ffi::SEXP
where
    F: FnOnce() -> Result<miniextendr_api::ffi::SendSEXP, ()> + Send + 'static,
{
    use std::thread::Builder;
    let handle = Builder::new().name(format!("rust worker")).spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || f()));
        let _ = miniextendr_api::unwind::MAIN_TX
            .get()
            .unwrap()
            .0
            .send(miniextendr_api::unwind::MainRequest::Done);
        result
    });

    // main thread loop: service requests until Done or worker panic/exit
    loop {
        let lock = miniextendr_api::unwind::MAIN_RX.get().unwrap().lock();
        let lock = lock.unwrap();
        match lock.recv() {
            Ok(::miniextendr_api::unwind::MainRequest::RGuard { task, reply }) => {
                ::miniextendr_api::unwind::run_on_main(task, reply);
            }
            Ok(::miniextendr_api::unwind::MainRequest::Done) => {
                break;
            }
            Err(error) => {
                dbg!(error);
                break;
            }
        }
    }

    // TODO: flatten this...
    match handle {
        Ok(handle) => match handle.join() {
            Ok(answer) => match answer {
                Ok(answer) => match answer {
                    Ok(answer) => {
                        let result: ::miniextendr_api::ffi::SEXP = answer.inner;
                        result
                    }
                    Err(()) => unsafe {
                        ::miniextendr_api::ffi::Rf_error(
                            c"%s".as_ptr(),
                            c"R error during guarded call".as_ptr(),
                        )
                    },
                },
                Err(payload) => ::miniextendr_api::unwind::payload_to_r_error(payload),
            },
            Err(payload) => ::miniextendr_api::unwind::payload_to_r_error(payload),
        },
        Err(spawn_error) => unsafe {
            let spawn_error_msg = spawn_error.to_string();
            let spawn_error_cmsg = std::ffi::CString::new(spawn_error_msg).unwrap();
            ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), spawn_error_cmsg.as_ptr())
        },
    }
}

#[miniextendr]
#[unsafe(no_mangle)]
pub extern "C" fn C_rust_worker2() -> miniextendr_api::ffi::SEXP {
    use miniextendr_api::ffi::SendSEXP;

    with_rust_worker(|| unsafe {
        use miniextendr_api::ffi::R_NilValue;

        // panic!("nothing happened");
        // Rf_error(std::ptr::null());

        // note: this will not work, because R is not present on the new thread.
        miniextendr_api::ffi::Rf_error(c"asd".as_ptr());

        Ok(SendSEXP::new(R_NilValue))
    })
}

#[miniextendr]
#[unsafe(no_mangle)]
pub extern "C" fn C_rust_worker1() -> miniextendr_api::ffi::SEXP {
    // note: everything outside of the thread will not drop in case of an R error.
    // note: a rust panic here is not good.

    // spawn worker
    let handle = std::thread::spawn(move || -> Result<::miniextendr_api::ffi::SendSEXP, ()> {
        // note: allocations here will deallocate in case of a panic

        // #<number>: cases to consider

        // #3
        // let a = MsgOnDrop;
        #[allow(unreachable_code)] // tests!
        let sexp: ::miniextendr_api::ffi::SendSEXP =
            ::miniextendr_api::unwind::with_r_guard(move || unsafe {
                // limitation: dropped on a panic, not on an Rf_error!
                // let a = MsgOnDrop;

                // #1
                // panic!("rust panic while running r task");

                // #2
                // ::miniextendr_api::ffi::Rf_error(c"an r error occurred".as_ptr());

                ::miniextendr_api::ffi::R_NilValue
            })?;
        // more Rust work...

        let sexp: ::miniextendr_api::ffi::SendSEXP =
            ::miniextendr_api::unwind::with_r_guard_ref(move || unsafe {
                // limitation: dropped on a panic, not on an Rf_error!
                // let a = MsgOnDrop;

                // #1
                // panic!("rust panic while running r task");

                // #2
                // ::miniextendr_api::ffi::Rf_error(c"an r error occurred".as_ptr());

                ::miniextendr_api::ffi::R_NilValue
            })?;
        // more Rust work...
        let sexp: ::miniextendr_api::ffi::SendSEXP =
            ::miniextendr_api::unwind::with_r_guard_mut(move || unsafe {
                // limitation: dropped on a panic, not on an Rf_error!
                // let a = MsgOnDrop;

                // #1
                // panic!("rust panic while running r task");

                // #2
                // ::miniextendr_api::ffi::Rf_error(c"an r error occurred".as_ptr());

                ::miniextendr_api::ffi::R_NilValue
            })?;
        // more Rust work...

        let _ = miniextendr_api::unwind::MAIN_TX
            .get()
            .unwrap()
            .0
            .send(::miniextendr_api::unwind::MainRequest::Done);
        Ok(sexp)
    });

    // main thread loop: service requests until Done or worker panic/exit
    loop {
        let lock = miniextendr_api::unwind::MAIN_RX.get().unwrap().lock();
        match lock.unwrap().recv() {
            Ok(::miniextendr_api::unwind::MainRequest::RGuard { task, reply }) => {
                ::miniextendr_api::unwind::run_on_main(task, reply);
            }
            Ok(::miniextendr_api::unwind::MainRequest::Done) => break,
            Err(_) => {
                break;
            }
        }
    }

    // join worker; on panic report via Rf_error
    match handle.join() {
        Ok(Ok(ans)) => {
            let ans: ::miniextendr_api::ffi::SEXP = ans.inner;
            ans
        }
        handle @ Ok(Err(())) => unsafe {
            drop(handle);
            ::miniextendr_api::ffi::Rf_error(
                c"%s".as_ptr(),
                c"R error during guarded call".as_ptr(),
            )
        },
        Err(payload) => ::miniextendr_api::unwind::payload_to_r_error(payload),
    }
}

// endregion
