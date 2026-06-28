//! The `SEXP` newtype and its inherent methods.
//!
//! `SEXP` is the central pointer type for R values. It's a thin newtype over
//! `*mut SEXPREC` that implements `Send`/`Sync` for cross-thread plumbing
//! (the pointed-to data must still only be touched on R's main thread).
//!
//! For the broader vocabulary of safe SEXP accessors, see [`crate::sexp_ext::SexpExt`].

use crate::sexp_types::{CE_UTF8, R_xlen_t, Rcomplex, SEXPTYPE};
// Pull in the extern bindings and module-level fns that the inherent methods
// dispatch to. The type-level re-exports in `sys` (SEXP, SEXPREC, SEXPTYPE,
// etc.) shadow but are identical to what we're defining locally — that's
// fine because they're the same items.
use crate::sys::{
    R_BaseNamespace, R_BlankString, R_ClassSymbol, R_DimNamesSymbol, R_DimSymbol, R_LevelsSymbol,
    R_MissingArg, R_NaString, R_NamesSymbol, R_NilValue, R_TspSymbol, R_altrep_data1,
    R_altrep_data1_unchecked, R_altrep_data2, R_altrep_data2_unchecked, R_set_altrep_data1,
    R_set_altrep_data2, R_set_altrep_data2_unchecked, Rf_ScalarComplex, Rf_ScalarComplex_unchecked,
    Rf_ScalarInteger, Rf_ScalarInteger_unchecked, Rf_ScalarLogical, Rf_ScalarLogical_unchecked,
    Rf_ScalarRaw, Rf_ScalarRaw_unchecked, Rf_ScalarReal, Rf_ScalarReal_unchecked, Rf_ScalarString,
    Rf_ScalarString_unchecked, Rf_allocVector, Rf_installChar, Rf_mkCharLenCE,
};

#[repr(transparent)]
#[derive(Debug)]
/// Opaque underlying S-expression header type.
pub struct SEXPREC(::std::os::raw::c_void);

/// R's pointer type for S-expressions.
///
/// This is a newtype wrapper around `*mut SEXPREC` that implements Send and Sync.
/// SEXP is just a handle (pointer) - the actual data it points to is managed by R's
/// garbage collector and should only be accessed on R's main thread.
///
/// # Safety
///
/// While SEXP is Send+Sync (allowing it to be passed between threads), the data
/// it points to must only be accessed on R's main thread. The miniextendr runtime
/// enforces this through the worker thread pattern.
///
/// # Equality Semantics
///
/// IMPORTANT: The derived `PartialEq` compares **pointer equality**, not semantic equality.
/// For proper R semantics (comparing object contents), use `R_compute_identical`.
///
/// ```ignore
/// // Pointer equality (fast, often wrong for R semantics)
/// if sexp1 == sexp2 { ... }  // Only true if same pointer
///
/// // Semantic equality (correct R semantics)
/// if R_compute_identical(sexp1, sexp2, 16) != 0 { ... }
/// ```
///
/// **Hash trait removed**: SEXP no longer implements `Hash` because proper hashing
/// would require deep content inspection via `R_compute_identical`, which is too
/// expensive for general use. If you need SEXP as a HashMap key, use pointer identity:
///
/// ```ignore
/// // Store by pointer identity (common pattern for R symbol lookups)
/// let mut map: HashMap<*mut SEXPREC, Value> = HashMap::new();
/// map.insert(sexp.as_ptr(), value);
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SEXP(pub *mut SEXPREC);

// SAFETY: SEXP is just a pointer (memory address). Passing the address between
// threads is safe. The actual data access is protected by miniextendr's runtime
// which ensures R API calls happen on the main thread.
unsafe impl Send for SEXP {}
unsafe impl Sync for SEXP {}

