//! Integration tests for the `R_ext/Applic.h` declarations in
//! `miniextendr_api::sys` (`lbfgsb` and the `optimfn` / `optimgr` callback
//! types), on analytic quadratics with known minima.
//!
//! The objective is `f(x) = sum_i w_i (x_i - c_i)^2`, so the unconstrained
//! minimum is `x = c` with `f = 0`, and with bounds each coordinate is `c_i`
//! clamped into its interval. The callbacks count their calls and catch
//! panics into the `ex` state, which is the pattern the declarations' docs
//! ask for. No case here makes `lbfgsb` raise an R error (`nreport >= 1`,
//! finite objective); those paths are exercised through `.Call` in the rpkg
//! testthat suite, where the framework catches the longjmp.

mod r_test_utils;

use miniextendr_api::sys::{lbfgsb, vmaxget, vmaxset};
use std::any::Any;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

#[test]
fn lbfgsb_suite() {
    r_test_utils::with_r_thread(|| {
        interior_minimum();
        bound_constrained_minimum();
        fixed_parameter();
        iteration_limit();
        invalid_bound_code();
        infeasible_bounds();
        zero_parameters();
        callback_panic_is_caught_and_resumed();
    });
}

// region: objective state and callbacks

/// Weighted quadratic objective, passed to the callbacks through `ex`.
struct Quadratic {
    centre: Vec<f64>,
    weight: Vec<f64>,
    fn_calls: c_int,
    gr_calls: c_int,
    /// Panic inside the objective on this (1-based) call.
    panic_on_fn_call: Option<c_int>,
    /// First panic payload caught in a callback, resumed after `lbfgsb`.
    panic: Option<Box<dyn Any + Send>>,
}

impl Quadratic {
    fn new(centre: &[f64], weight: &[f64]) -> Self {
        assert_eq!(centre.len(), weight.len());
        Self {
            centre: centre.to_vec(),
            weight: weight.to_vec(),
            fn_calls: 0,
            gr_calls: 0,
            panic_on_fn_call: None,
            panic: None,
        }
    }

    fn value(&mut self, par: &[f64]) -> f64 {
        self.fn_calls += 1;
        if self.panic_on_fn_call == Some(self.fn_calls) {
            panic!("objective panicked on call {}", self.fn_calls);
        }
        par.iter()
            .zip(&self.centre)
            .zip(&self.weight)
            .map(|((x, c), w)| w * (x - c) * (x - c))
            .sum()
    }

    fn gradient(&mut self, par: &[f64], gr: &mut [f64]) {
        self.gr_calls += 1;
        for (((g, x), c), w) in gr.iter_mut().zip(par).zip(&self.centre).zip(&self.weight) {
            *g = 2.0 * w * (x - c);
        }
    }
}

/// `optimfn`: the objective. A panic is caught and stored in the state, and
/// a finite value is returned so `lbfgsb` can finish (it raises an R error on
/// a non-finite value).
unsafe extern "C-unwind" fn quadratic_fn(n: c_int, par: *mut f64, ex: *mut c_void) -> f64 {
    // SAFETY: `ex` is the `&mut Quadratic` given to `lbfgsb`, alive for the
    // whole call, and R passes `n` parameter values.
    let state = unsafe { &mut *ex.cast::<Quadratic>() };
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

/// `optimgr`: the gradient, with the same panic handling (zeros on panic).
unsafe extern "C-unwind" fn quadratic_gr(n: c_int, par: *mut f64, gr: *mut f64, ex: *mut c_void) {
    // SAFETY: as in `quadratic_fn`; `gr` has `n` writable slots.
    let state = unsafe { &mut *ex.cast::<Quadratic>() };
    let n = usize::try_from(n).expect("lbfgsb passes n >= 0");
    let par = unsafe { std::slice::from_raw_parts(par, n) };
    let gr = unsafe { std::slice::from_raw_parts_mut(gr, n) };
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| state.gradient(par, gr))) {
        gr.fill(0.0);
        state.panic.get_or_insert(payload);
    }
}

/// What `lbfgsb` reports back.
struct Outcome {
    x: Vec<f64>,
    fmin: f64,
    fail: c_int,
    fncount: c_int,
    grcount: c_int,
    msg: String,
}

/// Bounds and limits for one run; `optim`'s defaults otherwise.
struct Problem<'a> {
    x0: &'a [f64],
    lower: &'a [f64],
    upper: &'a [f64],
    nbd: &'a [c_int],
    maxit: c_int,
}

