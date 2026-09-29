//! Compile-pass test for the three spellings of a parameter's R-side type
//! checks (#1566): the `Checked<T>` / `Unchecked<T>` markers, the
//! per-parameter `preconditions` / `no_preconditions`, and the method-level
//! bare and list forms, with agreeing combinations, borrowed inner types,
//! `coerce` and `no_na`. The generated conversion reads the inner type and
//! wraps the value back for the call. (The worker path needs the
//! `worker-thread` feature; rpkg covers it.)

#![allow(dead_code)]

use miniextendr_api::{Checked, Unchecked, miniextendr};

/// A `Checked` parameter keeps its guards under the function's `no_preconditions`.
#[miniextendr(no_preconditions)]
pub fn fit(n_iter: Checked<i32>, tol: f64) -> f64 {
    f64::from(*n_iter) * tol
}

/// `Unchecked` drops one parameter's guards; `no_na` stays.
#[miniextendr]
pub fn scale_by(#[miniextendr(no_na)] factor: f64, xs: Unchecked<Vec<f64>>) -> Vec<f64> {
    xs.into_inner().into_iter().map(|x| x * factor).collect()
}

/// The per-parameter keywords, and a keyword agreeing with its marker.
#[miniextendr]
pub fn keywords(
    #[miniextendr(no_preconditions)] a: i32,
    #[miniextendr(preconditions)] b: Checked<i32>,
    #[miniextendr(no_na, no_preconditions)] c: Unchecked<Vec<f64>>,
) -> i32 {
    a + *b + c.len() as i32
}

/// The widened guard of a coerced parameter, an optional one and borrows.
#[miniextendr]
pub fn shapes(
    #[miniextendr(coerce)] n: Checked<u16>,
    o: Checked<Option<i32>>,
    s: Checked<&str>,
    xs: Unchecked<&[f64]>,
) -> f64 {
    f64::from(*n) + f64::from(o.unwrap_or(0)) + s.len() as f64 + xs.iter().sum::<f64>()
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct Sampler {
    seed: i32,
}

/// Method-level spellings under an impl that drops the checks.
#[miniextendr(r6, no_preconditions)]
impl Sampler {
    pub fn new(seed: i32) -> Self {
        Sampler { seed }
    }

    #[miniextendr(preconditions(n))]
    pub fn draw(&self, n: i32, scale: f64) -> f64 {
        f64::from(n + self.seed) * scale
    }

    pub fn reseed(&mut self, seed: Checked<i32>) {
        self.seed = seed.into_inner();
    }

    #[miniextendr(preconditions, no_preconditions(k))]
    pub fn mixed(&self, k: i32, j: i32) -> i32 {
        k + j
    }

    #[miniextendr(preconditions(seed))]
    pub fn agree(&mut self, seed: Checked<i32>) {
        self.seed = *seed;
    }
}

fn main() {}
