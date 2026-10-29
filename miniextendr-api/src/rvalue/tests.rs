use super::*;

// Accessors and TryFrom operate on owned `RValue` — no SEXP, no R runtime.

#[test]
fn as_scalar_accessors() {
    assert_eq!(RValue::from(7i32).as_i32(), Some(7));
    assert_eq!(RValue::from(1.5).as_f64(), Some(1.5));
    assert_eq!(RValue::from(true).as_bool(), Some(true));
    assert_eq!(RValue::from("hi").as_str(), Some("hi"));

    // wrong variant
    assert_eq!(RValue::from("hi").as_i32(), None);
    // wrong length
    assert_eq!(RValue::Integer(vec![Some(1), Some(2)]).as_i32(), None);
    // NA
    assert_eq!(RValue::Integer(vec![None]).as_i32(), None);
    assert_eq!(
        RValue::Double(vec![crate::altrep_traits::NA_REAL]).as_f64(),
        None
    );
    assert!(RValue::Null.is_null());
}

#[test]
fn try_from_scalar_ok_and_errors() {
    assert_eq!(i32::try_from(RValue::from(7i32)).unwrap(), 7);
    assert_eq!(f64::try_from(RValue::from(2.5)).unwrap(), 2.5);
    assert!(!bool::try_from(RValue::from(false)).unwrap());
    assert_eq!(String::try_from(RValue::from("x")).unwrap(), "x");

    // wrong variant → Type
    assert!(matches!(
        i32::try_from(RValue::from("x")),
        Err(SexpError::Type(_))
    ));
    // wrong length → Length
    assert!(matches!(
        i32::try_from(RValue::Integer(vec![Some(1), Some(2)])),
        Err(SexpError::Length(_))
    ));
    // NA → Na
    assert!(matches!(
        i32::try_from(RValue::Integer(vec![None])),
        Err(SexpError::Na(_))
    ));
    assert!(matches!(
        f64::try_from(RValue::Double(vec![crate::altrep_traits::NA_REAL])),
        Err(SexpError::Na(_))
    ));
}

/// A computed NA (`NA_real_ * 1` quiets the NaN: bits `0x7FF8_0000_0000_07A2`)
/// is NA by R's `R_IsNA` rule; a plain NaN is a value.
#[test]
fn double_na_follows_r_is_na() {
    let computed = f64::from_bits(0x7FF8_0000_0000_07A2);
    assert_eq!(RValue::Double(vec![computed]).as_f64(), None);
    assert!(matches!(
        f64::try_from(RValue::Double(vec![computed])),
        Err(SexpError::Na(_))
    ));
    assert!(
        RValue::Double(vec![f64::NAN])
            .as_f64()
            .is_some_and(f64::is_nan)
    );
    assert!(f64::try_from(RValue::Double(vec![f64::NAN])).is_ok_and(f64::is_nan));
}

#[test]
fn from_option_scalars() {
    assert!(matches!(RValue::from(Some(7_i32)), RValue::Integer(v) if v == vec![Some(7)]));
    assert!(matches!(RValue::from(None::<i32>), RValue::Integer(v) if v == vec![None]));
    assert!(matches!(RValue::from(Some(true)), RValue::Logical(v) if v == vec![Some(true)]));
    assert!(matches!(RValue::from(None::<bool>), RValue::Logical(v) if v == vec![None]));
    assert!(
        matches!(RValue::from(Some("x")), RValue::Character(v) if v == vec![Some("x".to_string())])
    );
    assert!(matches!(RValue::from(None::<&str>), RValue::Character(v) if v == vec![None]));
    assert!(
        matches!(RValue::from(Some("y".to_string())), RValue::Character(v) if v == vec![Some("y".to_string())])
    );

    // f64 has no Option variant: Some → value, None → NA_REAL bit pattern.
    assert_eq!(RValue::from(Some(1.5_f64)).as_f64(), Some(1.5));
    assert_eq!(RValue::from(None::<f64>).as_f64(), None);
}

#[test]
fn from_vec_option() {
    assert!(
        matches!(RValue::from(vec![Some(1_i32), None, Some(3)]), RValue::Integer(v) if v == vec![Some(1), None, Some(3)])
    );
    assert!(
        matches!(RValue::from(vec![Some(true), None]), RValue::Logical(v) if v == vec![Some(true), None])
    );
    assert!(
        matches!(RValue::from(vec![Some("a".to_string()), None]), RValue::Character(v) if v == vec![Some("a".to_string()), None])
    );
    assert!(
        matches!(RValue::from(vec![Some("b"), None]), RValue::Character(v) if v == vec![Some("b".to_string()), None])
    );

    // Vec<Option<f64>> → Double with NA_REAL in the None slot.
    let RValue::Double(v) = RValue::from(vec![Some(0.5_f64), None]) else {
        panic!("expected Double");
    };
    assert_eq!(v[0], 0.5);
    assert_eq!(v[1].to_bits(), crate::altrep_traits::NA_REAL.to_bits());
}

#[test]
fn from_wide_integer_ladder() {
    // Fits in i32 (and not i32::MIN) → Integer.
    assert!(matches!(RValue::from(42_i64), RValue::Integer(v) if v == vec![Some(42)]));
    assert!(
        matches!(RValue::from(i32::MAX as i64), RValue::Integer(v) if v == vec![Some(i32::MAX)])
    );
    assert!(matches!(RValue::from(7_u32), RValue::Integer(v) if v == vec![Some(7)]));
    // i32::MIN is NA_integer_ → promote to Double rather than emit NA.
    assert!(matches!(RValue::from(i32::MIN as i64), RValue::Double(_)));
    // Just past i32::MAX → Double.
    assert!(
        matches!(RValue::from(i32::MAX as i64 + 1), RValue::Double(v) if v == vec![(i32::MAX as i64 + 1) as f64])
    );
    // u32::MAX exceeds i32::MAX → Double.
    assert!(matches!(RValue::from(u32::MAX), RValue::Double(v) if v == vec![f64::from(u32::MAX)]));
}

#[test]
fn debug_stringifies() {
    assert_eq!(RValue::debug(0..=100).as_str(), Some("0..=100"));
    assert_eq!(RValue::debug(vec![1, 2]).as_str(), Some("[1, 2]"));
}

#[test]
fn try_from_whole_vector() {
    let v: Vec<Option<i32>> = RValue::Integer(vec![Some(1), None]).try_into().unwrap();
    assert_eq!(v, vec![Some(1), None]);

    let raw: Vec<u8> = RValue::Raw(vec![1, 2, 3]).try_into().unwrap();
    assert_eq!(raw, vec![1, 2, 3]);

    // wrong variant → Type
    let bad: Result<Vec<u8>, _> = RValue::from(1i32).try_into();
    assert!(matches!(bad, Err(SexpError::Type(_))));
}
