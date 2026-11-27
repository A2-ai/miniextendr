//! ALTREP vectors read from a slice field: what `#[derive(Altrep)]` builds.
//!
//! `#[derive(Altrep)]` on a struct with one array, `Vec<T>`, `Box<[T]>` or
//! `&'static [T]` field implements [`AltrepSlice`] for it: the struct names the
//! slice and its element type. Everything else comes from the impls here, one
//! per ALTREP trait, chosen by the element's [`AltrepElement::Family`]:
//!
//! | Element | R vector | `Family` |
//! |---|---|---|
//! | `i32`, `f64`, `u8`, `RLogical`, `Rcomplex` | that type; R reads the Rust slice | the type's family |
//! | `#[derive(RNativeType)]` newtype | its inner type's; R reads the Rust slice | the inner type's |
//! | `bool` | logical | [`family::Logical`] |
//! | `String` | character | [`family::Character`] |
//! | `#[derive(RFactor)]` enum | factor (codes, `levels`, `class`) | [`family::Integer`] |
//! | `#[derive(MatchArg)]` enum | character (the choice names) | [`family::Character`] |
//!
//! Each trait has exactly one impl over `T: AltrepSlice`. Rust doesn't tell
//! impls apart by an associated type, so two impls of one trait for
//! different families would conflict; the family-specific traits
//! (`AltInteger`, `AltReal`, ...) are different traits, and the shared ones
//! (`Altrep`, `AltVec`, `InferBase`) dispatch through [`AltrepFamily`].
//!
//! A native element (`AltrepElement::NATIVE`) lets R read the slice in place.
//! R writes through the pointer when the slice is owned; a `&'static [T]`
//! field is copied into an R vector (the class's data2) before R writes, so
//! the borrowed data is never written.

use crate::altrep::RBase;
use crate::altrep_data::{
    AltComplexData, AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltStringData,
    AltrepExtract, AltrepLen, InferBase, Logical,
};
use crate::externalptr::TypedExternal;
use crate::sys::altrep::R_altrep_class_t;
use crate::{RLogical, RNativeType, Rcomplex, SEXP};

// region: The owner and element traits

/// A struct whose ALTREP vector is the elements of one slice field.
///
/// `#[derive(Altrep)]` implements it; there is no reason to write it by hand.
#[doc(hidden)]
pub trait AltrepSlice: TypedExternal + Sized {
    /// The element type; its [`AltrepElement::Family`] picks the R vector type.
    type Element: AltrepElement;

    /// The elements.
    fn altrep_slice(&self) -> &[Self::Element];

    /// The elements, for R to write through when they are native; `None`
    /// for borrowed data, which R gets a copy of instead.
    fn altrep_slice_mut(&mut self) -> Option<&mut [Self::Element]>;
}

/// The R form of one element of a `#[derive(Altrep)]` slice.
///
/// The `RFactor`, `MatchArg` and `RNativeType` derives implement it, and the
/// api does for `i32`, `f64`, `u8`, `RLogical`, `Rcomplex`, `bool` and
/// `String`. It is not meant to be implemented by hand.
///
/// # Safety
///
/// When [`NATIVE`](Self::NATIVE) is `true`, `Self` has the size and alignment
/// of the family's R storage element (`i32`, `f64`, `RLogical`, `u8`,
/// `Rcomplex`) and every bit pattern of it is a valid `Self`: R reads, and
/// for an owned slice writes, a `[Self]` in place.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no R vector form for `#[derive(Altrep)]`",
    label = "no R vector form for this element type",
    note = "an enum element takes `#[derive(RFactor)]` (a factor) or `#[derive(MatchArg)]` (a character vector); a `#[repr(transparent)]` newtype of `i32`, `f64`, `u8`, `RLogical` or `Rcomplex` takes `#[derive(RNativeType)]`",
    note = "for a mapping only you know, implement `Coerce<i32>` (or `Coerce<f64>`, `Coerce<Rboolean>`, `Coerce<u8>`, `Coerce<Rcomplex>`) for the element and derive the matching family (`#[derive(AltrepInteger)]`, ...) instead"
)]
pub unsafe trait AltrepElement: Sized + 'static {
    /// Which R vector the elements make.
    type Family: AltrepFamily;

    /// Whether R can read a `[Self]` in place (see the trait's safety section).
    const NATIVE: bool = false;

    /// The element's R value.
    fn altrep_value(&self) -> <Self::Family as AltrepFamily>::Value<'_>;

    /// A factor element's levels: a preserved `STRSXP`, set as the vector's
    /// `levels` attribute along with `class = "factor"`. On R's main thread.
    fn altrep_levels() -> Option<SEXP> {
        None
    }
}

