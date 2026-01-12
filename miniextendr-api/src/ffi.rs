pub mod altrep;

#[allow(non_camel_case_types)]
pub type R_xlen_t = isize;
pub type Rbyte = ::std::os::raw::c_uchar;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rcomplex {
    pub r: f64,
    pub i: f64,
}

#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SEXPTYPE {
    #[doc = " nil = NULL"]
    NILSXP = 0,
    #[doc = " symbols"]
    SYMSXP = 1,
    #[doc = " lists of dotted pairs"]
    LISTSXP = 2,
    #[doc = " closures"]
    CLOSXP = 3,
    #[doc = " environments"]
    ENVSXP = 4,
    #[doc = r" promises: \[un\]evaluated closure arguments"]
    PROMSXP = 5,
    #[doc = " language constructs (special lists)"]
    LANGSXP = 6,
    #[doc = " special forms"]
    SPECIALSXP = 7,
    #[doc = " builtin non-special forms"]
    BUILTINSXP = 8,
    #[doc = " \"scalar\" string type (internal only)"]
    CHARSXP = 9,
    #[doc = " logical vectors"]
    LGLSXP = 10,
    #[doc = " integer vectors"]
    INTSXP = 13,
    #[doc = " real variables"]
    REALSXP = 14,
    #[doc = " complex variables"]
    CPLXSXP = 15,
    #[doc = " string vectors"]
    STRSXP = 16,
    #[doc = " dot-dot-dot object"]
    DOTSXP = 17,
    #[doc = " make \"any\" args work"]
    ANYSXP = 18,
    #[doc = " generic vectors"]
    VECSXP = 19,
    #[doc = " expressions vectors"]
    EXPRSXP = 20,
    #[doc = " byte code"]
    BCODESXP = 21,
    #[doc = " external pointer"]
    EXTPTRSXP = 22,
    #[doc = " weak reference"]
    WEAKREFSXP = 23,
    #[doc = " raw bytes"]
    RAWSXP = 24,
    #[doc = " S4 non-vector"]
    S4SXP = 25,
    #[doc = " fresh node created in new page"]
    NEWSXP = 30,
    #[doc = " node released by GC"]
    FREESXP = 31,
    #[doc = " Closure or Builtin"]
    FUNSXP = 99,
}

impl SEXPTYPE {
    /// Alias for `S4SXP` (value 25).
    ///
    /// R defines both `OBJSXP` and `S4SXP` as value 25. `S4SXP` is retained
    /// for backwards compatibility; `OBJSXP` is the preferred name.
    pub const OBJSXP: SEXPTYPE = SEXPTYPE::S4SXP;
}

#[repr(transparent)]
#[derive(Debug)]
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
/// For proper R semantics (comparing object contents), use [`R_compute_identical`].
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
    /// Create a null SEXP.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if this SEXP is null.
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

/// Extension trait for SEXP providing safe(r) accessors and type checking.
///
/// This trait provides idiomatic Rust methods for working with SEXPs,
/// equivalent to R's inline macros and type checking functions.
pub trait SexpExt {
    /// Get the type of this SEXP.
    ///
    /// Equivalent to `TYPEOF(x)` macro.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid (not null and not freed).
    fn type_of(&self) -> SEXPTYPE;

    /// Check if this SEXP is null or R_NilValue.
    fn is_null_or_nil(&self) -> bool;

    /// Get the length of this SEXP as `usize`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    fn len(&self) -> usize;

    /// Get the length as `R_xlen_t`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    fn xlength(&self) -> R_xlen_t;

    /// Get the length without thread checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. No debug assertions.
    unsafe fn len_unchecked(&self) -> usize;

    /// Get a slice view of this SEXP's data.
    ///
    /// # Safety
    ///
    /// - The SEXP must be valid and of the correct type for `T`
    /// - The SEXP must be protected from R's garbage collector for the entire
    ///   duration the returned slice is used. This typically means the SEXP must
    ///   be either:
    ///   - An argument to a `.Call` function (protected by R's calling convention)
    ///   - Explicitly protected via `PROTECT`/`UNPROTECT` or `R_PreserveObject`
    ///   - Part of a protected container (e.g., element of a protected list)
    /// - The returned slice has `'static` lifetime for API convenience, but this
    ///   is a lie - the actual lifetime is tied to the SEXP's protection status.
    ///   Holding the slice after the SEXP is unprotected is undefined behavior.
    unsafe fn as_slice<T: RNativeType>(&self) -> &'static [T];

    /// Get a slice view without thread checks.
    ///
    /// # Safety
    ///
    /// - All safety requirements of [`as_slice`](Self::as_slice) apply
    /// - Additionally, must be called from R's main thread (no debug assertions)
    unsafe fn as_slice_unchecked<T: RNativeType>(&self) -> &'static [T];

    // Type checking methods (equivalent to R's type check macros)

    /// Check if this SEXP is an integer vector (INTSXP).
    fn is_integer(&self) -> bool;

    /// Check if this SEXP is a real/numeric vector (REALSXP).
    fn is_real(&self) -> bool;

    /// Check if this SEXP is a logical vector (LGLSXP).
    fn is_logical(&self) -> bool;

    /// Check if this SEXP is a character/string vector (STRSXP).
    fn is_character(&self) -> bool;

    /// Check if this SEXP is a raw vector (RAWSXP).
    fn is_raw(&self) -> bool;

    /// Check if this SEXP is a complex vector (CPLXSXP).
    fn is_complex(&self) -> bool;

    /// Check if this SEXP is a list/generic vector (VECSXP).
    fn is_list(&self) -> bool;

    /// Check if this SEXP is an external pointer (EXTPTRSXP).
    fn is_external_ptr(&self) -> bool;

    /// Check if this SEXP is an environment (ENVSXP).
    fn is_environment(&self) -> bool;

    /// Check if this SEXP is a symbol (SYMSXP).
    fn is_symbol(&self) -> bool;

    /// Check if this SEXP is a language object (LANGSXP).
    fn is_language(&self) -> bool;

    /// Check if this SEXP is an ALTREP object.
    ///
    /// Equivalent to R's `ALTREP(x)` macro.
    fn is_altrep(&self) -> bool;

    /// Check if this `SEXP` contains any elements.
    ///
    fn is_empty(&self) -> bool;
}

impl SexpExt for SEXP {
    #[inline]
    fn type_of(&self) -> SEXPTYPE {
        unsafe { TYPEOF(*self) }
    }

    #[inline]
    fn is_null_or_nil(&self) -> bool {
        self.is_null() || std::ptr::addr_eq(self.0, unsafe { R_NilValue.0 })
    }

    #[inline]
    fn len(&self) -> usize {
        unsafe { Rf_xlength(*self) as usize }
    }

    #[inline]
    fn xlength(&self) -> R_xlen_t {
        unsafe { Rf_xlength(*self) }
    }

    #[inline]
    unsafe fn len_unchecked(&self) -> usize {
        unsafe { Rf_xlength_unchecked(*self) as usize }
    }

    #[inline]
    unsafe fn as_slice<T: RNativeType>(&self) -> &'static [T] {
        debug_assert!(
            self.type_of() == T::SEXP_TYPE,
            "SEXP type mismatch: expected {:?}, got {:?}",
            T::SEXP_TYPE,
            self.type_of()
        );
        let len = self.len();
        if len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(DATAPTR_RO(*self).cast(), len) }
        }
    }

    #[inline]
    unsafe fn as_slice_unchecked<T: RNativeType>(&self) -> &'static [T] {
        debug_assert!(
            self.type_of() == T::SEXP_TYPE,
            "SEXP type mismatch: expected {:?}, got {:?}",
            T::SEXP_TYPE,
            self.type_of()
        );
        let len = unsafe { self.len_unchecked() };
        if len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(DATAPTR_RO_unchecked(*self).cast(), len) }
        }
    }

    // Type checking methods

    #[inline]
    fn is_integer(&self) -> bool {
        self.type_of() == SEXPTYPE::INTSXP
    }

    #[inline]
    fn is_real(&self) -> bool {
        self.type_of() == SEXPTYPE::REALSXP
    }

    #[inline]
    fn is_logical(&self) -> bool {
        self.type_of() == SEXPTYPE::LGLSXP
    }

    #[inline]
    fn is_character(&self) -> bool {
        self.type_of() == SEXPTYPE::STRSXP
    }

    #[inline]
    fn is_raw(&self) -> bool {
        self.type_of() == SEXPTYPE::RAWSXP
    }

    #[inline]
    fn is_complex(&self) -> bool {
        self.type_of() == SEXPTYPE::CPLXSXP
    }

    #[inline]
    fn is_list(&self) -> bool {
        self.type_of() == SEXPTYPE::VECSXP
    }

    #[inline]
    fn is_external_ptr(&self) -> bool {
        self.type_of() == SEXPTYPE::EXTPTRSXP
    }

    #[inline]
    fn is_environment(&self) -> bool {
        self.type_of() == SEXPTYPE::ENVSXP
    }

    #[inline]
    fn is_symbol(&self) -> bool {
        self.type_of() == SEXPTYPE::SYMSXP
    }

    #[inline]
    fn is_language(&self) -> bool {
        self.type_of() == SEXPTYPE::LANGSXP
    }

    #[inline]
    fn is_altrep(&self) -> bool {
        unsafe { ALTREP(*self) != 0 }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Marker trait for types that correspond to R's native vector element types.
///
/// This enables blanket implementations for `TryFromSexp` and safe conversions.
pub trait RNativeType: Sized + Copy + 'static {
    /// The SEXPTYPE for vectors containing this element type.
    const SEXP_TYPE: SEXPTYPE;

    /// Get mutable pointer to vector data.
    ///
    /// For empty vectors (length 0), returns an aligned dangling pointer rather than
    /// R's internal 0x1 sentinel, which isn't properly aligned for most types.
    /// This allows safe creation of zero-length slices with `std::slice::from_raw_parts_mut`.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid, non-null SEXP of the corresponding vector type.
    /// - For ALTREP vectors, this may trigger materialization.
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self;
}

/// R's logical element type (the contents of a `LGLSXP` vector).
///
/// In R, logical vectors are stored as `int` with possible values:
/// - `0` for FALSE
/// - `1` for TRUE
/// - `NA_LOGICAL` (typically `INT_MIN`) for NA
///
/// **Important:** R may also contain other non-zero values in logical vectors
/// (e.g., from low-level code). Those should be interpreted as TRUE.
///
/// This type is `repr(transparent)` over `i32` so *any* raw value is valid,
/// avoiding UB when viewing `LGLSXP` data as a slice.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct RLogical(i32);

impl RLogical {
    pub const FALSE: Self = Self(0);
    pub const TRUE: Self = Self(1);
    pub const NA: Self = Self(i32::MIN);

    #[inline]
    pub const fn from_i32(raw: i32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        self.0
    }

    #[inline]
    pub const fn is_na(self) -> bool {
        self.0 == i32::MIN
    }

    #[inline]
    pub const fn to_option_bool(self) -> Option<bool> {
        match self.0 {
            0 => Some(false),
            i32::MIN => None,
            _ => Some(true),
        }
    }
}

impl From<bool> for RLogical {
    #[inline]
    fn from(value: bool) -> Self {
        if value { Self::TRUE } else { Self::FALSE }
    }
}

