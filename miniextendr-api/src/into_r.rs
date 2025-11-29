pub trait IntoR {
    fn into_sexp(self) -> crate::ffi::SEXP;
}

impl IntoR for crate::ffi::SEXP {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self
    }
}

impl IntoR for () {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::R_NilValue }
    }
}

impl IntoR for std::convert::Infallible {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::R_NilValue }
    }
}

impl IntoR for i32 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarInteger(self) }
    }
}

impl IntoR for f64 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarReal(self) }
    }
}

impl IntoR for u8 {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarRaw(self) }
    }
}

impl IntoR for bool {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical(self as i32) }
    }
}

impl IntoR for crate::ffi::Rboolean {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe { crate::ffi::Rf_ScalarLogical(self as i32) }
    }
}

impl<T: crate::externalptr::TypedExternal> IntoR for crate::externalptr::ExternalPtr<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_sexp()
    }
}

/// Helper to convert a string slice to R CHARSXP.
/// Uses UTF-8 encoding. Empty strings return R_BlankString equivalent.
#[inline]
fn str_to_charsxp(s: &str) -> crate::ffi::SEXP {
    unsafe {
        if s.is_empty() {
            // For empty string, still use mkCharLenCE with length 0
            crate::ffi::Rf_mkCharLenCE(s.as_ptr().cast(), 0, crate::ffi::CE_UTF8)
        } else {
            crate::ffi::Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, crate::ffi::CE_UTF8)
        }
    }
}

impl IntoR for String {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_str().into_sexp()
    }
}

impl IntoR for &str {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = str_to_charsxp(self);
            crate::ffi::Rf_ScalarString(charsxp)
        }
    }
}