/// The R vector types an [`AltrepElement`] can make.
#[doc(hidden)]
pub mod family {
    /// An integer vector (`INTSXP`); factors too.
    pub struct Integer;
    /// A double vector (`REALSXP`).
    pub struct Real;
    /// A logical vector (`LGLSXP`).
    pub struct Logical;
    /// A raw vector (`RAWSXP`).
    pub struct Raw;
    /// A complex vector (`CPLXSXP`).
    pub struct Complex;
    /// A character vector (`STRSXP`).
    pub struct Character;
}

mod sealed {
    pub trait Sealed {}
}

/// What each [`family`] contributes to a slice ALTREP class.
#[doc(hidden)]
pub trait AltrepFamily: sealed::Sealed + 'static {
    /// The element's R value: `i32`, `f64`, `Logical`, `u8`, `Rcomplex`, or
    /// `Option<&str>` (`None` is `NA`).
    type Value<'a>;

    /// The R base type.
    const BASE: RBase;

    /// Install the class methods.
    ///
    /// # Safety
    /// `cls` is a class of this family, on R's main thread.
    unsafe fn install<T>(cls: R_altrep_class_t)
    where
        T: AltrepSlice,
        T::Element: AltrepElement<Family = Self>;

    /// The `Dataptr` method.
    ///
    /// # Safety
    /// `x` is a live ALTREP of `T`, on R's main thread.
    unsafe fn dataptr<T>(x: SEXP, writable: bool) -> *mut core::ffi::c_void
    where
        T: AltrepSlice,
        T::Element: AltrepElement<Family = Self>;

    /// The `Dataptr_or_null` method.
    ///
    /// # Safety
    /// As for [`dataptr`](Self::dataptr).
    unsafe fn dataptr_or_null<T>(x: SEXP) -> *const core::ffi::c_void
    where
        T: AltrepSlice,
        T::Element: AltrepElement<Family = Self>;
}

/// The family of `T`'s elements.
type FamilyOf<T> = <<T as AltrepSlice>::Element as AltrepElement>::Family;
// endregion

// region: Families

/// `Dataptr` for a numeric family with R storage `S`: the materialized copy
/// once there is one, else the Rust slice for a native element (owned only,
/// when R asks to write), else a copy R owns from then on.
///
/// # Safety
/// `x` is a live ALTREP of `T`, on R's main thread; `S` is the family's storage.
unsafe fn native_dataptr<T, S>(x: SEXP, writable: bool) -> *mut core::ffi::c_void
where
    T: AltrepSlice,
    S: RNativeType,
{
    unsafe {
        if let Some(data2) = crate::altrep_data::materialized_altrep_data2(x) {
            return crate::sys::DATAPTR_RO(data2).cast_mut();
        }
        if <T::Element as AltrepElement>::NATIVE {
            if !writable {
                // R reads only: the shared borrow serves an owned and a
                // borrowed slice alike.
                return T::altrep_extract_ref(x)
                    .altrep_slice()
                    .as_ptr()
                    .cast_mut()
                    .cast();
            }
            if let Some(values) = T::altrep_extract_mut(x).altrep_slice_mut() {
                return values.as_mut_ptr().cast();
            }
        }
        crate::altrep_data::materialize_altrep_data2::<S>(x)
    }
}

/// `Dataptr_or_null` for a numeric family: a pointer only where reading one
/// costs nothing.
///
/// # Safety
/// As for [`native_dataptr`].
unsafe fn native_dataptr_or_null<T>(x: SEXP) -> *const core::ffi::c_void
where
    T: AltrepSlice,
{
    unsafe {
        if let Some(data2) = crate::altrep_data::materialized_altrep_data2(x) {
            return crate::sys::DATAPTR_RO(data2);
        }
        if <T::Element as AltrepElement>::NATIVE {
            return T::altrep_extract_ref(x).altrep_slice().as_ptr().cast();
        }
        core::ptr::null()
    }
}

/// The elements as R's storage type, when they have its layout.
fn native_slice<T: AltrepSlice, S>(owner: &T) -> Option<&[S]> {
    <T::Element as AltrepElement>::NATIVE.then(|| {
        let values = owner.altrep_slice();
        // SAFETY: a NATIVE element has the layout of the family's storage
        // (the `AltrepElement` contract); callers pass that storage as `S`.
        unsafe { core::slice::from_raw_parts(values.as_ptr().cast::<S>(), values.len()) }
    })
}