impl RNativeType for i32 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::INTSXP;

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        // R returns 0x1 for empty vectors, which isn't properly aligned.
        // Return an aligned dangling pointer for empty case.
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                INTEGER(sexp)
            }
        }
    }
}

impl RNativeType for f64 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        // R returns 0x1 for empty vectors, which isn't properly aligned.
        // Return an aligned dangling pointer for empty case.
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                REAL(sexp)
            }
        }
    }
}

impl RNativeType for u8 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::RAWSXP;

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        // R returns 0x1 for empty vectors, which isn't properly aligned.
        // Return an aligned dangling pointer for empty case.
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                RAW(sexp)
            }
        }
    }
}

impl RNativeType for RLogical {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::LGLSXP;

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        // R returns 0x1 for empty vectors, which isn't properly aligned.
        // Return an aligned dangling pointer for empty case.
        // LOGICAL returns *mut c_int, RLogical is repr(transparent) over i32
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                LOGICAL(sexp).cast()
            }
        }
    }
}

impl RNativeType for Rcomplex {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::CPLXSXP;

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        // R returns 0x1 for empty vectors, which isn't properly aligned.
        // Return an aligned dangling pointer for empty case.
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                COMPLEX(sexp)
            }
        }
    }
}

#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Rboolean {
    FALSE = 0,
    TRUE = 1,
}

impl From<bool> for Rboolean {
    fn from(value: bool) -> Self {
        match value {
            true => Rboolean::TRUE,
            false => Rboolean::FALSE,
        }
    }
}

impl From<Rboolean> for bool {
    fn from(value: Rboolean) -> Self {
        match value {
            Rboolean::FALSE => false,
            Rboolean::TRUE => true,
        }
    }
}

#[allow(non_camel_case_types)]
pub type R_CFinalizer_t = ::std::option::Option<unsafe extern "C-unwind" fn(s: SEXP)>;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum cetype_t {
    CE_NATIVE = 0,
    CE_UTF8 = 1,
    CE_LATIN1 = 2,
    CE_BYTES = 3,
    CE_SYMBOL = 5,
    CE_ANY = 99,
}
pub use cetype_t::CE_UTF8;

// region: Connections types (gated behind `connections` feature)
// WARNING: R's connection API is explicitly marked as UNSTABLE.

/// Opaque R connection implementation (from R_ext/Connections.h).
///
/// This is an opaque type representing R's internal connection structure.
/// The actual structure is explicitly unstable and may change between R versions.
#[cfg(feature = "connections")]
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct Rconnection_impl(::std::os::raw::c_void);

/// Pointer to an R connection handle.
///
/// This is the typed equivalent of R's `Rconnection` type, which is a pointer
/// to the opaque `Rconn` struct. Using this instead of `*mut c_void` provides
/// type safety for connection APIs.
#[cfg(feature = "connections")]
#[allow(non_camel_case_types)]
pub type Rconnection = *mut Rconnection_impl;

/// R connections API version constant.
///
/// From R_ext/Connections.h: "you *must* check the version and proceed only
/// if it matches what you expect. We explicitly reserve the right to change
/// the connection implementation without a compatibility layer."
///
/// Before using any connection APIs, check that this equals the expected version (1).
#[cfg(feature = "connections")]
#[allow(non_upper_case_globals)]
pub const R_CONNECTIONS_VERSION: ::std::os::raw::c_int = 1;

// endregion

use miniextendr_macros::r_ffi_checked;

// Unchecked variadic functions (internal use only, no thread check)
#[allow(clashing_extern_declarations)]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    #[link_name = "Rf_error"]
    pub fn Rf_error_unchecked(arg1: *const ::std::os::raw::c_char, ...) -> !;
    #[link_name = "Rf_errorcall"]
    pub fn Rf_errorcall_unchecked(arg1: SEXP, arg2: *const ::std::os::raw::c_char, ...) -> !;
    #[link_name = "Rf_warning"]
    pub fn Rf_warning_unchecked(arg1: *const ::std::os::raw::c_char, ...);
    #[link_name = "Rprintf"]
    pub fn Rprintf_unchecked(arg1: *const ::std::os::raw::c_char, ...);
    #[link_name = "REprintf"]
    pub fn REprintf_unchecked(arg1: *const ::std::os::raw::c_char, ...);
}

// Error message access (non-API, declared in Rinternals.h but flagged by R CMD check)
#[cfg(feature = "nonapi")]
unsafe extern "C-unwind" {
    /// Get the current R error message buffer.
    ///
    /// Returns a pointer to R's internal error message buffer.
    /// Used by Rserve and other embedding applications.
    ///
    /// # Safety
    ///
    /// - The returned pointer is only valid until the next R error
    /// - Must not be modified
    /// - Should be copied if needed beyond the immediate scope
    ///
    /// # Feature Gate
    ///
    /// This is a non-API function and requires the `nonapi` feature.
    #[allow(non_snake_case)]
    pub fn R_curErrorBuf() -> *const ::std::os::raw::c_char;
}

// Console hooks (non-API; declared in Rinterface.h)
#[cfg(feature = "nonapi")]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    pub static ptr_R_WriteConsoleEx: Option<
        unsafe extern "C-unwind" fn(
            *const ::std::os::raw::c_char,
            ::std::os::raw::c_int,
            ::std::os::raw::c_int,
        ),
    >;
}

/// Checked wrapper for `Rf_error` - panics if called from non-main thread.
/// Common usage: `Rf_error(c"%s".as_ptr(), message.as_ptr())`
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_error(
    fmt: *const ::std::os::raw::c_char,
    arg1: *const ::std::os::raw::c_char,
) -> ! {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_error called from non-main thread");
    }
    unsafe { Rf_error_unchecked(fmt, arg1) }
}

/// Checked wrapper for `Rf_errorcall` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `call` must be a valid SEXP or R_NilValue
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_errorcall(
    call: SEXP,
    fmt: *const ::std::os::raw::c_char,
    arg1: *const ::std::os::raw::c_char,
) -> ! {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_errorcall called from non-main thread");
    }
    unsafe { Rf_errorcall_unchecked(call, fmt, arg1) }
}

/// Checked wrapper for `Rf_warning` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_warning(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_warning called from non-main thread");
    }
    unsafe { Rf_warning_unchecked(fmt, arg1) }
}

/// Checked wrapper for `Rprintf` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rprintf(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) {
    if !crate::worker::is_r_main_thread() {
        panic!("Rprintf called from non-main thread");
    }
    unsafe { Rprintf_unchecked(fmt, arg1) }
}

/// Print to R's stderr (via R_ShowMessage or error console).
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn REprintf(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) {
    if !crate::worker::is_r_main_thread() {
        panic!("REprintf called from non-main thread");
    }
    unsafe { REprintf_unchecked(fmt, arg1) }
}

