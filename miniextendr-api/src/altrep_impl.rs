//! ALTREP implementation utilities.
//!
//! This module provides helper functions used by the declarative macros
//! generated for ALTREP classes.  The proc-macro derives (`#[derive(AltrepInteger)]`
//! etc.) lower to invocations of `impl_alt*_from_data!` macros which expand
//! to code that references items in this module via `$crate::altrep_impl::*`
//! paths.
//!
//! Use `crate::altrep_data1_as` (re-exported from externalptr) to extract
//! data from an ALTREP's data1 slot.
//!
//! ## Layout
//!
//! - [`macros`] — `#[macro_export]` declarative macros (`impl_alt*_from_data!`
//!   and their `__impl_*` helpers).
//! - [`builtins`] — crate-private meta-macro + invocations for `Vec<T>` /
//!   `Box<[T]>` / `Cow<T>` / `Range<T>` ALTREP families.
//! - [`arrays`] — const-generic `[T; N]` ALTREP impls.
//! - [`static_slices`] — hand-written `&'static [T]` ALTREP impls.

// region: Checked string-to-CHARSXP helper

/// Create a CHARSXP from a Rust string, with checked length conversion.
///
/// # Safety
///
/// Must be called from R's main thread.
///
/// # Panics
///
/// Panics if `s.len() > i32::MAX`.
#[inline]
pub unsafe fn checked_mkchar(s: &str) -> crate::SEXP {
    let _len = i32::try_from(s.len()).unwrap_or_else(|_| {
        panic!(
            "string length {} exceeds i32::MAX for Rf_mkCharLenCE",
            s.len()
        )
    });
    crate::SEXP::charsxp(s)
}
// endregion

// region: String ALTREP materialization

/// Publish a complete STRSXP as a bridge-generated string ALTREP's data2.
///
/// Element reads stay lazy until a pointer or setter needs writable storage.
/// A partial buffer is never published: NA cannot double as a cache-miss marker,
/// because R can write NA after materialization.
///
/// # Safety
///
/// `x` must be a rooted, live ALTSTRING using the bridge's data2 convention.
/// Must run on R's main thread inside the class's unwind guard.
#[doc(hidden)]
pub unsafe fn materialize_altstring_data2<T: crate::altrep_traits::AltString>(
    x: crate::SEXP,
) -> crate::SEXP {
    use crate::SexpExt;
    use crate::altrep_ext::AltrepSexpExt;

    if let Some(data2) = unsafe { crate::altrep_data::materialized_altrep_data2(x) } {
        return data2;
    }

    let n = T::length(x);
    let buffer = unsafe {
        crate::OwnedProtect::new(crate::sys::Rf_allocVector_unchecked(
            crate::SEXPTYPE::STRSXP,
            n,
        ))
    };
    // R initializes STRSXP slots to R_BlankString, so the protected vector is
    // safe for GC while later CHARSXPs are allocated. Keep data2 absent until done.
    for i in 0..n {
        let value = T::elt(x, i);
        buffer.get().set_string_elt(i, value);
    }
    unsafe { AltrepSexpExt::set_altrep_data2(&x, buffer.get()) };
    buffer.get()
}
// endregion

// region: Centralized ALTREP buffer access helper

/// Create a mutable slice from an ALTREP `get_region` output buffer pointer.
///
/// Called by the bridge trampolines (`altrep_bridge.rs`) to convert the raw
/// `*mut T` buffer from R's ALTREP dispatch into a `&mut [T]` before passing
/// it to the trait methods.
///
/// # Safety
///
/// - `buf` must be a valid, aligned, writable pointer to at least `len` elements of `T`.
/// - The caller must ensure no aliasing references to the same memory exist.
/// - This is guaranteed when called from R's ALTREP `Get_region` dispatch, which
///   provides a freshly allocated buffer.
#[inline]
pub unsafe fn altrep_region_buf<T>(buf: *mut T, len: usize) -> &'static mut [T] {
    unsafe { std::slice::from_raw_parts_mut(buf, len) }
}
// endregion

// `macros` is declared first so its `#[macro_export]` items are textually
// in-scope before the `builtins`/`arrays`/`static_slices` modules that invoke
// them via `$crate::__impl_alt_*` paths.
pub mod macros;

pub mod arrays;
pub mod builtins;
pub mod static_slices;
