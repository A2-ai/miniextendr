//! Fixtures for the `Rmath.h` declarations in `miniextendr_api::sys`.
//!
//! Each export maps one scalar routine over a numeric vector through
//! `.Call`, so the package build links the libR symbols (`Rf_dnorm4`,
//! `Rf_pnorm5`, `Rf_qnorm5`, `Rf_qchisq`) the way a downstream package
//! would, and the testthat suite can compare them with `stats::`.
//! Arguments go to R unchanged: no `1 - p`, no exponentiating log values.

use miniextendr_api::miniextendr;
use miniextendr_api::sys::{Rf_dnorm4, Rf_pnorm5, Rf_qchisq, Rf_qnorm5};
use std::os::raw::c_int;

/// Normal density through `Rf_dnorm4` (the C routine behind `stats::dnorm`).
/// @param x Numeric vector of quantiles.
/// @param mean Numeric scalar mean.
/// @param sd Numeric scalar standard deviation.
/// @param give_log Logical scalar: return the log density.
#[miniextendr]
pub fn rmath_dnorm(x: Vec<f64>, mean: f64, sd: f64, give_log: bool) -> Vec<f64> {
    let give_log = c_int::from(give_log);
    x.into_iter()
        .map(|x| unsafe { Rf_dnorm4(x, mean, sd, give_log) })
        .collect()
}

/// Normal distribution function through `Rf_pnorm5` (behind `stats::pnorm`).
/// @param q Numeric vector of quantiles.
/// @param mean Numeric scalar mean.
/// @param sd Numeric scalar standard deviation.
/// @param lower_tail Logical scalar: lower tail (`TRUE`) or upper tail (`FALSE`) probabilities.
/// @param log_p Logical scalar: probabilities are on the log scale.
#[miniextendr]
pub fn rmath_pnorm(q: Vec<f64>, mean: f64, sd: f64, lower_tail: bool, log_p: bool) -> Vec<f64> {
    let (lower_tail, log_p) = (c_int::from(lower_tail), c_int::from(log_p));
    q.into_iter()
        .map(|q| unsafe { Rf_pnorm5(q, mean, sd, lower_tail, log_p) })
        .collect()
}

/// Normal quantile function through `Rf_qnorm5` (behind `stats::qnorm`).
/// @param p Numeric vector of probabilities (log probabilities if `log_p`).
/// @param mean Numeric scalar mean.
/// @param sd Numeric scalar standard deviation.
/// @param lower_tail Logical scalar: lower tail (`TRUE`) or upper tail (`FALSE`) probabilities.
/// @param log_p Logical scalar: probabilities are on the log scale.
#[miniextendr]
pub fn rmath_qnorm(p: Vec<f64>, mean: f64, sd: f64, lower_tail: bool, log_p: bool) -> Vec<f64> {
    let (lower_tail, log_p) = (c_int::from(lower_tail), c_int::from(log_p));
    p.into_iter()
        .map(|p| unsafe { Rf_qnorm5(p, mean, sd, lower_tail, log_p) })
        .collect()
}

/// Central chi-squared quantile function through `Rf_qchisq` (behind
/// `stats::qchisq` without `ncp`).
/// @param p Numeric vector of probabilities (log probabilities if `log_p`).
/// @param df Numeric scalar degrees of freedom.
/// @param lower_tail Logical scalar: lower tail (`TRUE`) or upper tail (`FALSE`) probabilities.
/// @param log_p Logical scalar: probabilities are on the log scale.
#[miniextendr]
pub fn rmath_qchisq(p: Vec<f64>, df: f64, lower_tail: bool, log_p: bool) -> Vec<f64> {
    let (lower_tail, log_p) = (c_int::from(lower_tail), c_int::from(log_p));
    p.into_iter()
        .map(|p| unsafe { Rf_qchisq(p, df, lower_tail, log_p) })
        .collect()
}
