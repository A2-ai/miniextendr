//! Wrapper helpers to force specific `IntoR` representations.

use crate::externalptr::{ExternalPtr, IntoExternalPtr};
use crate::ffi::RNativeType;
use crate::into_r::IntoR;
use crate::list::IntoList;

/// Wrap a value and convert it to an R list via `IntoList` when returned from Rust.
#[derive(Debug, Clone, Copy)]
pub struct AsList<T: IntoList>(pub T);

impl<T: IntoList> From<T> for AsList<T> {
    fn from(value: T) -> Self {
        AsList(value)
    }
}

impl<T: IntoList> IntoR for AsList<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.0.into_list().into_sexp()
    }
}

/// Wrap a value and convert it to an external pointer when returned from Rust.
#[derive(Debug, Clone, Copy)]
pub struct AsExternalPtr<T: IntoExternalPtr>(pub T);

impl<T: IntoExternalPtr> From<T> for AsExternalPtr<T> {
    fn from(value: T) -> Self {
        AsExternalPtr(value)
    }
}

impl<T: IntoExternalPtr> IntoR for AsExternalPtr<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        ExternalPtr::new(self.0).into_sexp()
    }
}

/// Wrap a scalar `RNativeType` and force native vector/scalar conversion,
/// even if the type also implements `IntoExternalPtr` or other `IntoR` paths.
#[derive(Debug, Clone, Copy)]
pub struct AsRNative<T: RNativeType>(pub T);

impl<T: RNativeType> From<T> for AsRNative<T> {
    fn from(value: T) -> Self {
        AsRNative(value)
    }
}

impl<T: RNativeType> IntoR for AsRNative<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        // TODO: this needs to be done differently.. maybe array?
        // Convert via a length-1 vector to avoid picking up other `IntoR` impls
        vec![self.0].into_sexp()
    }
}
