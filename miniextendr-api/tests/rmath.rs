//! Integration tests for the `Rmath.h` declarations in `miniextendr_api::sys`
//! (`Rf_dnorm4`, `Rf_pnorm5`, `Rf_qnorm5`, `Rf_qchisq`).
//!
//! Each routine is compared with R's own `stats::` function on the same
//! doubles. The inputs are built in R and read back, so both sides see
//! identical bits (no decimal round trip through the parser). `stats::`
//! calls the same libR symbol, so finite and infinite results must match
//! exactly. For NA / NaN inputs only the class is compared: `stats::`
//! normalises the result to `NA_real_` or `R_NaN`, while the C routine
//! returns whatever NaN the IEEE arithmetic propagates.
//!
//! The independent checks (`pnorm(0) = 1/2`, reflection through the
//! upper-tail flag, the df = 2 chi-squared upper quantile on the log scale,
//! quantile/probability round trips) don't use R's results at all.

mod r_test_utils;

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::r_str;
use miniextendr_api::sys::{Rf_dnorm4, Rf_pnorm5, Rf_qchisq, Rf_qnorm5};
use std::os::raw::c_int;

#[test]
fn rmath_suite() {
    r_test_utils::with_r_thread(|| {
        define_inputs();
        dnorm_matches_r();
        pnorm_matches_r();
        qnorm_matches_r();
        qchisq_matches_r();
        invalid_domains_return_nan();
        endpoint_probabilities();
        pnorm_at_zero_is_half();
        pnorm_reflection_via_upper_tail();
        qchisq_df2_upper_log_quantile();
        qnorm_inverts_pnorm_in_the_tails();
    });
}

// region: helpers

/// Evaluate R code that returns a double vector and copy it out.
fn r_reals(code: &str) -> Vec<f64> {
    let sexp = r_str!(code).unwrap_or_else(|e| panic!("R failed on `{code}`: {e}"));
    Vec::<f64>::try_from_sexp(sexp).unwrap_or_else(|e| panic!("`{code}` is not double: {e:?}"))
}

/// Evaluate R code that returns a logical vector (no NA) and copy it out.
fn r_flags(code: &str) -> Vec<bool> {
    let sexp = r_str!(code).unwrap_or_else(|e| panic!("R failed on `{code}`: {e}"));
    Vec::<bool>::try_from_sexp(sexp).unwrap_or_else(|e| panic!("`{code}` is not logical: {e:?}"))
}

/// R's logical spelling of a C flag.
fn r_bool(flag: c_int) -> &'static str {
    if flag != 0 { "TRUE" } else { "FALSE" }
}

/// The R values `-Inf`, `Inf`, `NA` and `NaN` render through `format!` as
/// `-inf`, `inf` and `NaN`; spell them for R's parser instead. Parameters
/// are only ever these specials or short exact decimals.
fn r_num(x: f64) -> String {
    if x == f64::INFINITY {
        "Inf".to_string()
    } else if x == f64::NEG_INFINITY {
        "-Inf".to_string()
    } else {
        format!("{x:?}")
    }
}

/// Compare one C result with R's result for the same input.
///
/// `na` / `nan` say which class R reports for that element (`is.na` and
/// `is.nan`). An NA or NaN in R must be a NaN in C; anything else must be
/// bit-for-bit the same value (`==`, so `0.0 == -0.0` is accepted).
fn assert_same(what: &str, input: f64, c_value: f64, r_value: f64, r_na: bool) {
    if r_na {
        assert!(
            c_value.is_nan(),
            "{what}: input {input:?} gives NA/NaN in R but {c_value:?} in C"
        );
    } else {
        assert!(
            c_value == r_value,
            "{what}: input {input:?} gives {r_value:?} in R but {c_value:?} in C"
        );
    }
}

/// Relative closeness with an absolute floor for values near zero.
fn assert_close(what: &str, got: f64, want: f64, rel: f64) {
    let tol = rel * want.abs().max(f64::MIN_POSITIVE);
    assert!(
        (got - want).abs() <= tol,
        "{what}: got {got:?}, want {want:?} (rel tol {rel:e})"
    );
}

