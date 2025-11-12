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
    /// When true, the global panic hook prints a concise file:line banner.
    pub static PANIC_BANNER_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    ///
    ///
    /// See [`R_TASK_TX`]
    pub static R_TASK_RX_SLOT: std::cell::RefCell<Option<std::sync::mpsc::Receiver<RTask>>>  = const { std::cell::RefCell::new(None) };
}

static WORKER_TX: std::sync::OnceLock<std::sync::mpsc::SyncSender<WorkerCommand>> =
    std::sync::OnceLock::new();

type WorkerReply = Result<SendSEXP, std::borrow::Cow<'static, str>>;
type ReplySender = std::sync::mpsc::SyncSender<WorkerReply>;

#[doc(hidden)]
enum WorkerCommand {
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
enum RTask {
    Call {
        job: Box<dyn FnOnce() -> SendSEXP + Send>,
        reply: ReplySender,
    },
    Wake,
    /// FIFO barrier used to drain all pending R tasks before continuing.
    /// The dispatcher acks this by sending `()` when the barrier is reached.
    Barrier {
        ack: std::sync::mpsc::SyncSender<()>,
    },
}

impl std::fmt::Debug for RTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Call { job: _, reply } => f
                .debug_struct("Call")
                .field("job", &"Box<dyn FnOnce() -> SendSEXP + Send>")
                .field("reply", reply)
                .finish(),
            Self::Wake => f.debug_struct("Wake").finish(),
            Self::Barrier { .. } => f.debug_struct("Barrier").finish(),
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn miniextendr_runtime_init() {
    RUNTIME_ONCE.call_once_force(|_once_state| {
        // TODO: use _once_state for tracing messages
        // Install a single global panic hook that is gated by a TLS flag so that
        // only threads that opt-in print the concise banner.
        let old = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            if PANIC_BANNER_ENABLED.with(|flag| flag.get()) {
                if let Some(location) = panic_info.location() {
                    eprintln!(
                        "Rust panic at src/rust/{}:{}",
                        location.file(),
                        location.line()
                    );
                }
            } else {
                (old)(panic_info);
            }
        }));

        let (r_task_tx, r_task_rx) = std::sync::mpsc::sync_channel(1);
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

        // Enable concise panic banner on the R dispatcher thread as well.
        PANIC_BANNER_ENABLED.with(|flag| flag.set(true));

        std::thread::Builder::new()
            .name("miniextendr worker".to_string())
            .spawn(move || {
                // Enable panic banner printing in the worker thread.
                PANIC_BANNER_ENABLED.with(|flag| flag.set(true));
                while let Ok(cmd) = worker_rx.recv() {
                    match cmd {
                        WorkerCommand::Run { job, reply } => {
                            // Catch panics and, on error, insert a FIFO barrier so
                            // the dispatcher drains all pending R tasks before we
                            // propagate the error back to R.
                            let result =
                                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)) {
                                    Ok(job_result) => job_result,
                                    Err(panic) => {
                                        // Request the dispatcher to drain pending R tasks.
                                        if let Some(tx) = R_TASK_TX.get() {
                                            let (ack_tx, ack_rx) = std::sync::mpsc::sync_channel(0);
                                            // This blocks until the dispatcher reaches the barrier,
                                            // guaranteeing FIFO drain of earlier tasks.
                                            let _ = tx.send(RTask::Barrier { ack: ack_tx });
                                            let _ = ack_rx.recv();
                                        }
                                        Err(miniextendr_api::unwind::panic_payload_to_string(panic))
                                    }
                                };
                            reply.send(result).unwrap();
                            if let Some(tx) = R_TASK_TX.get() {
                                // Wake the dispatcher in case it's blocked waiting for work.
                                let _ = tx.try_send(RTask::Wake);
                            }
                        }
                    }
                }
            })
            .expect("failed to spawn miniextendr worker thread");
    });
}

pub fn with_r<R, T>(r: R) -> std::result::Result<SEXP, std::borrow::Cow<'static, str>>
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

/// Convenience for exported FFI wrappers: run `r` on the R dispatcher and
/// throw via `Rf_errorcall` on failure so that errors surface as R conditions.
pub unsafe fn with_r_throw<R, T>(call: SEXP, r: R) -> SEXP
where
    R: FnOnce() -> T + Send + 'static,
    T: IntoR + 'static,
{
    match with_r(r) {
        Ok(sexp) => sexp,
        Err(message) => unsafe { raise_r_error_call(call, &message) },
    }
}

pub unsafe fn call_worker<F>(call: SEXP, job: F) -> SEXP
where
    F: FnOnce() -> WorkerReply + Send + 'static,
{
    // Enable panic banner printing on the R dispatcher thread for this call.
    PANIC_BANNER_ENABLED.with(|flag| flag.set(true));
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
                RTask::Wake => continue,
                RTask::Barrier { ack } => {
                    // Acknowledge the barrier to let the worker continue.
                    ack.send(()).unwrap();
                    continue;
                }
            }
        }
    });

    match worker_result {
        Ok(result) => result.get(),
        Err(message) => unsafe { raise_r_error_call(call, &message) },
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

// Deprecated: per-call panic hook swapping removed in favor of a single global
// hook gated by a thread-local flag.