#[r_ffi_checked]
#[allow(clashing_extern_declarations)]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    pub static R_NilValue: SEXP;

    #[doc(alias = "NA_STRING")]
    pub static R_NaString: SEXP;
    /// Empty string CHARSXP (length 0).
    pub static R_BlankString: SEXP;
    pub static R_NamesSymbol: SEXP;
    pub static R_DimSymbol: SEXP;
    pub static R_DimNamesSymbol: SEXP;
    pub static R_ClassSymbol: SEXP;
    pub static R_RowNamesSymbol: SEXP;
    pub static R_LevelsSymbol: SEXP;
    pub static R_TspSymbol: SEXP;

    pub static R_GlobalEnv: SEXP;
    pub static R_BaseEnv: SEXP;
    pub static R_EmptyEnv: SEXP;

    // Rinterface.h
    pub fn R_FlushConsole();

    // Special logical values (from internal Defn.h, not public API)
    // These are gated behind `nonapi` feature as they may change across R versions.
    #[cfg(feature = "nonapi")]
    pub static R_TrueValue: SEXP;
    #[cfg(feature = "nonapi")]
    pub static R_FalseValue: SEXP;
    #[cfg(feature = "nonapi")]
    pub static R_LogicalNAValue: SEXP;

    // Rinternals.h
    #[doc(alias = "mkChar")]
    pub fn Rf_mkChar(s: *const ::std::os::raw::c_char) -> SEXP;
    #[doc(alias = "mkCharLen")]
    pub fn Rf_mkCharLen(s: *const ::std::os::raw::c_char, len: i32) -> SEXP;
    #[doc(alias = "mkCharLenCE")]
    pub fn Rf_mkCharLenCE(
        x: *const ::std::os::raw::c_char,
        len: ::std::os::raw::c_int,
        ce: cetype_t,
    ) -> SEXP;
    #[doc(alias = "xlength")]
    #[doc(alias = "XLENGTH")]
    pub fn Rf_xlength(x: SEXP) -> R_xlen_t;
    #[doc(alias = "translateCharUTF8")]
    pub fn Rf_translateCharUTF8(x: SEXP) -> *const ::std::os::raw::c_char;
    #[doc(alias = "getCharCE")]
    pub fn Rf_getCharCE(x: SEXP) -> cetype_t;
    #[doc(alias = "charIsASCII")]
    pub fn Rf_charIsASCII(x: SEXP) -> Rboolean;
    #[doc(alias = "charIsUTF8")]
    pub fn Rf_charIsUTF8(x: SEXP) -> Rboolean;
    #[doc(alias = "charIsLatin1")]
    pub fn Rf_charIsLatin1(x: SEXP) -> Rboolean;

    pub fn R_MakeUnwindCont() -> SEXP;
    pub fn R_ContinueUnwind(cont: SEXP) -> !;
    pub fn R_UnwindProtect(
        fun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void) -> SEXP,
        >,
        fun_data: *mut ::std::os::raw::c_void,
        cleanfun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void, Rboolean),
        >,
        cleanfun_data: *mut ::std::os::raw::c_void,
        cont: SEXP,
    ) -> SEXP;

    /// Version of `R_UnwindProtect` that accepts `extern "C-unwind"` function pointers
    #[link_name = "R_UnwindProtect"]
    pub fn R_UnwindProtect_C_unwind(
        fun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void) -> SEXP,
        >,
        fun_data: *mut ::std::os::raw::c_void,
        cleanfun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void, Rboolean),
        >,
        cleanfun_data: *mut ::std::os::raw::c_void,
        cont: SEXP,
    ) -> SEXP;

    // Rinternals.h
    #[doc = " External pointer interface"]
    pub fn R_MakeExternalPtr(p: *mut ::std::os::raw::c_void, tag: SEXP, prot: SEXP) -> SEXP;
    pub fn R_ExternalPtrAddr(s: SEXP) -> *mut ::std::os::raw::c_void;
    pub fn R_ExternalPtrTag(s: SEXP) -> SEXP;
    pub fn R_ExternalPtrProtected(s: SEXP) -> SEXP;
    pub fn R_ClearExternalPtr(s: SEXP);
    pub fn R_SetExternalPtrAddr(s: SEXP, p: *mut ::std::os::raw::c_void);
    pub fn R_SetExternalPtrTag(s: SEXP, tag: SEXP);
    pub fn R_SetExternalPtrProtected(s: SEXP, p: SEXP);
    #[doc = " Added in R 3.4.0"]
    pub fn R_MakeExternalPtrFn(p: DL_FUNC, tag: SEXP, prot: SEXP) -> SEXP;
    pub fn R_ExternalPtrAddrFn(s: SEXP) -> DL_FUNC;
    pub fn R_RegisterFinalizer(s: SEXP, fun: SEXP);
    pub fn R_RegisterCFinalizer(s: SEXP, fun: R_CFinalizer_t);
    pub fn R_RegisterFinalizerEx(s: SEXP, fun: SEXP, onexit: Rboolean);
    pub fn R_RegisterCFinalizerEx(s: SEXP, fun: R_CFinalizer_t, onexit: Rboolean);

    // R_ext/Rdynload.h - C-callable interface
    /// Register a C-callable function for cross-package access.
    pub fn R_RegisterCCallable(
        package: *const ::std::os::raw::c_char,
        name: *const ::std::os::raw::c_char,
        fptr: DL_FUNC,
    );
    /// Get a C-callable function from another package.
    pub fn R_GetCCallable(
        package: *const ::std::os::raw::c_char,
        name: *const ::std::os::raw::c_char,
    ) -> DL_FUNC;

    // Rinternals.h
    pub fn R_PreserveObject(object: SEXP);
    pub fn R_ReleaseObject(object: SEXP);

    #[doc(alias = "PROTECT")]
    #[doc(alias = "protect")]
    pub fn Rf_protect(s: SEXP) -> SEXP;
    #[doc(alias = "UNPROTECT")]
    #[doc(alias = "unprotect")]
    pub fn Rf_unprotect(l: ::std::os::raw::c_int);
    #[doc(alias = "UNPROTECT_PTR")]
    pub fn Rf_unprotect_ptr(s: SEXP);
    // Vector allocation functions
    #[doc(alias = "allocVector")]
    pub fn Rf_allocVector(sexptype: SEXPTYPE, length: R_xlen_t) -> SEXP;
    #[doc(alias = "allocMatrix")]
    pub fn Rf_allocMatrix(
        sexptype: SEXPTYPE,
        nrow: ::std::os::raw::c_int,
        ncol: ::std::os::raw::c_int,
    ) -> SEXP;
    #[doc(alias = "allocArray")]
    pub fn Rf_allocArray(sexptype: SEXPTYPE, dims: SEXP) -> SEXP;
    #[doc(alias = "alloc3DArray")]
    pub fn Rf_alloc3DArray(
        sexptype: SEXPTYPE,
        nrow: ::std::os::raw::c_int,
        ncol: ::std::os::raw::c_int,
        nface: ::std::os::raw::c_int,
    ) -> SEXP;

    // Pairlist allocation
    #[doc(alias = "allocList")]
    pub fn Rf_allocList(n: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "allocLang")]
    pub fn Rf_allocLang(n: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "allocS4Object")]
    pub fn Rf_allocS4Object() -> SEXP;
    #[doc(alias = "allocSExp")]
    pub fn Rf_allocSExp(sexptype: SEXPTYPE) -> SEXP;

    // Pairlist construction
    #[doc(alias = "CONS")]
    #[doc(alias = "cons")]
    pub fn Rf_cons(car: SEXP, cdr: SEXP) -> SEXP;
    #[doc(alias = "LCONS")]
    #[doc(alias = "lcons")]
    pub fn Rf_lcons(car: SEXP, cdr: SEXP) -> SEXP;

    // Attribute manipulation
    #[doc(alias = "setAttrib")]
    pub fn Rf_setAttrib(vec: SEXP, name: SEXP, val: SEXP) -> SEXP;

    // Rinternals.h
    #[doc(alias = "ScalarComplex")]
    pub fn Rf_ScalarComplex(x: Rcomplex) -> SEXP;
    #[doc(alias = "ScalarInteger")]
    pub fn Rf_ScalarInteger(x: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "ScalarLogical")]
    pub fn Rf_ScalarLogical(x: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "ScalarRaw")]
    pub fn Rf_ScalarRaw(x: Rbyte) -> SEXP;
    #[doc(alias = "ScalarReal")]
    pub fn Rf_ScalarReal(x: f64) -> SEXP;
    #[doc(alias = "ScalarString")]
    pub fn Rf_ScalarString(x: SEXP) -> SEXP;

    // Rinternals.h
    /// Non-API function - use DATAPTR_RO or DATAPTR_OR_NULL instead.
    /// Only available with `nonapi` feature.
    #[cfg(feature = "nonapi")]
    pub fn DATAPTR(x: SEXP) -> *mut ::std::os::raw::c_void;
    pub fn DATAPTR_RO(x: SEXP) -> *const ::std::os::raw::c_void;
    pub fn DATAPTR_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_void;

    // =========================================================================
    // Cons Cell (Pairlist) Accessors
    // =========================================================================
    //
    // R's pairlists (LISTSXP) are cons cells like in Lisp/Scheme. Each node has:
    // - CAR: The value/head element
    // - CDR: The rest/tail of the list (another pairlist or R_NilValue)
    // - TAG: An optional name (symbol) for named lists/arguments
    //
    // Example R pairlist: list(a = 1, b = 2, 3)
    // - First node:  CAR=1,    TAG="a",  CDR=<next node>
    // - Second node: CAR=2,    TAG="b",  CDR=<next node>
    // - Third node:  CAR=3,    TAG=NULL, CDR=R_NilValue
    //
    // Pairlists are used for:
    // - Function arguments (formal parameters and actual arguments)
    // - Language objects (calls)
    // - Dotted pairs in old-style lists
    //
    // The names CAR/CDR come from Lisp:
    // - CAR = "Contents of Address part of Register"
    // - CDR = "Contents of Decrement part of Register" (pronounced "could-er")
    //
    // Modern R mostly uses generic vectors (VECSXP) instead of pairlists,
    // but pairlists are still used internally for function calls.

    /// Get the CAR (head/value) of a pairlist node.
    ///
    /// Returns the value stored in this cons cell.
    /// For argument lists, this is the argument value.
    /// For language objects, this is the function or first element.
    ///
    /// # Safety
    ///
    /// `e` must be a valid pairlist (LISTSXP, LANGSXP) or R_NilValue
    pub fn CAR(e: SEXP) -> SEXP;

    /// Get the CDR (tail/rest) of a pairlist node.
    ///
    /// Returns the remainder of the list after this node.
    /// This is either another pairlist node or R_NilValue (end of list).
    ///
    /// # Safety
    ///
    /// `e` must be a valid pairlist (LISTSXP, LANGSXP) or R_NilValue
    pub fn CDR(e: SEXP) -> SEXP;

    /// Get the CAR of the CAR (value of the first element's value).
    ///
    /// Equivalent to `CAR(CAR(e))`. Useful for nested lists.
    ///
    /// # Safety
    ///
    /// `e` must be a valid nested pairlist
    pub fn CAAR(e: SEXP) -> SEXP;

    /// Get the CDR of the CAR (tail of the first element).
    ///
    /// Equivalent to `CDR(CAR(e))`.
    ///
    /// # Safety
    ///
    /// `e` must be a valid nested pairlist
    pub fn CDAR(e: SEXP) -> SEXP;

    /// Get the CAR of the CDR (second element's value).
    ///
    /// Equivalent to `CAR(CDR(e))`. This gets the value of the 2nd list element.
    ///
    /// # Safety
    ///
    /// `e` must be a pairlist with at least 2 elements
    pub fn CADR(e: SEXP) -> SEXP;

    /// Get the CDR of the CDR (list starting from 3rd element).
    ///
    /// Equivalent to `CDR(CDR(e))`. Skips first two elements.
    ///
    /// # Safety
    ///
    /// `e` must be a pairlist with at least 2 elements
    pub fn CDDR(e: SEXP) -> SEXP;

    /// Get the value of the third element.
    ///
    /// Equivalent to `CAR(CDR(CDR(e)))`.
    ///
    /// # Safety
    ///
    /// `e` must be a pairlist with at least 3 elements
    pub fn CADDR(e: SEXP) -> SEXP;

    /// Get the value of the fourth element.
    ///
    /// Equivalent to `CAR(CDR(CDR(CDR(e))))`.
    ///
    /// # Safety
    ///
    /// `e` must be a pairlist with at least 4 elements
    pub fn CADDDR(e: SEXP) -> SEXP;

    /// Get the value of the fifth element.
    ///
    /// Equivalent to `CAR(CDR(CDR(CDR(CDR(e)))))`.
    ///
    /// # Safety
    ///
    /// `e` must be a pairlist with at least 5 elements
    pub fn CAD4R(e: SEXP) -> SEXP;

    /// Get the TAG (name) of a pairlist node.
    ///
    /// Returns the symbol associated with this element, or R_NilValue if unnamed.
    /// For named arguments like `f(x = 5)`, TAG is the symbol "x".
    ///
    /// # Safety
    ///
    /// `e` must be a valid pairlist (LISTSXP, LANGSXP) or R_NilValue
    pub fn TAG(e: SEXP) -> SEXP;

    /// Set the TAG (name) of a pairlist node.
    ///
    /// # Safety
    ///
    /// - `x` must be a valid mutable pairlist node
    /// - `y` must be a symbol (SYMSXP) or R_NilValue
    pub fn SET_TAG(x: SEXP, y: SEXP);

    /// Set the CAR (value) of a pairlist node.
    ///
    /// # Safety
    ///
    /// - `x` must be a valid mutable pairlist node
    /// - `y` must be a valid SEXP
    /// - Returns `y` for convenience
    pub fn SETCAR(x: SEXP, y: SEXP) -> SEXP;

    /// Set the CDR (tail) of a pairlist node.
    ///
    /// # Safety
    ///
    /// - `x` must be a valid mutable pairlist node
    /// - `y` must be a pairlist or R_NilValue
    /// - Returns `y` for convenience
    pub fn SETCDR(x: SEXP, y: SEXP) -> SEXP;

    /// Set the value of the second element.
    ///
    /// Equivalent to `SETCAR(CDR(x), y)`.
    ///
    /// # Safety
    ///
    /// `x` must be a pairlist with at least 2 elements
    pub fn SETCADR(x: SEXP, y: SEXP) -> SEXP;

    /// Set the value of the third element.
    ///
    /// Equivalent to `SETCAR(CDDR(x), y)`.
    ///
    /// # Safety
    ///
    /// `x` must be a pairlist with at least 3 elements
    pub fn SETCADDR(x: SEXP, y: SEXP) -> SEXP;

    /// Set the value of the fourth element.
    ///
    /// Equivalent to `SETCAR(CDR(CDDR(x)), y)`.
    ///
    /// # Safety
    ///
    /// `x` must be a pairlist with at least 4 elements
    pub fn SETCADDDR(x: SEXP, y: SEXP) -> SEXP;

    /// Set the value of the fifth element.
    ///
    /// Equivalent to `SETCAR(CAD4R(x), y)`.
    ///
    /// # Safety
    ///
    /// `x` must be a pairlist with at least 5 elements
    pub fn SETCAD4R(e: SEXP, y: SEXP) -> SEXP;
    pub fn LOGICAL_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_int;
    pub fn INTEGER_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_int;
    pub fn REAL_OR_NULL(x: SEXP) -> *const f64;
    pub fn COMPLEX_OR_NULL(x: SEXP) -> *const Rcomplex;
    pub fn RAW_OR_NULL(x: SEXP) -> *const Rbyte;

    // Element-wise accessors (ALTREP-aware)
    pub fn INTEGER_ELT(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int;
    pub fn REAL_ELT(x: SEXP, i: R_xlen_t) -> f64;
    pub fn LOGICAL_ELT(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int;
    pub fn COMPLEX_ELT(x: SEXP, i: R_xlen_t) -> Rcomplex;
    pub fn RAW_ELT(x: SEXP, i: R_xlen_t) -> Rbyte;
    pub fn VECTOR_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    pub fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    pub fn SET_STRING_ELT(x: SEXP, i: R_xlen_t, v: SEXP);
    pub fn SET_LOGICAL_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
    pub fn SET_INTEGER_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
    pub fn SET_REAL_ELT(x: SEXP, i: R_xlen_t, v: f64);
    pub fn SET_COMPLEX_ELT(x: SEXP, i: R_xlen_t, v: Rcomplex);
    pub fn SET_RAW_ELT(x: SEXP, i: R_xlen_t, v: Rbyte);
    pub fn SET_VECTOR_ELT(x: SEXP, i: R_xlen_t, v: SEXP) -> SEXP;

    // endregion

    // region: SEXP metadata accessors

    /// Get the length of a SEXP as `int` (for short vectors < 2^31).
    ///
    /// For long vectors, use `Rf_xlength()` instead.
    /// Returns 0 for R_NilValue.
    pub fn LENGTH(x: SEXP) -> ::std::os::raw::c_int;

    /// Get the length of a SEXP as `R_xlen_t` (supports long vectors).
    ///
    /// ALTREP-aware: will call ALTREP Length method if needed.
    pub fn XLENGTH(x: SEXP) -> R_xlen_t;

    /// Get the true length (allocated capacity) of a vector.
    ///
    /// May be larger than LENGTH for vectors with reserved space.
    /// ALTREP-aware.
    pub fn TRUELENGTH(x: SEXP) -> R_xlen_t;

    /// Get the attributes pairlist of a SEXP.
    ///
    /// Returns R_NilValue if no attributes.
    pub fn ATTRIB(x: SEXP) -> SEXP;

    /// Set the attributes pairlist of a SEXP.
    ///
    /// # Safety
    ///
    /// `v` must be a pairlist or R_NilValue
    pub fn SET_ATTRIB(x: SEXP, v: SEXP);

    /// Check if SEXP has the "object" bit set (has a class).
    ///
    /// Returns non-zero if object has a class attribute.
    pub fn OBJECT(x: SEXP) -> ::std::os::raw::c_int;

    /// Set the "object" bit.
    pub fn SET_OBJECT(x: SEXP, v: ::std::os::raw::c_int);

    /// Get the LEVELS field (for factors).
    pub fn LEVELS(x: SEXP) -> ::std::os::raw::c_int;

    /// Set the LEVELS field (for factors).
    ///
    /// Returns the value that was set.
    pub fn SETLEVELS(x: SEXP, v: ::std::os::raw::c_int) -> ::std::os::raw::c_int;

    // endregion

    // region: ALTREP support

    pub fn ALTREP_CLASS(x: SEXP) -> SEXP;
    pub fn R_altrep_data1(x: SEXP) -> SEXP;
    pub fn R_altrep_data2(x: SEXP) -> SEXP;
    pub fn R_set_altrep_data1(x: SEXP, v: SEXP);
    pub fn R_set_altrep_data2(x: SEXP, v: SEXP);

    /// Check if a SEXP is an ALTREP object (returns non-zero if true).
    pub fn ALTREP(x: SEXP) -> ::std::os::raw::c_int;

    // endregion

    // region: Vector data accessors (mutable pointers)

    /// Get mutable pointer to logical vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    pub fn LOGICAL(x: SEXP) -> *mut ::std::os::raw::c_int;

    /// Get mutable pointer to integer vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    pub fn INTEGER(x: SEXP) -> *mut ::std::os::raw::c_int;

    /// Get mutable pointer to real vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    pub fn REAL(x: SEXP) -> *mut f64;

    /// Get mutable pointer to complex vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    pub fn COMPLEX(x: SEXP) -> *mut Rcomplex;

    /// Get mutable pointer to raw vector data.
    ///
    /// For ALTREP vectors, this may force materialization.
    pub fn RAW(x: SEXP) -> *mut Rbyte;

    // endregion

    // region: User interrupt and utilities

    // utils.h
    pub fn R_CheckUserInterrupt();

    // endregion

    // region: Type checking

    pub fn TYPEOF(x: SEXP) -> SEXPTYPE;

    // endregion

    // Symbol creation and access
    #[doc(alias = "install")]
    pub fn Rf_install(name: *const ::std::os::raw::c_char) -> SEXP;
    /// Get the print name (CHARSXP) of a symbol (SYMSXP)
    pub fn PRINTNAME(x: SEXP) -> SEXP;
    /// Get the C string pointer from a CHARSXP
    #[doc(alias = "CHAR")]
    pub fn R_CHAR(x: SEXP) -> *const ::std::os::raw::c_char;

    // Attribute access
    /// Read an attribute from an object by symbol (e.g. `R_NamesSymbol`).
    ///
    /// Returns `R_NilValue` if the attribute is not set.
    #[doc(alias = "getAttrib")]
    pub fn Rf_getAttrib(vec: SEXP, name: SEXP) -> SEXP;
    /// Set the `names` attribute; returns the updated object.
    #[doc(alias = "namesgets")]
    pub fn Rf_namesgets(vec: SEXP, val: SEXP) -> SEXP;
    /// Set the `dim` attribute; returns the updated object.
    #[doc(alias = "dimgets")]
    pub fn Rf_dimgets(vec: SEXP, val: SEXP) -> SEXP;

    // Duplication
    #[doc(alias = "duplicate")]
    pub fn Rf_duplicate(s: SEXP) -> SEXP;
    #[doc(alias = "shallow_duplicate")]
    pub fn Rf_shallow_duplicate(s: SEXP) -> SEXP;

    // Object comparison
    /// Check if two R objects are identical (deep semantic equality).
    ///
    /// This is the C implementation of R's `identical()` function.
    ///
    /// # Flags
    ///
    /// Use the `IDENT_*` constants below. Flags are inverted: set bit = disable that check.
    ///
    /// **Default from R**: `IDENT_USE_CLOENV` (16) - ignore closure environments
    ///
    /// # Returns
    ///
    /// `TRUE` if identical, `FALSE` otherwise.
    ///
    /// # Performance
    ///
    /// Fast-path: Returns `TRUE` immediately if pointers are equal.
    pub fn R_compute_identical(x: SEXP, y: SEXP, flags: ::std::os::raw::c_int) -> Rboolean;
}

/// Flags for `R_compute_identical` (bitmask, inverted logic: set bit = disable check).
pub const IDENT_NUM_AS_BITS: ::std::os::raw::c_int = 1;
/// Treat all NAs as identical (ignore NA payload differences).
pub const IDENT_NA_AS_BITS: ::std::os::raw::c_int = 2;
/// Compare attributes in order (not as a set).
pub const IDENT_ATTR_BY_ORDER: ::std::os::raw::c_int = 4;
/// Include bytecode in comparison.
pub const IDENT_USE_BYTECODE: ::std::os::raw::c_int = 8;
/// Include closure environments in comparison.
pub const IDENT_USE_CLOENV: ::std::os::raw::c_int = 16;
/// Include source references in comparison.
pub const IDENT_USE_SRCREF: ::std::os::raw::c_int = 32;
/// Compare external pointers as references (not by address).
pub const IDENT_EXTPTR_AS_REF: ::std::os::raw::c_int = 64;

#[r_ffi_checked]
unsafe extern "C-unwind" {
    // Type coercion
    #[doc(alias = "asLogical")]
    pub fn Rf_asLogical(x: SEXP) -> ::std::os::raw::c_int;
    #[doc(alias = "asInteger")]
    pub fn Rf_asInteger(x: SEXP) -> ::std::os::raw::c_int;
    #[doc(alias = "asReal")]
    pub fn Rf_asReal(x: SEXP) -> f64;
    #[doc(alias = "asChar")]
    pub fn Rf_asChar(x: SEXP) -> SEXP;
    #[doc(alias = "coerceVector")]
    pub fn Rf_coerceVector(v: SEXP, sexptype: SEXPTYPE) -> SEXP;

    // Matrix utilities
    #[doc(alias = "nrows")]
    pub fn Rf_nrows(x: SEXP) -> ::std::os::raw::c_int;
    #[doc(alias = "ncols")]
    pub fn Rf_ncols(x: SEXP) -> ::std::os::raw::c_int;

    // Inheritance checking
    #[doc(alias = "inherits")]
    pub fn Rf_inherits(x: SEXP, klass: *const ::std::os::raw::c_char) -> Rboolean;

    // Type checking predicates
    #[doc(alias = "isNull")]
    pub fn Rf_isNull(s: SEXP) -> Rboolean;
    #[doc(alias = "isSymbol")]
    pub fn Rf_isSymbol(s: SEXP) -> Rboolean;
    #[doc(alias = "isLogical")]
    pub fn Rf_isLogical(s: SEXP) -> Rboolean;
    #[doc(alias = "isReal")]
    pub fn Rf_isReal(s: SEXP) -> Rboolean;
    #[doc(alias = "isComplex")]
    pub fn Rf_isComplex(s: SEXP) -> Rboolean;
    #[doc(alias = "isExpression")]
    pub fn Rf_isExpression(s: SEXP) -> Rboolean;
    #[doc(alias = "isEnvironment")]
    pub fn Rf_isEnvironment(s: SEXP) -> Rboolean;
    #[doc(alias = "isString")]
    pub fn Rf_isString(s: SEXP) -> Rboolean;

    // Composite type checking (from inline functions)
    #[doc(alias = "isArray")]
    pub fn Rf_isArray(s: SEXP) -> Rboolean;
    #[doc(alias = "isMatrix")]
    pub fn Rf_isMatrix(s: SEXP) -> Rboolean;
    #[doc(alias = "isList")]
    pub fn Rf_isList(s: SEXP) -> Rboolean;
    #[doc(alias = "isNewList")]
    pub fn Rf_isNewList(s: SEXP) -> Rboolean;
    #[doc(alias = "isPairList")]
    pub fn Rf_isPairList(s: SEXP) -> Rboolean;
    #[doc(alias = "isFunction")]
    pub fn Rf_isFunction(s: SEXP) -> Rboolean;
    #[doc(alias = "isPrimitive")]
    pub fn Rf_isPrimitive(s: SEXP) -> Rboolean;
    #[doc(alias = "isLanguage")]
    pub fn Rf_isLanguage(s: SEXP) -> Rboolean;
    #[doc(alias = "isDataFrame")]
    pub fn Rf_isDataFrame(s: SEXP) -> Rboolean;
    #[doc(alias = "isFactor")]
    pub fn Rf_isFactor(s: SEXP) -> Rboolean;
    #[doc(alias = "isInteger")]
    pub fn Rf_isInteger(s: SEXP) -> Rboolean;
    #[doc(alias = "isObject")]
    pub fn Rf_isObject(s: SEXP) -> Rboolean;

    // Pairlist utilities
    #[doc(alias = "elt")]
    pub fn Rf_elt(list: SEXP, i: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "lastElt")]
    pub fn Rf_lastElt(list: SEXP) -> SEXP;
    #[doc(alias = "nthcdr")]
    pub fn Rf_nthcdr(list: SEXP, n: ::std::os::raw::c_int) -> SEXP;
    #[doc(alias = "listAppend")]
    pub fn Rf_listAppend(s: SEXP, t: SEXP) -> SEXP;

    // More attribute setters (using R's "gets" suffix convention)
    //
    // See "Attribute access" section above for explanation of the "gets" suffix.
    // These are setter functions equivalent to R's `attr(x) <- value` syntax.

    /// Set the class attribute of a vector.
    ///
    /// Equivalent to R's `class(vec) <- klass` syntax.
    /// The "gets" suffix indicates this is a setter function.
    ///
    /// # Returns
    ///
    /// Returns the modified vector (like all "*gets" functions).
    #[doc(alias = "classgets")]
    pub fn Rf_classgets(vec: SEXP, klass: SEXP) -> SEXP;

    /// Set the dimnames attribute of an array/matrix.
    ///
    /// Equivalent to R's `dimnames(vec) <- val` syntax.
    /// The "gets" suffix indicates this is a setter function.
    ///
    /// # Returns
    ///
    /// Returns the modified vector.
    #[doc(alias = "dimnamesgets")]
    pub fn Rf_dimnamesgets(vec: SEXP, val: SEXP) -> SEXP;
    #[doc(alias = "GetRowNames")]
    pub fn Rf_GetRowNames(dimnames: SEXP) -> SEXP;
    #[doc(alias = "GetColNames")]
    pub fn Rf_GetColNames(dimnames: SEXP) -> SEXP;

    // Environment operations
    #[doc(alias = "findVar")]
    pub fn Rf_findVar(symbol: SEXP, rho: SEXP) -> SEXP;
    #[doc(alias = "findVarInFrame")]
    pub fn Rf_findVarInFrame(rho: SEXP, symbol: SEXP) -> SEXP;
    #[doc(alias = "findVarInFrame3")]
    pub fn Rf_findVarInFrame3(rho: SEXP, symbol: SEXP, doget: Rboolean) -> SEXP;
    #[doc(alias = "defineVar")]
    pub fn Rf_defineVar(symbol: SEXP, value: SEXP, rho: SEXP);
    #[doc(alias = "setVar")]
    pub fn Rf_setVar(symbol: SEXP, value: SEXP, rho: SEXP);
    #[doc(alias = "findFun")]
    pub fn Rf_findFun(symbol: SEXP, rho: SEXP) -> SEXP;

    // Evaluation
    #[doc(alias = "eval")]
    pub fn Rf_eval(expr: SEXP, rho: SEXP) -> SEXP;
    #[doc(alias = "applyClosure")]
    pub fn Rf_applyClosure(
        call: SEXP,
        op: SEXP,
        args: SEXP,
        rho: SEXP,
        suppliedvars: SEXP,
        check: Rboolean,
    ) -> SEXP;
    pub fn R_tryEval(expr: SEXP, env: SEXP, error_occurred: *mut ::std::os::raw::c_int) -> SEXP;
    pub fn R_tryEvalSilent(
        expr: SEXP,
        env: SEXP,
        error_occurred: *mut ::std::os::raw::c_int,
    ) -> SEXP;
    pub fn R_forceAndCall(e: SEXP, n: ::std::os::raw::c_int, rho: SEXP) -> SEXP;
}

