use super::*;

#[test]
fn markers_are_transparent_wrappers() {
    let i = Invisible::new(41).into_inner() + 1;
    assert_eq!(i, 42);
    let mut v: Visible<Vec<i32>> = Visible::from(vec![1]);
    v.push(2);
    assert_eq!(v.len(), 2);
    assert_eq!(*v, vec![1, 2]);
    assert_eq!(Invisible(()), Invisible::default());
    assert_eq!(
        std::mem::size_of::<Invisible<u64>>(),
        std::mem::size_of::<u64>()
    );
}
