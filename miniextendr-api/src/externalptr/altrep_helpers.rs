//! ALTREP helpers for `ExternalPtr` — data1/data2 slot access.
//!
//! Convenience functions for ALTREP implementations that store their data
//! in `ExternalPtr` slots. Also provides the `Sidecar` marker type for
//! `#[r_data]` fields.

use super::{ErasedExternalPtr, ExternalPtr, TypedExternal};
use crate::SEXP;

/// Extract the ALTREP data1 slot as a typed `ExternalPtr<T>`.
///
/// This is a convenience function for ALTREP implementations that store
/// their data in an `ExternalPtr` in the data1 slot.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
///
/// # Example
///
/// ```ignore
/// impl Altrep for MyAltrepClass {
///     const HAS_LENGTH: bool = true;
///     fn length(x: SEXP) -> R_xlen_t {
///         match unsafe { altrep_data1_as::<MyData>(x) } {
///             Some(ext) => ext.data.len() as R_xlen_t,
///             None => 0,
///         }
///     }
/// }
/// ```
#[inline]
pub unsafe fn altrep_data1_as<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp(SEXP::altrep_data1_raw(x)) }
}

/// Extract the ALTREP data1 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
#[inline]
pub unsafe fn altrep_data1_as_unchecked<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp_unchecked(SEXP::altrep_data1_raw_unchecked(x)) }
}

/// Extract the ALTREP data2 slot as a typed `ExternalPtr<T>`.
///
/// Similar to `altrep_data1_as`, but for the data2 slot.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
#[inline]
pub unsafe fn altrep_data2_as<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp(x.altrep_data2_raw()) }
}

/// Extract the ALTREP data2 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
#[inline]
pub unsafe fn altrep_data2_as_unchecked<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp_unchecked(x.altrep_data2_raw_unchecked()) }
}

/// Get a mutable reference to data in ALTREP data1 slot via `ErasedExternalPtr`.
///
/// This is useful for ALTREP methods that need to mutate the underlying data.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
/// - The caller must ensure no other references to the data exist
///
/// # Example
///
/// ```ignore
/// fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
///     match unsafe { altrep_data1_mut::<MyData>(x) } {
///         Some(data) => data.buffer.as_mut_ptr().cast(),
///         None => core::ptr::null_mut(),
///     }
/// }
/// ```
#[inline]
pub unsafe fn altrep_data1_mut<T: TypedExternal>(x: SEXP) -> Option<&'static mut T> {
    unsafe {
        let mut erased = ErasedExternalPtr::from_sexp(SEXP::altrep_data1_raw(x));
        // Transmute the lifetime to 'static - this is safe because:
        // 1. The ExternalPtr is protected by R's GC as part of the ALTREP object
        // 2. The ALTREP object `x` is kept alive by R during the callback
        erased.downcast_mut::<T>().map(|r| std::mem::transmute(r))
    }
}

/// Get a mutable reference to data in ALTREP data1 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
/// - The caller must ensure no other references to the data exist
#[inline]
pub unsafe fn altrep_data1_mut_unchecked<T: TypedExternal>(x: SEXP) -> Option<&'static mut T> {
    unsafe {
        let mut erased = ErasedExternalPtr::from_sexp(SEXP::altrep_data1_raw_unchecked(x));
        erased.downcast_mut::<T>().map(|r| std::mem::transmute(r))
    }
}

// Tests for ExternalPtr require R runtime, so they are in rpkg/src/rust/lib.rs
// endregion

// region: Sidecar Marker Type for #[r_data] Fields

