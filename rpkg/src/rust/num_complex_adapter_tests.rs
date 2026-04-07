//! num-complex adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::num_complex_impl::{Complex, RComplexOps};

/// Test Complex<f64> roundtrip through R.
/// @param c Complex number from R.
#[miniextendr]
pub fn complex_roundtrip(c: Complex<f64>) -> Complex<f64> {
    c
}

/// Test adding two complex numbers.
/// @param a First complex number.
/// @param b Second complex number.
#[miniextendr]
pub fn complex_add(a: Complex<f64>, b: Complex<f64>) -> Complex<f64> {
    a + b
}

/// Test computing the norm (absolute value) of a complex number.
/// @param c Complex number from R.
#[miniextendr]
pub fn complex_norm(c: Complex<f64>) -> f64 {
    RComplexOps::norm(&c)
}

/// Test computing the complex conjugate.
/// @param c Complex number from R.
#[miniextendr]
pub fn complex_conj(c: Complex<f64>) -> Complex<f64> {
    RComplexOps::conj(&c)
}

/// Test extracting the real part of a complex number.
/// @param c Complex number from R.
#[miniextendr]
pub fn complex_re(c: Complex<f64>) -> f64 {
    RComplexOps::re(&c)
}

/// Test extracting the imaginary part of a complex number.
/// @param c Complex number from R.
#[miniextendr]
pub fn complex_im(c: Complex<f64>) -> f64 {
    RComplexOps::im(&c)
}

/// Test Vec<Complex<f64>> roundtrip through R complex vector.
/// @param v Complex vector from R.
#[miniextendr]
pub fn complex_roundtrip_vec(v: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    v
}

/// Test whether a complex number has finite real and imaginary parts.
/// @param c Complex number from R.
#[miniextendr]
pub fn complex_is_finite(c: Complex<f64>) -> bool {
    RComplexOps::is_finite(&c)
}

/// Test constructing a complex number from real and imaginary parts.
/// @param re Real part.
/// @param im Imaginary part.
#[miniextendr]
pub fn complex_from_parts(re: f64, im: f64) -> Complex<f64> {
    Complex::new(re, im)
}
