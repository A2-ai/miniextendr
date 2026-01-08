use super::*;

#[test]
fn test_identity() {
    assert_eq!(Coerce::<i32>::coerce(42i32), 42i32);
    assert_eq!(
        Coerce::<f64>::coerce(std::f64::consts::PI),
        std::f64::consts::PI
    );
}

#[test]
fn test_widening() {
    let x: i32 = 42i8.coerce();
    assert_eq!(x, 42);

    let y: f64 = 42i32.coerce();
    assert_eq!(y, 42.0);
}

#[test]
fn test_bool() {
    assert_eq!(Coerce::<Rboolean>::coerce(true), Rboolean::TRUE);
    assert_eq!(Coerce::<i32>::coerce(true), 1);
    assert_eq!(Coerce::<f64>::coerce(false), 0.0);
}

fn takes_coercible<T: Coerce<i32>>(x: T) -> i32 {
    x.coerce()
}

#[test]
fn test_trait_bound() {
    assert_eq!(takes_coercible(42i8), 42);
    assert_eq!(takes_coercible(true), 1);
}

#[test]
fn test_try_coerce() {
    assert_eq!(TryCoerce::<i32>::try_coerce(42u32), Ok(42));
    assert_eq!(
        TryCoerce::<i32>::try_coerce(u32::MAX),
        Err(CoerceError::Overflow)
    );
}

#[test]
fn test_f64_to_i32() {
    assert_eq!(TryCoerce::<i32>::try_coerce(42.0f64), Ok(42));
    assert_eq!(
        TryCoerce::<i32>::try_coerce(42.5f64),
        Err(CoerceError::PrecisionLoss)
    );
    assert_eq!(
        TryCoerce::<i32>::try_coerce(f64::NAN),
        Err(CoerceError::NaN)
    );
}

#[test]
fn test_i64_to_f64() {
    assert_eq!(TryCoerce::<f64>::try_coerce(1000i64), Ok(1000.0));
    assert_eq!(
        TryCoerce::<f64>::try_coerce(i64::MAX),
        Err(CoerceError::PrecisionLoss)
    );
}

#[test]
fn test_slice_coerce() {
    let slice: &[i8] = &[1, 2, 3];
    let vec: Vec<i32> = slice.coerce();
    assert_eq!(vec, vec![1i32, 2, 3]);
}

#[test]
fn test_option_na_coerce() {
    assert_eq!(Coerce::<i32>::coerce(None::<i32>), NA_INTEGER);
    assert_eq!(Coerce::<i32>::coerce(None::<bool>), NA_LOGICAL);
    assert_eq!(Coerce::<i32>::coerce(None::<Rboolean>), NA_LOGICAL);

    let na_real = Coerce::<f64>::coerce(None::<f64>);
    assert_eq!(na_real.to_bits(), NA_REAL.to_bits());
}

#[test]
fn test_vec_coerce() {
    let v: Vec<i16> = vec![10, 20, 30];
    let result: Vec<f64> = v.coerce();
    assert_eq!(result, vec![10.0, 20.0, 30.0]);
}

#[test]
fn test_slice_try_coerce_via_blanket() {
    // When element type has Coerce, TryCoerce is provided via blanket impl
    let slice: &[i8] = &[1, 2, 3];
    let result: Result<Vec<i32>, _> = slice.try_coerce();
    assert_eq!(result, Ok(vec![1, 2, 3]));
}

#[test]
fn test_fallible_slice_coerce_manual() {
    // For types with only TryCoerce (not Coerce), use manual iteration
    let slice: &[u32] = &[1, u32::MAX, 3];
    let result: Result<Vec<i32>, _> = slice.iter().copied().map(TryCoerce::try_coerce).collect();
    assert_eq!(result, Err(CoerceError::Overflow));
}

