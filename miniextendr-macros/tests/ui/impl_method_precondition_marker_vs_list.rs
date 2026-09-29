//! Test: an impl method's `no_preconditions(seed)` that contradicts the
//! parameter's `Checked<T>` marker is rejected.

use miniextendr_api::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
pub struct Sampler;

#[miniextendr(env)]
impl Sampler {
    pub fn new() -> Self {
        Sampler
    }

    #[miniextendr(no_preconditions(seed))]
    pub fn reseed(&mut self, seed: miniextendr_api::Checked<i32>) {
        let _ = seed;
    }
}

fn main() {}
