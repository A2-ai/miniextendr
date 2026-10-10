//! Scalar and vector conversions for owned unsized UTF-8 strings.
use std::rc::Rc;
use std::sync::Arc;

use super::{
    IntoR, opt_str_iter_to_strsxp, opt_str_iter_to_strsxp_unchecked, str_iter_to_strsxp,
    str_iter_to_strsxp_unchecked,
};
use crate::SEXP;

macro_rules! impl_shared_str_scalar {
    ($owner:ident) => {
        impl IntoR for $owner<str> {
            type Error = crate::IntoRError;
            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                self.as_ref().try_into_sexp()
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
                unsafe { self.as_ref().try_into_sexp_unchecked() }
            }
        }
    };
}
// Box<str> already reuses its allocation through String::from.
impl_shared_str_scalar!(Rc);
impl_shared_str_scalar!(Arc);

macro_rules! impl_owned_str_vectors {
    ($owner:ident) => {
        impl IntoR for Option<$owner<str>> {
            type Error = <Option<&'static str> as IntoR>::Error;
            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                self.as_deref().try_into_sexp()
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
                unsafe { self.as_deref().try_into_sexp_unchecked() }
            }
        }
        impl IntoR for Vec<$owner<str>> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                Ok(str_iter_to_strsxp(self.iter().map(|s| s.as_ref())))
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
                Ok(unsafe { str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_ref())) })
            }
        }
        impl IntoR for Vec<Option<$owner<str>>> {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                Ok(opt_str_iter_to_strsxp(self.iter().map(|s| s.as_deref())))
            }
            unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
                Ok(unsafe { opt_str_iter_to_strsxp_unchecked(self.iter().map(|s| s.as_deref())) })
            }
        }
    };
}
impl_owned_str_vectors!(Box);
impl_owned_str_vectors!(Rc);
impl_owned_str_vectors!(Arc);
