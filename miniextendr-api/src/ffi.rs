pub mod altrep;

#[allow(non_camel_case_types)]
pub type R_xlen_t = isize;
pub type Rbyte = ::std::os::raw::c_uchar;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
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

#[repr(transparent)]
#[derive(Debug)]
pub struct SEXPREC(::std::os::raw::c_void);
pub type SEXP = *mut SEXPREC;

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
    /// - The SEXP must be valid and of the correct type for T
    /// - The returned slice borrows from R's memory; the SEXP must remain protected
    fn as_slice<T: RNativeType>(&self) -> &'static [T];

    /// Get a slice view without thread checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. No debug assertions.
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
}

impl SexpExt for SEXP {
    #[inline]
    fn type_of(&self) -> SEXPTYPE {
        unsafe { TYPEOF(*self) }
    }

    #[inline]
    fn is_null_or_nil(&self) -> bool {
        self.is_null() || std::ptr::addr_eq(*self, unsafe { R_NilValue })
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
    fn as_slice<T: RNativeType>(&self) -> &'static [T] {
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
}

/// Marker trait for types that correspond to R's native vector element types.
///
/// This enables blanket implementations for `TryFromSexp` and safe conversions.
pub(crate) trait RNativeType: Sized + Copy + 'static {
    /// The SEXPTYPE for vectors containing this element type.
    const SEXP_TYPE: SEXPTYPE;
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
}

impl RNativeType for f64 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;
}

impl RNativeType for u8 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::RAWSXP;
}

impl RNativeType for RLogical {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::LGLSXP;
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

#[r_ffi_checked]
#[allow(clashing_extern_declarations)]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    #[allow(dead_code)]
    pub static R_NilValue: SEXP;

    #[doc(alias = "NA_STRING")]
    pub static R_NaString: SEXP;
    pub static R_NamesSymbol: SEXP;
    pub static R_DimSymbol: SEXP;
    pub static R_DimNamesSymbol: SEXP;
    pub static R_ClassSymbol: SEXP;
    pub static R_RowNamesSymbol: SEXP;

    pub static R_GlobalEnv: SEXP;
    pub static R_BaseEnv: SEXP;
    pub static R_EmptyEnv: SEXP;

    // Special logical values
    pub static R_TrueValue: SEXP;
    pub static R_FalseValue: SEXP;
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

    // Rinternals.h
    pub fn R_PreserveObject(object: SEXP);
    pub fn R_ReleaseObject(object: SEXP);

    #[doc(alias = "PROTECT")]
    #[doc(alias = "protect")]
    pub fn Rf_protect(s: SEXP) -> SEXP;
    #[doc(alias = "UNPROTECT")]
    #[doc(alias = "unprotect")]
    pub fn Rf_unprotect(l: ::std::os::raw::c_int);
    #[doc(alias = "allocVector")]
    pub fn Rf_allocVector(sexptype: SEXPTYPE, length: R_xlen_t) -> SEXP;
    #[doc(alias = "CONS")]
    #[doc(alias = "cons")]
    pub fn Rf_cons(car: SEXP, cdr: SEXP) -> SEXP;
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
    pub fn SET_VECTOR_ELT(x: SEXP, i: R_xlen_t, v: SEXP);

    pub fn ALTREP_CLASS(x: SEXP) -> SEXP;
    pub fn R_altrep_data1(x: SEXP) -> SEXP;
    pub fn R_altrep_data2(x: SEXP) -> SEXP;
    pub fn R_set_altrep_data1(x: SEXP, v: SEXP);
    pub fn R_set_altrep_data2(x: SEXP, v: SEXP);

    // Vector data accessors (mutable pointers)
    pub fn LOGICAL(x: SEXP) -> *mut ::std::os::raw::c_int;
    pub fn INTEGER(x: SEXP) -> *mut ::std::os::raw::c_int;
    pub fn REAL(x: SEXP) -> *mut f64;
    pub fn COMPLEX(x: SEXP) -> *mut Rcomplex;
    pub fn RAW(x: SEXP) -> *mut Rbyte;
    pub fn ALTREP(x: SEXP) -> ::std::os::raw::c_int;

