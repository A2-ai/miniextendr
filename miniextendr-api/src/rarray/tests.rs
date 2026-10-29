use super::*;

#[test]
fn matrix_is_array2() {
    fn assert_matrix<T: RNativeType>(_: RMatrix<T>) {}
    fn assert_array2<T: RNativeType>(_: RArray<T, 2>) {}

    // These should compile - RMatrix<T> == RArray<T, 2>
    let m: RMatrix<f64> = unsafe { RArray::from_sexp_unchecked(SEXP(std::ptr::null_mut())) };
    assert_matrix(m);
    assert_array2(m);
}

#[test]
fn size_equals_sexp() {
    // RArray should be same size as SEXP (PhantomData is zero-sized)
    assert_eq!(
        std::mem::size_of::<RArray<f64, 2>>(),
        std::mem::size_of::<SEXP>()
    );
}

// Note: RArray is !Send and !Sync due to PhantomData<*const ()>.
// This is verified by the compiler - attempting to send RArray across
// threads will fail to compile.