#[test]
fn test_i32_to_u16() {
    // Success case
    assert_eq!(TryCoerce::<u16>::try_coerce(1000i32), Ok(1000u16));
    // Overflow - negative
    assert_eq!(
        TryCoerce::<u16>::try_coerce(-1i32),
        Err(CoerceError::Overflow)
    );
    // Overflow - too large
    assert_eq!(
        TryCoerce::<u16>::try_coerce(70000i32),
        Err(CoerceError::Overflow)
    );
}

#[test]
fn test_i32_slice_to_u16_vec() {
    // Using manual iteration for fallible element-wise coercion
    let slice: &[i32] = &[1, 100, 1000];
    let result: Result<Vec<u16>, _> = slice.iter().copied().map(TryCoerce::try_coerce).collect();
    assert_eq!(result, Ok(vec![1u16, 100, 1000]));

    // Failure case
    let slice2: &[i32] = &[1, -5, 1000];
    let result2: Result<Vec<u16>, _> = slice2.iter().copied().map(TryCoerce::try_coerce).collect();
    assert_eq!(result2, Err(CoerceError::Overflow));
}

#[test]
fn test_widening_to_u16() {
    let x: u16 = 42u8.coerce();
    assert_eq!(x, 42u16);
}

#[test]
fn test_i32_to_bool() {
    // TRUE (1)
    assert_eq!(TryCoerce::<bool>::try_coerce(1i32), Ok(true));
    // FALSE (0)
    assert_eq!(TryCoerce::<bool>::try_coerce(0i32), Ok(false));
    // NA (i32::MIN)
    assert_eq!(
        TryCoerce::<bool>::try_coerce(i32::MIN),
        Err(LogicalCoerceError::NAValue)
    );
    // Invalid value
    assert_eq!(
        TryCoerce::<bool>::try_coerce(42i32),
        Err(LogicalCoerceError::InvalidValue(42))
    );
}

#[test]
fn test_i32_to_i64() {
    // Coerce (infallible widening)
    let x: i64 = 42i32.coerce();
    assert_eq!(x, 42i64);

    let y: i64 = (-100i32).coerce();
    assert_eq!(y, -100i64);

    // Edge cases
    let max: i64 = i32::MAX.coerce();
    assert_eq!(max, i32::MAX as i64);

    let min: i64 = i32::MIN.coerce();
    assert_eq!(min, i32::MIN as i64);
}

#[test]
fn test_i32_to_isize() {
    // Coerce (infallible)
    let x: isize = 42i32.coerce();
    assert_eq!(x, 42isize);

    let y: isize = (-100i32).coerce();
    assert_eq!(y, -100isize);
}

#[test]
fn test_i32_to_u32() {
    // Success
    assert_eq!(TryCoerce::<u32>::try_coerce(42i32), Ok(42u32));
    assert_eq!(TryCoerce::<u32>::try_coerce(0i32), Ok(0u32));
    assert_eq!(TryCoerce::<u32>::try_coerce(i32::MAX), Ok(i32::MAX as u32));
    // Failure - negative
    assert_eq!(
        TryCoerce::<u32>::try_coerce(-1i32),
        Err(CoerceError::Overflow)
    );
}

#[test]
fn test_i32_to_u64() {
    // Success
    assert_eq!(TryCoerce::<u64>::try_coerce(42i32), Ok(42u64));
    assert_eq!(TryCoerce::<u64>::try_coerce(i32::MAX), Ok(i32::MAX as u64));
    // Failure - negative
    assert_eq!(
        TryCoerce::<u64>::try_coerce(-1i32),
        Err(CoerceError::Overflow)
    );
}

#[test]
fn test_i32_to_usize() {
    // Success
    assert_eq!(TryCoerce::<usize>::try_coerce(42i32), Ok(42usize));
    // Failure - negative
    assert_eq!(
        TryCoerce::<usize>::try_coerce(-1i32),
        Err(CoerceError::Overflow)
    );
}

