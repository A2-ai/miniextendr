//! num-complex adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::num_complex_impl::{Complex, RComplexOps};

/// @noRd
#[miniextendr]
pub fn complex_roundtrip(c: Complex<f64>) -> Complex<f64> {
    c
}

/// @noRd
#[miniextendr]
pub fn complex_add(a: Complex<f64>, b: Complex<f64>) -> Complex<f64> {
    a + b
}

/// @noRd
#[miniextendr]
pub fn complex_norm(c: Complex<f64>) -> f64 {
    RComplexOps::norm(&c)
}

/// @noRd
#[miniextendr]
pub fn complex_conj(c: Complex<f64>) -> Complex<f64> {
    RComplexOps::conj(&c)
}

/// @noRd
#[miniextendr]
pub fn complex_re(c: Complex<f64>) -> f64 {
    RComplexOps::re(&c)
}

/// @noRd
#[miniextendr]
pub fn complex_im(c: Complex<f64>) -> f64 {
    RComplexOps::im(&c)
}

/// @noRd
#[miniextendr]
pub fn complex_roundtrip_vec(v: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    v
}

/// @noRd
#[miniextendr]
pub fn complex_is_finite(c: Complex<f64>) -> bool {
    RComplexOps::is_finite(&c)
}

/// @noRd
#[miniextendr]
pub fn complex_from_parts(re: f64, im: f64) -> Complex<f64> {
    Complex::new(re, im)
}