    // utils.h
    pub fn R_CheckUserInterrupt();

    // Type checking
    pub fn TYPEOF(x: SEXP) -> SEXPTYPE;

    // Symbol creation and access
    #[doc(alias = "install")]
    pub fn Rf_install(name: *const ::std::os::raw::c_char) -> SEXP;
    /// Get the print name (CHARSXP) of a symbol (SYMSXP)
    pub fn PRINTNAME(x: SEXP) -> SEXP;
    /// Get the C string pointer from a CHARSXP
    #[doc(alias = "CHAR")]
    pub fn R_CHAR(x: SEXP) -> *const ::std::os::raw::c_char;

    // Attribute access
    #[doc(alias = "getAttrib")]
    pub fn Rf_getAttrib(vec: SEXP, name: SEXP) -> SEXP;
    #[doc(alias = "namesgets")]
    pub fn Rf_namesgets(vec: SEXP, val: SEXP) -> SEXP;
    #[doc(alias = "dimgets")]
    pub fn Rf_dimgets(vec: SEXP, val: SEXP) -> SEXP;

    // Duplication
    #[doc(alias = "duplicate")]
    pub fn Rf_duplicate(s: SEXP) -> SEXP;
    #[doc(alias = "shallow_duplicate")]
    pub fn Rf_shallow_duplicate(s: SEXP) -> SEXP;

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
}

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

#[allow(non_camel_case_types)]
pub type DL_FUNC = ::std::option::Option<unsafe extern "C-unwind" fn(...) -> SEXP>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CallMethodDef {
    pub name: *const ::std::os::raw::c_char,
    pub fun: DL_FUNC,
    pub numArgs: ::std::os::raw::c_int,
}

// SAFETY: R_CallMethodDef contains raw pointers which don't impl Sync by default.
// However, Sync is required to store these in static arrays for R's method registration.
// This is safe because:
// 1. The name pointer points to static C string literals (&'static CStr)
// 2. The fun pointer is a static function pointer
// 3. These are read-only after initialization during R_init_*
unsafe impl Sync for R_CallMethodDef {}

#[r_ffi_checked]
#[allow(clashing_extern_declarations)]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    pub fn R_registerRoutines(
        info: *mut DllInfo,
        croutines: *const ::std::os::raw::c_void,
        callRoutines: *const R_CallMethodDef,
        fortranRoutines: *const ::std::os::raw::c_void,
        externalRoutines: *const ::std::os::raw::c_void,
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
    use super::{Rboolean, SEXP};

    #[allow(non_camel_case_types)]
    pub type R_CFinalizer_t_C = ::std::option::Option<unsafe extern "C" fn(s: SEXP)>;

    #[allow(non_camel_case_types)]
    pub type DL_FUNC_C = ::std::option::Option<unsafe extern "C" fn(...) -> SEXP>;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    #[allow(non_camel_case_types)]
    #[allow(non_snake_case)]
    pub struct R_CallMethodDef_C {
        pub name: *const ::std::os::raw::c_char,
        pub fun: DL_FUNC_C,
        pub numArgs: ::std::os::raw::c_int,
    }

    // SAFETY: Same as R_CallMethodDef - contains only static pointers
    unsafe impl Sync for R_CallMethodDef_C {}

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
///     let saved = R_CStackLimit;
///     R_CStackLimit = usize::MAX; // disable checking
///     // ... call R APIs ...
///     R_CStackLimit = saved; // restore
/// }
/// ```
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
        pub static mut R_CStackStart: usize;

        /// Stack size limit. Set to `usize::MAX` to disable stack checking.
        ///
        /// From R source: `if(R_CStackStart == -1) R_CStackLimit = -1; /* never set */`
        #[allow(non_upper_case_globals)]
        pub static mut R_CStackLimit: usize;

        /// Stack growth direction: 1 = grows up, -1 = grows down.
        ///
        /// Most systems (x86, ARM) grow down (-1).
        #[allow(non_upper_case_globals)]
        pub static mut R_CStackDir: ::std::os::raw::c_int;
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
    unsafe {
        Rf_isNumeric(x) || TYPEOF(x) == SEXPTYPE::CPLXSXP
    }
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

// endregion

