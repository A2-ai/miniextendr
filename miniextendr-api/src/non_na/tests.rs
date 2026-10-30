use super::*;

#[test]
fn new_refuses_what_any_na_counts() {
    assert!(NonNa::new(1.5_f64).is_some());
    assert!(NonNa::new(f64::NAN).is_none());
    assert!(NonNa::new(<f64 as crate::RNativeType>::R_NA).is_none());
    assert!(NonNa::new(7_i32).is_some());
    assert!(NonNa::new(i32::MIN).is_none(), "NA_integer_");
    assert!(NonNa::new(RLogical::TRUE).is_some());
    assert!(NonNa::new(RLogical::NA).is_none());
    assert!(
        NonNa::new(Rcomplex {
            r: 1.0,
            i: f64::NAN
        })
        .is_none()
    );
    assert!(
        NonNa::new(String::from("NA")).is_some(),
        "a string is never NA"
    );
    assert!(NonNa::new(true).is_some());
}

#[test]
fn new_looks_into_containers() {
    assert!(NonNa::new(vec![1.0, 2.0]).is_some());
    assert!(NonNa::new(vec![1.0, f64::NAN]).is_none());
    assert!(NonNa::new(vec![Some(1), Some(2)]).is_some());
    assert!(
        NonNa::new(vec![Some(1), None]).is_none(),
        "None is an NA element"
    );
    assert!(NonNa::new(Vec::<f64>::new()).is_some());
    assert!(NonNa::new(&[1_i32, i32::MIN][..]).is_none());
    assert!(NonNa::new([1.0_f64; 3]).is_some());
    assert!(NonNa::new(vec![1.0, 2.0].into_boxed_slice()).is_some());
}

#[test]
fn accessors_give_the_value() {
    let x = NonNa::new(2.5_f64).unwrap();
    assert_eq!(*x, 2.5);
    assert_eq!(x.get(), 2.5);
    assert_eq!(x.into_inner(), 2.5);
    assert_eq!(x.unwrap(), 2.5);
    let xs = NonNa::new(vec![1, 2, 3]).unwrap();
    assert_eq!(xs.len(), 3, "Deref");
    assert_eq!(AsRef::<Vec<i32>>::as_ref(&xs), &vec![1, 2, 3]);
    assert_eq!(Borrow::<Vec<i32>>::borrow(&xs), &vec![1, 2, 3]);
    assert_eq!(xs.into_inner(), vec![1, 2, 3]);
    // SAFETY: 4 is not NA.
    let y = unsafe { NonNa::new_unchecked(4_i32) };
    assert_eq!(y.get(), 4);
}

#[test]
fn transparent_over_the_value() {
    assert_eq!(
        std::mem::size_of::<NonNa<f64>>(),
        std::mem::size_of::<f64>()
    );
    assert_eq!(
        std::mem::size_of::<NonNa<Vec<u8>>>(),
        std::mem::size_of::<Vec<u8>>()
    );
}
