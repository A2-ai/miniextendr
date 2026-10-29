use super::*;

#[test]
fn either_can_be_created() {
    let _left: Either<i32, String> = Left(42);
    let _right: Either<i32, String> = Right("hello".to_string());
}

#[test]
fn either_forwards_no_na_to_its_side() {
    use crate::convert::{AsCharacter, AsNumeric, AsNumericVec};

    let left: Either<AsNumericVec, AsCharacter> = Left(AsNumericVec(vec![Some(1.0), None]));
    assert!(left.__mx_has_na());
    let left: Either<AsNumericVec, AsCharacter> = Left(AsNumericVec(vec![Some(1.0)]));
    assert!(!left.__mx_has_na());
    let right: Either<AsNumericVec, AsCharacter> = Right(AsCharacter(None));
    assert!(right.__mx_has_na());
    assert!(!Either::<AsNumericVec, i32>::Right(1).__mx_has_na());

    // An `Option` arm: `NULL` (`None`) is not given and passes, and a
    // number the marker reads as missing is refused.
    assert!(!Either::<Option<AsNumeric>, i32>::Left(None).__mx_has_na());
    assert!(Either::<Option<AsNumeric>, i32>::Left(Some(AsNumeric(None))).__mx_has_na());
    assert!(!Either::<Option<AsNumeric>, i32>::Left(Some(AsNumeric(Some(3.0)))).__mx_has_na());
}

/// A `List` arm converts its error to `SexpError`, as both arms must.
#[test]
fn either_takes_a_list_arm() {
    fn is_param<T: TryFromSexp>() {}
    is_param::<Either<i32, crate::list::List>>();
    is_param::<Either<crate::list::List, String>>();
}