fn run(problem: &Problem<'_>, state: &mut Quadratic) -> Outcome {
    let n_usize = problem.x0.len();
    assert_eq!(problem.lower.len(), n_usize);
    assert_eq!(problem.upper.len(), n_usize);
    assert_eq!(problem.nbd.len(), n_usize);
    let n = c_int::try_from(n_usize).expect("small problem");

    let mut x = problem.x0.to_vec();
    let mut lower = problem.lower.to_vec();
    let mut upper = problem.upper.to_vec();
    let mut nbd = problem.nbd.to_vec();
    let mut fmin = f64::NAN;
    let mut fail: c_int = -1;
    let mut fncount: c_int = -1;
    let mut grcount: c_int = -1;
    let mut msg: [c_char; 60] = [0; 60];

    // The workspace is R_alloc'd; outside `.Call` nothing reclaims it, so
    // restore the transient-allocation watermark afterwards.
    let watermark = unsafe { vmaxget() };
    unsafe {
        lbfgsb(
            n,
            5,
            x.as_mut_ptr(),
            lower.as_mut_ptr(),
            upper.as_mut_ptr(),
            nbd.as_mut_ptr(),
            &mut fmin,
            quadratic_fn,
            quadratic_gr,
            &mut fail,
            std::ptr::from_mut(state).cast::<c_void>(),
            1e7,
            0.0,
            &mut fncount,
            &mut grcount,
            problem.maxit,
            msg.as_mut_ptr(),
            0,
            10,
        );
        vmaxset(watermark);
    }

    // SAFETY: lbfgsb strcpy's a NUL-terminated status (< 60 bytes) into msg.
    let msg = unsafe { CStr::from_ptr(msg.as_ptr()) }
        .to_str()
        .expect("ASCII status")
        .to_owned();
    Outcome {
        x,
        fmin,
        fail,
        fncount,
        grcount,
        msg,
    }
}

/// Each `FG` request evaluates the objective and the gradient once, and R
/// reports that request count as both `fncount` and `grcount`.
fn assert_counts(out: &Outcome, state: &Quadratic) {
    assert_eq!(out.fncount, out.grcount, "R stores one count in both");
    assert_eq!(state.fn_calls, out.fncount, "objective calls");
    assert_eq!(state.gr_calls, out.grcount, "gradient calls");
}

fn assert_close(what: &str, got: &[f64], want: &[f64], tol: f64) {
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!((g - w).abs() <= tol, "{what}[{i}]: got {g}, want {w}");
    }
}

const INF: f64 = f64::INFINITY;

// endregion

// region: cases

fn interior_minimum() {
    let centre = [1.0, -2.0, 3.0];
    let mut state = Quadratic::new(&centre, &[1.0, 2.0, 0.5]);
    let out = run(
        &Problem {
            x0: &[0.0, 0.0, 0.0],
            lower: &[-INF, -INF, -INF],
            upper: &[INF, INF, INF],
            nbd: &[0, 0, 0],
            maxit: 100,
        },
        &mut state,
    );
    assert_eq!(out.fail, 0, "converged: {}", out.msg);
    assert!(out.msg.starts_with("CONVERGENCE"), "{}", out.msg);
    assert_close("x", &out.x, &centre, 1e-6);
    assert!(out.fmin.abs() <= 1e-10, "fmin {}", out.fmin);
    assert!(out.fncount > 0);
    assert_counts(&out, &state);
}

fn bound_constrained_minimum() {
    // Centre (1, -2, 3). Coordinate 0 is boxed in [2, 5] (code 2), 1 is
    // capped at -3 (code 3, upper only) and 2 is floored at 0 (code 1,
    // lower only, inactive). The minimum clamps to (2, -3, 3) with
    // f = 1 * 1^2 + 2 * 1^2 + 0 = 3.
    let mut state = Quadratic::new(&[1.0, -2.0, 3.0], &[1.0, 2.0, 0.5]);
    let out = run(
        &Problem {
            x0: &[4.0, -4.0, 1.0],
            lower: &[2.0, -INF, 0.0],
            upper: &[5.0, -3.0, INF],
            nbd: &[2, 3, 1],
            maxit: 100,
        },
        &mut state,
    );
    assert_eq!(out.fail, 0, "converged: {}", out.msg);
    assert!(out.msg.starts_with("CONVERGENCE"), "{}", out.msg);
    // Active bounds are hit exactly (the iterates are projected onto them).
    assert_eq!(out.x[0], 2.0);
    assert_eq!(out.x[1], -3.0);
    assert!((out.x[2] - 3.0).abs() <= 1e-6, "x[2] = {}", out.x[2]);
    assert!((out.fmin - 3.0).abs() <= 1e-10, "fmin {}", out.fmin);
    assert_counts(&out, &state);
}

