//! Fixtures for the `R_ext/Applic.h` declarations in `miniextendr_api::sys`
//! (`lbfgsb` with `optimfn` / `optimgr` callbacks), called through `.Call`
//! so the package build links `lbfgsb` the way a downstream package would.
//!
//! The objective is the weighted quadratic `sum_i w_i (x_i - c_i)^2`, whose
//! constrained minimum is known in closed form, so the testthat suite can
//! check results against both the closed form and `stats::optim`.
//!
//! Two fixtures drive `lbfgsb` into its own R errors (`nreport = 0`, and a
//! non-finite objective). An R error longjmps over the Rust frames between
//! `lbfgsb` and the framework's `R_UnwindProtect`, so their state lives in
//! stack arrays with no destructors to skip.

use miniextendr_api::list::List;
use miniextendr_api::sys::lbfgsb;
use miniextendr_api::{IntoList, miniextendr};
use std::any::Any;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

// region: quadratic objective

/// Objective state passed to the callbacks through `ex`.
struct Quadratic<'a> {
    centre: &'a [f64],
    weight: &'a [f64],
    fn_calls: i32,
    gr_calls: i32,
    /// Panic inside the objective on this (1-based) call; 0 never.
    panic_on_fn_call: i32,
    /// First panic payload caught in a callback, resumed after `lbfgsb`.
    panic: Option<Box<dyn Any + Send>>,
}

impl Quadratic<'_> {
    fn value(&mut self, par: &[f64]) -> f64 {
        self.fn_calls += 1;
        if self.fn_calls == self.panic_on_fn_call {
            panic!("objective panicked on call {}", self.fn_calls);
        }
        par.iter()
            .zip(self.centre)
            .zip(self.weight)
            .map(|((x, c), w)| w * (x - c) * (x - c))
            .sum()
    }

    fn gradient(&mut self, par: &[f64], gr: &mut [f64]) {
        self.gr_calls += 1;
        for (((g, x), c), w) in gr.iter_mut().zip(par).zip(self.centre).zip(self.weight) {
            *g = 2.0 * w * (x - c);
        }
    }
}

/// `optimfn` callback. A panic is caught here (it must not unwind through
/// R's C frames), stored in the state, and a finite value is returned so
/// `lbfgsb` does not raise its "finite values" error.
unsafe extern "C-unwind" fn quadratic_fn(n: c_int, par: *mut f64, ex: *mut c_void) -> f64 {
    // SAFETY: `ex` is the `Quadratic` passed to `lbfgsb`, alive for the call;
    // R passes `n` parameter values.
    let state = unsafe { &mut *ex.cast::<Quadratic<'_>>() };
    let n = usize::try_from(n).expect("lbfgsb passes n >= 0");
    let par = unsafe { std::slice::from_raw_parts(par, n) };
    match catch_unwind(AssertUnwindSafe(|| state.value(par))) {
        Ok(value) => value,
        Err(payload) => {
            state.panic.get_or_insert(payload);
            0.0
        }
    }
}

/// `optimgr` callback, with the same panic handling (zeros on panic).
unsafe extern "C-unwind" fn quadratic_gr(n: c_int, par: *mut f64, gr: *mut f64, ex: *mut c_void) {
    // SAFETY: as in `quadratic_fn`; `gr` has `n` writable slots.
    let state = unsafe { &mut *ex.cast::<Quadratic<'_>>() };
    let n = usize::try_from(n).expect("lbfgsb passes n >= 0");
    let par = unsafe { std::slice::from_raw_parts(par, n) };
    let gr = unsafe { std::slice::from_raw_parts_mut(gr, n) };
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| state.gradient(par, gr))) {
        gr.fill(0.0);
        state.panic.get_or_insert(payload);
    }
}

/// What `lbfgsb` reports, plus the callbacks' own call counts.
#[derive(IntoList)]
struct LbfgsbOutcome {
    par: Vec<f64>,
    value: f64,
    fail: i32,
    fncount: i32,
    grcount: i32,
    message: String,
    fn_calls: i32,
    gr_calls: i32,
}

