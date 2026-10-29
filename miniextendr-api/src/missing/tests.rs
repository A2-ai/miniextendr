use super::*;

#[test]
fn missing_methods() {
    let present: Missing<i32> = Missing::Present(42);
    let absent: Missing<i32> = Missing::Absent;

    assert!(present.is_present());
    assert!(!present.is_missing());

    assert!(!absent.is_present());
    assert!(absent.is_missing());

    assert_eq!(present.into_option(), Some(42));
    assert_eq!(absent.into_option(), None);
}

#[test]
fn missing_unwrap_or() {
    let present: Missing<i32> = Missing::Present(42);
    let absent: Missing<i32> = Missing::Absent;

    assert_eq!(present.unwrap_or(0), 42);
    assert_eq!(absent.unwrap_or(0), 0);
}

#[test]
fn missing_map() {
    let present: Missing<i32> = Missing::Present(21);
    let absent: Missing<i32> = Missing::Absent;

    assert_eq!(present.map(|x| x * 2), Missing::Present(42));
    assert_eq!(absent.map(|x| x * 2), Missing::Absent);
}

#[test]
fn missing_default() {
    let m: Missing<i32> = Missing::default();
    assert!(m.is_missing());
}

#[test]
fn missing_from_option() {
    let some: Missing<i32> = Some(42).into();
    let none: Missing<i32> = None.into();

    assert_eq!(some, Missing::Present(42));
    assert_eq!(none, Missing::Absent);
}

/// `Missing<List>` is a parameter type: `List`'s conversion error (a
/// type error, the only way a `List` conversion fails) reaches
/// `SexpError::Type`.
#[test]
fn missing_list_converts() {
    fn is_param<T: TryFromSexp>() {}
    is_param::<Missing<crate::list::List>>();

    let type_err: <crate::list::List as TryFromSexp>::Error = crate::from_r::SexpTypeError {
        expected: crate::SEXPTYPE::VECSXP,
        actual: crate::SEXPTYPE::STRSXP,
    };
    assert!(matches!(SexpError::from(type_err), SexpError::Type(_)));
}
