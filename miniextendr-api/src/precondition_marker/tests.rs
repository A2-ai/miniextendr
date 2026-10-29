use super::*;

#[test]
fn markers_are_transparent_over_the_value() {
    assert_eq!(std::mem::size_of::<Checked<f64>>(), 8);
    assert_eq!(std::mem::size_of::<Unchecked<u16>>(), 2);
    let mut n = Checked::from_inner(3_i32);
    *n += 1;
    assert_eq!(*n, 4);
    assert_eq!(n.into_inner(), 4);
    let s = Unchecked::from_inner("a");
    assert_eq!(s.len(), 1);
    assert_eq!(s.into_inner(), "a");
}