macro_rules! numeric_family {
    ($family:ident, $value:ty, $storage:ty, $base:ident, $install:ident) => {
        impl sealed::Sealed for family::$family {}

        impl AltrepFamily for family::$family {
            type Value<'a> = $value;
            const BASE: RBase = RBase::$base;

            unsafe fn install<T>(cls: R_altrep_class_t)
            where
                T: AltrepSlice,
                T::Element: AltrepElement<Family = Self>,
            {
                unsafe {
                    crate::altrep_bridge::install_base::<T>(cls);
                    crate::altrep_bridge::install_vec::<T>(cls);
                    crate::altrep_bridge::$install::<T>(cls);
                }
            }

            unsafe fn dataptr<T>(x: SEXP, writable: bool) -> *mut core::ffi::c_void
            where
                T: AltrepSlice,
                T::Element: AltrepElement<Family = Self>,
            {
                unsafe { native_dataptr::<T, $storage>(x, writable) }
            }

            unsafe fn dataptr_or_null<T>(x: SEXP) -> *const core::ffi::c_void
            where
                T: AltrepSlice,
                T::Element: AltrepElement<Family = Self>,
            {
                unsafe { native_dataptr_or_null::<T>(x) }
            }
        }
    };
}

numeric_family!(Integer, i32, i32, Int, install_int);
numeric_family!(Real, f64, f64, Real, install_real);
numeric_family!(Logical, Logical, RLogical, Logical, install_lgl);
numeric_family!(Raw, u8, u8, Raw, install_raw);
numeric_family!(Complex, Rcomplex, Rcomplex, Complex, install_cplx);

impl sealed::Sealed for family::Character {}

impl AltrepFamily for family::Character {
    type Value<'a> = Option<&'a str>;
    const BASE: RBase = RBase::String;

    unsafe fn install<T>(cls: R_altrep_class_t)
    where
        T: AltrepSlice,
        T::Element: AltrepElement<Family = Self>,
    {
        unsafe {
            crate::altrep_bridge::install_base::<T>(cls);
            crate::altrep_bridge::install_vec::<T>(cls);
            crate::altrep_bridge::install_str::<T>(cls);
        }
    }

    unsafe fn dataptr<T>(x: SEXP, writable: bool) -> *mut core::ffi::c_void
    where
        T: AltrepSlice,
        T::Element: AltrepElement<Family = Self>,
    {
        unsafe { crate::altrep_impl::altstring_dataptr::<T>(x, writable) }
    }

    unsafe fn dataptr_or_null<T>(_x: SEXP) -> *const core::ffi::c_void
    where
        T: AltrepSlice,
        T::Element: AltrepElement<Family = Self>,
    {
        // As for every bridge string class: R reads through Elt.
        core::ptr::null()
    }
}
// endregion

// region: Elements the api knows

macro_rules! native_element {
    ($ty:ty, $family:ident, |$value:ident| $r_value:expr) => {
        // SAFETY: the family's own storage type.
        unsafe impl AltrepElement for $ty {
            type Family = family::$family;
            const NATIVE: bool = true;

            #[inline]
            fn altrep_value(&self) -> <Self::Family as AltrepFamily>::Value<'_> {
                let $value = *self;
                $r_value
            }
        }
    };
}

native_element!(i32, Integer, |v| v);
native_element!(f64, Real, |v| v);
native_element!(u8, Raw, |v| v);
native_element!(Rcomplex, Complex, |v| v);
native_element!(RLogical, Logical, |v| Logical::from(v));

// SAFETY: not NATIVE.
unsafe impl AltrepElement for bool {
    type Family = family::Logical;

    #[inline]
    fn altrep_value(&self) -> Logical {
        Logical::from_bool(*self)
    }
}

// SAFETY: not NATIVE.
unsafe impl AltrepElement for String {
    type Family = family::Character;

    #[inline]
    fn altrep_value(&self) -> Option<&str> {
        Some(self.as_str())
    }
}
// endregion

// region: Data traits

impl<T: AltrepSlice> AltrepLen for T {
    fn altrep_len(&self) -> usize {
        self.altrep_slice().len()
    }
}

impl<T> AltIntegerData for T
where
    T: AltrepSlice,
    T::Element: AltrepElement<Family = family::Integer>,
{
    fn elt(&self, i: usize) -> i32 {
        self.altrep_slice()[i].altrep_value()
    }

    fn as_slice(&self) -> Option<&[i32]> {
        native_slice(self)
    }
}

impl<T> AltRealData for T
where
    T: AltrepSlice,
    T::Element: AltrepElement<Family = family::Real>,
{
    fn elt(&self, i: usize) -> f64 {
        self.altrep_slice()[i].altrep_value()
    }

    fn as_slice(&self) -> Option<&[f64]> {
        native_slice(self)
    }
}

impl<T> AltLogicalData for T
where
    T: AltrepSlice,
    T::Element: AltrepElement<Family = family::Logical>,
{
    fn elt(&self, i: usize) -> Logical {
        self.altrep_slice()[i].altrep_value()
    }

    fn as_r_slice(&self) -> Option<&[i32]> {
        // `RLogical` is transparent over `i32` and takes every bit pattern.
        native_slice(self)
    }
}