// region: Connections API (R_ext/Connections.h)
//
// Gated behind `connections` feature because R's connection API is explicitly UNSTABLE.
// From R_ext/Connections.h:
//   "IMPORTANT: we do not expect future connection APIs to be
//    backward-compatible so if you use this, you *must* check the
//    version and proceeds only if it matches what you expect.
//
//    We explicitly reserve the right to change the connection
//    implementation without a compatibility layer."
//
// Use with caution and always check R_CONNECTIONS_VERSION.
#[r_ffi_checked]
#[cfg(feature = "connections")]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    /// Create a new custom connection.
    ///
    /// # WARNING
    ///
    /// This API is UNSTABLE. Check `R_CONNECTIONS_VERSION` before use.
    /// The connection implementation may change without notice.
    ///
    /// # Safety
    ///
    /// - `description`, `mode`, and `class_name` must be valid C strings
    /// - `ptr` must be a valid pointer to store the connection handle
    pub fn R_new_custom_connection(
        description: *const ::std::os::raw::c_char,
        mode: *const ::std::os::raw::c_char,
        class_name: *const ::std::os::raw::c_char,
        ptr: *mut Rconnection,
    ) -> SEXP;

    /// Read from a connection.
    ///
    /// # WARNING
    ///
    /// This API is UNSTABLE and may change.
    ///
    /// # Safety
    ///
    /// - `con` must be a valid Rconnection handle
    /// - `buf` must be a valid buffer with at least `n` bytes
    pub fn R_ReadConnection(con: Rconnection, buf: *mut ::std::os::raw::c_void, n: usize) -> usize;

    /// Write to a connection.
    ///
    /// # WARNING
    ///
    /// This API is UNSTABLE and may change.
    ///
    /// # Safety
    ///
    /// - `con` must be a valid Rconnection handle
    /// - `buf` must contain at least `n` valid bytes
    pub fn R_WriteConnection(
        con: Rconnection,
        buf: *const ::std::os::raw::c_void,
        n: usize,
    ) -> usize;

    /// Get a connection from a SEXP.
    ///
    /// # WARNING
    ///
    /// This API is UNSTABLE and may change.
    /// Added in R 3.3.0.
    ///
    /// # Safety
    ///
    /// - `sConn` must be a valid connection SEXP
    pub fn R_GetConnection(sConn: SEXP) -> Rconnection;
}
// endregion: Connections API