#[test]
fn test_i32_slice_to_bool_vec() {
    let slice: &[i32] = &[1, 0, 1, 0];
    let result: Result<Vec<bool>, _> = slice.iter().copied().map(TryCoerce::try_coerce).collect();
    assert_eq!(result, Ok(vec![true, false, true, false]));

    // With NA
    let slice_na: &[i32] = &[1, i32::MIN, 0];
    let result_na: Result<Vec<bool>, LogicalCoerceError> = slice_na
        .iter()
        .copied()
        .map(TryCoerce::try_coerce)
        .collect();
    assert_eq!(result_na, Err(LogicalCoerceError::NAValue));
}

#[test]
fn test_nonzero_i32() {
    use core::num::NonZeroI32;

    // Success
    assert_eq!(
        TryCoerce::<NonZeroI32>::try_coerce(42i32),
        Ok(NonZeroI32::new(42).unwrap())
    );
    assert_eq!(
        TryCoerce::<NonZeroI32>::try_coerce(-5i32),
        Ok(NonZeroI32::new(-5).unwrap())
    );

    // Failure - zero
    assert_eq!(
        TryCoerce::<NonZeroI32>::try_coerce(0i32),
        Err(CoerceError::Zero)
    );
}

#[test]
fn test_nonzero_u32_from_i32() {
    use core::num::NonZeroU32;

    // Success
    assert_eq!(
        TryCoerce::<NonZeroU32>::try_coerce(42i32),
        Ok(NonZeroU32::new(42).unwrap())
    );

    // Failure - zero
    assert_eq!(
        TryCoerce::<NonZeroU32>::try_coerce(0i32),
        Err(CoerceError::Zero)
    );

    // Failure - negative (overflow before zero check)
    assert_eq!(
        TryCoerce::<NonZeroU32>::try_coerce(-1i32),
        Err(CoerceError::Overflow)
    );
}

#[test]
fn test_nonzero_i64_from_i32() {
    use core::num::NonZeroI64;

    // Success - widening
    assert_eq!(
        TryCoerce::<NonZeroI64>::try_coerce(42i32),
        Ok(NonZeroI64::new(42).unwrap())
    );
    assert_eq!(
        TryCoerce::<NonZeroI64>::try_coerce(-100i32),
        Ok(NonZeroI64::new(-100).unwrap())
    );

    // Failure - zero
    assert_eq!(
        TryCoerce::<NonZeroI64>::try_coerce(0i32),
        Err(CoerceError::Zero)
    );
}

#[test]
fn test_nonzero_usize_from_i32() {
    use core::num::NonZeroUsize;

    // Success
    assert_eq!(
        TryCoerce::<NonZeroUsize>::try_coerce(42i32),
        Ok(NonZeroUsize::new(42).unwrap())
    );

    // Failure - zero
    assert_eq!(
        TryCoerce::<NonZeroUsize>::try_coerce(0i32),
        Err(CoerceError::Zero)
    );

    // Failure - negative
    assert_eq!(
        TryCoerce::<NonZeroUsize>::try_coerce(-1i32),
        Err(CoerceError::Overflow)
    );
}

#[test]
fn test_f64_to_u8() {
    // Success
    assert_eq!(TryCoerce::<u8>::try_coerce(42.0f64), Ok(42u8));
    assert_eq!(TryCoerce::<u8>::try_coerce(0.0f64), Ok(0u8));
    assert_eq!(TryCoerce::<u8>::try_coerce(255.0f64), Ok(255u8));

    // Failure - negative
    assert_eq!(
        TryCoerce::<u8>::try_coerce(-1.0f64),
        Err(CoerceError::Overflow)
    );
    // Failure - too large
    assert_eq!(
        TryCoerce::<u8>::try_coerce(256.0f64),
        Err(CoerceError::Overflow)
    );
    // Failure - fractional
    assert_eq!(
        TryCoerce::<u8>::try_coerce(1.5f64),
        Err(CoerceError::PrecisionLoss)
    );
    // Failure - NaN
    assert_eq!(TryCoerce::<u8>::try_coerce(f64::NAN), Err(CoerceError::NaN));
}

