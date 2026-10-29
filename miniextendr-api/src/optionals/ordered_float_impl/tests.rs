use super::*;

#[test]
fn ordered_float_basic() {
    let of = OrderedFloat(std::f64::consts::PI);
    assert!(!of.is_nan());
    assert!(of.is_finite());
    assert!(!of.is_infinite());
}

#[test]
fn rorderedfloatops_predicates() {
    let positive = OrderedFloat(std::f64::consts::PI);
    let negative = OrderedFloat(-2.5f64);
    let nan = OrderedFloat(f64::NAN);
    let inf = OrderedFloat(f64::INFINITY);

    assert!(ROrderedFloatOps::r_is_positive(&positive));
    assert!(!ROrderedFloatOps::r_is_negative(&positive));
    assert!(ROrderedFloatOps::r_is_negative(&negative));
    assert!(!ROrderedFloatOps::r_is_positive(&negative));

    assert!(ROrderedFloatOps::r_is_nan(&nan));
    assert!(!ROrderedFloatOps::r_is_finite(&nan));

    assert!(ROrderedFloatOps::r_is_infinite(&inf));
    assert!(!ROrderedFloatOps::r_is_finite(&inf));
}

#[test]
fn rorderedfloatops_rounding() {
    let of = OrderedFloat(3.7f64);
    assert_eq!(ROrderedFloatOps::r_floor(&of), 3.0);
    assert_eq!(ROrderedFloatOps::r_ceil(&of), 4.0);
    assert_eq!(ROrderedFloatOps::r_round(&of), 4.0);
    assert_eq!(ROrderedFloatOps::r_trunc(&of), 3.0);
    assert!((ROrderedFloatOps::r_fract(&of) - 0.7).abs() < 0.001);
}

#[test]
fn rorderedfloatops_abs_signum() {
    let neg = OrderedFloat(-5.0f64);
    assert_eq!(ROrderedFloatOps::r_abs(&neg), 5.0);
    assert_eq!(ROrderedFloatOps::r_signum(&neg), -1.0);

    let pos = OrderedFloat(5.0f64);
    assert_eq!(ROrderedFloatOps::r_signum(&pos), 1.0);
}

#[test]
fn rorderedfloatops_min_max_clamp() {
    let of = OrderedFloat(5.0f64);

    assert_eq!(ROrderedFloatOps::r_min_with(&of, 3.0), 3.0);
    assert_eq!(ROrderedFloatOps::r_min_with(&of, 7.0), 5.0);

    assert_eq!(ROrderedFloatOps::r_max_with(&of, 3.0), 5.0);
    assert_eq!(ROrderedFloatOps::r_max_with(&of, 7.0), 7.0);

    assert_eq!(ROrderedFloatOps::r_clamp_to(&of, 0.0, 10.0), 5.0);
    assert_eq!(ROrderedFloatOps::r_clamp_to(&of, 6.0, 10.0), 6.0);
    assert_eq!(ROrderedFloatOps::r_clamp_to(&of, 0.0, 3.0), 3.0);
}

#[test]
fn rorderedfloatops_nan_handling() {
    let nan = OrderedFloat(f64::NAN);
    let val = OrderedFloat(5.0f64);

    // In OrderedFloat's Ord impl, NaN is greater than all other values
    // So min(5.0, NaN) = 5.0, and max(NaN, 5.0) = NaN
    assert_eq!(ROrderedFloatOps::r_min_with(&val, f64::NAN), 5.0);
    assert!(ROrderedFloatOps::r_max_with(&nan, 5.0).is_nan());
}

#[test]
fn rorderedfloatops_f32() {
    let of = OrderedFloat(std::f32::consts::PI);
    assert!(!ROrderedFloatOps::r_is_nan(&of));
    assert!(ROrderedFloatOps::r_is_finite(&of));
    assert!((ROrderedFloatOps::r_floor(&of) - 3.0).abs() < 0.001);
}
