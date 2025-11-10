use miniextendr_api::{miniextendr, miniextendr_module};

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
    let _a = MsgOnDrop;
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

type WorkerReply = Result<SendSEXP, String>;

enum RuntimeMessage {
    Task {
        job: Box<dyn FnOnce() -> SendSEXP + Send>,
        reply: std::sync::mpsc::SyncSender<WorkerReply>,
    },
    Flush,
}

static DISPATCHER_TX: std::sync::OnceLock<std::sync::mpsc::SyncSender<RuntimeMessage>> =
    std::sync::OnceLock::new();
static RUNTIME_ONCE: std::sync::Once = std::sync::Once::new();

thread_local! {
    static DISPATCHER_RX: std::cell::RefCell<Option<std::sync::mpsc::Receiver<RuntimeMessage>>> = std::cell::RefCell::new(None);
    static WORKER_MAILBOX: std::cell::RefCell<Option<(std::sync::mpsc::SyncSender<RuntimeMessage>, std::sync::mpsc::SyncSender<WorkerReply>, std::sync::mpsc::Receiver<WorkerReply>)>> = std::cell::RefCell::new(None);
}

fn with_r<R>(r: R) -> SEXP
where
    R: FnOnce() -> SEXP + Send + 'static,
{
    WORKER_MAILBOX.with(|cell| {
        let slot = cell.borrow();
        let (dispatcher, reply_tx, reply_rx) = slot
            .as_ref()
            .expect("with_r called outside a registered worker thread");

        dispatcher
            .send(RuntimeMessage::Task {
                job: Box::new(|| unsafe { SendSEXP::new(r()) }),
                reply: reply_tx.clone(),
            })
            .expect("dispatcher channel closed unexpectedly");

        match reply_rx
            .recv()
            .expect("dispatcher dropped worker reply channel")
        {
            Ok(value) => value.get(),
            Err(message) => panic!("{message}"),
        }
    })
}

fn do_nothing() -> SEXP {
    let allocated_r_scalar = with_r(|| {
        use miniextendr_api::ffi;

        unsafe { ffi::Rf_ScalarInteger(42) }
    });
    allocated_r_scalar
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn miniextendr_runtime_init() {
    RUNTIME_ONCE.call_once_force(|_once_state| {
        // TODO: use `_once_state` to inform about issues.. this will
        // be relevant when the rust-runtime is converted to a runtime amongst
        // all rust-based r-packages
        let (dispatcher_tx, dispatcher_rx) = std::sync::mpsc::sync_channel(0);
        DISPATCHER_TX
            .set(dispatcher_tx)
            .expect("dispatcher already initialised");
        DISPATCHER_RX.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot.is_some() {
                panic!("dispatcher receiver already initialised on this thread");
            }
            *slot = Some(dispatcher_rx);
        });
    });
}

#[miniextendr]
#[unsafe(no_mangle)]
unsafe extern "C" fn C_do_nothing() -> ::miniextendr_api::ffi::SEXP {
    let mut hook_slot = Some(std::panic::take_hook());
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            println!(
                "Rust error occurred in file 'src/rust/{}' at line {}",
                location.file(),
                location.line()
            )
        };
    }));

    let dispatcher = DISPATCHER_TX
        .get()
        .expect("dispatcher not initialised")
        .clone();

    let rust_worker_thread = std::thread::Builder::new()
        .name("rust worker thread".to_string())
        .spawn({
            let dispatcher_for_worker = dispatcher.clone();
            move || {
                WORKER_MAILBOX.with(|cell| {
                    let mut slot = cell.borrow_mut();
                    if slot.is_some() {
                        panic!("worker mailbox already registered on this thread");
                    }
                    let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(0);
                    *slot = Some((dispatcher_for_worker.clone(), reply_tx, reply_rx));
                });

                let rust_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let rust_result = do_nothing();
                    // TODO: insert rust_result -> r conversion!
                    unsafe { miniextendr_api::ffi::SendSEXP::new(rust_result) }
                }))
                .map_err(|panic| match panic.downcast::<String>() {
                    Ok(message) => *message,
                    Err(panic) => match panic.downcast::<&'static str>() {
                        Ok(message) => (*message).to_string(),
                        Err(_) => "panic payload could not be unpacked".to_string(),
                    },
                });

                WORKER_MAILBOX.with(|cell| {
                    cell.borrow_mut().take();
                });
                let _ = dispatcher.send(RuntimeMessage::Flush);
                rust_result
            }
        })
        .expect("failed to spawn rust worker thread");

    let mut dispatcher_error: Option<String> = None;

    DISPATCHER_RX.with(|cell| {
        let receiver_slot = cell.borrow();
        let receiver = receiver_slot
            .as_ref()
            .expect("dispatcher receiver not initialised on this thread");

        unsafe {
            ::miniextendr_api::unwind_protect::with_unwind_protect(
                || {
                    let pending_error = &mut dispatcher_error;

                    loop {
                        let message = match receiver.recv() {
                            Ok(msg) => msg,
                            Err(_) => {
                                *pending_error =
                                    Some("dispatcher channel closed unexpectedly".to_string());
                                break miniextendr_api::ffi::R_NilValue;
                            }
                        };

                        match message {
                            RuntimeMessage::Task { job, reply } => {
                                let result =
                                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(job));
                                match result {
                                    Ok(value) => {
                                        if reply.send(Ok(value)).is_err() && pending_error.is_none()
                                        {
                                            *pending_error = Some(
                                                "worker dropped reply channel unexpectedly"
                                                    .to_string(),
                                            );
                                        }
                                    }
                                    Err(panic) => {
                                        let message = match panic.downcast::<String>() {
                                            Ok(message) => *message,
                                            Err(panic) => match panic.downcast::<&'static str>() {
                                                Ok(message) => (*message).to_string(),
                                                Err(_) => "panic payload could not be unpacked"
                                                    .to_string(),
                                            },
                                        };
                                        reply.send(Err(message.clone())).unwrap();
                                        *pending_error = Some(message);
                                        break miniextendr_api::ffi::R_NilValue;
                                    }
                                }
                            }
                            RuntimeMessage::Flush => break miniextendr_api::ffi::R_NilValue,
                        }
                    }
                },
                {
                    let hook_ptr: *mut Option<
                        Box<dyn Fn(&std::panic::PanicHookInfo) + Send + Sync + 'static>,
                    > = &mut hook_slot;
                    move |_jump| {
                        let hook = (*hook_ptr).take();
                        if let Some(hook) = hook {
                            std::panic::set_hook(hook);
                        }
                    }
                },
            )
        };
    });

    if let Some(hook) = hook_slot.take() {
        std::panic::set_hook(hook);
    }

    let worker_result: Result<SendSEXP, String> = match rust_worker_thread.join() {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(message)) => Err(message),
        Err(panic) => Err(match panic.downcast::<String>() {
            Ok(message) => *message,
            Err(panic) => match panic.downcast::<&'static str>() {
                Ok(message) => (*message).to_string(),
                Err(_) => "panic payload could not be unpacked".to_string(),
            },
        }),
    };

    if let Some(message) = dispatcher_error {
        let c_message = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("<invalid panic message>").unwrap());
        loop {
            unsafe {
                ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c_message.as_ptr());
            }
        }
    }

    match worker_result {
        Ok(result) => result.get(),
        Err(message) => {
            let c_message = std::ffi::CString::new(message)
                .unwrap_or_else(|_| std::ffi::CString::new("<invalid panic message>").unwrap());
            loop {
                unsafe {
                    ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c_message.as_ptr());
                }
            }
        }
    }
}

// endregion