impl SEXP {
    /// Create a C null pointer SEXP (0x0).
    ///
    /// This is **not** R's `NULL` value (`R_NilValue`). R's `NULL` is a real
    /// heap-allocated singleton; a C null pointer is just address zero. Passing
    /// `SEXP::null()` where R expects `R_NilValue` will corrupt R's GC state
    /// and likely segfault.
    ///
    /// Use [`SEXP::nil()`] for R's `NULL`. Only use `null()` for low-level
    /// pointer initialization, ALTREP Sum/Min/Max "can't compute" returns
    /// (R checks `!= NULL`, not `!= R_NilValue`), or comparison against
    /// uninitialized pointers.
    ///
    /// See also: [`SEXP::nil()`], [`SEXP::is_null()`], [`crate::SexpExt::is_nil()`]
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Return R's `NULL` singleton (`R_NilValue`).
    ///
    /// This is **not** a C null pointer — it points to R's actual nil object
    /// on the heap. Use this for `.Call()` return values, SEXP arguments to
    /// R API functions, and any slot in R data structures.
    ///
    /// See also: [`SEXP::null()`], [`crate::SexpExt::is_nil()`], [`SEXP::is_null()`]
    #[inline]
    pub fn nil() -> Self {
        unsafe { R_NilValue }
    }

    /// Check if this SEXP is a C null pointer (0x0).
    ///
    /// To check if an SEXP is R's `NULL` (`R_NilValue`), use
    /// [`crate::SexpExt::is_nil()`] instead.
    ///
    /// See also: [`crate::SexpExt::is_nil()`], [`crate::SexpExt::is_null_or_nil()`]
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut SEXPREC {
        self.0
    }

    /// Create from a raw pointer.
    #[inline]
    pub const fn from_ptr(ptr: *mut SEXPREC) -> Self {
        Self(ptr)
    }

    // region: String construction

    /// Create a CHARSXP from a Rust `&str` (UTF-8).
    #[inline]
    pub fn charsxp(s: &str) -> SEXP {
        let len: i32 = s.len().try_into().expect("string exceeds i32::MAX bytes");
        unsafe { Rf_mkCharLenCE(s.as_ptr().cast(), len, CE_UTF8) }
    }

    /// R's `NA_character_` singleton.
    #[inline]
    pub fn na_string() -> SEXP {
        unsafe { R_NaString }
    }

    /// R's empty string `""` singleton.
    #[inline]
    pub fn blank_string() -> SEXP {
        unsafe { R_BlankString }
    }

    /// Create an R symbol (SYMSXP) from a CHARSXP.
    ///
    /// Equivalent to `Rf_installChar(charsxp)`. The symbol is interned
    /// in R's global symbol table and never garbage collected.
    #[inline]
    pub fn install_char(charsxp: SEXP) -> SEXP {
        unsafe { Rf_installChar(charsxp) }
    }

    /// Create an R symbol (SYMSXP) from a Rust `&str`.
    ///
    /// Combines `SEXP::charsxp()` + `Rf_installChar` into one call.
    /// The symbol is interned and never garbage collected.
    #[inline]
    pub fn symbol(name: &str) -> SEXP {
        Self::install_char(Self::charsxp(name))
    }

    // endregion

    // region: Scalar construction

    /// Create a length-1 integer vector.
    #[inline]
    pub fn scalar_integer(x: i32) -> SEXP {
        unsafe { Rf_ScalarInteger(x) }
    }

    /// Create a length-1 real vector.
    #[inline]
    pub fn scalar_real(x: f64) -> SEXP {
        unsafe { Rf_ScalarReal(x) }
    }

    /// Create a length-1 logical vector.
    ///
    /// Produces only `TRUE` or `FALSE`; a `bool` cannot represent R's `NA`.
    /// For an NA logical, use [`scalar_logical_raw`](Self::scalar_logical_raw)
    /// with `NA_LOGICAL` (`i32::MIN`).
    #[inline]
    pub fn scalar_logical(x: bool) -> SEXP {
        unsafe { Rf_ScalarLogical(if x { 1 } else { 0 }) }
    }

