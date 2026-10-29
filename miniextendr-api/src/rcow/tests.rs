use super::*;

// The borrowed arm's behavior (try_from_root / zero-copy IntoR) requires a
// live R session and is exercised by the rpkg `zero_copy_rcow_*` fixtures
// and the `gc_stress_rcow_roundtrip` gctorture guard. These unit tests cover
// the R-independent surface (the owned arm and the Cow-like helpers).

#[test]
fn owned_deref_and_into_owned() {
    let c: RCow<'static, i32> = RCow::Owned(vec![1, 2, 3]);
    assert!(c.is_owned());
    assert!(!c.is_borrowed());
    assert_eq!(&*c, &[1, 2, 3]);
    assert_eq!(c.into_owned(), vec![1, 2, 3]);
}

#[test]
fn to_mut_on_owned_mutates_in_place() {
    let mut c: RCow<'static, f64> = RCow::Owned(vec![1.0]);
    c.to_mut().push(2.0);
    assert_eq!(&*c, &[1.0, 2.0]);
}

#[test]
fn from_vec_is_owned() {
    let c: RCow<'static, i32> = vec![9, 8, 7].into();
    assert!(c.is_owned());
    assert_eq!(c.len(), 3);
}
