//! Test: MXL120 — vctrs constructor returning Self is rejected.
//!
//! The generated R wrapper passes the constructor result to `vctrs::new_vctr()`,
//! which requires a plain vector payload, not an ExternalPtr (Self).

use miniextendr_macros::miniextendr;

struct Currency;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "$"))]
impl Currency {
    pub fn new(amounts: Vec<f64>) -> Self {
        Currency
    }
}

fn main() {}