#[test]
fn test_f64_to_u32() {
    // Success
    assert_eq!(TryCoerce::<u32>::try_coerce(42.0f64), Ok(42u32));
    assert_eq!(TryCoerce::<u32>::try_coerce(0.0f64), Ok(0u32));

    // Failure - negative
    assert_eq!(
        TryCoerce::<u32>::try_coerce(-1.0f64),
        Err(CoerceError::Overflow)
    );
    // Failure - fractional
    assert_eq!(
        TryCoerce::<u32>::try_coerce(1.5f64),
        Err(CoerceError::PrecisionLoss)
    );
}

#[test]
fn test_f64_to_i64() {
    // Success
    assert_eq!(TryCoerce::<i64>::try_coerce(42.0f64), Ok(42i64));
    assert_eq!(TryCoerce::<i64>::try_coerce(-100.0f64), Ok(-100i64));
    assert_eq!(TryCoerce::<i64>::try_coerce(0.0f64), Ok(0i64));

    // Failure - fractional
    assert_eq!(
        TryCoerce::<i64>::try_coerce(1.5f64),
        Err(CoerceError::PrecisionLoss)
    );
    // Failure - NaN
    assert_eq!(
        TryCoerce::<i64>::try_coerce(f64::NAN),
        Err(CoerceError::NaN)
    );
}

#[test]
fn test_f64_to_u64() {
    // Success
    assert_eq!(TryCoerce::<u64>::try_coerce(42.0f64), Ok(42u64));
    assert_eq!(TryCoerce::<u64>::try_coerce(0.0f64), Ok(0u64));

    // Failure - negative
    assert_eq!(
        TryCoerce::<u64>::try_coerce(-1.0f64),
        Err(CoerceError::Overflow)
    );
    // Failure - fractional
    assert_eq!(
        TryCoerce::<u64>::try_coerce(1.5f64),
        Err(CoerceError::PrecisionLoss)
    );
}

#[test]
fn test_nonzero_smaller_from_i32() {
    use core::num::{NonZeroI8, NonZeroI16, NonZeroU8, NonZeroU16};

    // NonZeroI8
    assert_eq!(
        TryCoerce::<NonZeroI8>::try_coerce(42i32),
        Ok(NonZeroI8::new(42).unwrap())
    );
    assert_eq!(
        TryCoerce::<NonZeroI8>::try_coerce(0i32),
        Err(CoerceError::Zero)
    );
    assert_eq!(
        TryCoerce::<NonZeroI8>::try_coerce(200i32),
        Err(CoerceError::Overflow)
    );

    // NonZeroI16
    assert_eq!(
        TryCoerce::<NonZeroI16>::try_coerce(1000i32),
        Ok(NonZeroI16::new(1000).unwrap())
    );
    assert_eq!(
        TryCoerce::<NonZeroI16>::try_coerce(0i32),
        Err(CoerceError::Zero)
    );
    assert_eq!(
        TryCoerce::<NonZeroI16>::try_coerce(40000i32),
        Err(CoerceError::Overflow)
    );

    // NonZeroU8
    assert_eq!(
        TryCoerce::<NonZeroU8>::try_coerce(42i32),
        Ok(NonZeroU8::new(42).unwrap())
    );
    assert_eq!(
        TryCoerce::<NonZeroU8>::try_coerce(0i32),
        Err(CoerceError::Zero)
    );
    assert_eq!(
        TryCoerce::<NonZeroU8>::try_coerce(-1i32),
        Err(CoerceError::Overflow)
    );
    assert_eq!(
        TryCoerce::<NonZeroU8>::try_coerce(300i32),
        Err(CoerceError::Overflow)
    );

    // NonZeroU16
    assert_eq!(
        TryCoerce::<NonZeroU16>::try_coerce(1000i32),
        Ok(NonZeroU16::new(1000).unwrap())
    );
    assert_eq!(
        TryCoerce::<NonZeroU16>::try_coerce(0i32),
        Err(CoerceError::Zero)
    );
    assert_eq!(
        TryCoerce::<NonZeroU16>::try_coerce(-1i32),
        Err(CoerceError::Overflow)
    );
}