impl<T> AltRawData for T
where
    T: AltrepSlice,
    T::Element: AltrepElement<Family = family::Raw>,
{
    fn elt(&self, i: usize) -> u8 {
        self.altrep_slice()[i].altrep_value()
    }

    fn as_slice(&self) -> Option<&[u8]> {
        native_slice(self)
    }
}

impl<T> AltComplexData for T
where
    T: AltrepSlice,
    T::Element: AltrepElement<Family = family::Complex>,
{
    fn elt(&self, i: usize) -> Rcomplex {
        self.altrep_slice()[i].altrep_value()
    }

    fn as_slice(&self) -> Option<&[Rcomplex]> {
        native_slice(self)
    }
}

impl<T> AltStringData for T
where
    T: AltrepSlice,
    T::Element: AltrepElement<Family = family::Character>,
{
    fn elt(&self, i: usize) -> Option<&str> {
        self.altrep_slice()[i].altrep_value()
    }
}
// endregion

// region: Low-level traits

crate::__impl_altrep_base!({T} T {T: AltrepSlice}, RUnwind);

impl<T: AltrepSlice> crate::altrep_traits::AltVec for T {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: SEXP, writable: bool) -> *mut core::ffi::c_void {
        unsafe { <FamilyOf<T> as AltrepFamily>::dataptr::<T>(x, writable) }
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        unsafe { <FamilyOf<T> as AltrepFamily>::dataptr_or_null::<T>(x) }
    }
}

crate::__impl_altinteger_methods!({T} T {T: AltrepSlice, <T as AltrepSlice>::Element: AltrepElement<Family = family::Integer>});
crate::__impl_altreal_methods!({T} T {T: AltrepSlice, <T as AltrepSlice>::Element: AltrepElement<Family = family::Real>});
crate::__impl_altlogical_methods!({T} T {T: AltrepSlice, <T as AltrepSlice>::Element: AltrepElement<Family = family::Logical>});
crate::__impl_altraw_methods!({T} T {T: AltrepSlice, <T as AltrepSlice>::Element: AltrepElement<Family = family::Raw>});
crate::__impl_altcomplex_methods!({T} T {T: AltrepSlice, <T as AltrepSlice>::Element: AltrepElement<Family = family::Complex>});
crate::__impl_altstring_methods!({T} T {T: AltrepSlice, <T as AltrepSlice>::Element: AltrepElement<Family = family::Character>});

impl<T: AltrepSlice> InferBase for T {
    const BASE: RBase = <FamilyOf<T> as AltrepFamily>::BASE;

    unsafe fn make_class(
        class_name: *const core::ffi::c_char,
        pkg_name: *const core::ffi::c_char,
    ) -> R_altrep_class_t {
        unsafe { crate::altrep::make_class_by_base(class_name, pkg_name, Self::BASE) }
    }

    unsafe fn install_methods(cls: R_altrep_class_t) {
        unsafe { <FamilyOf<T> as AltrepFamily>::install::<T>(cls) }
    }
}
// endregion

// region: Conversion

/// Give a new slice ALTREP its element's attributes: `levels` and
/// `class = "factor"` for a factor element, nothing otherwise.
///
/// # Safety
/// `x` is a new ALTREP of `T`, on R's main thread. It is protected here.
#[doc(hidden)]
pub unsafe fn set_slice_altrep_attributes<T: AltrepSlice>(x: SEXP) {
    use crate::SexpExt;

    // Protected before the levels are read: the first read builds the levels
    // vector, which allocates. Setting an attribute allocates its cell too.
    // The levels and the class vector are preserved.
    let x = unsafe { crate::OwnedProtect::new(x) };
    if let Some(levels) = <T::Element as AltrepElement>::altrep_levels() {
        x.get().set_levels(levels);
        x.get().set_class(crate::factor::factor_class_sexp());
    }
}

/// The elements as the family's R storage `S`, when `E` is `S`: the
/// zero-copy view the family derives use for an element of the family's
/// own type (an `i32` element of `#[derive(AltrepInteger)]`).
#[doc(hidden)]
#[inline]
pub fn native_view<E: 'static, S: 'static>(values: &[E]) -> Option<&[S]> {
    (core::any::TypeId::of::<E>() == core::any::TypeId::of::<S>()).then(|| {
        // SAFETY: `E` and `S` are the same type.
        unsafe { core::slice::from_raw_parts(values.as_ptr().cast::<S>(), values.len()) }
    })
}

/// The mutable form of [`native_view`].
#[doc(hidden)]
#[inline]
pub fn native_view_mut<E: 'static, S: 'static>(values: &mut [E]) -> Option<&mut [S]> {
    (core::any::TypeId::of::<E>() == core::any::TypeId::of::<S>()).then(|| {
        // SAFETY: `E` and `S` are the same type.
        unsafe { core::slice::from_raw_parts_mut(values.as_mut_ptr().cast::<S>(), values.len()) }
    })
}
// endregion

#[cfg(test)]
mod tests;