    /// Create a length-1 logical vector from raw i32 (0=FALSE, 1=TRUE, NA_LOGICAL=NA).
    #[inline]
    /// Accepts 0 (FALSE), 1 (TRUE), or `NA_LOGICAL` (`i32::MIN`) for NA.
    /// Prefer [`scalar_logical`](Self::scalar_logical) for non-NA values.
    pub fn scalar_logical_raw(x: i32) -> SEXP {
        unsafe { Rf_ScalarLogical(x) }
    }

    /// Create a length-1 raw vector.
    #[inline]
    pub fn scalar_raw(x: u8) -> SEXP {
        unsafe { Rf_ScalarRaw(x) }
    }

    /// Create a length-1 complex vector.
    #[inline]
    pub fn scalar_complex(x: Rcomplex) -> SEXP {
        unsafe { Rf_ScalarComplex(x) }
    }

    /// Create a length-1 character vector from a CHARSXP.
    #[inline]
    pub fn scalar_string(charsxp: SEXP) -> SEXP {
        unsafe { Rf_ScalarString(charsxp) }
    }

    /// Create a length-1 character vector from a Rust `&str`.
    #[inline]
    pub fn scalar_string_from_str(s: &str) -> SEXP {
        Self::scalar_string(Self::charsxp(s))
    }

    // Unchecked scalar constructors — skip the `with_r_thread` check.
    // Use only inside ALTREP callbacks, `with_r_unwind_protect`, or `with_r_thread` blocks
    // where the R-thread invariant is already established (see `#[r_ffi_checked]` docs).