/// Check if a SEXP is an S4 object.
///
/// # Safety
///
/// - `arg1` must be a valid SEXP
#[allow(non_snake_case)]
pub unsafe fn Rf_isS4(arg1: SEXP) -> Rboolean {
    unsafe extern "C-unwind" {
        #[link_name = "Rf_isS4"]
        pub fn Rf_isS4_original(arg1: SEXP) -> u32;
    }

    unsafe {
        if Rf_isS4_original(arg1) == 0 {
            Rboolean::FALSE
        } else {
            Rboolean::TRUE
        }
    }
}

// region: registration!

#[repr(C)]
#[derive(Debug)]
pub struct DllInfo(::std::os::raw::c_void);

/// Generic dynamic library function pointer.
///
/// R defines this as `void *(*)(void)` - a function taking no arguments and
/// returning `void*`. This is used for method registration and external pointer
/// functions. The actual function signatures vary; callers cast to the appropriate
/// concrete function type before calling.
///
/// We use `fn() -> *mut c_void` to match R's signature. The function pointer is
/// stored generically and cast to the appropriate type when called by R.
#[allow(non_camel_case_types)]
pub type DL_FUNC =
    ::std::option::Option<unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void>;

/// Type descriptor for native primitive arguments in .C/.Fortran calls.
///
/// This is used in `R_CMethodDef` and `R_FortranMethodDef` to specify
/// argument types for type checking.
#[allow(non_camel_case_types)]
pub type R_NativePrimitiveArgType = ::std::os::raw::c_uint;

/// Method definition for .C interface routines.
///
/// Used to register C functions callable via `.C()` from R.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CMethodDef {
    pub name: *const ::std::os::raw::c_char,
    pub fun: DL_FUNC,
    pub numArgs: ::std::os::raw::c_int,
    /// Optional array of argument types for type checking. May be null.
    pub types: *const R_NativePrimitiveArgType,
}

/// Method definition for .Fortran interface routines.
///
/// Structurally identical to `R_CMethodDef`.
#[allow(non_camel_case_types)]
pub type R_FortranMethodDef = R_CMethodDef;

/// Method definition for .Call interface routines.
///
/// Used to register C functions callable via `.Call()` from R.
/// Unlike `.C()` routines, `.Call()` functions receive and return SEXP values directly.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CallMethodDef {
    pub name: *const ::std::os::raw::c_char,
    pub fun: DL_FUNC,
    pub numArgs: ::std::os::raw::c_int,
}

/// Method definition for .External interface routines.
///
/// Structurally identical to `R_CallMethodDef`.
#[allow(non_camel_case_types)]
pub type R_ExternalMethodDef = R_CallMethodDef;

#[r_ffi_checked]
#[allow(clashing_extern_declarations)]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    pub fn R_registerRoutines(
        info: *mut DllInfo,
        croutines: *const R_CMethodDef,
        callRoutines: *const R_CallMethodDef,
        fortranRoutines: *const R_FortranMethodDef,
        externalRoutines: *const R_ExternalMethodDef,
    ) -> ::std::os::raw::c_int;

    pub fn R_useDynamicSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
    pub fn R_forceSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
}

// endregion

// region: Legacy `extern "C"` types (kept for compatibility testing)

/// Legacy types using `extern "C"` ABI instead of `extern "C-unwind"`.
///
/// These are kept for compatibility testing. The main codebase uses
/// `extern "C-unwind"` everywhere to properly propagate Rust panics.
#[allow(clashing_extern_declarations)]
pub mod legacy_c {
    use super::{Rboolean, SEXP, r_ffi_checked};

    #[allow(non_camel_case_types)]
    pub type R_CFinalizer_t_C = ::std::option::Option<unsafe extern "C" fn(s: SEXP)>;

    #[allow(non_camel_case_types)]
    pub type DL_FUNC_C =
        ::std::option::Option<unsafe extern "C" fn() -> *mut ::std::os::raw::c_void>;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    #[allow(non_camel_case_types)]
    #[allow(non_snake_case)]
    pub struct R_CallMethodDef_C {
        pub name: *const ::std::os::raw::c_char,
        pub fun: DL_FUNC_C,
        pub numArgs: ::std::os::raw::c_int,
    }

    #[r_ffi_checked]
    unsafe extern "C" {
        #[link_name = "R_RegisterCFinalizer"]
        pub fn R_RegisterCFinalizer_C(s: SEXP, fun: R_CFinalizer_t_C);

        #[link_name = "R_RegisterCFinalizerEx"]
        pub fn R_RegisterCFinalizerEx_C(s: SEXP, fun: R_CFinalizer_t_C, onexit: Rboolean);

        #[link_name = "R_MakeExternalPtrFn"]
        pub fn R_MakeExternalPtrFn_C(p: DL_FUNC_C, tag: SEXP, prot: SEXP) -> SEXP;

        #[link_name = "R_ExternalPtrAddrFn"]
        pub fn R_ExternalPtrAddrFn_C(s: SEXP) -> DL_FUNC_C;

        #[link_name = "R_registerRoutines"]
        pub fn R_registerRoutines_C(
            info: *mut super::DllInfo,
            croutines: *const ::std::os::raw::c_void,
            callRoutines: *const R_CallMethodDef_C,
            fortranRoutines: *const ::std::os::raw::c_void,
            externalRoutines: *const ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int;
    }
}

// endregion

// region: Non-API encoding/locale state (Defn.h)

/// Non-API encoding / locale helpers from R's `Defn.h`.
///
/// These are not part of the stable R API and may break across R versions.
#[cfg(feature = "nonapi")]
pub mod nonapi_encoding {
    use super::r_ffi_checked;

