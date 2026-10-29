use super::*;

#[test]
fn test_to_from_rcomplex() {
    let c = Complex::new(3.0, 4.0);
    let r = to_rcomplex(c);
    assert_eq!(r.r, 3.0);
    assert_eq!(r.i, 4.0);

    let c2 = from_rcomplex(r);
    assert_eq!(c2.re, 3.0);
    assert_eq!(c2.im, 4.0);
}

#[test]
fn test_na_rcomplex() {
    let na = na_rcomplex();
    assert!(IsNA::is_na(&na));

    let normal = Rcomplex { r: 1.0, i: 2.0 };
    assert!(!IsNA::is_na(&normal));

    // NA in real part only
    let partial_na = Rcomplex { r: NA_REAL, i: 2.0 };
    assert!(IsNA::is_na(&partial_na));

    // NA in imaginary part only
    let partial_na2 = Rcomplex { r: 1.0, i: NA_REAL };
    assert!(IsNA::is_na(&partial_na2));
}

#[test]
fn test_is_na_complex() {
    let na = Complex::new(NA_REAL, NA_REAL);
    assert!(is_na_complex(&na));

    let normal = Complex::new(1.0, 2.0);
    assert!(!is_na_complex(&normal));

    // Regular NaN is NOT NA
    let nan = Complex::new(f64::NAN, 0.0);
    assert!(!is_na_complex(&nan));
    assert!(!IsNA::is_na(&Rcomplex {
        r: 0.0,
        i: f64::NAN
    }));

    // A computed NA (`NA_real_ * 1` quiets the NaN) is still NA, in either part.
    let computed = f64::from_bits(0x7FF8_0000_0000_07A2);
    assert!(is_na_complex(&Complex::new(computed, 1.0)));
    assert!(is_na_complex(&Complex::new(1.0, computed)));
    assert!(IsNA::is_na(&Rcomplex {
        r: computed,
        i: 2.0
    }));
}

#[test]
fn test_rcomplexops_basics() {
    let c = Complex::new(3.0, 4.0);

    assert_eq!(RComplexOps::re(&c), 3.0);
    assert_eq!(RComplexOps::im(&c), 4.0);
    assert_eq!(RComplexOps::norm(&c), 5.0);
    assert_eq!(RComplexOps::norm_sqr(&c), 25.0);
    assert!(RComplexOps::is_finite(&c));
    assert!(!RComplexOps::is_infinite(&c));
    assert!(!RComplexOps::is_nan(&c));
}

#[test]
fn test_rcomplexops_conj_inv() {
    let c = Complex::new(3.0, 4.0);

    let conj = RComplexOps::conj(&c);
    assert_eq!(conj.re, 3.0);
    assert_eq!(conj.im, -4.0);

    let inv = RComplexOps::inv(&c);
    // 1/(3+4i) = (3-4i)/25 = 0.12 - 0.16i
    assert!((inv.re - 0.12).abs() < 1e-10);
    assert!((inv.im + 0.16).abs() < 1e-10);
}

#[test]
fn test_rcomplexops_special_values() {
    let inf = Complex::new(f64::INFINITY, 0.0);
    assert!(RComplexOps::is_infinite(&inf));
    assert!(!RComplexOps::is_finite(&inf));

    let nan = Complex::new(f64::NAN, 0.0);
    assert!(RComplexOps::is_nan(&nan));
    assert!(!RComplexOps::is_finite(&nan));
}
