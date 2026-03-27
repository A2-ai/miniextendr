//! ALTREP marker type (Altrep<T> / Lazy<T>).

use crate::into_r::IntoR;

/// Marker type to opt-in to ALTREP representation for types that have both
/// eager-copy and ALTREP implementations.
///
/// # Motivation
///
/// Types like `Vec<i32>` have two possible conversions to R:
/// 1. **Eager copy** (default): copies all data to R immediately
/// 2. **ALTREP**: keeps data in Rust, provides it on-demand to R
///
/// The default `IntoR` for `Vec<i32>` does eager copy. To get ALTREP behavior,
/// wrap your value in `Altrep<T>`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, Altrep};
///
/// // Returns an ALTREP-backed integer vector (data stays in Rust)
/// #[miniextendr]
/// fn altrep_vec() -> Altrep<Vec<i32>> {
///     Altrep((0..1_000_000).collect())
/// }
///
/// // Returns a regular R vector (data copied to R)
/// #[miniextendr]
/// fn regular_vec() -> Vec<i32> {
///     (0..1_000_000).collect()
/// }
/// ```
///
/// # Supported Types
///
/// `Altrep<T>` works with any type that implements both:
/// - [`RegisterAltrep`](crate::altrep::RegisterAltrep) - for ALTREP class registration
/// - [`TypedExternal`](crate::externalptr::TypedExternal) - for wrapping in ExternalPtr
///
/// Built-in supported types:
/// - `Vec<i32>`, `Vec<f64>`, `Vec<bool>`, `Vec<u8>`, `Vec<String>`
/// - `Box<[i32]>`, `Box<[f64]>`, `Box<[bool]>`, `Box<[u8]>`, `Box<[String]>`
/// - `Range<i32>`, `Range<i64>`, `Range<f64>`
///
/// Opt-in lazy materialization via ALTREP.
///
/// Wrapping a return type in `Lazy<T>` causes it to be returned as an
/// ALTREP vector backed by Rust-owned memory. R reads elements on demand;
/// full materialization only happens if R needs a contiguous pointer.
///
/// # When to use
/// - Large vectors (>1000 elements)
/// - Data R may only partially read
/// - Computed/external data (Arrow, ndarray, nalgebra)
///
/// # When NOT to use
/// - Small vectors (<100 elements, ALTREP overhead dominates)
/// - Data R will immediately modify (triggers instant materialization)
///
/// # Example
/// ```rust,ignore
/// #[miniextendr]
/// fn big_result() -> Lazy<Vec<f64>> {
///     Lazy(vec![0.0; 1_000_000])
/// }
/// ```
pub type Lazy<T> = Altrep<T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Altrep<T>(pub T);

impl<T> Altrep<T> {
    /// Create a new ALTREP marker wrapper.
    #[inline]
    pub fn new(value: T) -> Self {
        Altrep(value)
    }

    /// Unwrap and return the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Convert to R ALTREP and wrap in [`AltrepSexp`](crate::altrep_sexp::AltrepSexp) (`!Send + !Sync`).
    ///
    /// This creates the ALTREP SEXP and wraps it in an `AltrepSexp` that
    /// prevents the result from being sent to non-R threads. Use this when
    /// you need to keep the ALTREP vector in Rust code and want compile-time
    /// thread safety guarantees.
    ///
    /// For returning directly to R from `#[miniextendr]` functions, use
    /// `Altrep<T>` as the return type (which implements `IntoR`) or call
    /// `.into_sexp()` / `.into_sexp_altrep()` instead.
    pub fn into_altrep_sexp(self) -> crate::altrep_sexp::AltrepSexp
    where
        T: crate::altrep::RegisterAltrep + crate::externalptr::TypedExternal,
    {
        let sexp = self.into_sexp();
        // Safety: we just created an ALTREP SEXP via R_new_altrep
        unsafe { crate::altrep_sexp::AltrepSexp::from_raw(sexp) }
    }
}

impl<T> From<T> for Altrep<T> {
    #[inline]
    fn from(value: T) -> Self {
        Altrep(value)
    }
}

impl<T> std::ops::Deref for Altrep<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Altrep<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Convert `Altrep<T>` to R using ALTREP representation.
///
/// This creates an ALTREP object where the data stays in Rust and is
/// provided to R on-demand through ALTREP callbacks.
impl<T> IntoR for Altrep<T>
where
    T: crate::altrep::RegisterAltrep + crate::externalptr::TypedExternal,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        let cls = <T as crate::altrep::RegisterAltrep>::get_or_init_class();
        let ext_ptr = crate::externalptr::ExternalPtr::new(self.0);
        let data1 = ext_ptr.as_sexp();
        // Protect data1 across R_new_altrep — it may allocate and trigger GC.
        unsafe {
            crate::ffi::Rf_protect_unchecked(data1);
            let out = crate::ffi::altrep::R_new_altrep(cls, data1, crate::ffi::SEXP::null());
            crate::ffi::Rf_unprotect_unchecked(1);
            out
        }
    }
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let cls = <T as crate::altrep::RegisterAltrep>::get_or_init_class();
        let ext_ptr = crate::externalptr::ExternalPtr::new(self.0);
        let data1 = ext_ptr.as_sexp();
        unsafe {
            crate::ffi::Rf_protect_unchecked(data1);
            let out =
                crate::ffi::altrep::R_new_altrep_unchecked(cls, data1, crate::ffi::SEXP::null());
            crate::ffi::Rf_unprotect_unchecked(1);
            out
        }
    }
}

/// Convert `AltrepSexp` to R by returning the inner SEXP.
///
/// This allows `AltrepSexp` to be used as a return type from `#[miniextendr]`
/// functions, transparently passing the ALTREP SEXP back to R.
impl IntoR for crate::altrep_sexp::AltrepSexp {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        // Safety: returning to R which is always the main thread context
        unsafe { self.as_raw() }
    }
}
// endregion
