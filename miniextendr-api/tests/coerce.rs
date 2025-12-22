//! Integration tests for coercion traits.

use miniextendr_api::coerce::{Coerce, CoerceError, TryCoerce};

#[test]
fn coerce_suite() {
    // Infallible coercions
    assert_eq!(Coerce::<i32>::coerce(5i8), 5i32);
    assert_eq!(Coerce::<f64>::coerce(true), 1.0f64);
    assert_eq!(Coerce::<i32>::coerce(false), 0i32);

    // Fallible coercions
    assert_eq!(TryCoerce::<i32>::try_coerce(123u16).unwrap(), 123i32);
    assert!(matches!(
        TryCoerce::<i32>::try_coerce(u32::MAX),
        Err(CoerceError::Overflow)
    ));

    assert!(matches!(
        TryCoerce::<i32>::try_coerce(1.5f64),
        Err(CoerceError::PrecisionLoss)
    ));

    assert!(matches!(
        TryCoerce::<i32>::try_coerce(f64::NAN),
        Err(CoerceError::NaN)
    ));
}
