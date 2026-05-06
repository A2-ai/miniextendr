//! Test: MXL120 — `&self` / `&mut self` receivers on vctrs impls are rejected.
//!
//! A vctrs object is an S3-classed base vector (REALSXP, INTSXP, etc.). There is no
//! Rust `Self` stored inside the R SEXP — the vector payload IS the R object. The C
//! wrapper would call `ErasedExternalPtr::from_sexp` on a base vector, which panics at
//! runtime. MXL120 catches this at macro-expansion time instead.

use miniextendr_macros::miniextendr;

struct Currency;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "$"))]
impl Currency {
    pub fn new(amounts: Vec<f64>) -> Vec<f64> {
        amounts
    }

    pub fn symbol(&self) -> String {
        String::new()
    }
}

fn main() {}
