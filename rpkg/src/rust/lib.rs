use std::{
    any::Any,
    borrow::Cow,
    cell::RefCell,
    ffi::CString,
    panic::AssertUnwindSafe,
    sync::{
        Once, OnceLock,
        mpsc::{Receiver, SyncSender},
    },
};

use miniextendr_api::ffi::{R_NilValue, SEXP, SendSEXP};
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
            miniextendr_api::ffi::Rprintf(c"%s".as_ptr(), c"Dropped `MsgOnDrop`!\n".as_ptr())
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

type WorkerReply = Result<SendSEXP, Cow<'static, str>>;
type ReplySender = SyncSender<WorkerReply>;

enum WorkerCommand {
    Run {
        job: Box<dyn FnOnce() -> WorkerReply + Send>,
    },
}
impl std::fmt::Debug for WorkerCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Run { job: _ } => f
                .debug_struct("Run")
                .field("job", &"Box<dyn FnOnce() -> WorkerReply + Send>")
                .finish(),
        }
    }
}

enum RTask {
    Call {
        job: Box<dyn FnOnce() -> SendSEXP + Send>,
        reply: ReplySender,
    },
    Result(WorkerReply),
}

impl std::fmt::Debug for RTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Call { job: _, reply } => f
                .debug_struct("Call")
                .field("job", &"Box<dyn FnOnce() -> SendSEXP + Send>")
                .field("reply", reply)
                .finish(),
            Self::Result(arg0) => f.debug_tuple("Result").field(arg0).finish(),
        }
    }
}

///
///
/// See [`miniextendr_runtime_init`]
static RUNTIME_ONCE: Once = Once::new();
static WORKER_TX: OnceLock<SyncSender<WorkerCommand>> = OnceLock::new();
///
///
/// See [`R_TASK_RX_SLOT`]
static R_TASK_TX: OnceLock<SyncSender<RTask>> = OnceLock::new();

thread_local! {
    ///
    ///
    ///
    /// See [`R_TASK_TX`]
    static R_TASK_RX_SLOT: RefCell<Option<Receiver<RTask>>> = const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
extern "C" fn miniextendr_runtime_init() {
    RUNTIME_ONCE.call_once_force(|_once_state| {
        // TODO: use _once_state for tracing messages
        let (r_task_tx, r_task_rx) = std::sync::mpsc::sync_channel(0);
        R_TASK_TX
            .set(r_task_tx)
            .expect("R task sender already initialised");
        R_TASK_RX_SLOT.with(|slot| {
            *slot.borrow_mut() = Some(r_task_rx);
        });

        let (worker_tx, worker_rx) = std::sync::mpsc::sync_channel(0);
        WORKER_TX
            .set(worker_tx)
            .expect("worker runtime already initialised");

        std::thread::Builder::new()
            .name("miniextendr worker".to_string())
            .spawn(move || {
                while let Ok(cmd) = worker_rx.recv() {
                    match cmd {
                        WorkerCommand::Run { job } => {
                            let result = match std::panic::catch_unwind(AssertUnwindSafe(job)) {
                                Ok(job_result) => job_result,
                                Err(panic) => Err(panic_payload_to_string(panic)),
                            };
                            R_TASK_TX
                                .get()
                                .unwrap()
                                .send(RTask::Result(result))
                                .unwrap();
                        }
                    }
                }
            })
            .expect("failed to spawn miniextendr worker thread");
    });
}

pub fn with_r<R>(r: R) -> Result<SEXP, Cow<'static, str>>
where
    R: FnOnce() -> SEXP + Send + 'static,
{
    let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(0);
    R_TASK_TX
        .get()
        .expect("runtime not initialised (R task sender)")
        .send(RTask::Call {
            job: Box::new(|| unsafe { SendSEXP::new(r()) }),
            reply: reply_tx,
        })
        .expect("failed to send R task");

    reply_rx
        .recv()
        .expect("R dispatcher dropped reply channel")
        .map(|value| value.get())
}

fn do_nothing() -> Result<SEXP, Cow<'static, str>> {
    with_r(|| {
        use miniextendr_api::ffi;

        // panic!("intentional panic inside with_r"); // #2: test
        unsafe { miniextendr_api::ffi::Rf_error(c"intentional r error inside with_r".as_ptr()) }; // #1: test!

        unsafe { ffi::Rf_ScalarInteger(42) }
    })
}

#[miniextendr]
#[unsafe(no_mangle)]
unsafe extern "C" fn C_do_nothing() -> ::miniextendr_api::ffi::SEXP {
    let hook_guard = RefCell::new(Some(PanicHookGuard::new()));

    WORKER_TX
        .get()
        .expect("worker runtime not initialised")
        .send(WorkerCommand::Run {
            job: Box::new(|| {
                let rust_result = do_nothing();
                rust_result.map(|sexp| unsafe { SendSEXP::new(sexp) })
            }),
        })
        .expect("worker thread exited unexpectedly");

    let worker_result = R_TASK_RX_SLOT.with(|slot| {
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
                RTask::Call { job, reply } => {
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
                                        Err(panic) => Err(panic_payload_to_string(panic)),
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
                                    Ok(RTask::Result(_)) => {}
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
                RTask::Result(result) => break result,
            }
        }
    });

    match worker_result {
        Ok(result) => result.get(),
        Err(message) => raise_r_error(&message),
    }
}

fn panic_payload_to_string(panic: Box<dyn Any + Send>) -> Cow<'static, str> {
    match panic.downcast::<String>() {
        Ok(message) => Cow::Owned(*message),
        Err(panic) => match panic.downcast::<&'static str>() {
            Ok(ref message) => Cow::Borrowed(message),
            Err(_) => Cow::Borrowed("panic payload could not be unpacked"),
        },
    }
}

fn raise_r_error(message: &str) -> ! {
    let c_message =
        CString::new(message).unwrap_or_else(|_| CString::new("<invalid panic message>").unwrap());
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c_message.as_ptr());
    }
}

type PanicHook = Box<dyn Fn(&std::panic::PanicHookInfo) + Send + Sync + 'static>;
struct PanicHookGuard(Option<PanicHook>);

impl PanicHookGuard {
    fn new() -> Self {
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
        Self(Some(old))
    }
}

impl Drop for PanicHookGuard {
    fn drop(&mut self) {
        if let Some(old) = self.0.take() {
            unsafe {
                miniextendr_api::ffi::Rprintf(c"%s".as_ptr(), c"Reset panic hook!\n".as_ptr())
            };

            std::panic::set_hook(old);
        }
    }
}

// endregion