/// Distribution parameters `(mean, sd)` the comparisons run over: standard,
/// shifted and scaled, narrow, degenerate (`sd = 0`) and infinitely wide.
const NORMAL_PARAMS: &[(f64, f64)] = &[
    (0.0, 1.0),
    (1.5, 2.5),
    (-3.0, 0.25),
    (1.0, 0.0),
    (0.0, f64::INFINITY),
];

/// Store the input vectors in R's global environment.
///
/// `.mx_x`: quantiles from far tail to far tail, plus the specials.
/// `.mx_p`: probabilities including both endpoints, tiny and near-one values,
/// and invalid ones. `.mx_lp`: log probabilities, `-Inf` and `0` endpoints,
/// extreme lower tails and an invalid positive value.
fn define_inputs() {
    r_str!(
        ".mx_x <- c(-Inf, -1e10, -40, -38.5, -10, -2.5, -1, -0.3, 0, 0.3, 1, 1.5, \
         2.5, 8.25, 10, 38.5, 40, 1e10, Inf, NA, NaN)"
    )
    .expect("define .mx_x");
    r_str!(
        ".mx_p <- c(0, 1e-300, 1e-100, 1e-10, 0.001, 0.025, 0.3, 0.5, 0.7, 0.975, \
         0.999, 1 - 1e-10, 1 - 2^-53, 1, -0.1, 1.1, NA, NaN)"
    )
    .expect("define .mx_p");
    r_str!(
        ".mx_lp <- c(-Inf, -1e5, -800, -50, -5, log(0.5), -0.01, -1e-10, -1e-300, 0, \
         0.5, NA, NaN)"
    )
    .expect("define .mx_lp");
}

// endregion

// region: comparisons with R's stats functions

fn dnorm_matches_r() {
    let xs = r_reals(".mx_x");
    for &(mu, sigma) in NORMAL_PARAMS {
        for give_log in [0, 1] {
            let call = format!(
                "dnorm(.mx_x, {}, {}, log = {})",
                r_num(mu),
                r_num(sigma),
                r_bool(give_log)
            );
            let want = r_reals(&format!("suppressWarnings({call})"));
            let na = r_flags(&format!("is.na(suppressWarnings({call}))"));
            for (i, &x) in xs.iter().enumerate() {
                let got = unsafe { Rf_dnorm4(x, mu, sigma, give_log) };
                assert_same(&call, x, got, want[i], na[i]);
            }
        }
    }
}

fn pnorm_matches_r() {
    let xs = r_reals(".mx_x");
    for &(mu, sigma) in NORMAL_PARAMS {
        for lower_tail in [1, 0] {
            for log_p in [0, 1] {
                let call = format!(
                    "pnorm(.mx_x, {}, {}, lower.tail = {}, log.p = {})",
                    r_num(mu),
                    r_num(sigma),
                    r_bool(lower_tail),
                    r_bool(log_p)
                );
                let want = r_reals(&format!("suppressWarnings({call})"));
                let na = r_flags(&format!("is.na(suppressWarnings({call}))"));
                for (i, &x) in xs.iter().enumerate() {
                    let got = unsafe { Rf_pnorm5(x, mu, sigma, lower_tail, log_p) };
                    assert_same(&call, x, got, want[i], na[i]);
                }
            }
        }
    }
}

fn qnorm_matches_r() {
    for &(mu, sigma) in NORMAL_PARAMS {
        for lower_tail in [1, 0] {
            for log_p in [0, 1] {
                let input = if log_p != 0 { ".mx_lp" } else { ".mx_p" };
                let ps = r_reals(input);
                let call = format!(
                    "qnorm({input}, {}, {}, lower.tail = {}, log.p = {})",
                    r_num(mu),
                    r_num(sigma),
                    r_bool(lower_tail),
                    r_bool(log_p)
                );
                let want = r_reals(&format!("suppressWarnings({call})"));
                let na = r_flags(&format!("is.na(suppressWarnings({call}))"));
                for (i, &p) in ps.iter().enumerate() {
                    let got = unsafe { Rf_qnorm5(p, mu, sigma, lower_tail, log_p) };
                    assert_same(&call, p, got, want[i], na[i]);
                }
            }
        }
    }
}