    #[r_ffi_checked]
    #[allow(clashing_extern_declarations)]
    #[allow(non_snake_case)]
    unsafe extern "C-unwind" {
        pub fn R_nativeEncoding() -> *const ::std::os::raw::c_char;

        // Locale flags
        pub static utf8locale: super::Rboolean;
        pub static latin1locale: super::Rboolean;

        // Set when R "knows" it is running in UTF-8.
        pub static known_to_be_utf8: super::Rboolean;
    }
}

// endregion

// region: Non-API stack checking variables (Rinterface.h)

/// Non-API stack checking variables from `Rinterface.h`.
///
/// R uses these to detect stack overflow. When calling R from a thread other
/// than the main R thread, stack checking will fail because these values are
/// set for the main thread's stack.
///
/// # Usage
///
/// To safely call R from a worker thread, disable stack checking:
/// ```ignore
/// #[cfg(feature = "nonapi")]
/// unsafe {
///     use miniextendr_api::ffi::nonapi_stack::*;
///     let saved = get_r_cstack_limit();
///     set_r_cstack_limit(usize::MAX); // disable checking
///     // ... call R APIs ...
///     set_r_cstack_limit(saved); // restore
/// }
/// ```
///
/// Or use the higher-level [`StackCheckGuard`](crate::thread::StackCheckGuard) which handles this automatically.
///
/// Setting `R_CStackLimit` to `usize::MAX` (i.e., `-1` as `uintptr_t`) disables
/// stack checking entirely.
#[cfg(feature = "nonapi")]
pub mod nonapi_stack {
    unsafe extern "C" {
        /// Top of the stack (set during `Rf_initialize_R` for main thread).
        ///
        /// On Unix, determined via `__libc_stack_end`, `KERN_USRSTACK`, or
        /// `thr_stksegment`. On Windows, via `VirtualQuery`.
        #[allow(non_upper_case_globals)]
        pub static R_CStackStart: usize;

        /// Stack size limit. Set to `usize::MAX` to disable stack checking.
        ///
        /// From R source: `if(R_CStackStart == -1) R_CStackLimit = -1; /* never set */`
        #[allow(non_upper_case_globals)]
        pub static R_CStackLimit: usize;

        /// Stack growth direction: 1 = grows up, -1 = grows down.
        ///
        /// Most systems (x86, ARM) grow down (-1).
        #[allow(non_upper_case_globals)]
        pub static R_CStackDir: ::std::os::raw::c_int;
    }

    /// Write to `R_CStackLimit`.
    ///
    /// # Safety
    /// Must be called from R's main thread.
    #[inline]
    pub unsafe fn set_r_cstack_limit(value: usize) {
        unsafe {
            let ptr = &raw const R_CStackLimit as *mut usize;
            ptr.write(value);
        }
    }

    /// Read `R_CStackLimit`.
    #[inline]
    pub fn get_r_cstack_limit() -> usize {
        unsafe { R_CStackLimit }
    }

    /// Read `R_CStackStart`.
    #[inline]
    pub fn get_r_cstack_start() -> usize {
        unsafe { R_CStackStart }
    }

    /// Read `R_CStackDir`.
    #[inline]
    pub fn get_r_cstack_dir() -> ::std::os::raw::c_int {
        unsafe { R_CStackDir }
    }
}

// endregion

// region: Inline Helper Functions (Rust implementations of R's inline functions)

/// Create a length-1 string vector from a C string.
///
/// Rust equivalent of R's inline `Rf_mkString(s)`, which is
/// shorthand for `ScalarString(mkChar(s))`.
///
/// # Safety
///
/// - `s` must be a valid null-terminated C string
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "mkString")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_mkString(s: *const ::std::os::raw::c_char) -> SEXP {
    unsafe {
        let charsxp = Rf_mkChar(s);
        let protected = Rf_protect(charsxp);
        let result = Rf_ScalarString(protected);
        Rf_unprotect(1);
        result
    }
}

/// Build a pairlist with 1 element.
///
/// Rust equivalent of R's inline `Rf_list1(s)`.
///
/// # Safety
///
/// - `s` must be a valid SEXP
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "list1")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_list1(s: SEXP) -> SEXP {
    unsafe { Rf_cons(s, R_NilValue) }
}

/// Build a pairlist with 2 elements.
///
/// Rust equivalent of R's inline `Rf_list2(s, t)`.
///
/// # Safety
///
/// - Both SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "list2")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_list2(s: SEXP, t: SEXP) -> SEXP {
    unsafe { Rf_cons(s, Rf_cons(t, R_NilValue)) }
}

/// Build a pairlist with 3 elements.
///
/// Rust equivalent of R's inline `Rf_list3(s, t, u)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "list3")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_list3(s: SEXP, t: SEXP, u: SEXP) -> SEXP {
    unsafe { Rf_cons(s, Rf_cons(t, Rf_cons(u, R_NilValue))) }
}

/// Build a pairlist with 4 elements.
///
/// Rust equivalent of R's inline `Rf_list4(s, t, u, v)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "list4")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_list4(s: SEXP, t: SEXP, u: SEXP, v: SEXP) -> SEXP {
    unsafe { Rf_cons(s, Rf_cons(t, Rf_cons(u, Rf_cons(v, R_NilValue)))) }
}

/// Check if a SEXP is a numeric type (integer, logical, or real, excluding factors).
///
/// Rust equivalent of R's inline `Rf_isNumeric()`.
///
/// # Safety
///
/// - `x` must be a valid SEXP
/// - Must be called from R's main thread
#[doc(alias = "isNumeric")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_isNumeric(x: SEXP) -> bool {
    unsafe {
        let typ = TYPEOF(x);
        (typ == SEXPTYPE::INTSXP || typ == SEXPTYPE::LGLSXP || typ == SEXPTYPE::REALSXP)
            && Rf_inherits(x, c"factor".as_ptr()) == Rboolean::FALSE
    }
}

/// Check if a SEXP is a number type (numeric or complex).
///
/// Rust equivalent of R's inline `Rf_isNumber()`.
///
/// # Safety
///
/// - `x` must be a valid SEXP
/// - Must be called from R's main thread
#[doc(alias = "isNumber")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_isNumber(x: SEXP) -> bool {
    unsafe { Rf_isNumeric(x) || TYPEOF(x) == SEXPTYPE::CPLXSXP }
}

/// Check if a SEXP is an atomic vector.
///
/// Rust equivalent of R's inline `Rf_isVectorAtomic()`.
/// Returns true for logical, integer, real, complex, character, and raw vectors.
///
/// # Safety
///
/// - `x` must be a valid SEXP
/// - Must be called from R's main thread
#[doc(alias = "isVectorAtomic")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_isVectorAtomic(x: SEXP) -> bool {
    unsafe {
        let typ = TYPEOF(x);
        matches!(
            typ,
            SEXPTYPE::LGLSXP
                | SEXPTYPE::INTSXP
                | SEXPTYPE::REALSXP
                | SEXPTYPE::CPLXSXP
                | SEXPTYPE::STRSXP
                | SEXPTYPE::RAWSXP
        )
    }
}

/// Check if a SEXP is a vector list (VECSXP or EXPRSXP).
///
/// Rust equivalent of R's inline `Rf_isVectorList()`.
///
/// # Safety
///
/// - `x` must be a valid SEXP
/// - Must be called from R's main thread
#[doc(alias = "isVectorList")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_isVectorList(x: SEXP) -> bool {
    unsafe {
        let typ = TYPEOF(x);
        typ == SEXPTYPE::VECSXP || typ == SEXPTYPE::EXPRSXP
    }
}

/// Check if a SEXP is a vector (atomic vector or list).
///
/// Rust equivalent of R's inline `Rf_isVector()`.
///
/// # Safety
///
/// - `x` must be a valid SEXP
/// - Must be called from R's main thread
#[doc(alias = "isVector")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_isVector(x: SEXP) -> bool {
    unsafe { Rf_isVectorAtomic(x) || Rf_isVectorList(x) }
}

/// Build a language object (call) with 1 element (the function).
///
/// Rust equivalent of R's inline `Rf_lang1(s)`.
/// Creates a call like `f()` where `s` is the function.
///
/// # Safety
///
/// - `s` must be a valid SEXP (typically a symbol or closure)
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang1")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang1(s: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, R_NilValue) }
}

/// Build a language object (call) with function and 1 argument.
///
/// Rust equivalent of R's inline `Rf_lang2(s, t)`.
/// Creates a call like `f(arg)` where `s` is the function and `t` is the argument.
///
/// # Safety
///
/// - Both SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang2")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang2(s: SEXP, t: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, Rf_list1(t)) }
}

/// Build a language object (call) with function and 2 arguments.
///
/// Rust equivalent of R's inline `Rf_lang3(s, t, u)`.
/// Creates a call like `f(arg1, arg2)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang3")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang3(s: SEXP, t: SEXP, u: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, Rf_list2(t, u)) }
}

/// Build a language object (call) with function and 3 arguments.
///
/// Rust equivalent of R's inline `Rf_lang4(s, t, u, v)`.
/// Creates a call like `f(arg1, arg2, arg3)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang4")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang4(s: SEXP, t: SEXP, u: SEXP, v: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, Rf_list3(t, u, v)) }
}

/// Build a language object (call) with function and 4 arguments.
///
/// Rust equivalent of R's inline `Rf_lang5(s, t, u, v, w)`.
/// Creates a call like `f(arg1, arg2, arg3, arg4)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang5")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang5(s: SEXP, t: SEXP, u: SEXP, v: SEXP, w: SEXP) -> SEXP {
    unsafe { Rf_lcons(s, Rf_list4(t, u, v, w)) }
}

/// Build a language object (call) with function and 5 arguments.
///
/// Rust equivalent of R's inline `Rf_lang6(s, t, u, v, w, x)`.
/// Creates a call like `f(arg1, arg2, arg3, arg4, arg5)`.
///
/// # Safety
///
/// - All SEXPs must be valid
/// - Must be called from R's main thread
/// - Result must be protected from GC
#[doc(alias = "lang6")]
#[allow(non_snake_case)]
#[inline]
pub unsafe fn Rf_lang6(s: SEXP, t: SEXP, u: SEXP, v: SEXP, w: SEXP, x: SEXP) -> SEXP {
    unsafe {
        let protected = Rf_protect(s);
        let list = Rf_cons(t, Rf_list4(u, v, w, x));
        let result = Rf_lcons(protected, list);
        Rf_unprotect(1);
        result
    }
}

// endregion

// region: RNG functions (R_ext/Random.h)

/// RNG type enum from R_ext/Random.h
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum RNGtype {
    WICHMANN_HILL = 0,
    MARSAGLIA_MULTICARRY = 1,
    SUPER_DUPER = 2,
    MERSENNE_TWISTER = 3,
    KNUTH_TAOCP = 4,
    USER_UNIF = 5,
    KNUTH_TAOCP2 = 6,
    LECUYER_CMRG = 7,
}

/// Normal distribution generator type enum from R_ext/Random.h
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum N01type {
    BUGGY_KINDERMAN_RAMAGE = 0,
    AHRENS_DIETER = 1,
    BOX_MULLER = 2,
    USER_NORM = 3,
    INVERSION = 4,
    KINDERMAN_RAMAGE = 5,
}

/// Discrete uniform sample method enum from R_ext/Random.h
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Sampletype {
    ROUNDING = 0,
    REJECTION = 1,
}

