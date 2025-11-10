use crate::{
    ffi::{R_NilValue, SEXP, SendSEXP},
    into_r::IntoR,
    unwind_protect::with_unwind_protect,
};

///
///
/// See [`miniextendr_runtime_init`]
static RUNTIME_ONCE: std::sync::Once = std::sync::Once::new();

///
///
/// See [`R_TASK_RX_SLOT`]
static R_TASK_TX: std::sync::OnceLock<std::sync::mpsc::SyncSender<RTask>> =
    std::sync::OnceLock::new();

thread_local! {
    ///
    ///
    ///
    /// See [`R_TASK_TX`]
    pub static R_TASK_RX_SLOT: std::cell::RefCell<Option<std::sync::mpsc::Receiver<RTask>>>  = const { std::cell::RefCell::new(None) };
}

pub static WORKER_TX: std::sync::OnceLock<std::sync::mpsc::SyncSender<WorkerCommand>> =
    std::sync::OnceLock::new();

type WorkerReply = Result<SendSEXP, std::borrow::Cow<'static, str>>;
type ReplySender = std::sync::mpsc::SyncSender<WorkerReply>;

#[doc(hidden)]
pub enum WorkerCommand {
    Run {
        job: Box<dyn FnOnce() -> WorkerReply + Send>,
        reply: ReplySender,
    },
}
impl std::fmt::Debug for WorkerCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Run { job: _, .. } => f
                .debug_struct("Run")
                .field("job", &"Box<dyn FnOnce() -> WorkerReply + Send>")
                .finish(),
        }
    }
}

#[doc(hidden)]
pub enum RTask {
    Call {
        job: Box<dyn FnOnce() -> SendSEXP + Send>,
        reply: ReplySender,
    },
    Result(WorkerReply),
    Wake,
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
            Self::Wake => f.debug_struct("Wake").finish(),
        }
    }
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

        // Use a small buffer to avoid circular deadlocks when the worker is
        // blocked sending a result back to the dispatcher while the dispatcher
        // is attempting to enqueue the next job.
        let (worker_tx, worker_rx) = std::sync::mpsc::sync_channel(1);
        WORKER_TX
            .set(worker_tx)
            .expect("worker runtime already initialised");

        std::thread::Builder::new()
            .name("miniextendr worker".to_string())
            .spawn(move || {
                while let Ok(cmd) = worker_rx.recv() {
                    match cmd {
                        WorkerCommand::Run { job, reply } => {
                            let result =
                                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)) {
                                    Ok(job_result) => job_result,
                                    Err(panic) => {
                                        Err(miniextendr_api::unwind::panic_payload_to_string(panic))
                                    }
                                };
                            reply.send(result).unwrap();
                            if let Some(tx) = R_TASK_TX.get() {
                                // Wake the dispatcher in case it's blocked waiting for work.
                                let _ = tx.send(RTask::Wake);
                            }
                        }
                    }
                }
            })
            .expect("failed to spawn miniextendr worker thread");
    });
}

pub fn with_r<R, T>(r: R) -> Result<SEXP, std::borrow::Cow<'static, str>>
where
    R: FnOnce() -> T + Send + 'static,
    T: IntoR + 'static,
{
    let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(0);
    R_TASK_TX
        .get()
        .expect("runtime not initialised (R task sender)")
        .send(RTask::Call {
            job: Box::new(|| unsafe { SendSEXP::new(r().into_sexp()) }),
            reply: reply_tx,
        })
        .expect("failed to send R task");

    reply_rx
        .recv()
        .expect("R dispatcher dropped reply channel")
        .map(|value| value.get())
}

pub unsafe fn call_worker<F>(call: SEXP, job: F) -> SEXP
where
    F: FnOnce() -> WorkerReply + Send + 'static,
{
    let hook_guard = std::cell::RefCell::new(Some(PanicHookGuard::print_error_location()));

    let (worker_reply_tx, worker_reply_rx) = std::sync::mpsc::sync_channel(1);

    WORKER_TX
        .get()
        .expect("worker runtime not initialised")
        .send(WorkerCommand::Run {
            job: Box::new(job),
            reply: worker_reply_tx,
        })
        .expect("worker thread exited unexpectedly");

    let worker_result = R_TASK_RX_SLOT.with(|slot| {
        loop {
            // If the worker has finished, stop dispatching R tasks and return.
            if let Ok(done) = worker_reply_rx.try_recv() {
                break done;
            }

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
                    let reply_slot = std::cell::RefCell::new(Some(reply));
                    let reply_slot = &reply_slot;

                    unsafe {
                        with_unwind_protect(
                            move || {
                                let result = match std::panic::catch_unwind(
                                    std::panic::AssertUnwindSafe(|| {
                                        job.take().expect("R task already consumed by dispatcher")()
                                    }),
                                ) {
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
                                        .send(Err(std::borrow::Cow::Borrowed(
                                            "non-local jump while executing R task",
                                        )))
                                        .unwrap();
                                }
                                // Do not touch the R task receiver here.
                                // Let the outer loop receive the next message (likely the worker result).
                            },
                        );
                    };
                }
                // Prior versions sent worker results over this channel.
                // If encountered, ignore and continue; the real result is read
                // from `worker_reply_rx` above which is correctly associated to
                // this call.
                RTask::Result(_result) => continue,
                RTask::Wake => continue,
            }
        }
    });

    match worker_result {
        Ok(result) => {
            hook_guard.borrow_mut().take();
            result.get()
        }
        Err(message) => {
            hook_guard.borrow_mut().take();
            unsafe { raise_r_error_call(call, &message) }
        }
    }
}

#[doc(hidden)]
pub fn panic_payload_to_string(
    panic: Box<dyn std::any::Any + Send>,
) -> std::borrow::Cow<'static, str> {
    match panic.downcast::<String>() {
        Ok(message) => std::borrow::Cow::Owned(*message),
        Err(panic) => match panic.downcast::<&'static str>() {
            Ok(ref message) => std::borrow::Cow::Borrowed(message),
            Err(_) => std::borrow::Cow::Borrowed("panic payload could not be unpacked"),
        },
    }
}

#[doc(hidden)]
pub unsafe fn raise_r_error_call(call: SEXP, message: &str) -> ! {
    let c_message = std::ffi::CString::new(message)
        .unwrap_or_else(|_| std::ffi::CString::new("<invalid panic message>").unwrap());
    unsafe {
        ::miniextendr_api::ffi::Rf_errorcall(call, c"%s".as_ptr(), c_message.as_ptr());
    }
}

#[doc(hidden)]
pub fn raise_r_error(message: &str) -> ! {
    let c_message = std::ffi::CString::new(message)
        .unwrap_or_else(|_| std::ffi::CString::new("<invalid panic message>").unwrap());
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c_message.as_ptr());
    }
}

type PanicHook = Box<dyn Fn(&std::panic::PanicHookInfo) + Send + Sync + 'static>;

pub struct PanicHookGuard(Option<PanicHook>);

impl PanicHookGuard {
    pub fn print_error_location() -> Self {
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

    pub fn print_nothing() -> Self {
        let old = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_panic_info| {}));
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
