//! Compile-pass test for the `&RngGuard` parameter (#2002): under
//! `#[miniextendr(rng)]` the wrapper passes the guard it holds, at any
//! position, on a standalone function and on instance and static methods in
//! every class system. The parameter is no R formal.

#![allow(dead_code, deprecated)]

use miniextendr_api::rng::RngGuard;
use miniextendr_api::{ExternalPtr, miniextendr};

#[miniextendr(rng)]
pub fn guard_first(rng: &RngGuard, n: i32) -> Vec<f64> {
    (0..n).map(|_| rng.unif_rand()).collect()
}

#[miniextendr(rng)]
pub fn guard_between(n: i32, rng: &miniextendr_api::RngGuard, scale: f64) -> f64 {
    f64::from(n) * scale * rng.exp_rand()
}

#[miniextendr(rng)]
pub fn guard_last<'a>(n: i32, _rng: &'a RngGuard) -> i32 {
    n
}

#[miniextendr(rng)]
pub fn guard_only(rng: &RngGuard) -> f64 {
    rng.norm_rand()
}

#[derive(ExternalPtr)]
pub struct EnvSampler(f64);

#[miniextendr(env)]
impl EnvSampler {
    pub fn new(scale: f64) -> Self {
        EnvSampler(scale)
    }
    #[miniextendr(rng)]
    pub fn draw(&self, n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| self.0 * rng.unif_rand()).collect()
    }
    #[miniextendr(rng)]
    pub fn reset(&mut self, rng: &RngGuard) {
        self.0 = rng.unif_rand();
    }
    #[miniextendr(rng)]
    pub fn sampled(rng: &RngGuard, n: i32) -> Vec<f64> {
        (0..n).map(|_| rng.norm_rand()).collect()
    }
}

#[derive(ExternalPtr)]
pub struct R6Sampler(f64);

#[miniextendr(r6)]
impl R6Sampler {
    pub fn new(scale: f64) -> Self {
        R6Sampler(scale)
    }
    #[miniextendr(rng)]
    pub fn draw(&self, rng: &RngGuard) -> f64 {
        self.0 * rng.unif_rand()
    }
    #[miniextendr(rng, r6(active))]
    pub fn sample(&self, rng: &RngGuard) -> f64 {
        self.0 * rng.norm_rand()
    }
}

#[derive(ExternalPtr)]
pub struct S3Sampler(f64);

#[miniextendr(s3)]
impl S3Sampler {
    pub fn new(scale: f64) -> Self {
        S3Sampler(scale)
    }
    #[miniextendr(rng)]
    pub fn draw(&self, rng: &RngGuard, n: i32) -> Vec<f64> {
        (0..n).map(|_| self.0 * rng.unif_rand()).collect()
    }
}

#[derive(ExternalPtr)]
pub struct S4Sampler(f64);

#[miniextendr(s4)]
impl S4Sampler {
    pub fn new(scale: f64) -> Self {
        S4Sampler(scale)
    }
    #[miniextendr(rng)]
    pub fn draw(&self, rng: &RngGuard) -> f64 {
        self.0 * rng.unif_rand()
    }
}

#[derive(ExternalPtr)]
pub struct S7Sampler(f64);

#[miniextendr(s7)]
impl S7Sampler {
    pub fn new(scale: f64) -> Self {
        S7Sampler(scale)
    }
    #[miniextendr(rng)]
    pub fn draw(&self, rng: &RngGuard) -> f64 {
        self.0 * rng.unif_rand()
    }
    #[miniextendr(rng)]
    pub fn drawn(rng: &RngGuard) -> f64 {
        rng.exp_rand()
    }
}

#[derive(ExternalPtr)]
pub struct Noise;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "ns"))]
impl Noise {
    #[miniextendr(rng)]
    pub fn new(n: i32, rng: &RngGuard) -> Vec<f64> {
        (0..n).map(|_| rng.norm_rand()).collect()
    }
}

fn main() {}
