use miniextendr_api::RNativeType;

#[repr(transparent)]
#[derive(Clone, Copy, RNativeType)]
pub struct Tuple(pub i32);

#[repr(transparent)]
#[derive(Clone, Copy, RNativeType)]
pub struct Named { pub value: f64 }

#[repr(transparent)]
#[derive(Clone, Copy, RNativeType)]
pub struct Generic<T: RNativeType>(pub T);

fn main() {
    assert_eq!(std::mem::size_of::<Tuple>(), std::mem::size_of::<i32>());
    assert_eq!(std::mem::align_of::<Named>(), std::mem::align_of::<f64>());
    assert_eq!(std::mem::size_of::<Generic<i32>>(), std::mem::size_of::<i32>());
}
