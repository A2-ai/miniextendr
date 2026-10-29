use super::*;

#[test]
fn decimal_from_str() {
    let d = Decimal::from_str("123.456").unwrap();
    assert_eq!(RDecimalOps::as_string(&d), "123.456");
}

#[test]
fn rdecimalops_sign() {
    let pos = Decimal::from_str("123.45").unwrap();
    let neg = Decimal::from_str("-67.89").unwrap();
    let zero = Decimal::ZERO;

    assert!(RDecimalOps::is_positive(&pos));
    assert!(!RDecimalOps::is_negative(&pos));
    assert_eq!(RDecimalOps::sign(&pos), 1);

    assert!(!RDecimalOps::is_positive(&neg));
    assert!(RDecimalOps::is_negative(&neg));
    assert_eq!(RDecimalOps::sign(&neg), -1);

    assert!(RDecimalOps::is_zero(&zero));
    assert_eq!(RDecimalOps::sign(&zero), 0);
}

#[test]
fn rdecimalops_arithmetic() {
    let d = Decimal::from_str("100.5").unwrap();

    let sum = RDecimalOps::add_str(&d, "0.5").unwrap();
    assert_eq!(ToString::to_string(&sum), "101.0");

    let diff = RDecimalOps::sub_str(&d, "0.5").unwrap();
    assert_eq!(ToString::to_string(&diff), "100.0");

    let prod = RDecimalOps::mul_str(&d, "2").unwrap();
    assert_eq!(ToString::to_string(&prod), "201.0");

    let quot = RDecimalOps::div_str(&d, "2").unwrap();
    // Note: Decimal preserves scale, so 100.5 / 2 = 50.250
    assert_eq!(ToString::to_string(&RDecimalOps::normalize(&quot)), "50.25");

    let rem = RDecimalOps::rem_str(&d, "7").unwrap();
    assert_eq!(ToString::to_string(&rem), "2.5"); // 100.5 % 7 = 2.5
}

#[test]
fn rdecimalops_rounding() {
    let d = Decimal::from_str("123.456").unwrap();

    assert_eq!(ToString::to_string(&RDecimalOps::round(&d, 2)), "123.46");
    assert_eq!(ToString::to_string(&RDecimalOps::floor(&d)), "123");
    assert_eq!(ToString::to_string(&RDecimalOps::ceil(&d)), "124");
    assert_eq!(ToString::to_string(&RDecimalOps::trunc(&d)), "123");
    assert_eq!(ToString::to_string(&RDecimalOps::fract(&d)), "0.456");
}

#[test]
fn rdecimalops_scale() {
    let d1 = Decimal::from_str("123.456").unwrap();
    assert_eq!(RDecimalOps::scale(&d1), 3);

    let d2 = Decimal::from_str("100").unwrap();
    assert_eq!(RDecimalOps::scale(&d2), 0);
}

#[test]
fn rdecimalops_abs_neg() {
    let neg = Decimal::from_str("-123.45").unwrap();
    assert_eq!(ToString::to_string(&RDecimalOps::abs(&neg)), "123.45");
    assert_eq!(ToString::to_string(&RDecimalOps::neg(&neg)), "123.45");

    let pos = Decimal::from_str("67.89").unwrap();
    assert_eq!(ToString::to_string(&RDecimalOps::abs(&pos)), "67.89");
    assert_eq!(ToString::to_string(&RDecimalOps::neg(&pos)), "-67.89");
}

#[test]
fn rdecimalops_conversions() {
    let d = Decimal::from_str("123.456").unwrap();
    assert!((RDecimalOps::as_f64(&d) - 123.456).abs() < 0.0001);
    assert_eq!(RDecimalOps::as_i64(&d), None); // Has decimal part

    let int_d = Decimal::from_str("100").unwrap();
    assert_eq!(RDecimalOps::as_i64(&int_d), Some(100));
    assert!(RDecimalOps::is_integer(&int_d));
    assert!(!RDecimalOps::is_integer(&d));
}

#[test]
fn rdecimalops_normalize() {
    let d = Decimal::from_str("100.500").unwrap();
    let normalized = RDecimalOps::normalize(&d);
    assert_eq!(ToString::to_string(&normalized), "100.5");
}

#[test]
fn rdecimalops_division_by_zero() {
    let d = Decimal::from_str("100").unwrap();
    assert!(RDecimalOps::div_str(&d, "0").is_err());
    assert!(RDecimalOps::rem_str(&d, "0").is_err());
}