fn qchisq_matches_r() {
    // df = 0 puts all mass at 0; the others span heavy-tailed to near-normal.
    for df in [0.0, 0.5, 1.0, 2.0, 7.5, 100.0] {
        for lower_tail in [1, 0] {
            for log_p in [0, 1] {
                let input = if log_p != 0 { ".mx_lp" } else { ".mx_p" };
                let ps = r_reals(input);
                let call = format!(
                    "qchisq({input}, {}, lower.tail = {}, log.p = {})",
                    r_num(df),
                    r_bool(lower_tail),
                    r_bool(log_p)
                );
                let want = r_reals(&format!("suppressWarnings({call})"));
                let na = r_flags(&format!("is.na(suppressWarnings({call}))"));
                for (i, &p) in ps.iter().enumerate() {
                    let got = unsafe { Rf_qchisq(p, df, lower_tail, log_p) };
                    assert_same(&call, p, got, want[i], na[i]);
                }
            }
        }
    }
}

// endregion

// region: domains and endpoints

/// A negative sd (or df, or a probability outside its range) returns NaN.
/// `stats::` reports "NaNs produced" for these (checked here so the
/// comparison is explicit); the C routines themselves return NaN silently,
/// which the rpkg testthat suite checks through `.Call`.
fn invalid_domains_return_nan() {
    unsafe {
        assert!(Rf_dnorm4(1.0, 0.0, -1.0, 0).is_nan());
        assert!(Rf_pnorm5(1.0, 0.0, -1.0, 1, 0).is_nan());
        assert!(Rf_qnorm5(0.5, 0.0, -1.0, 1, 0).is_nan());
        assert!(Rf_qnorm5(1.5, 0.0, 1.0, 1, 0).is_nan());
        assert!(Rf_qnorm5(0.5, 0.0, 1.0, 1, 1).is_nan());
        assert!(Rf_qchisq(0.5, -1.0, 1, 0).is_nan());
        assert!(Rf_qchisq(1.5, 2.0, 1, 0).is_nan());
    }
    let warned = r_flags(
        "c(tryCatch({ dnorm(1, 0, -1); FALSE }, warning = function(w) TRUE), \
         tryCatch({ qchisq(1.5, 2); FALSE }, warning = function(w) TRUE))",
    );
    assert_eq!(warned, vec![true, true], "stats:: warns on these domains");
}

fn endpoint_probabilities() {
    let inf = f64::INFINITY;
    unsafe {
        // Probabilities 0 and 1 map to the support's ends, per tail.
        assert_eq!(Rf_qnorm5(0.0, 0.0, 1.0, 1, 0), -inf);
        assert_eq!(Rf_qnorm5(1.0, 0.0, 1.0, 1, 0), inf);
        assert_eq!(Rf_qnorm5(0.0, 0.0, 1.0, 0, 0), inf);
        assert_eq!(Rf_qnorm5(1.0, 0.0, 1.0, 0, 0), -inf);
        // On the log scale the endpoints are -Inf and 0.
        assert_eq!(Rf_qnorm5(-inf, 0.0, 1.0, 1, 1), -inf);
        assert_eq!(Rf_qnorm5(0.0, 0.0, 1.0, 1, 1), inf);
        assert_eq!(Rf_qchisq(0.0, 3.0, 1, 0), 0.0);
        assert_eq!(Rf_qchisq(1.0, 3.0, 1, 0), inf);
        assert_eq!(Rf_qchisq(0.0, 3.0, 0, 0), inf);
        assert_eq!(Rf_qchisq(-inf, 3.0, 0, 1), inf);
        // Infinite quantiles give the limiting probabilities and density.
        assert_eq!(Rf_pnorm5(-inf, 0.0, 1.0, 1, 0), 0.0);
        assert_eq!(Rf_pnorm5(inf, 0.0, 1.0, 1, 0), 1.0);
        assert_eq!(Rf_pnorm5(inf, 0.0, 1.0, 0, 0), 0.0);
        assert_eq!(Rf_pnorm5(-inf, 0.0, 1.0, 1, 1), -inf);
        assert_eq!(Rf_pnorm5(inf, 0.0, 1.0, 1, 1), 0.0);
        assert_eq!(Rf_dnorm4(inf, 0.0, 1.0, 0), 0.0);
        assert_eq!(Rf_dnorm4(-inf, 0.0, 1.0, 1), -inf);
        // Extreme tails: the plain scale underflows, the log scale doesn't.
        assert_eq!(Rf_pnorm5(-40.0, 0.0, 1.0, 1, 0), 0.0);
        let log_tail = Rf_pnorm5(-40.0, 0.0, 1.0, 1, 1);
        assert!(log_tail.is_finite() && log_tail < -800.0, "{log_tail}");
        let deep_quantile = Rf_qnorm5(-1e5, 0.0, 1.0, 1, 1);
        assert!(
            deep_quantile.is_finite() && deep_quantile < -400.0,
            "{deep_quantile}"
        );
    }
}