#[test]
fn test_tuple_coerce() {
    // 2-tuple
    let t: (i8, i16) = (1, 2);
    let coerced: (i32, i32) = t.coerce();
    assert_eq!(coerced, (1i32, 2i32));

    // Mixed types
    let t2: (i8, f32) = (42, std::f32::consts::PI);
    let coerced2: (i32, f64) = t2.coerce();
    assert_eq!(coerced2.0, 42i32);
    assert!((coerced2.1 - std::f64::consts::PI).abs() < 0.001);

    // 3-tuple
    let t3: (i8, i16, u8) = (1, 2, 3);
    let coerced3: (i32, i32, i32) = t3.coerce();
    assert_eq!(coerced3, (1, 2, 3));

    // Identity coercion
    let t4: (i32, f64) = (10, 20.0);
    let coerced4: (i32, f64) = t4.coerce();
    assert_eq!(coerced4, (10, 20.0));
}

#[test]
fn test_option_f64_coerce() {
    // Some value passes through
    let x: f64 = Some(42.0).coerce();
    assert_eq!(x, 42.0);

    // None becomes NA_real_ (NaN)
    let na: f64 = None::<f64>.coerce();
    assert!(na.is_nan());
}

#[test]
fn test_option_i32_coerce() {
    // Some value passes through
    let x: i32 = Some(42).coerce();
    assert_eq!(x, 42);

    // None becomes NA_integer_ (i32::MIN)
    let na: i32 = None::<i32>.coerce();
    assert_eq!(na, i32::MIN);
}

#[test]
fn test_option_bool_coerce() {
    // Some(true) -> 1
    let t: i32 = Some(true).coerce();
    assert_eq!(t, 1);

    // Some(false) -> 0
    let f: i32 = Some(false).coerce();
    assert_eq!(f, 0);

    // None -> NA_LOGICAL (i32::MIN)
    let na: i32 = None::<bool>.coerce();
    assert_eq!(na, i32::MIN);
}

#[test]
fn test_option_rboolean_coerce() {
    // Some values pass through
    let t: i32 = Some(Rboolean::TRUE).coerce();
    assert_eq!(t, 1);

    let f: i32 = Some(Rboolean::FALSE).coerce();
    assert_eq!(f, 0);

    // None -> NA_LOGICAL (i32::MIN)
    let na: i32 = None::<Rboolean>.coerce();
    assert_eq!(na, i32::MIN);
}

#[test]
fn test_option_vec_coerce() {
    // Vec<Option<T>> element-wise coercion via blanket impl
    let v: Vec<Option<f64>> = vec![Some(1.0), None, Some(3.0)];
    let coerced: Vec<f64> = v.coerce();
    assert_eq!(coerced[0], 1.0);
    assert!(coerced[1].is_nan());
    assert_eq!(coerced[2], 3.0);

    let v2: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    let coerced2: Vec<i32> = v2.coerce();
    assert_eq!(coerced2, vec![1, i32::MIN, 3]);
}

#[test]
fn test_option_slice_coerce() {
    // &[Option<T>] element-wise coercion to Vec<R>
    let slice: &[Option<f64>] = &[Some(1.0), None, Some(3.0)];
    let coerced: Vec<f64> = slice.coerce();
    assert_eq!(coerced[0], 1.0);
    assert!(coerced[1].is_nan());
    assert_eq!(coerced[2], 3.0);

    let slice2: &[Option<i32>] = &[Some(1), None, Some(3)];
    let coerced2: Vec<i32> = slice2.coerce();
    assert_eq!(coerced2, vec![1, i32::MIN, 3]);

    let slice3: &[Option<bool>] = &[Some(true), None, Some(false)];
    let coerced3: Vec<i32> = slice3.coerce();
    assert_eq!(coerced3, vec![1, i32::MIN, 0]);
}
