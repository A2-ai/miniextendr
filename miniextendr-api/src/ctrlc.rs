//! Opt-in cooperative Ctrl+C handling.
//!
//! Call [`check_interrupt`] periodically in long-running Rust loops. A checkpoint
//! catches R's interrupt below the caller's Rust frames, then unwinds those frames
//! normally. The generated wrapper receives a tagged value and signals an R
//! condition with classes `rust_interrupt`, `interrupt`, `condition`.
//!
//! This does not preempt Rust code or protect arbitrary R API calls. Interrupts
//! raised inside other R calls still need their own leaf fences (see issue #1507).
//! Checkpoints work on the R thread and on the framework's worker thread.
//!
//! Registration tries `ctrlc::try_set_handler` once on Unix. R normally already
//! owns SIGINT, so this declines without replacing it. Windows uses R's handler
//! directly: `try_set_handler` there would add a handler ahead of R's, consuming
//! console events. Wrapper-generation loads never register a signal handler.

use std::ffi::c_void;
use std::sync::OnceLock;

use crate::condition::RCondition;
use crate::{IntoR, SEXP, SexpExt, sys};

#[cfg(all(unix, not(target_family = "wasm")))]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(unix, not(target_family = "wasm")))]
static PENDING: AtomicBool = AtomicBool::new(false);
static HANDLER_INSTALLED: OnceLock<bool> = OnceLock::new();
static CONDITION_CLASSES: OnceLock<SEXP> = OnceLock::new();

/// Initialize once per package image, on R's main thread during registration.
pub(crate) fn initialize() {
    CONDITION_CLASSES.get_or_init(|| unsafe {
        let classes = ["interrupt", "error"].into_sexp();
        let protected = crate::gc_protect::OwnedProtect::new(classes);
        sys::R_PreserveObject(protected.get());
        classes
    });
    HANDLER_INSTALLED.get_or_init(|| {
        #[cfg(all(unix, not(target_family = "wasm")))]
        {
            // Never overwrite R's or another package's signal handler. In a
            // normal R console MultipleHandlers is expected; native polling is
            // still available when installation fails for any reason.
            ::ctrlc::try_set_handler(|| PENDING.store(true, Ordering::Release)).is_ok()
        }
        #[cfg(not(all(unix, not(target_family = "wasm"))))]
        {
            false
        }
    });
}

/// Whether registration installed a Unix handler (normally false inside R).
///
/// An embedding host with R's signal handlers disabled can allow installation.
/// Such a host must keep the package DLL loaded for the process lifetime: the
/// ctrlc crate owns a permanent handler thread and has no unregister operation.
pub fn handler_installed() -> bool {
    *HANDLER_INSTALLED
        .get()
        .expect("package routines are registered")
}

/// Check for cancellation and unwind Rust to the generated wrapper if interrupted.
///
/// Call this in long-running loops, for example once per chunk of work. Rust
/// destructors run before the R wrapper signals the interrupt. The checkpoint
/// also processes R GUI events and respects `suspendInterrupts()` when R owns
/// the handler. R time-limit errors remain errors, not interrupts.
///
/// Call only from an initialized miniextendr entry point, on R's main thread or
/// the framework worker. A loop with no checkpoints remains non-interruptible.
///
/// ```ignore
/// #[miniextendr]
/// fn compute() -> i32 {
///     let resource = acquire_resource();
///     for chunk in work_chunks() {
///         miniextendr_api::ctrlc::check_interrupt();
///         process(chunk, &resource);
///     }
///     42
/// }
/// ```
pub fn check_interrupt() {
    #[cfg(all(unix, not(target_family = "wasm")))]
    if PENDING.swap(false, Ordering::AcqRel) {
        std::panic::resume_unwind(Box::new(RCondition::Interrupt));
    }
    // Return owned, Send-safe condition data to a worker before unwinding it.
    // No Rust panic crosses R_tryCatch's live R contexts.
    let condition = crate::worker::with_r_thread(|| unsafe { poll_r() });
    if let Some(condition) = condition {
        std::panic::resume_unwind(Box::new(condition));
    }
}

unsafe fn poll_r() -> Option<RCondition> {
    unsafe extern "C-unwind" fn poll(_: *mut c_void) -> SEXP {
        unsafe { sys::R_CheckUserInterrupt_unchecked() };
        SEXP::nil()
    }
    unsafe extern "C-unwind" fn caught(condition: SEXP, _: *mut c_void) -> SEXP {
        condition
    }
    let condition = unsafe {
        sys::R_tryCatch(
            Some(poll),
            std::ptr::null_mut(),
            *CONDITION_CLASSES
                .get()
                .expect("package routines are registered"),
            Some(caught),
            std::ptr::null_mut(),
            None,
            std::ptr::null_mut(),
        )
    };
    if condition == SEXP::nil() {
        return None;
    }
    let _root = unsafe { crate::gc_protect::OwnedProtect::new(condition) };
    if condition.inherits_class(c"interrupt") {
        Some(RCondition::Interrupt)
    } else {
        // R_CheckUserInterrupt also checks stack and elapsed/CPU time limits.
        // Those produce simpleError objects; preserve their message separately
        // from interrupts rather than misclassifying every non-local exit.
        let message = unsafe { crate::list::List::from_raw(condition) }
            .get_named::<String>("message")
            .expect("R_CheckUserInterrupt error has a message");
        Some(RCondition::Error {
            message,
            class: Vec::new(),
            data: None,
        })
    }
}

/// Raise only after a legacy/ALTREP guard has already unwound its Rust body.
pub(crate) unsafe fn raise_interrupt(call: Option<SEXP>) -> ! {
    unsafe {
        let scope = crate::gc_protect::ProtectScope::new();
        let message = scope.protect_raw("Interrupted".into_sexp());
        let condition = scope.protect_raw(
            crate::list::List::from_values(vec![message, call.unwrap_or(SEXP::nil())]).as_sexp(),
        );
        let names = scope.protect_raw(["message", "call"].into_sexp());
        condition.set_names(names);
        let classes = scope.protect_raw(["rust_interrupt", "interrupt", "condition"].into_sexp());
        condition.set_class(classes);
        let stop = sys::Rf_install(c"stop".as_ptr());
        let expr = scope.protect_raw(sys::Rf_lang2(stop, condition));
        sys::Rf_eval(expr, sys::R_BaseEnv);
        unreachable!("base::stop does not return")
    }
}
