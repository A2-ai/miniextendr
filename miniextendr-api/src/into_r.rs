pub trait IntoR {
    fn into_sexp(self) -> crate::ffi::SEXP;
}

impl IntoR for crate::ffi::SEXP {
    fn into_sexp(self) -> crate::ffi::SEXP {
        self
    }
}

impl IntoR for () {
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::R_NilValue }
    }
}
