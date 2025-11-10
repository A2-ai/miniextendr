use std::{
    panic::AssertUnwindSafe,
    sync::{
        OnceLock,
        mpsc::{Receiver, SyncSender},
    },
};

use miniextendr_api::{
    ffi::{R_NilValue, SEXP, SendSEXP},
    miniextendr, miniextendr_module,
};

use miniextendr_api::unwind_protect::with_unwind_protect;

// region

#[derive(Debug)]
struct MsgOnDrop;

impl Drop for MsgOnDrop {
    fn drop(&mut self) {
        // FiXME: use thread-local for Rprintf, and make Rprintf private!
        // put an alias on the macro that uses the thread-local buffer to Rprintf!

        unsafe {
            miniextendr_api::ffi::Rprintf(c"%s".as_ptr(), c"Dropped `MsgOnDrop`!\n\n".as_ptr())
        };
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

enum RustRuntimeStates {
    RTask(Box<dyn FnOnce() -> SendSEXP + Send>),
    /// No more R tasks from Rust worker
    Done,
}

static R_TASK_TX: OnceLock<SyncSender<RustRuntimeStates>> = OnceLock::new();
thread_local! {
    static R_TASK_RESULT_RX: std::cell::OnceCell<Receiver<SendSEXP>> = std::cell::OnceCell::new();
}

fn with_r<R>(r: R) -> SEXP
where
    R: FnOnce() -> SEXP + Send + 'static,
    // F: Fn() -> SEXP + Send + 'static,
    // F: FnMut() -> SEXP + Send + 'static,
{
    // Send r task
    R_TASK_TX
        .get()
        .unwrap()
        .send(RustRuntimeStates::RTask(Box::new(|| unsafe {
            SendSEXP::new(r())
        })))
        .unwrap();
    // Receive the result of the R task!
    R_TASK_RESULT_RX
        .with(|x| x.get().unwrap().recv().unwrap())
        .get()
}

fn do_nothing() -> SEXP {
    let allocated_r_scalar = with_r(|| {
        use miniextendr_api::ffi;

        unsafe { ffi::Rf_ScalarInteger(42) }
    });
    allocated_r_scalar
}

#[miniextendr]
#[unsafe(no_mangle)]
unsafe extern "C" fn C_do_nothing() -> ::miniextendr_api::ffi::SEXP {
    let old = std::panic::take_hook();
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            println!(
                "Rust error occurred in file 'src/rust/{}' at line {}",
                location.file(),
                location.line()
            )
        };
    }));

    let rust_worker_thread = std::thread::Builder::new()
        .name("rust worker thread".to_string())
        .spawn(move || {
            let rust_result = std::panic::catch_unwind(move || {
                let rust_result = do_nothing();
                // TODO: add result conversion here
                unsafe { SendSEXP::new(rust_result) }
            });
            // no more R tasks to be processed
            R_TASK_TX
                .get()
                .unwrap()
                .send(RustRuntimeStates::Done)
                .unwrap();

            rust_result
        });

    // blocking on receiving r tasks.. maybe this can be expanded to have a buffer (e.g. sync_channel(100))
    let (r_task_tx, r_task_rx) = std::sync::mpsc::sync_channel(0);
    R_TASK_TX.set(r_task_tx).unwrap();

    // only one main-thread, so I assume we don't need a buffer
    let (r_task_result_tx, r_task_result_rx) = std::sync::mpsc::sync_channel(0);
    R_TASK_RESULT_RX.with(|x| x.set(r_task_result_rx)).unwrap();

    // receive r tasks from the rust worker thread
    unsafe {
        ::miniextendr_api::unwind_protect::with_unwind_protect(
            move || {
                loop {
                    let mut is_done = false;
                    let r_task_result = {
                        let catch_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                            let r_tasks_from_rust_worker = r_task_rx.recv();
                            match r_tasks_from_rust_worker {
                                Ok(RustRuntimeStates::Done) => {
                                    is_done = true;
                                    Ok(::miniextendr_api::ffi::R_NilValue)
                                }
                                Ok(RustRuntimeStates::RTask(rtask)) => Ok(rtask().get()),
                                Err(error) => Err(error),
                            }
                        }));

                        if let Ok(Ok(r_task_result)) = catch_result {
                            r_task_result
                        } else {
                            let error_message = if let Ok(Err(error)) = catch_result {
                                error.to_string()
                            } else if let Err(payload) = catch_result {
                                if let Some(&message) = payload.downcast_ref::<&str>() {
                                    message
                                } else if let Some(message) = payload.downcast_ref::<String>() {
                                    message.as_str()
                                } else if let Some(message) = payload.downcast_ref::<&String>() {
                                    message.as_str()
                                } else {
                                    "panic payload could not be unpacked"
                                }
                                .to_string()
                            } else {
                                unreachable!()
                            };

                            let c_error_message = std::ffi::CString::new(error_message)
                                .unwrap_or_else(|_| {
                                    std::ffi::CString::new("<invalid panic message>").unwrap()
                                });
                            ::miniextendr_api::ffi::Rf_error(
                                c"%s".as_ptr(),
                                c_error_message.as_ptr(),
                            );
                        }
                    };
                    if is_done {
                        break R_NilValue;
                    } else {
                        // Send r task result back!
                        r_task_result_tx.send(SendSEXP::new(r_task_result)).unwrap();
                    }
                }
            },
            move |_jump| {
                std::panic::set_hook(old);
            },
        );
    }
    rust_worker_thread.unwrap().join().unwrap().unwrap().get()
}

// endregion
