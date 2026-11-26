use super::*;

/// The step from a number to `i32`: whole numbers in R's integer range pass,
/// `NaN` is missing, everything else is refused.
#[test]
fn numbers_store_as_r_integers() {
    for (value, expected) in [
        (0.0, Some(0)),
        (-0.0, Some(0)),
        (3.0, Some(3)),
        (-7.0, Some(-7)),
        (2147483647.0, Some(i32::MAX)),
        (-2147483647.0, Some(-i32::MAX)),
    ] {
        assert!(
            matches!(i32::from_number(value), Ok(Some(v)) if Some(v) == expected),
            "{value}"
        );
    }
    assert!(matches!(i32::from_number(f64::NAN), Ok(None)));
    // `i32::MIN` is `NA_integer_`, so it is out of range.
    for value in [
        3.5,
        -0.5,
        f64::INFINITY,
        f64::NEG_INFINITY,
        2147483648.0,
        -2147483648.0,
        3e10,
    ] {
        assert!(matches!(i32::from_number(value), Err(Refused)), "{value}");
    }
}

/// `no_na` refuses every `None`.
#[test]
fn no_na_refuses_none() {
    assert!(!AsInteger(Some(0)).__mx_has_na());
    assert!(AsInteger(None).__mx_has_na());

    assert!(!AsIntegerVec(vec![]).__mx_has_na());
    assert!(!AsIntegerVec(vec![Some(1), Some(-1)]).__mx_has_na());
    assert!(AsIntegerVec(vec![Some(1), None]).__mx_has_na());

    assert!(!None::<AsInteger>.__mx_has_na());
    assert!(Some(AsInteger(None)).__mx_has_na());
    assert!(Some(AsIntegerVec(vec![None])).__mx_has_na());
}
