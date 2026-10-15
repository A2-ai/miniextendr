//! Concrete argument types inferred from an inherited trait method.
//!
//! Metadata describes the R-facing shape, but a copied Rust path may name a
//! private declaration-local alias. This zero-sized witness gets its type from
//! the actual method item instead. Conversion errors remain concrete at the
//! generated call site, preserving the condition-class probes.

use crate::from_r::NativeBorrow;
use crate::gc_protect::Root;
use crate::{SEXP, TryFromRoot, TryFromSexp};
use core::marker::PhantomData;

/// An inferred method argument type, used only by generated wrappers.
#[doc(hidden)]
pub struct Argument<T>(PhantomData<fn() -> T>);

impl<T> Argument<T> {
    /// Construct a type witness without constructing an argument.
    pub const NEW: Self = Self(PhantomData);

    /// Convert through the inferred type's ordinary converter.
    pub fn convert(&self, input: SEXP) -> Result<T, T::Error>
    where
        T: TryFromSexp,
    {
        T::try_from_sexp(input)
    }

    /// Convert a borrow whose lifetime is tied to the wrapper's root.
    pub fn rooted<'a>(&self, input: Root<'a>) -> Result<T, T::Error>
    where
        T: TryFromRoot<'a>,
    {
        T::try_from_root(input)
    }

    /// The ordinary converter's alias-preflight metadata.
    pub fn native_borrow(&self) -> Option<NativeBorrow>
    where
        T: TryFromSexp,
    {
        T::NATIVE_BORROW
    }

    /// The rooted converter's alias-preflight metadata.
    pub fn rooted_native_borrow<'a>(&self) -> Option<NativeBorrow>
    where
        T: TryFromRoot<'a>,
    {
        T::NATIVE_BORROW
    }

    /// The ordinary converter's R-facing diagnostic expectation.
    pub fn expectation(&self) -> Option<&'static str>
    where
        T: TryFromSexp,
    {
        T::__MX_EXPECTATION
    }
}