fn fixed_parameter() {
    // Code 2 with l == u fixes coordinate 1 at 0.5 although the centre is -2;
    // the start value 0 is projected onto it. The others reach their centre.
    let mut state = Quadratic::new(&[1.0, -2.0, 3.0], &[1.0, 2.0, 0.5]);
    let out = run(
        &Problem {
            x0: &[0.0, 0.0, 0.0],
            lower: &[-INF, 0.5, -INF],
            upper: &[INF, 0.5, INF],
            nbd: &[0, 2, 0],
            maxit: 100,
        },
        &mut state,
    );
    assert_eq!(out.fail, 0, "converged: {}", out.msg);
    assert_eq!(out.x[1], 0.5, "fixed coordinate never moves");
    assert_close("free coordinates", &[out.x[0], out.x[2]], &[1.0, 3.0], 1e-6);
    // f = 2 * (0.5 - (-2))^2 = 12.5 at the constrained minimum.
    assert!((out.fmin - 12.5).abs() <= 1e-10, "fmin {}", out.fmin);
    assert_counts(&out, &state);
}

fn iteration_limit() {
    // Badly scaled, so one iteration cannot reach the minimum.
    let mut state = Quadratic::new(&[1.0, -2.0], &[1.0, 1000.0]);
    let out = run(
        &Problem {
            x0: &[10.0, 10.0],
            lower: &[-INF, -INF],
            upper: &[INF, INF],
            nbd: &[0, 0],
            maxit: 1,
        },
        &mut state,
    );
    assert_eq!(out.fail, 1, "maxit reached: {}", out.msg);
    assert_eq!(out.msg, "NEW_X");
    assert!(out.fmin > 0.0);
    assert_counts(&out, &state);
}

fn invalid_bound_code() {
    let mut state = Quadratic::new(&[1.0, -2.0], &[1.0, 1.0]);
    let out = run(
        &Problem {
            x0: &[0.0, 0.0],
            lower: &[0.0, 0.0],
            upper: &[1.0, 1.0],
            nbd: &[0, 4],
            maxit: 100,
        },
        &mut state,
    );
    assert_eq!(out.fail, 52);
    assert_eq!(out.msg, "ERROR: INVALID NBD");
    assert_eq!(state.fn_calls, 0, "input checks run before any evaluation");
    assert_eq!(out.fncount, out.grcount);
    assert_eq!(out.x, vec![0.0, 0.0], "x is left as given");
}

fn infeasible_bounds() {
    let mut state = Quadratic::new(&[1.0], &[1.0]);
    let out = run(
        &Problem {
            x0: &[0.0],
            lower: &[2.0],
            upper: &[1.0],
            nbd: &[2],
            maxit: 100,
        },
        &mut state,
    );
    assert_eq!(out.fail, 52);
    assert_eq!(out.msg, "ERROR: NO FEASIBLE SOLUTION");
    assert_eq!(state.fn_calls, 0);
}

fn zero_parameters() {
    let mut state = Quadratic::new(&[], &[]);
    let out = run(
        &Problem {
            x0: &[],
            lower: &[],
            upper: &[],
            nbd: &[],
            maxit: 100,
        },
        &mut state,
    );
    assert_eq!(out.fail, 0);
    assert_eq!(out.msg, "NOTHING TO DO");
    assert_eq!((out.fncount, out.grcount), (1, 0));
    assert_eq!((state.fn_calls, state.gr_calls), (1, 0));
    assert_eq!(out.fmin, 0.0, "empty sum");
}

fn callback_panic_is_caught_and_resumed() {
    let mut state = Quadratic::new(&[1.0, -2.0, 3.0], &[1.0, 2.0, 0.5]);
    state.panic_on_fn_call = Some(3);
    let out = run(
        &Problem {
            x0: &[0.0, 0.0, 0.0],
            lower: &[-INF, -INF, -INF],
            upper: &[INF, INF, INF],
            nbd: &[0, 0, 0],
            maxit: 100,
        },
        &mut state,
    );
    // lbfgsb returned normally; its result is meaningless after the panic.
    assert!([0, 1, 51, 52].contains(&out.fail), "fail {}", out.fail);
    assert!(state.fn_calls >= 3);
    let payload = state
        .panic
        .take()
        .expect("the panic was caught in the callback");
    // Resume it now that no R frame is on the stack, as a caller would.
    let resumed = catch_unwind(AssertUnwindSafe(|| resume_unwind(payload)))
        .expect_err("resume_unwind re-raises the payload");
    let message = resumed
        .downcast_ref::<String>()
        .map(String::as_str)
        .unwrap_or_default();
    assert_eq!(message, "objective panicked on call 3");
}

// endregion
