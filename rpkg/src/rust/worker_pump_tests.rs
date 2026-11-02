//! Fixtures for `WorkerPump`: a worker thread computes in pure Rust and sends
//! progress messages; the pump runs on R's main thread for each one and is
//! the only side that calls R (here, an R callback through
//! `RCall::eval_with_handlers`, so an R error in it unwinds through the pump
//! and the worker is joined before the error reaches R).
//!
//! See `tests/testthat/test-worker-pump.R` and docs/THREADS.md.

use miniextendr_api::expression::RCall;
use miniextendr_api::pump::WorkerPump;
use miniextendr_api::{NamedListBuilder, SEXP, is_r_main_thread, miniextendr};

/// The sum of the squares of `1..=n`, computed on a worker thread that sends
/// each partial sum to the pump. The pump calls `on_progress(i, partial)` on
/// R's main thread (unless it is `NULL`). Returns
/// `list(total, ticks, worker_on_main, pump_on_main)`: the sum, how many
/// messages the pump got, and on which thread each side ran.
/// @param n Number of terms (non-negative).
/// @param capacity Channel capacity; `0` makes every send wait for the pump.
/// @param on_progress `NULL`, or a function of the term and the partial sum.
#[miniextendr(noexport)]
pub fn worker_pump_sum_squares(
    n: i32,
    capacity: i32,
    on_progress: SEXP,
) -> Result<NamedListBuilder, String> {
    if n < 0 {
        return Err(format!("`n` must be non-negative, not {n}"));
    }
    let capacity = usize::try_from(capacity)
        .map_err(|_| format!("`capacity` must be non-negative, not {capacity}"))?;
    let mut ticks = 0_i32;
    let mut pump_on_main = true;
    let (total, worker_on_main) = WorkerPump::<(i32, f64)>::new()
        .channel_capacity(capacity)
        .run(
            |tx| {
                let mut total = 0.0;
                for i in 1..=n {
                    total += f64::from(i) * f64::from(i);
                    // The pump is gone (an R error unwound it): stop.
                    if tx.send((i, total)).is_err() {
                        break;
                    }
                }
                Ok((total, is_r_main_thread()))
            },
            |(i, partial)| {
                ticks += 1;
                pump_on_main &= is_r_main_thread();
                if !on_progress.is_nil() {
                    // SAFETY: the pump runs on R's main thread, inside this
                    // call's boundary; `on_progress` is the rooted `.Call()`
                    // argument. An R error unwinds as a Rust panic, which
                    // drops the receiver and joins the worker.
                    let _ = unsafe {
                        RCall::from_sexp(on_progress)
                            .arg_value(i)
                            .arg_value(partial)
                            .eval_with_handlers(miniextendr_api::sys::R_GlobalEnv)
                    };
                }
            },
        )
        .map_err(|e| e.to_string())?;
    let mut out = NamedListBuilder::with_capacity(4);
    out.push("total", total);
    out.push("ticks", ticks);
    out.push("worker_on_main", worker_on_main);
    out.push("pump_on_main", pump_on_main);
    Ok(out)
}

/// A worker that sends `sent` messages and then fails: it returns an error,
/// or panics when `panic` is `TRUE`. The failure reaches R as an error
/// naming the number of messages the pump got.
/// @param sent Number of messages the worker sends before failing.
/// @param panic Whether the worker panics instead of returning an error.
#[miniextendr(noexport)]
pub fn worker_pump_failing(sent: i32, panic: bool) -> Result<i32, String> {
    let mut ticks = 0_i32;
    let result = WorkerPump::<i32>::new().run(
        |tx| -> Result<(), miniextendr_api::pump::WorkerError> {
            for i in 1..=sent {
                tx.send(i).ok();
            }
            if panic {
                panic!("the worker gave up after {sent} messages");
            }
            Err(format!("the worker gave up after {sent} messages").into())
        },
        |_| ticks += 1,
    );
    match result {
        Ok(()) => Ok(ticks),
        Err(e) => Err(format!("{e} (the pump got {ticks})")),
    }
}
