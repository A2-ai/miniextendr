//!
//!
//!
use std::{cell::LazyCell, sync::mpsc};

pub type RTask = Box<dyn FnOnce() -> crate::ffi::SEXP + Send>;

pub static MAIN_TX: std::sync::OnceLock<crate::unwind::MainSender> = std::sync::OnceLock::new();
pub static MAIN_RX: std::sync::OnceLock<
    std::sync::Mutex<mpsc::Receiver<crate::unwind::MainRequest>>,
> = std::sync::OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn rpkg_main_queue_init() {
    // TODO: retrieve these statics from a symbol attached to the R runtime
    // in order to achieve persistence _across_ extendr-based R packages.
    if MAIN_TX.get().is_some() || MAIN_RX.get().is_some() {
        return;
    }
    let (tx, rx) = mpsc::sync_channel::<crate::unwind::MainRequest>(1);
    let _ = MAIN_TX.set(crate::unwind::MainSender(tx));
    let _ = MAIN_RX.set(std::sync::Mutex::new(rx));
}

#[derive(Clone)]
pub struct MainSender(pub mpsc::SyncSender<MainRequest>);

#[inline]
pub fn with_r_guard<F>(f: F) -> Result<crate::ffi::SendSEXP, ()>
where
    F: FnOnce() -> crate::ffi::SEXP + Send + 'static,
{
    let task: RTask = Box::new(f);
    let (tx, rx) = mpsc::sync_channel(1);
    let _ = MAIN_TX
        .get()
        .unwrap()
        .0
        .send(MainRequest::RGuard { task, reply: tx })
        .ok()
        .unwrap();
    rx.recv().unwrap_or(Err(()))
}

// TODO: make sure that there is a reason to use with_r_guard_ref / with_r_guard_mut...
// obviously those would be better because an Rf_error will result in them deallocating things properly.
// TODO: test if with_r_guard_ref/with_r_guard_mut work with the current setup...

#[inline]
pub fn with_r_guard_ref<F>(f: F) -> Result<crate::ffi::SendSEXP, ()>
where
    F: Fn() -> crate::ffi::SEXP + Send + 'static,
{
    let task: RTask = Box::new(f);
    let (tx, rx) = mpsc::sync_channel(1);
    let _ = MAIN_TX
        .get()
        .unwrap()
        .0
        .send(MainRequest::RGuard { task, reply: tx })
        .ok()
        .unwrap();
    rx.recv().unwrap_or(Err(()))
}

#[inline]
pub fn with_r_guard_mut<F>(f: F) -> Result<crate::ffi::SendSEXP, ()>
where
    F: FnMut() -> crate::ffi::SEXP + Send + 'static,
{
    let task: RTask = Box::new(f);
    let (tx, rx) = mpsc::sync_channel(1);
    let _ = MAIN_TX
        .get()
        .unwrap()
        .0
        .send(MainRequest::RGuard { task, reply: tx })
        .ok()
        .unwrap();
    rx.recv().unwrap_or(Err(()))
}

pub enum MainRequest {
    /// Run a batch of R API calls on main under one guard.
    RGuard {
        task: RTask,
        reply: mpsc::SyncSender<Result<crate::ffi::SendSEXP, ()>>,
    },
    Done,
}

impl std::fmt::Debug for MainRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RGuard { task: _, reply } => f
                .debug_struct("RGuard")
                .field("task", &())
                .field("reply", reply)
                .finish(),
            Self::Done => write!(f, "Done"),
        }
    }
}

#[repr(C)]
pub struct TaskCtx {
    task: Option<RTask>,
    reply: Option<mpsc::SyncSender<Result<crate::ffi::SendSEXP, ()>>>,
}

#[inline(always)]
pub fn payload_to_r_error(payload: Box<dyn std::any::Any + Send>) -> ! {
    let msg: String = if let Some(panic_str) = payload.downcast_ref::<&str>() {
        panic_str.to_string()
    } else if let Some(panic_string) = payload.downcast_ref::<String>() {
        panic_string.clone()
    } else {
        "rust panic".to_string()
    };

    // TODO: add panic_any here, so that packages can hook in their own handling of
    // error messages

    let cmsg = std::ffi::CString::new(msg).unwrap();
    // Triggers R’s nonlocal exit; clean_tramp will run and signal Err(()):
    unsafe { crate::ffi::Rf_error(c"%s".as_ptr(), cmsg.as_ptr()) };
}

#[inline(always)]
pub unsafe extern "C" fn r_fun_tramp(p: *mut std::ffi::c_void) -> crate::ffi::SEXP {
    // normal path: run task, send Ok, return SEXP
    let ctx = unsafe { p.cast::<TaskCtx>().as_mut().unwrap() };
    let f = ctx.task.take().unwrap();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || (f)())) {
        Ok(ans) => {
            let _ = ctx
                .reply
                .take()
                .and_then(|tx| tx.send(Ok(unsafe { crate::ffi::SendSEXP::new(ans) })).ok())
                .unwrap();
            ans
        }
        Err(payload) => {
            let _ = ctx
                .reply
                .take()
                .and_then(|tx| tx.send(Err(())).ok())
                .unwrap();
            payload_to_r_error(payload)
        }
    }
}

#[inline(always)]
pub unsafe extern "C" fn r_clean_tramp(p: *mut std::ffi::c_void, _jumping: crate::ffi::Rboolean) {
    let ctx = unsafe { p.cast::<TaskCtx>().as_mut().unwrap() };
    if let Some(tx) = ctx.reply.take() {
        let _ = tx.try_send(Err(()));
    }
    let _ = ctx.task.take();
}

thread_local! {
    static R_CONTINUATION_TOKEN: LazyCell<crate::ffi::SEXP> = LazyCell::new(|| unsafe {
        // FIXME: protect this token forever using R_PreserveObject
        crate::ffi::R_MakeUnwindCont()
    });
}
// Run one task on main under R_UnwindProtect.
#[inline(always)]
pub fn run_on_main(task: RTask, reply: mpsc::SyncSender<Result<crate::ffi::SendSEXP, ()>>) {
    let mut ctx = TaskCtx {
        task: Some(task),
        reply: Some(reply),
    };
    let p = std::ptr::from_mut(&mut ctx).cast();
    unsafe {
        let _ = crate::ffi::R_UnwindProtect(
            Some(r_fun_tramp),
            p,
            Some(r_clean_tramp),
            p,
            R_CONTINUATION_TOKEN.with(|x| **x),
        );
    }
    // normal return just falls through
}
