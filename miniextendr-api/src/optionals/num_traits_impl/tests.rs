use super::*;

#[test]
fn test_rnum_i32() {
    assert_eq!(<i32 as RNum>::r_zero(), 0);
    assert_eq!(<i32 as RNum>::r_one(), 1);
    assert!(RNum::r_is_zero(&0i32));
    assert!(!RNum::r_is_zero(&1i32));
    assert!(RNum::r_is_one(&1i32));
    assert!(!RNum::r_is_one(&2i32));
}

#[test]
fn test_rnum_f64() {
    assert_eq!(<f64 as RNum>::r_zero(), 0.0);
    assert_eq!(<f64 as RNum>::r_one(), 1.0);
    assert!(RNum::r_is_zero(&0.0f64));
    assert!(!RNum::r_is_zero(&0.1f64));
}

#[test]
fn test_rsigned_i32() {
    assert_eq!((-5i32).r_abs(), 5);
    assert_eq!(5i32.r_abs(), 5);
    assert_eq!((-5i32).r_signum(), -1);
    assert_eq!(0i32.r_signum(), 0);
    assert_eq!(5i32.r_signum(), 1);
    assert!(5i32.r_is_positive());
    assert!(!(-5i32).r_is_positive());
    assert!((-5i32).r_is_negative());
    assert!(!5i32.r_is_negative());
}

#[test]
fn test_rsigned_f64() {
    // Use explicit trait qualification since f64 implements both RSigned and RFloat
    assert_eq!(RSigned::r_abs(&-std::f64::consts::PI), std::f64::consts::PI);
    assert_eq!(RSigned::r_signum(&-std::f64::consts::PI), -1.0);
    assert_eq!(RSigned::r_signum(&std::f64::consts::PI), 1.0);
}

#[test]
fn test_rfloat_classification() {
    assert!(f64::NAN.r_is_nan());
    assert!(!1.0f64.r_is_nan());
    assert!(f64::INFINITY.r_is_infinite());
    assert!(f64::NEG_INFINITY.r_is_infinite());
    assert!(!1.0f64.r_is_infinite());
    assert!(1.0f64.r_is_finite());
    assert!(!f64::INFINITY.r_is_finite());
    assert!(!f64::NAN.r_is_finite());
    assert!(1.0f64.r_is_normal());
    assert!(!0.0f64.r_is_normal());
}

#[test]
fn test_rfloat_sign() {
    assert!(1.0f64.r_is_sign_positive());
    assert!(!(-1.0f64).r_is_sign_positive());
    assert!((-1.0f64).r_is_sign_negative());
    assert!(!1.0f64.r_is_sign_negative());
}

#[test]
fn test_rfloat_rounding() {
    assert_eq!(3.7f64.r_floor(), 3.0);
    assert_eq!((-3.7f64).r_floor(), -4.0);
    assert_eq!(3.2f64.r_ceil(), 4.0);
    assert_eq!((-3.2f64).r_ceil(), -3.0);
    assert_eq!(3.5f64.r_round(), 4.0);
    assert_eq!(3.4f64.r_round(), 3.0);
    assert_eq!(3.7f64.r_trunc(), 3.0);
    assert_eq!((-3.7f64).r_trunc(), -3.0);
    let fract = 3.7f64.r_fract();
    assert!((fract - 0.7).abs() < 1e-10);
}

#[test]
fn test_rfloat_math() {
    // Use explicit trait qualification for abs/signum since f64 implements both RSigned and RFloat
    assert_eq!(RFloat::r_abs(&-3.0f64), 3.0);
    assert_eq!(RFloat::r_signum(&-1.0f64), -1.0);
    assert_eq!(RFloat::r_signum(&1.0f64), 1.0);
    assert_eq!(4.0f64.r_sqrt(), 2.0);
    assert_eq!(8.0f64.r_cbrt(), 2.0);
}

#[test]
fn test_rfloat_exp_log() {
    assert!((1.0f64.r_exp() - std::f64::consts::E).abs() < 1e-10);
    assert_eq!(3.0f64.r_exp2(), 8.0);
    assert!((std::f64::consts::E.r_ln() - 1.0).abs() < 1e-10);
    assert_eq!(8.0f64.r_log2(), 3.0);
    assert_eq!(100.0f64.r_log10(), 2.0);
}

#[test]
fn test_rfloat_trig() {
    let pi = std::f64::consts::PI;
    assert!(0.0f64.r_sin().abs() < 1e-10);
    assert!((1.0f64 - 0.0f64.r_cos()).abs() < 1e-10);
    assert!(0.0f64.r_tan().abs() < 1e-10);
    assert!((0.0f64.r_asin()).abs() < 1e-10);
    assert!((1.0f64.r_acos()).abs() < 1e-10);
    assert!((pi / 4.0 - 1.0f64.r_atan()).abs() < 1e-10);
}

#[test]
fn test_rfloat_hyperbolic() {
    assert!(0.0f64.r_sinh().abs() < 1e-10);
    assert!((1.0 - 0.0f64.r_cosh()).abs() < 1e-10);
    assert!(0.0f64.r_tanh().abs() < 1e-10);
    assert!(0.0f64.r_asinh().abs() < 1e-10);
    assert!(1.0f64.r_acosh().abs() < 1e-10);
    assert!(0.0f64.r_atanh().abs() < 1e-10);
}

#[test]
fn test_rfloat_special_values() {
    assert!(<f64 as RFloat>::r_infinity().r_is_infinite());
    assert!(<f64 as RFloat>::r_infinity() > 0.0);
    assert!(<f64 as RFloat>::r_neg_infinity().r_is_infinite());
    assert!(<f64 as RFloat>::r_neg_infinity() < 0.0);
    assert!(<f64 as RFloat>::r_nan().r_is_nan());
    assert!(<f64 as RFloat>::r_min_value() < 0.0);
    assert!(<f64 as RFloat>::r_max_value() > 0.0);
    assert!(<f64 as RFloat>::r_epsilon() > 0.0);
    assert!(<f64 as RFloat>::r_epsilon() < 1e-10);
}

#[test]
fn test_rfloat_power() {
    assert_eq!(2.0f64.r_powi(3), 8.0);
    assert_eq!(2.0f64.r_powi(-1), 0.5);
    assert_eq!(<f64 as RFloat>::r_powf(&2.0f64, &3.0), 8.0);
    assert_eq!(<f64 as RFloat>::r_powf(&4.0f64, &0.5), 2.0);
    assert_eq!(<f64 as RFloat>::r_recip(&2.0f64), 0.5);
}

#[test]
fn test_rfloat_f32() {
    // Verify f32 also works
    assert_eq!(<f32 as RNum>::r_zero(), 0.0f32);
    assert_eq!(<f32 as RNum>::r_one(), 1.0f32);
    assert!(f32::NAN.r_is_nan());
    assert_eq!(4.0f32.r_sqrt(), 2.0f32);
}
