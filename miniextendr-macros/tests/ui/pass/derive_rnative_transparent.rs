use miniextendr_api::{IntoR, IsNA, RLogical, RNativeType, Rcomplex};

#[repr(transparent)]
#[derive(Clone, Copy, RNativeType, IsNA, IntoR)]
pub struct Tuple(pub i32);

#[repr(transparent)]
#[derive(Clone, Copy, RNativeType, IsNA, IntoR)]
pub struct Named { pub value: f64 }

#[repr(transparent)]
#[derive(Clone, Copy, RNativeType, IsNA, IntoR)]
pub struct Generic<T: RNativeType>(pub T)
where
    T: std::fmt::Debug;

fn needs_na<T: IsNA>() {}

fn needs_output<T: IntoR>() {}

fn main() {
    needs_output::<Tuple>();
    needs_output::<Vec<Tuple>>();
    needs_output::<Box<[Tuple]>>();
    needs_output::<Option<Vec<Tuple>>>();
    needs_output::<Vec<Vec<Tuple>>>();
    needs_output::<Vec<Option<Vec<Tuple>>>>();
    needs_output::<Vec<Box<[Tuple]>>>();
    needs_output::<Vec<[Tuple; 2]>>();
    needs_output::<Vec<Named>>();
    needs_output::<Vec<Generic<RLogical>>>();
    needs_output::<Vec<Generic<u8>>>();
    needs_na::<Tuple>();
    needs_na::<Named>();
    needs_na::<Generic<i32>>();
    needs_na::<Generic<f64>>();
    needs_na::<Generic<RLogical>>();
    needs_na::<Generic<Rcomplex>>();
    // The conditional IsNA impl must not prevent raw native storage.
    let raw = Generic::<u8>::R_NA;
    assert_eq!(raw.0, 0);
    assert!(Tuple::R_NA.is_na());
    assert!(Named::R_NA.is_na());
    assert!(!Named { value: f64::NAN }.is_na());
    assert_eq!(std::mem::size_of::<Tuple>(), std::mem::size_of::<i32>());
    assert_eq!(std::mem::align_of::<Named>(), std::mem::align_of::<f64>());
    assert_eq!(std::mem::size_of::<Generic<i32>>(), std::mem::size_of::<i32>());
}
