use std::{panic::catch_unwind, sync::mpsc};

use miniextendr_api::{miniextendr, miniextendr_module};

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

type RTask = Box<dyn FnMut() -> ::miniextendr_api::ffi::SEXP + Send>;

enum MainReq {
    /// Run a batch of R API calls on main under one guard.
    RGuard {
        task: RTask,
        reply: mpsc::SyncSender<Result<::miniextendr_api::ffi::SendSEXP, ()>>,
    },
}

// ---------- R guard trampolines ----------
#[repr(C)]
struct TaskCtx {
    task: Option<RTask>,
    reply: Option<mpsc::SyncSender<Result<::miniextendr_api::ffi::SendSEXP, ()>>>,
}

pub fn payload_to_r_error(payload: Box<dyn std::any::Any + Send + 'static>) -> ! {
    let msg: String = if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "rust panic".to_string()
    };

    // TODO: add panic_any here, so that packages can hook in their own handling of
    // error messages

    let cmsg = std::ffi::CString::new(msg)
        .map(|x| x.as_ptr())
        .unwrap_or_else(|_| c"unexplained rust panic".as_ptr());
    // Triggers R’s nonlocal exit; clean_tramp will run and signal Err(()):
    unsafe { ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), cmsg) };
}

unsafe extern "C" fn r_fun_tramp(p: *mut std::ffi::c_void) -> ::miniextendr_api::ffi::SEXP {
    // normal path: run task, send Ok, return SEXP
    let ctx = unsafe { p.cast::<TaskCtx>().as_mut().unwrap() };
    let Some(f) = ctx.task.as_mut() else {
        unreachable!()
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || f())) {
        Ok(ans) => {
            let _ = ctx.reply.take().and_then(|tx| {
                tx.send(Ok(unsafe { ::miniextendr_api::ffi::SendSEXP::new(ans) }))
                    .ok()
            });
            ans
        }
        Err(payload) => {
            let _ = ctx.reply.take().and_then(|tx| tx.send(Err(())).ok());
            payload_to_r_error(payload)
        }
    }
}

unsafe extern "C" fn r_clean_tramp(
    p: *mut std::ffi::c_void,
    _jumping: ::miniextendr_api::ffi::Rboolean,
) {
    // longjmp path: signal Err to unblock worker and drop the task
    let ctx = unsafe { p.cast::<TaskCtx>().as_mut().unwrap() };
    if let Some(tx) = ctx.reply.take() {
        let _ = tx.send(Err(()));
    }
    // release the task runner captures
    drop(ctx.task.take());
}

// Run one task on main under R_UnwindProtect.
fn run_on_main(task: RTask, reply: mpsc::SyncSender<Result<::miniextendr_api::ffi::SendSEXP, ()>>) {
    let mut ctx = TaskCtx {
        task: Some(task),
        reply: Some(reply),
    };
    let p = std::ptr::from_mut(&mut ctx).cast();
    unsafe {
        // TODO: put this in a thread-local and retrieve it/reset it.
        // is resetting necessary? I don't think so.
        let cont = ::miniextendr_api::ffi::R_MakeUnwindCont();
        // If R longjmps, this never returns to Rust; clean_tramp runs.
        let _ = ::miniextendr_api::ffi::R_UnwindProtect(
            Some(r_fun_tramp),
            p,
            Some(r_clean_tramp),
            p,
            cont,
        );
    }
    // normal return just falls through
}

// ---------- worker helpers ----------
#[derive(Clone)]
struct MainSender(mpsc::SyncSender<MainReq>);

impl MainSender {
    fn with_r_guard<F>(&self, f: F) -> Result<::miniextendr_api::ffi::SendSEXP, ()>
    where
        F: FnOnce() -> ::miniextendr_api::ffi::SEXP + Send + 'static,
    {
        // box as FnMut once
        let mut opt = Some(f);
        let task: RTask = Box::new(move || {
            let f = opt.take().unwrap();
            f()
        });
        let (tx, rx) = mpsc::sync_channel(1);
        self.0.send(MainReq::RGuard { task, reply: tx }).ok();
        rx.recv().unwrap_or(Err(()))
    }
}

// ---------- example exported entry ----------
#[miniextendr]
#[unsafe(no_mangle)]
pub extern "C" fn C_rust_worker1() -> miniextendr_api::ffi::SEXP {
    // channel main<->worker
    // TODO: make `bound` configurable
    let (tx, rx) = mpsc::sync_channel::<MainReq>(1);
    let main_tx = MainSender(tx);

    // note: everything outside of the thread will not drop in case of an R error.
    // note: a rust panic here is not good.

    // spawn worker
    let handle = std::thread::spawn(move || -> Result<::miniextendr_api::ffi::SendSEXP, ()> {
        // note: allocations here will deallocate in case of a panic

        // #<number>: cases to consider

        // #3
        // let a = MsgOnDrop;
        #[allow(unreachable_code)] // tests!
        let sexp: ::miniextendr_api::ffi::SendSEXP = main_tx.with_r_guard(move || unsafe {
            // limitation: dropped on a panic, not on an Rf_error!
            // let a = MsgOnDrop;

            // #1
            // panic!("rust panic while running r task");

            // #2
            // ::miniextendr_api::ffi::Rf_error(c"an r error occurred".as_ptr());

            ::miniextendr_api::ffi::R_NilValue
        })?;
        // more Rust work...
        // Finish: send final SEXP (could be `s` or another)
        Ok(sexp)
    });

    // main thread loop: service requests until Done or worker panic/exit
    loop {
        match rx.recv() {
            Ok(MainReq::RGuard { task, reply }) => {
                // Each batch is guarded; R error longjmps, clean_tramp signals Err to worker.
                run_on_main(task, reply);
                // If R longjmps, this frame is unwound by R. Worker is already unblocked.
            }
            Err(_) => {
                // sender closed: worker ended or panicked
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
            drop(rx);
            ::miniextendr_api::ffi::Rf_error(c"R error during guarded call".as_ptr())
        },
        Err(payload) => payload_to_r_error(payload),
    }
}

// endregion