    /// Create a length-1 integer vector (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_integer_unchecked(x: i32) -> SEXP {
        unsafe { Rf_ScalarInteger_unchecked(x) }
    }

    /// Create a length-1 real vector (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_real_unchecked(x: f64) -> SEXP {
        unsafe { Rf_ScalarReal_unchecked(x) }
    }

    /// Create a length-1 logical vector from raw i32 (unchecked — no thread routing).
    ///
    /// Accepts 0 (FALSE), 1 (TRUE), or `NA_LOGICAL` (`i32::MIN`) for NA.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_logical_raw_unchecked(x: i32) -> SEXP {
        unsafe { Rf_ScalarLogical_unchecked(x) }
    }

    /// Create a length-1 raw vector (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_raw_unchecked(x: u8) -> SEXP {
        unsafe { Rf_ScalarRaw_unchecked(x) }
    }

    /// Create a length-1 complex vector (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_complex_unchecked(x: Rcomplex) -> SEXP {
        unsafe { Rf_ScalarComplex_unchecked(x) }
    }

    /// Create a length-1 character vector from a CHARSXP (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_string_unchecked(charsxp: SEXP) -> SEXP {
        unsafe { Rf_ScalarString_unchecked(charsxp) }
    }

    // endregion

    // region: Vector allocation

    /// Allocate a fresh R vector of the given type and length.
    ///
    /// Direct wrapper over `Rf_allocVector`. For typed allocations, prefer
    /// helpers like [`SEXP::alloc_list`], [`SEXP::alloc_strsxp`], or wrap the
    /// result in [`OwnedProtect`](crate::gc_protect::OwnedProtect) immediately
    /// — the returned SEXP is unprotected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The returned SEXP is unprotected;
    /// any subsequent allocation may collect it.
    #[inline]
    pub unsafe fn alloc(ty: SEXPTYPE, n: R_xlen_t) -> SEXP {
        unsafe { Rf_allocVector(ty, n) }
    }

    /// Allocate an R list (VECSXP) of length `n`. Unprotected.
    ///
    /// Equivalent to `Rf_allocVector(VECSXP, n)`. Elements are initialised to `R_NilValue`.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The returned SEXP is unprotected —
    /// wrap it in [`OwnedProtect`](crate::gc_protect::OwnedProtect) before any
    /// other allocation that could trigger GC.
    #[inline]
    pub unsafe fn alloc_list(n: R_xlen_t) -> SEXP {
        unsafe { Rf_allocVector(SEXPTYPE::VECSXP, n) }
    }

    /// Allocate an R character vector (STRSXP) of length `n`. Unprotected.
    ///
    /// Equivalent to `Rf_allocVector(STRSXP, n)`. Elements are initialised to `R_BlankString`.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The returned SEXP is unprotected —
    /// wrap it in [`OwnedProtect`](crate::gc_protect::OwnedProtect) before any
    /// other allocation that could trigger GC.
    #[inline]
    pub unsafe fn alloc_strsxp(n: R_xlen_t) -> SEXP {
        unsafe { Rf_allocVector(SEXPTYPE::STRSXP, n) }
    }

    // endregion

    // region: R global symbols and singletons

    /// R's `names` attribute symbol.
    #[inline]
    pub fn names_symbol() -> SEXP {
        unsafe { R_NamesSymbol }
    }

    /// R's `dim` attribute symbol.
    #[inline]
    pub fn dim_symbol() -> SEXP {
        unsafe { R_DimSymbol }
    }

    /// R's `dimnames` attribute symbol.
    #[inline]
    pub fn dimnames_symbol() -> SEXP {
        unsafe { R_DimNamesSymbol }
    }

    /// R's `class` attribute symbol.
    #[inline]
    pub fn class_symbol() -> SEXP {
        unsafe { R_ClassSymbol }
    }

    /// R's `levels` attribute symbol (factors).
    #[inline]
    pub fn levels_symbol() -> SEXP {
        unsafe { R_LevelsSymbol }
    }

    /// R's `tsp` attribute symbol (time series).
    #[inline]
    pub fn tsp_symbol() -> SEXP {
        unsafe { R_TspSymbol }
    }

    /// R's base namespace environment.
    #[inline]
    pub fn base_namespace() -> SEXP {
        unsafe { R_BaseNamespace }
    }

    /// R's missing argument sentinel.
    #[inline]
    pub fn missing_arg() -> SEXP {
        unsafe { R_MissingArg }
    }

    // endregion

    // region: ALTREP data slot access

    /// Get the raw SEXP in the ALTREP data1 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn altrep_data1_raw(self) -> SEXP {
        unsafe { R_altrep_data1(self) }
    }

    /// Get the raw SEXP in the ALTREP data1 slot (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn altrep_data1_raw_unchecked(self) -> SEXP {
        unsafe { R_altrep_data1_unchecked(self) }
    }

    /// Set the ALTREP data1 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn set_altrep_data1(self, v: SEXP) {
        unsafe { R_set_altrep_data1(self, v) }
    }

    /// Get the raw SEXP in the ALTREP data2 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn altrep_data2_raw(self) -> SEXP {
        unsafe { R_altrep_data2(self) }
    }

    /// Get the raw SEXP in the ALTREP data2 slot (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn altrep_data2_raw_unchecked(self) -> SEXP {
        unsafe { R_altrep_data2_unchecked(self) }
    }

    /// Set the ALTREP data2 slot.
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn set_altrep_data2(self, v: SEXP) {
        unsafe { R_set_altrep_data2(self, v) }
    }

    /// Set the ALTREP data2 slot (unchecked — no thread routing).
    ///
    /// # Safety
    ///
    /// - `self` must be a valid ALTREP SEXP
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn set_altrep_data2_unchecked(self, v: SEXP) {
        unsafe { R_set_altrep_data2_unchecked(self, v) }
    }

    // endregion
}

impl Default for SEXP {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

impl From<*mut SEXPREC> for SEXP {
    #[inline]
    fn from(ptr: *mut SEXPREC) -> Self {
        Self(ptr)
    }
}

impl From<SEXP> for *mut SEXPREC {
    #[inline]
    fn from(sexp: SEXP) -> Self {
        sexp.0
    }
}
