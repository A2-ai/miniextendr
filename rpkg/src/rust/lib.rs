use std::borrow::Cow;
use std::cell::RefCell;
use std::panic::AssertUnwindSafe;

use miniextendr_api::ffi::{R_NilValue, SEXP, SendSEXP};
use miniextendr_api::{miniextendr, miniextendr_module};

use miniextendr_api::unwind_protect::with_unwind_protect;

// region

#[derive(Debug)]
struct MsgOnDrop;

impl Drop for MsgOnDrop {
    fn drop(&mut self) {
        let _ = miniextendr_api::unwind::with_r(|| unsafe {
            miniextendr_api::ffi::Rprintf(c"%s".as_ptr(), c"Dropped `MsgOnDrop`!\n".as_ptr());
            miniextendr_api::ffi::R_NilValue
        });
    }
}

#[miniextendr]
fn just() -> i32 {
    let _a = MsgOnDrop;
    42
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
    let _a = MsgOnDrop;
    panic!();
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
    // WARNING: doesn't drop
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
    // WARNING: doesn't drop
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"r error in `add_r_error`".as_ptr())
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
#[allow(clippy::diverging_sub_expression)]
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
#[allow(clippy::diverging_sub_expression)]
#[unsafe(no_mangle)]
extern "C" fn C_r_error_in_thread() -> ::miniextendr_api::ffi::SEXP {
    std::thread::spawn(|| unsafe { miniextendr_api::ffi::Rf_error(c"arg1".as_ptr()) })
        .join()
        .unwrap();
    unsafe { miniextendr_api::ffi::R_NilValue }
}

/// This will segfault, as R is not present on the spawned thread.
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn C_r_print_in_thread() -> ::miniextendr_api::ffi::SEXP {
    std::thread::spawn(|| unsafe { miniextendr_api::ffi::Rprintf(c"arg1".as_ptr()) })
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
        with_unwind_protect(
            || {
                R_CheckUserInterrupt();
                R_NilValue
            },
            |jump| {
                if jump {
                    println!("jump occurred, i.e. an interupt!");
                }
            },
        );
        R_NilValue
    }
}

// region: miniextendr_module! tests

miniextendr_module! {
    mod rpkg;
    use altrep;

    fn add;
    fn add2;
    fn add3;
    fn add4;
    fn add_panic;
    fn add_r_error;

    fn add_panic_heap;
    fn add_r_error_heap;

    fn add_left_mut;
    fn add_right_mut;
    fn add_left_right_mut;

    fn take_and_return_nothing;

    extern "C" fn C_just_panic;
    extern "C" fn C_panic_and_catch;

    fn just;
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

    extern fn C_r_error;
    extern fn C_r_error_in_catch;
    extern fn C_r_error_in_thread;
    extern fn C_r_print_in_thread;

    extern fn C_check_interupt_after;
    extern fn C_check_interupt_unwind;

    extern "C" fn C_do_nothing;

}

mod altrep {
    use miniextendr_api::miniextendr_module;

    miniextendr_module! {
        mod altrep;
    }
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

// endregion

// region: weird

// FIXME: should compile...
// #[miniextendr]
// fn underscore_it_all(_: i32, _: f64) {}

// endregion

// region: rust worker thread

fn do_nothing() -> Result<SEXP, Cow<'static, str>> {
    miniextendr_api::unwind::with_r(|| {
        use miniextendr_api::ffi;

        // panic!("intentional panic inside with_r"); // #2: test
        // unsafe { miniextendr_api::ffi::Rf_error(c"intentional r error inside with_r".as_ptr()) }; // #1: test!

        unsafe { ffi::Rf_ScalarInteger(42) }
    })
}

#[miniextendr]
#[unsafe(no_mangle)]
unsafe extern "C" fn C_do_nothing() -> ::miniextendr_api::ffi::SEXP {
    // TODO: give this an attribute
    let hook_guard = RefCell::new(Some(
        miniextendr_api::unwind::PanicHookGuard::print_error_location(),
        // miniextendr_api::unwind::PanicHookGuard::print_nothing(),
    ));

    miniextendr_api::unwind::WORKER_TX
        .get()
        .expect("worker runtime not initialised")
        .send(miniextendr_api::unwind::WorkerCommand::Run {
            job: Box::new(|| {
                let rust_result = do_nothing();
                rust_result.map(|sexp| unsafe { SendSEXP::new(sexp) })
            }),
        })
        .expect("worker thread exited unexpectedly");

    let worker_result = miniextendr_api::unwind::R_TASK_RX_SLOT.with(|slot| {
        loop {
            let message = {
                let mut slot_borrow = slot.borrow_mut();
                let receiver = slot_borrow
                    .as_mut()
                    .expect("runtime not initialised (R task receiver)");
                receiver
                    .recv()
                    .expect("R dispatcher channel closed unexpectedly")
            };

            match message {
                miniextendr_api::unwind::RTask::Call { job, reply } => {
                    let hook_guard = &hook_guard;
                    let mut job = Some(job);
                    let reply_slot = RefCell::new(Some(reply));
                    let reply_slot = &reply_slot;

                    unsafe {
                        with_unwind_protect(
                            move || {
                                let result =
                                    match std::panic::catch_unwind(AssertUnwindSafe(|| {
                                        job.take().expect("R task already consumed by dispatcher")()
                                    })) {
                                        Ok(value) => Ok(value),
                                        Err(panic) => Err(
                                            miniextendr_api::unwind::panic_payload_to_string(panic),
                                        ),
                                    };

                                reply_slot
                                    .borrow_mut()
                                    .take()
                                    .expect("R task reply already sent")
                                    .send(result)
                                    .unwrap();

                                R_NilValue
                            },
                            move |jump| {
                                hook_guard.borrow_mut().take();

                                if !jump {
                                    return;
                                }

                                if let Some(reply) = reply_slot.borrow_mut().take() {
                                    reply
                                        .send(Err(Cow::Borrowed(
                                            "non-local jump while executing R task",
                                        )))
                                        .unwrap();
                                }

                                let mut slot = slot.borrow_mut();
                                let receiver = slot
                                    .as_mut()
                                    .expect("runtime not initialised (R task receiver)");

                                // ensuring that the r task receiver is not waiting after an error
                                match receiver.recv() {
                                    Ok(miniextendr_api::unwind::RTask::Result(_)) => {}
                                    Ok(_) => {}
                                    Err(_) => {}
                                }
                                // Example:
                                // [lib.rs:575:42] a = Ok(
                                //     Result(
                                //         Err(
                                //             "non-local jump while executing R task",
                                //         ),
                                //     ),
                                // )
                            },
                        );
                    };
                }
                miniextendr_api::unwind::RTask::Result(result) => break result,
            }
        }
    });

    match worker_result {
        Ok(result) => result.get(),
        Err(message) => miniextendr_api::unwind::raise_r_error(&message),
    }
}

// endregion
