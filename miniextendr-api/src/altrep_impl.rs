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

// region: String ALTREP element cache

/// Return a bridge string ALTREP's data2 element cache, allocating it on first use.
///
/// Slots not computed yet hold C `NULL`, as in R's own deferred string class
/// (`altclasses.c`). `NA_character_` is therefore an ordinary cached value, so
/// an NA that R writes through `Set_elt` or `Dataptr` stays NA. Every CHARSXP
/// handed to R stays reachable from `x`, as R code expects of `STRING_ELT`.
///
/// # Safety
///
/// `x` must be a live ALTSTRING using the bridge's data2 convention, on R's
/// main thread.
unsafe fn altstring_cache(x: crate::SEXP, n: crate::R_xlen_t) -> crate::SEXP {
    use crate::SexpExt;
    use crate::altrep_ext::AltrepSexpExt;

    let data2 = unsafe { AltrepSexpExt::altrep_data2_raw(&x) };
    if !data2.is_null_ptr() && data2.type_of() == crate::SEXPTYPE::STRSXP {
        return data2;
    }
    // Inside ALTREP dispatch — _unchecked variants skip the redundant
    // with_r_thread check/route (MXL301 permits).
    unsafe {
        let cache = crate::sys::Rf_protect_unchecked(crate::sys::Rf_allocVector_unchecked(
            crate::SEXPTYPE::STRSXP,
            n,
        ));
        if n > 0 {
            // A fresh STRSXP holds only R_BlankString, so clearing its slots
            // drops no reference the write barrier tracks.
            let slots = crate::sys::DATAPTR_RO_unchecked(cache)
                .cast_mut()
                .cast::<crate::SEXP>();
            let len = usize::try_from(n).expect("R vector length is non-negative");
            std::ptr::write_bytes(slots, 0, len);
        }
        AltrepSexpExt::set_altrep_data2(&x, cache);
        crate::sys::Rf_unprotect_unchecked(1);
        cache
    }
}

/// Read cache slot `i`, computing it from the Rust data on first use.
///
/// # Safety
///
/// As for [`altstring_cache`]; `cache` is `x`'s cache and `0 <= i < length`.
unsafe fn altstring_cached_elt<T>(
    x: crate::SEXP,
    cache: crate::SEXP,
    i: crate::R_xlen_t,
) -> crate::SEXP
where
    T: crate::altrep_data::AltrepExtract + crate::altrep_data::AltStringData + 'static,
{
    use crate::SexpExt;

    let cached = cache.string_elt(i);
    if !cached.is_null_ptr() {
        return cached;
    }
    let data = unsafe { T::altrep_extract_ref(x) };
    let index = usize::try_from(i).expect("ALTREP element index is non-negative");
    let value = match data.elt(index) {
        Some(s) => unsafe { checked_mkchar(s) },
        None => crate::SEXP::na_string(),
    };
    cache.set_string_elt(i, value);
    value
}

/// `Elt` for bridge-generated string ALTREPs: each element is computed once.
///
/// # Safety
///
/// `x` must be a live ALTSTRING using the bridge's data2 convention, on R's
/// main thread.
#[doc(hidden)]
pub unsafe fn altstring_elt<T>(x: crate::SEXP, i: crate::R_xlen_t) -> crate::SEXP
where
    T: crate::altrep_traits::Altrep
        + crate::altrep_data::AltrepExtract
        + crate::altrep_data::AltStringData
        + 'static,
{
    // Clamp before touching data2: it is a plain STRSXP, so STRING_ELT would
    // error on a negative index instead of using the clamped one (#1190).
    let i = i.max(0);
    unsafe {
        let cache = altstring_cache(x, T::length(x));
        altstring_cached_elt::<T>(x, cache, i)
    }
}

/// Fill every cache slot and return the cache, for `Dataptr`.
///
/// A panic part-way leaves the filled slots cached; a retry fills the rest.
///
/// # Safety
///
/// As for [`altstring_elt`].
#[doc(hidden)]
pub unsafe fn altstring_fill<T>(x: crate::SEXP) -> crate::SEXP
where
    T: crate::altrep_traits::Altrep
        + crate::altrep_data::AltrepExtract
        + crate::altrep_data::AltStringData
        + 'static,
{
    unsafe {
        let n = T::length(x);
        let cache = altstring_cache(x, n);
        for i in 0..n {
            altstring_cached_elt::<T>(x, cache, i);
        }
        cache
    }
}

/// `Set_elt` for bridge-generated string ALTREPs; R's default raises an error.
///
/// # Safety
///
/// As for [`altstring_elt`]; `v` is a CHARSXP and `0 <= i < length`.
#[doc(hidden)]
pub unsafe fn altstring_set_elt<T>(x: crate::SEXP, i: crate::R_xlen_t, v: crate::SEXP)
where
    T: crate::altrep_traits::Altrep,
{
    use crate::SexpExt;

    unsafe {
        // R disables GC around Set_elt, but a direct caller may not: keep `v`
        // live across the cache allocation.
        let v = crate::OwnedProtect::new(v);
        let cache = altstring_cache(x, T::length(x));
        cache.set_string_elt(i, v.get());
    }
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