// endregion

// region: identities independent of R's results

fn pnorm_at_zero_is_half() {
    unsafe {
        assert_eq!(Rf_pnorm5(0.0, 0.0, 1.0, 1, 0), 0.5);
        assert_eq!(Rf_pnorm5(0.0, 0.0, 1.0, 0, 0), 0.5);
        // Same point after shifting and scaling.
        assert_eq!(Rf_pnorm5(1.5, 1.5, 2.5, 1, 0), 0.5);
        assert_close(
            "log pnorm(0)",
            Rf_pnorm5(0.0, 0.0, 1.0, 1, 1),
            0.5f64.ln(),
            4.0 * f64::EPSILON,
        );
    }
}

/// `pnorm(-x) = 1 - pnorm(x)`, expressed through the upper-tail flag (never
/// by subtracting from 1): `P[X <= -x]` must equal `P[X > x]`.
fn pnorm_reflection_via_upper_tail() {
    let xs = [
        0.0, 0.1, 0.5, 0.67, 0.7, 1.0, 2.5, 5.0, 10.0, 20.0, 37.0, 38.5, 40.0,
    ];
    for x in xs {
        for log_p in [0, 1] {
            let lower = unsafe { Rf_pnorm5(-x, 0.0, 1.0, 1, log_p) };
            let upper = unsafe { Rf_pnorm5(x, 0.0, 1.0, 0, log_p) };
            assert_eq!(lower, upper, "reflection at x = {x}, log_p = {log_p}");
        }
    }
}

/// For df = 2 the survival function is `exp(-q / 2)`, so the upper-tail
/// quantile of the log probability `lp` is `-2 * lp`.
fn qchisq_df2_upper_log_quantile() {
    for lp in [-1e-8, -0.01, -0.5, -1.0, -5.0, -50.0, -700.0, -1e4] {
        let q = unsafe { Rf_qchisq(lp, 2.0, 0, 1) };
        assert_close(&format!("qchisq({lp}, 2) upper/log"), q, -2.0 * lp, 1e-14);
    }
}

/// The quantile function inverts the distribution function on the log
/// scale, on the accurate side of each tail.
fn qnorm_inverts_pnorm_in_the_tails() {
    for x in [-38.0, -20.0, -5.0, -0.5, 0.0] {
        let lp = unsafe { Rf_pnorm5(x, 0.0, 1.0, 1, 1) };
        let back = unsafe { Rf_qnorm5(lp, 0.0, 1.0, 1, 1) };
        assert!(
            (back - x).abs() <= 1e-12 * x.abs().max(1.0),
            "{x} -> {back}"
        );
    }
    for x in [0.5, 5.0, 20.0, 38.0] {
        let lp = unsafe { Rf_pnorm5(x, 0.0, 1.0, 0, 1) };
        let back = unsafe { Rf_qnorm5(lp, 0.0, 1.0, 0, 1) };
        assert!(
            (back - x).abs() <= 1e-12 * x.abs().max(1.0),
            "{x} -> {back}"
        );
    }
}

// endregion