#[r_ffi_checked]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    /// Save the current RNG state from R's global state.
    ///
    /// Must be called before using `unif_rand()`, `norm_rand()`, etc.
    /// The state is restored with `PutRNGstate()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe {
    ///     GetRNGstate();
    ///     let x = unif_rand();
    ///     let y = norm_rand();
    ///     PutRNGstate();
    /// }
    /// ```
    pub fn GetRNGstate();

    /// Restore the RNG state to R's global state.
    ///
    /// Must be called after using `unif_rand()`, `norm_rand()`, etc.
    /// to ensure R's `.Random.seed` is updated.
    pub fn PutRNGstate();

    /// Generate a uniform random number in (0, 1).
    ///
    /// # Important
    ///
    /// Must call `GetRNGstate()` before and `PutRNGstate()` after.
    pub fn unif_rand() -> f64;

    /// Generate a standard normal random number (mean 0, sd 1).
    ///
    /// # Important
    ///
    /// Must call `GetRNGstate()` before and `PutRNGstate()` after.
    pub fn norm_rand() -> f64;

    /// Generate an exponential random number with rate 1.
    ///
    /// # Important
    ///
    /// Must call `GetRNGstate()` before and `PutRNGstate()` after.
    pub fn exp_rand() -> f64;

    /// Generate a uniform random index in [0, dn).
    ///
    /// Used for sampling without bias for large n.
    ///
    /// # Important
    ///
    /// Must call `GetRNGstate()` before and `PutRNGstate()` after.
    pub fn R_unif_index(dn: f64) -> f64;

    /// Get the current discrete uniform sample method.
    pub fn R_sample_kind() -> Sampletype;
}

// endregion

// region: Memory allocation (R_ext/Memory.h)