/// Minimise `sum(weight * (x - centre)^2)` with `lbfgsb` (`optim`'s
/// defaults: `lmm = 5`, `factr = 1e7`, `pgtol = 0`).
///
/// Returns a list with `par`, `value`, `fail`, `fncount`, `grcount`,
/// `message` (as reported by `lbfgsb`) and `fn_calls` / `gr_calls` (counted
/// by the callbacks). If the objective panics, the panic is caught inside the
/// callback and re-raised after `lbfgsb` returns, as a Rust error.
/// @param centre Numeric vector, the unconstrained minimiser.
/// @param weight Numeric vector of positive weights, same length.
/// @param x0 Numeric vector of starting values, same length.
/// @param lower Numeric vector of lower bounds, same length.
/// @param upper Numeric vector of upper bounds, same length.
/// @param nbd Integer vector of bound codes: 0 none, 1 lower, 2 both, 3 upper.
/// @param maxit Integer scalar iteration limit.
/// @param panic_on_fn_call Integer scalar: panic in this objective call (0 never).
#[miniextendr]
#[allow(clippy::too_many_arguments)]
pub fn lbfgsb_quadratic(
    centre: Vec<f64>,
    weight: Vec<f64>,
    mut x0: Vec<f64>,
    mut lower: Vec<f64>,
    mut upper: Vec<f64>,
    mut nbd: Vec<i32>,
    maxit: i32,
    panic_on_fn_call: i32,
) -> List {
    let len = centre.len();
    for (name, other) in [
        ("weight", weight.len()),
        ("x0", x0.len()),
        ("lower", lower.len()),
        ("upper", upper.len()),
        ("nbd", nbd.len()),
    ] {
        assert_eq!(other, len, "`{name}` must have the length of `centre`");
    }
    let n = c_int::try_from(len).expect("too many parameters for lbfgsb");

    let mut state = Quadratic {
        centre: &centre,
        weight: &weight,
        fn_calls: 0,
        gr_calls: 0,
        panic_on_fn_call,
        panic: None,
    };
    let mut fmin = f64::NAN;
    let mut fail: c_int = -1;
    let mut fncount: c_int = -1;
    let mut grcount: c_int = -1;
    let mut msg: [c_char; 60] = [0; 60];
    // SAFETY: every array holds `n` elements, `msg` has the 60 bytes lbfgsb
    // writes, and `state` outlives the call. The R_alloc'd workspace is
    // reclaimed when this `.Call` returns.
    unsafe {
        lbfgsb(
            n,
            5,
            x0.as_mut_ptr(),
            lower.as_mut_ptr(),
            upper.as_mut_ptr(),
            nbd.as_mut_ptr(),
            &mut fmin,
            quadratic_fn,
            quadratic_gr,
            &mut fail,
            std::ptr::from_mut(&mut state).cast::<c_void>(),
            1e7,
            0.0,
            &mut fncount,
            &mut grcount,
            maxit,
            msg.as_mut_ptr(),
            0,
            10,
        );
    }
    if let Some(payload) = state.panic.take() {
        // No R frame is on the stack any more: let the framework turn it into
        // an R error.
        resume_unwind(payload);
    }
    // SAFETY: lbfgsb strcpy's a NUL-terminated status (< 60 bytes) into msg.
    let message = unsafe { CStr::from_ptr(msg.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    LbfgsbOutcome {
        par: x0,
        value: fmin,
        fail,
        fncount,
        grcount,
        message,
        fn_calls: state.fn_calls,
        gr_calls: state.gr_calls,
    }
    .into_list()
}

// endregion

// region: lbfgsb's own R errors

/// Objective that is never finite, to reach lbfgsb's "finite values" error.
unsafe extern "C-unwind" fn infinite_fn(_n: c_int, _par: *mut f64, _ex: *mut c_void) -> f64 {
    f64::INFINITY
}

/// Gradient that is never called before the objective's error.
unsafe extern "C-unwind" fn zero_gr(n: c_int, _par: *mut f64, gr: *mut f64, _ex: *mut c_void) {
    let n = usize::try_from(n).expect("lbfgsb passes n >= 0");
    // SAFETY: lbfgsb passes `n` writable gradient slots.
    unsafe { std::slice::from_raw_parts_mut(gr, n) }.fill(0.0);
}

/// Call `lbfgsb` on two unbounded parameters with stack-only state, so an R
/// error raised inside it skips no Rust destructor.
fn lbfgsb_on_stack(fn_: miniextendr_api::sys::optimfn, nreport: c_int) {
    let mut x = [1.0, 2.0];
    let mut lower = [0.0; 2];
    let mut upper = [0.0; 2];
    let mut nbd: [c_int; 2] = [0; 2];
    let mut fmin = 0.0;
    let mut fail: c_int = 0;
    let mut fncount: c_int = 0;
    let mut grcount: c_int = 0;
    let mut msg: [c_char; 60] = [0; 60];
    // SAFETY: arrays of n = 2, 60-byte msg; the callbacks ignore `ex`.
    unsafe {
        lbfgsb(
            2,
            5,
            x.as_mut_ptr(),
            lower.as_mut_ptr(),
            upper.as_mut_ptr(),
            nbd.as_mut_ptr(),
            &mut fmin,
            fn_,
            zero_gr,
            &mut fail,
            std::ptr::null_mut(),
            1e7,
            0.0,
            &mut fncount,
            &mut grcount,
            100,
            msg.as_mut_ptr(),
            0,
            nreport,
        );
    }
}

/// Call `lbfgsb` with `nREPORT = 0`, which it rejects with an R error.
#[miniextendr]
pub fn lbfgsb_report_zero() {
    lbfgsb_on_stack(infinite_fn, 0);
}

/// Call `lbfgsb` with an objective returning `Inf`, which it rejects with an
/// R error.
#[miniextendr]
pub fn lbfgsb_infinite_objective() {
    lbfgsb_on_stack(infinite_fn, 10);
}

// endregion
