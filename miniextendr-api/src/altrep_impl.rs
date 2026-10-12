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

/// Clone a derive-generated ALTREP owner while preserving its current R cache.
/// R's default DuplicateEX copies attributes and object flags after this hook.
///
/// # Safety
/// `x` must be a rooted ALTREP of `T` with ExternalPtr data1, on R's thread.
/// `T::clone` must detach any allocation that this class exposes for R writes.
#[doc(hidden)]
pub unsafe fn duplicate_altrep_owner<T>(x: crate::SEXP) -> crate::SEXP
where
    T: crate::externalptr::TypedExternal + crate::altrep::RegisterAltrep + Clone,
{
    unsafe {
        let original =
            crate::altrep_data1_as_unchecked::<T>(x).expect("ALTREP owner type mismatch");
        // Clone the handle as well as T so sidecar slots are preserved.
        let owner = original.clone();
        let data1 = crate::OwnedProtect::new(owner.as_sexp());
        let class = T::get_or_init_class();
        let out =
            crate::OwnedProtect::new(class.new_altrep_unchecked(data1.get(), crate::SEXP::nil()));
        if let Some(cache) = crate::altrep_data::materialized_altrep_data2(x) {
            out.get().set_altrep_data2(duplicate_altrep_data2(cache));
        }
        out.get()
    }
}

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
    unsafe {
        let cache = new_altstring_cache(n);
        // Attaching the cache allocates nothing, so it needs no protection.
        AltrepSexpExt::set_altrep_data2(&x, cache);
        cache
    }
}

/// Allocate a string cache of length `n` with every slot empty (C `NULL`).
///
/// # Safety
///
/// On R's main thread. The result is unprotected.
unsafe fn new_altstring_cache(n: crate::R_xlen_t) -> crate::SEXP {
    // Inside ALTREP dispatch — _unchecked variants skip the redundant
    // with_r_thread check/route (MXL301 permits).
    unsafe {
        let cache = crate::sys::Rf_allocVector_unchecked(crate::SEXPTYPE::STRSXP, n);
        if n > 0 {
            // A fresh STRSXP holds only R_BlankString, so clearing its slots
            // drops no reference the write barrier tracks.
            let slots = crate::sys::DATAPTR_RO_unchecked(cache)
                .cast_mut()
                .cast::<crate::SEXP>();
            let len = usize::try_from(n).expect("R vector length is non-negative");
            std::ptr::write_bytes(slots, 0, len);
        }
        cache
    }
}

/// Copy a bridge ALTREP's data2 for a `Duplicate` callback.
///
/// A string cache can hold C `NULL` slots, which `Rf_duplicate` can't copy:
/// it stores every slot with `SET_STRING_ELT`. Its copy keeps those slots
/// empty, so the new vector computes them from its own Rust data, and keeps
/// the written mark. Any other data2 (a materialized vector) goes through
/// `Rf_duplicate`.
///
/// # Safety
///
/// `data2` is a live ALTREP's data2, kept reachable by the caller, on R's
/// main thread. The result is unprotected.
#[doc(hidden)]
pub unsafe fn duplicate_altrep_data2(data2: crate::SEXP) -> crate::SEXP {
    use crate::SexpExt;

    if data2.type_of() != crate::SEXPTYPE::STRSXP {
        return unsafe { crate::sys::Rf_duplicate_unchecked(data2) };
    }
    unsafe {
        let n = data2.xlength();
        let copy = crate::OwnedProtect::new(new_altstring_cache(n));
        for i in 0..n {
            let value = data2.string_elt(i);
            if !value.is_null_ptr() {
                copy.get().set_string_elt(i, value);
            }
        }
        if altstring_cache_written(data2) {
            mark_altstring_written(copy.get());
        }
        copy.get()
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

/// `Dataptr` for bridge-generated string ALTREPs: fill every cache slot and
/// return the cache's pointer. A writable request marks the cache written
/// first (see [`altstring_cache_written`]), since R writes through it.
///
/// A panic part-way leaves the filled slots cached; a retry fills the rest.
///
/// # Safety
///
/// As for [`altstring_elt`].
#[doc(hidden)]
pub unsafe fn altstring_dataptr<T>(x: crate::SEXP, writable: bool) -> *mut core::ffi::c_void
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
        if writable {
            mark_altstring_written(cache);
        }
        crate::sys::DATAPTR_RO_unchecked(cache).cast_mut()
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
        mark_altstring_written(cache);
        cache.set_string_elt(i, v.get());
    }
}

/// Whether R has stored a value into a string ALTREP's cache.
///
/// Reads fill the cache too, with values computed from the Rust data, so a
/// cache alone doesn't mean the Rust data is stale. `Set_elt` and a writable
/// `Dataptr` mark it before R can store into it, so the mark errs towards
/// written.
///
/// # Safety
///
/// `cache` is a string ALTREP's data2 cache, on R's main thread.
pub(crate) unsafe fn altstring_cache_written(cache: crate::SEXP) -> bool {
    use crate::SexpExt;

    !unsafe { cache.get_attr_unchecked(crate::cached_class::altstring_written_symbol()) }.is_nil()
}

/// Mark a string ALTREP's cache written; see [`altstring_cache_written`].
///
/// # Safety
///
/// As for [`altstring_cache_written`].
unsafe fn mark_altstring_written(cache: crate::SEXP) {
    use crate::SexpExt;

    unsafe {
        if !altstring_cache_written(cache) {
            // The symbol is its own value: a symbol is never collected, so
            // setting it allocates nothing but the attribute's cell.
            let symbol = crate::cached_class::altstring_written_symbol();
            cache.set_attr_unchecked(symbol, symbol);
        }
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
pub mod buffers;
pub mod builtins;
pub mod shared_slices;
pub mod static_slices;