#[r_ffi_checked]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    /// Get the current R memory stack watermark.
    ///
    /// Use with `vmaxset()` to restore memory stack state.
    /// Memory allocated with `R_alloc()` between `vmaxget()` and `vmaxset()`
    /// will be freed when `vmaxset()` is called.
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe {
    ///     let watermark = vmaxget();
    ///     let buf = R_alloc(100, 1);
    ///     // ... use buf ...
    ///     vmaxset(watermark); // frees buf
    /// }
    /// ```
    pub fn vmaxget() -> *mut ::std::os::raw::c_void;

    /// Set the R memory stack watermark, freeing memory allocated since the mark.
    ///
    /// # Safety
    ///
    /// `ovmax` must be a value returned by `vmaxget()` called earlier in the
    /// same R evaluation context.
    pub fn vmaxset(ovmax: *const ::std::os::raw::c_void);

    /// Run the R garbage collector.
    ///
    /// Forces a full garbage collection cycle.
    pub fn R_gc();

    /// Check if the garbage collector is currently running.
    ///
    /// Returns non-zero if GC is in progress.
    pub fn R_gc_running() -> ::std::os::raw::c_int;

    /// Allocate memory on R's memory stack.
    ///
    /// This memory is automatically freed when the calling R function returns,
    /// or can be freed earlier with `vmaxset()`.
    ///
    /// # Parameters
    ///
    /// - `nelem`: Number of elements to allocate
    /// - `eltsize`: Size of each element in bytes
    ///
    /// # Returns
    ///
    /// Pointer to allocated memory (as `char*` for compatibility with S).
    pub fn R_alloc(nelem: usize, eltsize: ::std::os::raw::c_int) -> *mut ::std::os::raw::c_char;

    /// Allocate an array of long doubles on R's memory stack.
    ///
    /// # Parameters
    ///
    /// - `nelem`: Number of long double elements to allocate
    pub fn R_allocLD(nelem: usize) -> *mut f64; // Note: f64 is close enough for most uses

    /// S compatibility: allocate zeroed memory on R's memory stack.
    ///
    /// # Parameters
    ///
    /// - `nelem`: Number of elements
    /// - `eltsize`: Size of each element
    pub fn S_alloc(
        nelem: ::std::os::raw::c_long,
        eltsize: ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;

    /// S compatibility: reallocate memory on R's memory stack.
    ///
    /// # Safety
    ///
    /// `ptr` must have been allocated by `S_alloc`.
    pub fn S_realloc(
        ptr: *mut ::std::os::raw::c_char,
        newsize: ::std::os::raw::c_long,
        oldsize: ::std::os::raw::c_long,
        eltsize: ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;

    /// GC-aware malloc.
    ///
    /// Triggers GC if allocation fails, then retries.
    /// Memory must be freed with `free()`.
    pub fn R_malloc_gc(size: usize) -> *mut ::std::os::raw::c_void;

    /// GC-aware calloc.
    ///
    /// Triggers GC if allocation fails, then retries.
    /// Memory must be freed with `free()`.
    pub fn R_calloc_gc(nelem: usize, eltsize: usize) -> *mut ::std::os::raw::c_void;

    /// GC-aware realloc.
    ///
    /// Triggers GC if allocation fails, then retries.
    /// Memory must be freed with `free()`.
    pub fn R_realloc_gc(
        ptr: *mut ::std::os::raw::c_void,
        size: usize,
    ) -> *mut ::std::os::raw::c_void;
}

// endregion

// region: Sorting and utility functions (R_ext/Utils.h)

#[r_ffi_checked]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    /// Sort an integer vector in place (ascending order).
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to integer array
    /// - `n`: Number of elements
    pub fn R_isort(x: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int);

    /// Sort a double vector in place (ascending order).
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to double array
    /// - `n`: Number of elements
    pub fn R_rsort(x: *mut f64, n: ::std::os::raw::c_int);

    /// Sort a complex vector in place.
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to Rcomplex array
    /// - `n`: Number of elements
    pub fn R_csort(x: *mut Rcomplex, n: ::std::os::raw::c_int);

    /// Sort doubles in descending order, carrying along an index array.
    ///
    /// # Parameters
    ///
    /// - `a`: Pointer to double array (sorted in place, descending)
    /// - `ib`: Pointer to integer array (permuted alongside `a`)
    /// - `n`: Number of elements
    #[doc(alias = "Rf_revsort")]
    pub fn revsort(a: *mut f64, ib: *mut ::std::os::raw::c_int, n: ::std::os::raw::c_int);

    /// Sort doubles with index array.
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to double array (sorted in place)
    /// - `indx`: Pointer to integer array (permuted alongside `x`)
    /// - `n`: Number of elements
    pub fn rsort_with_index(
        x: *mut f64,
        indx: *mut ::std::os::raw::c_int,
        n: ::std::os::raw::c_int,
    );

    /// Partial sort integers (moves k-th smallest to position k).
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to integer array
    /// - `n`: Number of elements
    /// - `k`: Target position (0-indexed)
    #[doc(alias = "Rf_iPsort")]
    pub fn iPsort(
        x: *mut ::std::os::raw::c_int,
        n: ::std::os::raw::c_int,
        k: ::std::os::raw::c_int,
    );

    /// Partial sort doubles (moves k-th smallest to position k).
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to double array
    /// - `n`: Number of elements
    /// - `k`: Target position (0-indexed)
    #[doc(alias = "Rf_rPsort")]
    pub fn rPsort(x: *mut f64, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int);

    /// Partial sort complex numbers.
    ///
    /// # Parameters
    ///
    /// - `x`: Pointer to Rcomplex array
    /// - `n`: Number of elements
    /// - `k`: Target position (0-indexed)
    #[doc(alias = "Rf_cPsort")]
    pub fn cPsort(x: *mut Rcomplex, n: ::std::os::raw::c_int, k: ::std::os::raw::c_int);

    /// Quicksort doubles in place.
    ///
    /// # Parameters
    ///
    /// - `v`: Pointer to double array
    /// - `i`: Start index (1-indexed for R compatibility)
    /// - `j`: End index (1-indexed)
    pub fn R_qsort(v: *mut f64, i: usize, j: usize);

    /// Quicksort doubles with index array.
    ///
    /// # Parameters
    ///
    /// - `v`: Pointer to double array
    /// - `indx`: Pointer to index array (permuted alongside v)
    /// - `i`: Start index (1-indexed)
    /// - `j`: End index (1-indexed)
    #[allow(non_snake_case)]
    pub fn R_qsort_I(
        v: *mut f64,
        indx: *mut ::std::os::raw::c_int,
        i: ::std::os::raw::c_int,
        j: ::std::os::raw::c_int,
    );

    /// Quicksort integers in place.
    ///
    /// # Parameters
    ///
    /// - `iv`: Pointer to integer array
    /// - `i`: Start index (1-indexed)
    /// - `j`: End index (1-indexed)
    pub fn R_qsort_int(iv: *mut ::std::os::raw::c_int, i: usize, j: usize);

    /// Quicksort integers with index array.
    ///
    /// # Parameters
    ///
    /// - `iv`: Pointer to integer array
    /// - `indx`: Pointer to index array
    /// - `i`: Start index (1-indexed)
    /// - `j`: End index (1-indexed)
    #[allow(non_snake_case)]
    pub fn R_qsort_int_I(
        iv: *mut ::std::os::raw::c_int,
        indx: *mut ::std::os::raw::c_int,
        i: ::std::os::raw::c_int,
        j: ::std::os::raw::c_int,
    );

    /// Expand a filename, resolving `~` and environment variables.
    ///
    /// # Returns
    ///
    /// Pointer to expanded path (in R's internal buffer, do not free).
    pub fn R_ExpandFileName(s: *const ::std::os::raw::c_char) -> *const ::std::os::raw::c_char;

    /// Convert string to double, always using '.' as decimal point.
    ///
    /// Also accepts "NA" as input, returning NA_REAL.
    pub fn R_atof(str: *const ::std::os::raw::c_char) -> f64;

    /// Convert string to double with end pointer, using '.' as decimal point.
    ///
    /// Like `strtod()` but locale-independent.
    pub fn R_strtod(c: *const ::std::os::raw::c_char, end: *mut *mut ::std::os::raw::c_char)
    -> f64;

    /// Generate a temporary filename.
    ///
    /// # Parameters
    ///
    /// - `prefix`: Filename prefix
    /// - `tempdir`: Directory for temp file
    ///
    /// # Returns
    ///
    /// Newly allocated string (must be freed with `R_free_tmpnam`).
    pub fn R_tmpnam(
        prefix: *const ::std::os::raw::c_char,
        tempdir: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;

    /// Generate a temporary filename with extension.
    ///
    /// # Parameters
    ///
    /// - `prefix`: Filename prefix
    /// - `tempdir`: Directory for temp file
    /// - `fileext`: File extension (e.g., ".txt")
    ///
    /// # Returns
    ///
    /// Newly allocated string (must be freed with `R_free_tmpnam`).
    pub fn R_tmpnam2(
        prefix: *const ::std::os::raw::c_char,
        tempdir: *const ::std::os::raw::c_char,
        fileext: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;

    /// Free a temporary filename allocated by `R_tmpnam` or `R_tmpnam2`.
    pub fn R_free_tmpnam(name: *mut ::std::os::raw::c_char);

    /// Check for R stack overflow.
    ///
    /// Throws an R error if stack is nearly exhausted.
    pub fn R_CheckStack();

    /// Check for R stack overflow with extra space requirement.
    ///
    /// # Parameters
    ///
    /// - `extra`: Additional bytes needed
    pub fn R_CheckStack2(extra: usize);

    /// Find the interval containing a value (binary search).
    ///
    /// Used for interpolation and binning.
    ///
    /// # Parameters
    ///
    /// - `xt`: Sorted breakpoints array
    /// - `n`: Number of breakpoints
    /// - `x`: Value to find
    /// - `rightmost_closed`: If TRUE, rightmost interval is closed
    /// - `all_inside`: If TRUE, out-of-bounds values map to endpoints
    /// - `ilo`: Initial guess for interval (1-indexed)
    /// - `mflag`: Output flag (see R documentation)
    ///
    /// # Returns
    ///
    /// Interval index (1-indexed).
    pub fn findInterval(
        xt: *const f64,
        n: ::std::os::raw::c_int,
        x: f64,
        rightmost_closed: Rboolean,
        all_inside: Rboolean,
        ilo: ::std::os::raw::c_int,
        mflag: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;

    /// Extended interval finding with left-open option.
    #[allow(clippy::too_many_arguments)]
    pub fn findInterval2(
        xt: *const f64,
        n: ::std::os::raw::c_int,
        x: f64,
        rightmost_closed: Rboolean,
        all_inside: Rboolean,
        left_open: Rboolean,
        ilo: ::std::os::raw::c_int,
        mflag: *mut ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;

    /// Find column maxima in a matrix.
    ///
    /// # Parameters
    ///
    /// - `matrix`: Column-major matrix data
    /// - `nr`: Number of rows
    /// - `nc`: Number of columns
    /// - `maxes`: Output array for column maxima indices (1-indexed)
    /// - `ties_meth`: How to handle ties (1=first, 2=random, 3=last)
    pub fn R_max_col(
        matrix: *const f64,
        nr: *const ::std::os::raw::c_int,
        nc: *const ::std::os::raw::c_int,
        maxes: *mut ::std::os::raw::c_int,
        ties_meth: *const ::std::os::raw::c_int,
    );

    /// Check if a string represents FALSE in R.
    ///
    /// Recognizes "FALSE", "false", "False", "F", "f", etc.
    #[doc(alias = "Rf_StringFalse")]
    pub fn StringFalse(s: *const ::std::os::raw::c_char) -> Rboolean;

    /// Check if a string represents TRUE in R.
    ///
    /// Recognizes "TRUE", "true", "True", "T", "t", etc.
    #[doc(alias = "Rf_StringTrue")]
    pub fn StringTrue(s: *const ::std::os::raw::c_char) -> Rboolean;

    /// Check if a string is blank (empty or only whitespace).
    #[doc(alias = "Rf_isBlankString")]
    pub fn isBlankString(s: *const ::std::os::raw::c_char) -> Rboolean;
}

// endregion

// region: Additional Rinternals.h functions

#[r_ffi_checked]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    // String/character functions

    /// Create a CHARSXP with specified encoding.
    ///
    /// # Parameters
    ///
    /// - `s`: C string
    /// - `encoding`: Character encoding (CE_UTF8, CE_LATIN1, etc.)
    #[doc(alias = "mkCharCE")]
    pub fn Rf_mkCharCE(s: *const ::std::os::raw::c_char, encoding: cetype_t) -> SEXP;

    /// Get the number of characters in a string/character.
    ///
    /// # Parameters
    ///
    /// - `x`: A string SEXP
    /// - `ntype`: Type of count (0=bytes, 1=chars, 2=width)
    /// - `allowNA`: Whether to allow NA values
    /// - `keepNA`: Whether to keep NA in result
    /// - `msg_name`: Name for error messages
    ///
    /// # Returns
    ///
    /// Character count or -1 on error.
    pub fn R_nchar(
        x: SEXP,
        ntype: ::std::os::raw::c_int,
        allowNA: Rboolean,
        keepNA: Rboolean,
        msg_name: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;

    /// Convert SEXPTYPE to C string name.
    ///
    /// Returns a string like "INTSXP", "REALSXP", etc.
    #[doc(alias = "type2char")]
    pub fn Rf_type2char(sexptype: SEXPTYPE) -> *const ::std::os::raw::c_char;

    /// Print an R value to the console.
    ///
    /// Uses R's standard print method for the object.
    #[doc(alias = "PrintValue")]
    pub fn Rf_PrintValue(x: SEXP);

    // Environment functions

    /// Create a new environment.
    ///
    /// # Parameters
    ///
    /// - `enclos`: Enclosing environment
    /// - `hash`: Whether to use a hash table
    /// - `size`: Initial hash table size (if hash is TRUE)
    pub fn R_NewEnv(enclos: SEXP, hash: Rboolean, size: ::std::os::raw::c_int) -> SEXP;

    /// Check if a variable exists in an environment frame.
    ///
    /// Does not search enclosing environments.
    pub fn R_existsVarInFrame(rho: SEXP, symbol: SEXP) -> Rboolean;

    /// Remove a variable from an environment frame.
    ///
    /// # Returns
    ///
    /// The removed value, or R_NilValue if not found.
    pub fn R_removeVarFromFrame(symbol: SEXP, env: SEXP) -> SEXP;

    /// Get the top-level environment.
    ///
    /// Walks up enclosing environments until reaching a top-level env
    /// (global, namespace, or base).
    #[doc(alias = "topenv")]
    pub fn Rf_topenv(target: SEXP, envir: SEXP) -> SEXP;

    // Matching functions

    /// Match elements of first vector in second vector.
    ///
    /// Like R's `match()` function.
    ///
    /// # Parameters
    ///
    /// - `x`: Vector of values to match
    /// - `table`: Vector to match against
    /// - `nomatch`: Value to return for non-matches
    ///
    /// # Returns
    ///
    /// Integer vector of match positions (1-indexed, nomatch for non-matches).
    #[doc(alias = "match")]
    pub fn Rf_match(x: SEXP, table: SEXP, nomatch: ::std::os::raw::c_int) -> SEXP;

    // Duplication and copying

    /// Copy most attributes from source to target.
    ///
    /// Copies all attributes except names, dim, and dimnames.
    #[doc(alias = "copyMostAttrib")]
    pub fn Rf_copyMostAttrib(source: SEXP, target: SEXP);

    /// Find first duplicated element.
    ///
    /// # Parameters
    ///
    /// - `x`: Vector to search
    /// - `fromLast`: If TRUE, search from end
    ///
    /// # Returns
    ///
    /// 0 if no duplicates, otherwise 1-indexed position of first duplicate.
    #[doc(alias = "any_duplicated")]
    pub fn Rf_any_duplicated(x: SEXP, fromLast: Rboolean) -> R_xlen_t;

    // S4 functions

    /// Convert to an S4 object.
    ///
    /// # Parameters
    ///
    /// - `object`: Object to convert
    /// - `flag`: Conversion flag
    #[doc(alias = "asS4")]
    pub fn Rf_asS4(object: SEXP, flag: Rboolean, complete: ::std::os::raw::c_int) -> SEXP;

    /// Get the S3 class of an S4 object.
    #[doc(alias = "S3Class")]
    pub fn Rf_S3Class(object: SEXP) -> SEXP;

    // Option access

    /// Get an R option value.
    ///
    /// Equivalent to `getOption("name")` in R.
    ///
    /// # Parameters
    ///
    /// - `tag`: Symbol for option name
    #[doc(alias = "GetOption1")]
    pub fn Rf_GetOption1(tag: SEXP) -> SEXP;

    /// Get the `digits` option.
    ///
    /// Returns the value of `getOption("digits")`.
    #[doc(alias = "GetOptionDigits")]
    pub fn Rf_GetOptionDigits() -> ::std::os::raw::c_int;

    /// Get the `width` option.
    ///
    /// Returns the value of `getOption("width")`.
    #[doc(alias = "GetOptionWidth")]
    pub fn Rf_GetOptionWidth() -> ::std::os::raw::c_int;

    // Factor functions

    /// Check if a factor is ordered.
    #[doc(alias = "isOrdered")]
    pub fn Rf_isOrdered(s: SEXP) -> Rboolean;

    /// Check if a factor is unordered.
    #[doc(alias = "isUnordered")]
    pub fn Rf_isUnordered(s: SEXP) -> Rboolean;

    /// Check if a vector is unsorted.
    ///
    /// # Parameters
    ///
    /// - `x`: Vector to check
    /// - `strictly`: If TRUE, check for strictly increasing
    #[doc(alias = "isUnsorted")]
    pub fn Rf_isUnsorted(x: SEXP, strictly: Rboolean) -> ::std::os::raw::c_int;

    // Expression and evaluation

    /// Substitute in an expression.
    ///
    /// Like R's `substitute()` function.
    #[doc(alias = "substitute")]
    pub fn Rf_substitute(lang: SEXP, rho: SEXP) -> SEXP;

    /// Set vector length.
    ///
    /// For short vectors (length < 2^31).
    #[doc(alias = "lengthgets")]
    pub fn Rf_lengthgets(x: SEXP, newlen: R_xlen_t) -> SEXP;

    /// Set vector length (long vector version).
    #[doc(alias = "xlengthgets")]
    pub fn Rf_xlengthgets(x: SEXP, newlen: R_xlen_t) -> SEXP;

    // Protection

    /// Protect with saved index for later reprotection.
    ///
    /// # Parameters
    ///
    /// - `s`: SEXP to protect
    /// - `index`: Output parameter for protection index
    #[doc(alias = "PROTECT_WITH_INDEX")]
    pub fn R_ProtectWithIndex(s: SEXP, index: *mut ::std::os::raw::c_int);

    /// Reprotect a SEXP using a saved index.
    ///
    /// Allows updating a protected slot without unprotecting.
    ///
    /// # Safety
    ///
    /// `index` must be from a previous `R_ProtectWithIndex` call.
    #[doc(alias = "REPROTECT")]
    pub fn R_Reprotect(s: SEXP, index: ::std::os::raw::c_int);

    // Weak references

    /// Create a weak reference.
    ///
    /// # Parameters
    ///
    /// - `key`: The key object (weak reference target)
    /// - `val`: The value to associate
    /// - `fin`: Finalizer function (or R_NilValue)
    /// - `onexit`: Whether to run finalizer on R exit
    pub fn R_MakeWeakRef(key: SEXP, val: SEXP, fin: SEXP, onexit: Rboolean) -> SEXP;

    /// Create a weak reference with C finalizer.
    pub fn R_MakeWeakRefC(key: SEXP, val: SEXP, fin: R_CFinalizer_t, onexit: Rboolean) -> SEXP;

    /// Get the key from a weak reference.
    pub fn R_WeakRefKey(w: SEXP) -> SEXP;

    /// Get the value from a weak reference.
    pub fn R_WeakRefValue(w: SEXP) -> SEXP;

    /// Run pending finalizers.
    pub fn R_RunPendingFinalizers();

    // Conversion list/vector

    /// Convert a pairlist to a generic vector (list).
    #[doc(alias = "PairToVectorList")]
    pub fn Rf_PairToVectorList(x: SEXP) -> SEXP;

    /// Convert a generic vector (list) to a pairlist.
    #[doc(alias = "VectorToPairList")]
    pub fn Rf_VectorToPairList(x: SEXP) -> SEXP;

    // Install with CHARSXP

    /// Install a symbol from a CHARSXP.
    ///
    /// Like `Rf_install()` but takes a CHARSXP instead of C string.
    #[doc(alias = "installChar")]
    pub fn Rf_installChar(x: SEXP) -> SEXP;
}

// endregion