/// Marker type for enabling R sidecar accessors in an `ExternalPtr` struct.
///
/// When used with `#[derive(ExternalPtr)]` and `#[r_data]`, this field acts as
/// a selector that enables R-facing accessors for sibling `#[r_data]` fields.
///
/// # Supported Field Types
///
/// - **[`Sidecar<T>`](super::Sidecar)** - a typed value kept in the external
///   pointer's protection list (not in the struct), so the pointer roots it
///   and `saveRDS()` writes it; Rust reads and writes it through the
///   accessors the derive generates
/// - **`i32`, `f64`, `bool`, `u8`** - scalars read with `Rf_as*` on write and
///   returned as a fresh length-1 vector on read
/// - **Any `IntoR + TryFromSexp` type** - converted on every read and write
///   (e.g., `String`, `Vec<T>`)
/// - **[`Computed`]** - a read-only field computed from the Rust value by a
///   getter function (`#[r_data(get = "Self::f")]`)
///
/// Every field but a `Sidecar` lives in the Rust struct, so a reader always
/// sees the Rust value and a writer either converts into it or gets an error.
/// Those values sit behind the pointer's address, which `saveRDS` does not
/// write. `Sidecar` values travel with the pointer.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::externalptr::{RSidecar, Sidecar};
///
/// #[derive(ExternalPtr)]
/// pub struct MyType {
///     pub x: i32,
///
///     /// Selector field - enables R wrapper generation
///     #[r_data]
///     r: RSidecar,
///
///     /// Rooted value - MyType_get_keys() / MyType_set_keys(), and
///     /// `MyType::keys(&ptr)` / `MyType::set_keys(&mut ptr, v)` in Rust
///     #[r_data]
///     pub keys: Sidecar<Vec<i32>>,
///
///     /// Zero-overhead scalar - MyType_get_count() / MyType_set_count()
///     #[r_data]
///     pub count: i32,
///
///     /// Conversion type - MyType_get_name() / MyType_set_name()
///     #[r_data]
///     pub name: String,
/// }
/// ```
///
/// # Design Notes
///
/// - `RSidecar` is a ZST (zero-sized type) - no runtime cost
/// - Only `pub` `#[r_data]` fields get R wrapper functions generated
/// - Multiple `RSidecar` fields in one struct is a compile error
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RSidecar;

/// Marker type of a read-only R field computed from the Rust value
/// (#1883).
///
/// Declare it as `#[r_data(get = "Self::f")] pub f: Computed` on a
/// `#[derive(ExternalPtr)]` struct with an [`RSidecar`] selector. `get`
/// names a plain Rust function `fn(&Self) -> T` with `T: IntoR`, resolved in
/// the struct's module (a leading `Self` is the struct); it needs no
/// `#[miniextendr]`. The field holds no value: it is a zero-sized marker that
/// gives the computed field its place among the type's R fields, so every
/// struct literal writes `f: Computed` once, as it writes `_r: RSidecar`.
///
/// ```ignore
/// #[derive(ExternalPtr)]
/// #[externalptr(s3)]
/// pub struct Engine {
///     inner: Model,
///     #[r_data] _r: RSidecar,
///     #[r_data(get = "Self::group_by")] pub group_by: Computed,
///     #[r_data(name = "keys")] pub r_keys: Sidecar<SEXP>,
///     #[r_data(get = "Self::n_rows")] pub n_rows: Computed,
/// }
///
/// impl Engine {
///     fn group_by(&self) -> Vec<String> { self.inner.group_by.clone() }
///     fn n_rows(&self) -> i32 { self.inner.n_rows() }
/// }
/// ```
///
/// What the derive generates for it:
///
/// - the standalone getter `Engine_get_group_by(x)`, and an arm in the
///   `$` / `[[` field methods of `s3(r_data_accessors)` & co., at the field's
///   declared place among the `Sidecar` and struct fields;
/// - no setter: no `Engine_set_group_by()`, and under the get/set form an
///   assignment (`x$group_by <- v`) raises `miniextendr_read_only_field`.
///
/// The getter runs on R's main thread with the live Rust value, so a
/// computed field refuses a pointer restored from a saved session like a
/// struct field does. Its errors are its own: a `rust_error!(class = ...)`
/// in the getter reaches R with that class. The getter returns a value, not a
/// `Result`; it takes `&Self`, so it can't read `Sidecar` values (#1856),
/// which are R fields already.
///
/// Only a `pub` `Computed` field reaches R, and only on S3, S4, env and
/// vctrs types: R6 active bindings and S7 properties need a setter (#1888).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Computed;
// endregion
