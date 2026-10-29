use super::Altrep;

#[test]
fn array_marker_coerces_to_mutable_slice_view() {
    let mut array = Altrep([1, 2, 3]);
    let slice: &mut Altrep<[i32]> = &mut array;
    slice[1] = 9;
    assert_eq!(&**slice, &[1, 9, 3]);
    assert_eq!(array.into_inner(), [1, 9, 3]);
}
