//! `SexpExt` — the ergonomic extension trait on `SEXP`.
//!
//! Provides safe(r) accessors and type checks. This is the actual user-facing
//! API surface most callers reach for; it's re-exported from
//! [`crate::prelude`].

use crate::SEXP;
use crate::sexp_types::{R_xlen_t, RNativeType, Rboolean, Rcomplex, SEXPTYPE};
// Pull in everything raw from sys. Type re-exports (SEXP, SEXPTYPE, etc.)
// that `sys` itself promotes are identical items, so glob import is safe;
// we still bring SEXP/SEXPTYPE in via crate root above so the file reads
// naturally.
use crate::sys::{
    ALTREP, CAR, CAR_unchecked, CDR, CDR_unchecked, COMPLEX_ELT, DATAPTR_RO, DATAPTR_RO_unchecked,
    INTEGER_ELT, LOGICAL_ELT, PRINTNAME, R_CHAR, R_CHAR_unchecked, R_ClassSymbol, R_DimNamesSymbol,
    R_DimSymbol, R_LevelsSymbol, R_NaString, R_NamesSymbol, R_NilValue, R_RowNamesSymbol, RAW_ELT,
    REAL_ELT, Rf_asChar, Rf_asInteger, Rf_asLogical, Rf_asReal, Rf_classgets, Rf_coerceVector,
    Rf_cons, Rf_cons_unchecked, Rf_dimgets, Rf_dimnamesgets, Rf_duplicate, Rf_getAttrib,
    Rf_getAttrib_unchecked, Rf_inherits, Rf_isArray, Rf_isFactor, Rf_isFunction, Rf_isList,
    Rf_isMatrix, Rf_isObject, Rf_isS4, Rf_lcons, Rf_namesgets, Rf_setAttrib,
    Rf_setAttrib_unchecked, Rf_shallow_duplicate, Rf_xlength, Rf_xlength_unchecked, Rf_xlengthgets,
    SET_COMPLEX_ELT, SET_INTEGER_ELT, SET_LOGICAL_ELT, SET_RAW_ELT, SET_REAL_ELT, SET_STRING_ELT,
    SET_STRING_ELT_unchecked, SET_TAG, SET_TAG_unchecked, SET_VECTOR_ELT, SET_VECTOR_ELT_unchecked,
    SETCAR, SETCAR_unchecked, SETCDR, SETCDR_unchecked, STRING_ELT, STRING_ELT_unchecked, TAG,
    TYPEOF, VECTOR_ELT, VECTOR_ELT_unchecked,
};

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

    /// Get the length as `R_xlen_t` without thread checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. No debug assertions.
    unsafe fn xlength_unchecked(&self) -> R_xlen_t;

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

    /// Get a mutable slice view of this SEXP's data.
    ///
    /// # Safety
    ///
    /// - All safety requirements of [`as_slice`](Self::as_slice) apply.
    /// - The caller must ensure **exclusive access**: no other `&[T]` or `&mut [T]`
    ///   slices derived from this SEXP may exist simultaneously. Multiple calls to
    ///   `as_mut_slice` on the same SEXP without dropping the previous slice is UB.
    /// - The SEXP must not be shared (ALTREP or NAMED > 0 objects may alias).
    unsafe fn as_mut_slice<T: RNativeType>(&self) -> &'static mut [T];

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
    fn is_empty(&self) -> bool;

    /// Check if this SEXP is R's `NULL` (NILSXP).
    fn is_nil(&self) -> bool;

    /// Check if this SEXP is a factor.
    ///
    /// Equivalent to R's `Rf_isFactor(x)`.
    fn is_factor(&self) -> bool;

    /// Check if this SEXP is a pairlist (LISTSXP or NILSXP).
    ///
    /// Equivalent to R's `Rf_isList(x)`.
    fn is_pair_list(&self) -> bool;

    /// Check if this SEXP is a matrix.
    ///
    /// Equivalent to R's `Rf_isMatrix(x)`.
    fn is_matrix(&self) -> bool;

    /// Check if this SEXP is an array.
    ///
    /// Equivalent to R's `Rf_isArray(x)`.
    fn is_array(&self) -> bool;

    /// Check if this SEXP is a function (closure, builtin, or special).
    ///
    /// Equivalent to R's `Rf_isFunction(x)`.
    fn is_function(&self) -> bool;

    /// Check if this SEXP is an S4 object.
    ///
    /// Equivalent to R's `Rf_isS4(x)`.
    fn is_s4(&self) -> bool;

    /// Check if this SEXP is a data.frame.
    ///
    /// Equivalent to R's `Rf_isDataFrame(x)`.
    fn is_data_frame(&self) -> bool;

    /// Check if this SEXP is a numeric type (integer, logical, or real, excluding factors).
    ///
    /// Equivalent to R's `Rf_isNumeric(x)`.
    fn is_numeric(&self) -> bool;

    /// Check if this SEXP is a number type (numeric or complex).
    ///
    /// Equivalent to R's `Rf_isNumber(x)`.
    fn is_number(&self) -> bool;

    /// Check if this SEXP is an atomic vector.
    ///
    /// Returns true for logical, integer, real, complex, character, and raw vectors.
    fn is_vector_atomic(&self) -> bool;

    /// Check if this SEXP is a vector list (VECSXP or EXPRSXP).
    fn is_vector_list(&self) -> bool;

    /// Check if this SEXP is a vector (atomic vector or list).
    fn is_vector(&self) -> bool;

    /// Check if this SEXP is an R "object" (has a class attribute).
    fn is_object(&self) -> bool;

    // region: Coercion and scalar extraction

    /// Coerce this SEXP to the given type, returning a new SEXP.
    ///
    /// The result is guaranteed to have the requested SEXPTYPE.
    /// Equivalent to R's `Rf_coerceVector(x, target)`.
    fn coerce(&self, target: SEXPTYPE) -> SEXP;

    /// Extract a scalar logical value.
    ///
    /// Returns `None` for `NA`. Coerces non-logical inputs.
    /// Equivalent to R's `Rf_asLogical(x)`.
    fn as_logical(&self) -> Option<bool>;

    /// Extract a scalar integer value.
    ///
    /// Returns `None` for `NA_integer_`. Coerces non-integer inputs.
    /// Equivalent to R's `Rf_asInteger(x)`.
    fn as_integer(&self) -> Option<i32>;

    /// Extract a scalar real value.
    ///
    /// Returns `None` for `NA_real_` (NaN). Coerces non-real inputs.
    /// Equivalent to R's `Rf_asReal(x)`.
    fn as_real(&self) -> Option<f64>;

    /// Extract a scalar CHARSXP from this SEXP.
    ///
    /// The result is guaranteed to be a CHARSXP.
    /// Equivalent to R's `Rf_asChar(x)`.
    fn as_char(&self) -> SEXP;

    // endregion

    // region: Attribute access

    /// Get an attribute by symbol.
    fn get_attr(&self, name: SEXP) -> SEXP;

    /// Get an attribute by symbol, returning `None` for `R_NilValue`.
    fn get_attr_opt(&self, name: SEXP) -> Option<SEXP> {
        let attr = self.get_attr(name);
        if attr.is_nil() { None } else { Some(attr) }
    }

    /// Set an attribute by symbol.
    fn set_attr(&self, name: SEXP, val: SEXP);

    /// Get the `names` attribute.
    fn get_names(&self) -> SEXP;

    /// Set the `names` attribute.
    fn set_names(&self, names: SEXP);

    /// Get the `class` attribute.
    fn get_class(&self) -> SEXP;

    /// Set the `class` attribute.
    fn set_class(&self, class: SEXP);

    /// Get the `dim` attribute.
    fn get_dim(&self) -> SEXP;

    /// Set the `dim` attribute.
    fn set_dim(&self, dim: SEXP);

    /// Get the `dimnames` attribute.
    fn get_dimnames(&self) -> SEXP;

    /// Set the `dimnames` attribute.
    fn set_dimnames(&self, dimnames: SEXP);

    /// Get the `levels` attribute (factors).
    fn get_levels(&self) -> SEXP;

    /// Set the `levels` attribute (factors).
    fn set_levels(&self, levels: SEXP);

    /// Get the `row.names` attribute.
    fn get_row_names(&self) -> SEXP;

    /// Set the `row.names` attribute.
    fn set_row_names(&self, row_names: SEXP);

    /// Check if this SEXP inherits from a class.
    ///
    /// Equivalent to R's `inherits(x, "class_name")`.
    fn inherits_class(&self, class: &std::ffi::CStr) -> bool;

    // endregion

    // region: String element access

    /// Get the i-th CHARSXP element from a STRSXP.
    ///
    /// Equivalent to R's `STRING_ELT(x, i)`.
    fn string_elt(&self, i: isize) -> SEXP;

    /// Get the i-th string element as `Option<&str>`.
    ///
    /// Returns `None` for `NA_character_`. The returned `&str` borrows from R's
    /// internal string cache (CHARSXP global pool) and is valid as long as the
    /// parent STRSXP is protected from GC. The lifetime is tied to `&self` by
    /// the borrow checker, but the true validity depends on GC protection —
    /// do not hold the `&str` across allocation boundaries without ensuring
    /// the SEXP remains protected.
    fn string_elt_str(&self, i: isize) -> Option<&str>;

    /// Set the i-th CHARSXP element of a STRSXP.
    ///
    /// Equivalent to R's `SET_STRING_ELT(x, i, v)`.
    fn set_string_elt(&self, i: isize, charsxp: SEXP);

    /// Check if this CHARSXP is `NA_character_`.
    fn is_na_string(&self) -> bool;

    // endregion

    // region: List element access

    /// Get the i-th element of a VECSXP (generic vector / list).
    ///
    /// Equivalent to R's `VECTOR_ELT(x, i)`.
    fn vector_elt(&self, i: isize) -> SEXP;

    /// Set the i-th element of a VECSXP.
    ///
    /// Equivalent to R's `SET_VECTOR_ELT(x, i, v)`.
    fn set_vector_elt(&self, i: isize, val: SEXP);

    // endregion

    // region: Typed single-element access

    /// Get the i-th integer element.
    fn integer_elt(&self, i: isize) -> i32;
    /// Get the i-th real element.
    fn real_elt(&self, i: isize) -> f64;
    /// Get the i-th logical element (raw i32: 0/1/NA_LOGICAL).
    fn logical_elt(&self, i: isize) -> i32;
    /// Get the i-th complex element.
    fn complex_elt(&self, i: isize) -> Rcomplex;
    /// Get the i-th raw element.
    fn raw_elt(&self, i: isize) -> u8;

    /// Set the i-th integer element.
    fn set_integer_elt(&self, i: isize, v: i32);
    /// Set the i-th real element.
    fn set_real_elt(&self, i: isize, v: f64);
    /// Set the i-th logical element (raw i32: 0/1/NA_LOGICAL).
    fn set_logical_elt(&self, i: isize, v: i32);
    /// Set the i-th complex element.
    fn set_complex_elt(&self, i: isize, v: Rcomplex);
    /// Set the i-th raw element.
    fn set_raw_elt(&self, i: isize, v: u8);

    // endregion

    // region: Symbol and CHARSXP access

    /// Get the print name (CHARSXP) of a symbol (SYMSXP).
    ///
    /// # Safety
    ///
    /// The SEXP must be a valid SYMSXP.
    fn printname(&self) -> SEXP;

    /// Get the C string pointer from a CHARSXP.
    ///
    /// The returned pointer is valid as long as the CHARSXP is protected.
    ///
    /// # Safety
    ///
    /// The SEXP must be a valid CHARSXP.
    fn r_char(&self) -> *const ::std::os::raw::c_char;

    /// Get a `&str` from a CHARSXP. Returns `None` for `NA_character_`.
    fn r_char_str(&self) -> Option<&str>;

    // endregion

    // region: Vector resizing

    /// Resize a vector to a new length, returning a (possibly new) SEXP.
    ///
    /// If the new length is shorter, elements are truncated.
    /// If longer, new elements are filled with NA/NULL.
    /// Equivalent to R's `Rf_xlengthgets(x, newlen)`.
    fn resize(&self, newlen: R_xlen_t) -> SEXP;

    // endregion

    // region: Duplication

    /// Deep-copy this SEXP. Equivalent to R's `Rf_duplicate(x)`.
    fn duplicate(&self) -> SEXP;

    /// Shallow-copy this SEXP. Equivalent to R's `Rf_shallow_duplicate(x)`.
    fn shallow_duplicate(&self) -> SEXP;

    // endregion

    // region: Unchecked variants (bypass thread-check, for perf-critical paths)

    /// Get the i-th CHARSXP from a STRSXP. No thread check.
    ///
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn string_elt_unchecked(&self, i: isize) -> SEXP;
    /// Set the i-th CHARSXP of a STRSXP. No thread check.
    ///
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn set_string_elt_unchecked(&self, i: isize, charsxp: SEXP);
    /// Get the i-th element of a VECSXP. No thread check.
    ///
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn vector_elt_unchecked(&self, i: isize) -> SEXP;
    /// Set the i-th element of a VECSXP. No thread check.
    ///
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn set_vector_elt_unchecked(&self, i: isize, val: SEXP);
    /// Get an attribute by symbol. No thread check.
    ///
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn get_attr_unchecked(&self, name: SEXP) -> SEXP;
    /// Set an attribute by symbol. No thread check.
    ///
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn set_attr_unchecked(&self, name: SEXP, val: SEXP);

    /// Get C string pointer from a CHARSXP. No thread check.
    ///
    /// # Safety
    /// Must be called from R's main thread. The SEXP must be a valid CHARSXP.
    unsafe fn r_char_unchecked(&self) -> *const ::std::os::raw::c_char;

    // endregion
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
    unsafe fn xlength_unchecked(&self) -> R_xlen_t {
        unsafe { Rf_xlength_unchecked(*self) }
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
    unsafe fn as_mut_slice<T: RNativeType>(&self) -> &'static mut [T] {
        debug_assert!(
            self.type_of() == T::SEXP_TYPE,
            "SEXP type mismatch: expected {:?}, got {:?}",
            T::SEXP_TYPE,
            self.type_of()
        );
        let len = self.len();
        if len == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(T::dataptr_mut(*self), len) }
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

    #[inline]
    fn is_nil(&self) -> bool {
        // Pointer comparison, not type dereference — safe on dangling pointers.
        // R_NilValue is the singleton NILSXP; checking type_of() would crash
        // on freed SEXPs during cleanup.
        unsafe { std::ptr::addr_eq(self.0, R_NilValue.0) }
    }

    #[inline]
    fn is_factor(&self) -> bool {
        unsafe { Rf_isFactor(*self) != Rboolean::FALSE }
    }

    #[inline]
    fn is_pair_list(&self) -> bool {
        unsafe { Rf_isList(*self) != Rboolean::FALSE }
    }

    #[inline]
    fn is_matrix(&self) -> bool {
        unsafe { Rf_isMatrix(*self) != Rboolean::FALSE }
    }

    #[inline]
    fn is_array(&self) -> bool {
        unsafe { Rf_isArray(*self) != Rboolean::FALSE }
    }

    #[inline]
    fn is_function(&self) -> bool {
        unsafe { Rf_isFunction(*self) != Rboolean::FALSE }
    }

    #[inline]
    fn is_s4(&self) -> bool {
        unsafe { Rf_isS4(*self) != Rboolean::FALSE }
    }

    #[inline]
    fn is_data_frame(&self) -> bool {
        self.inherits_class(c"data.frame")
    }

    #[inline]
    fn is_numeric(&self) -> bool {
        let typ = self.type_of();
        (typ == SEXPTYPE::INTSXP || typ == SEXPTYPE::LGLSXP || typ == SEXPTYPE::REALSXP)
            && !self.is_factor()
    }

    #[inline]
    fn is_number(&self) -> bool {
        self.is_numeric() || self.is_complex()
    }

    #[inline]
    fn is_vector_atomic(&self) -> bool {
        matches!(
            self.type_of(),
            SEXPTYPE::LGLSXP
                | SEXPTYPE::INTSXP
                | SEXPTYPE::REALSXP
                | SEXPTYPE::CPLXSXP
                | SEXPTYPE::STRSXP
                | SEXPTYPE::RAWSXP
        )
    }

    #[inline]
    fn is_vector_list(&self) -> bool {
        let typ = self.type_of();
        typ == SEXPTYPE::VECSXP || typ == SEXPTYPE::EXPRSXP
    }

    #[inline]
    fn is_vector(&self) -> bool {
        self.is_vector_atomic() || self.is_vector_list()
    }

    #[inline]
    fn is_object(&self) -> bool {
        unsafe { Rf_isObject(*self) != Rboolean::FALSE }
    }

    // region: Coercion and scalar extraction

    #[inline]
    fn coerce(&self, target: SEXPTYPE) -> SEXP {
        unsafe { Rf_coerceVector(*self, target) }
    }

    #[inline]
    fn as_logical(&self) -> Option<bool> {
        let v = unsafe { Rf_asLogical(*self) };
        if v == crate::altrep_traits::NA_LOGICAL {
            None
        } else {
            Some(v != 0)
        }
    }

    #[inline]
    fn as_integer(&self) -> Option<i32> {
        let v = unsafe { Rf_asInteger(*self) };
        if v == crate::altrep_traits::NA_INTEGER {
            None
        } else {
            Some(v)
        }
    }

    #[inline]
    fn as_real(&self) -> Option<f64> {
        let v = unsafe { Rf_asReal(*self) };
        if v.to_bits() == crate::altrep_traits::NA_REAL.to_bits() {
            None
        } else {
            Some(v)
        }
    }

    #[inline]
    fn as_char(&self) -> SEXP {
        unsafe { Rf_asChar(*self) }
    }

    // endregion

    // region: Attribute access

    #[inline]
    fn get_attr(&self, name: SEXP) -> SEXP {
        unsafe { Rf_getAttrib(*self, name) }
    }

    #[inline]
    fn set_attr(&self, name: SEXP, val: SEXP) {
        unsafe {
            Rf_setAttrib(*self, name, val);
        }
    }

    #[inline]
    fn get_names(&self) -> SEXP {
        unsafe { Rf_getAttrib(*self, R_NamesSymbol) }
    }

    #[inline]
    fn set_names(&self, names: SEXP) {
        unsafe {
            Rf_namesgets(*self, names);
        }
    }

    #[inline]
    fn get_class(&self) -> SEXP {
        unsafe { Rf_getAttrib(*self, R_ClassSymbol) }
    }

    #[inline]
    fn set_class(&self, class: SEXP) {
        unsafe {
            Rf_classgets(*self, class);
        }
    }

    #[inline]
    fn get_dim(&self) -> SEXP {
        unsafe { Rf_getAttrib(*self, R_DimSymbol) }
    }

    #[inline]
    fn set_dim(&self, dim: SEXP) {
        unsafe {
            Rf_dimgets(*self, dim);
        }
    }

    #[inline]
    fn get_dimnames(&self) -> SEXP {
        unsafe { Rf_getAttrib(*self, R_DimNamesSymbol) }
    }

    #[inline]
    fn set_dimnames(&self, dimnames: SEXP) {
        unsafe {
            Rf_dimnamesgets(*self, dimnames);
        }
    }

    #[inline]
    fn get_levels(&self) -> SEXP {
        unsafe { Rf_getAttrib(*self, R_LevelsSymbol) }
    }

    #[inline]
    fn set_levels(&self, levels: SEXP) {
        unsafe {
            Rf_setAttrib(*self, R_LevelsSymbol, levels);
        }
    }

    #[inline]
    fn get_row_names(&self) -> SEXP {
        unsafe { Rf_getAttrib(*self, R_RowNamesSymbol) }
    }

    #[inline]
    fn set_row_names(&self, row_names: SEXP) {
        unsafe {
            Rf_setAttrib(*self, R_RowNamesSymbol, row_names);
        }
    }

    #[inline]
    fn inherits_class(&self, class: &std::ffi::CStr) -> bool {
        unsafe { Rf_inherits(*self, class.as_ptr()) != Rboolean::FALSE }
    }

    // endregion

    // region: String element access

    #[inline]
    fn string_elt(&self, i: isize) -> SEXP {
        unsafe { STRING_ELT(*self, i) }
    }

    #[inline]
    fn string_elt_str(&self, i: isize) -> Option<&str> {
        unsafe {
            let charsxp = STRING_ELT(*self, i);
            if std::ptr::addr_eq(charsxp.0, R_NaString.0) {
                return None;
            }
            let p = R_CHAR(charsxp);
            Some(std::ffi::CStr::from_ptr(p).to_str().unwrap_or(""))
        }
    }

    #[inline]
    fn set_string_elt(&self, i: isize, charsxp: SEXP) {
        unsafe { SET_STRING_ELT(*self, i, charsxp) }
    }

    #[inline]
    fn is_na_string(&self) -> bool {
        unsafe { std::ptr::addr_eq(self.0, R_NaString.0) }
    }

    // endregion

    // region: List element access

    #[inline]
    fn vector_elt(&self, i: isize) -> SEXP {
        unsafe { VECTOR_ELT(*self, i) }
    }

    #[inline]
    fn set_vector_elt(&self, i: isize, val: SEXP) {
        unsafe {
            SET_VECTOR_ELT(*self, i, val);
        }
    }

    // endregion

    // region: Typed single-element access

    #[inline]
    fn integer_elt(&self, i: isize) -> i32 {
        unsafe { INTEGER_ELT(*self, i) }
    }
    #[inline]
    fn real_elt(&self, i: isize) -> f64 {
        unsafe { REAL_ELT(*self, i) }
    }
    #[inline]
    fn logical_elt(&self, i: isize) -> i32 {
        unsafe { LOGICAL_ELT(*self, i) }
    }
    #[inline]
    fn complex_elt(&self, i: isize) -> Rcomplex {
        unsafe { COMPLEX_ELT(*self, i) }
    }
    #[inline]
    fn raw_elt(&self, i: isize) -> u8 {
        unsafe { RAW_ELT(*self, i) }
    }
    #[inline]
    fn set_integer_elt(&self, i: isize, v: i32) {
        unsafe { SET_INTEGER_ELT(*self, i, v) }
    }
    #[inline]
    fn set_real_elt(&self, i: isize, v: f64) {
        unsafe { SET_REAL_ELT(*self, i, v) }
    }
    #[inline]
    fn set_logical_elt(&self, i: isize, v: i32) {
        unsafe { SET_LOGICAL_ELT(*self, i, v) }
    }
    #[inline]
    fn set_complex_elt(&self, i: isize, v: Rcomplex) {
        unsafe { SET_COMPLEX_ELT(*self, i, v) }
    }
    #[inline]
    fn set_raw_elt(&self, i: isize, v: u8) {
        unsafe { SET_RAW_ELT(*self, i, v) }
    }

    // endregion

    // region: Symbol and CHARSXP access

    #[inline]
    fn printname(&self) -> SEXP {
        unsafe { PRINTNAME(*self) }
    }

    #[inline]
    fn r_char(&self) -> *const ::std::os::raw::c_char {
        unsafe { R_CHAR(*self) }
    }

    #[inline]
    fn r_char_str(&self) -> Option<&str> {
        if self.is_na_string() {
            return None;
        }
        let p = unsafe { R_CHAR(*self) };
        Some(
            unsafe { std::ffi::CStr::from_ptr(p) }
                .to_str()
                .unwrap_or(""),
        )
    }

    // endregion

    // region: Vector resizing

    #[inline]
    fn resize(&self, newlen: R_xlen_t) -> SEXP {
        unsafe { Rf_xlengthgets(*self, newlen) }
    }

    // endregion

    // region: Duplication

    #[inline]
    fn duplicate(&self) -> SEXP {
        unsafe { Rf_duplicate(*self) }
    }

    #[inline]
    fn shallow_duplicate(&self) -> SEXP {
        unsafe { Rf_shallow_duplicate(*self) }
    }

    // endregion

    // region: Unchecked variants

    #[inline]
    unsafe fn string_elt_unchecked(&self, i: isize) -> SEXP {
        unsafe { STRING_ELT_unchecked(*self, i) }
    }

    #[inline]
    unsafe fn set_string_elt_unchecked(&self, i: isize, charsxp: SEXP) {
        unsafe { SET_STRING_ELT_unchecked(*self, i, charsxp) }
    }

    #[inline]
    unsafe fn vector_elt_unchecked(&self, i: isize) -> SEXP {
        unsafe { VECTOR_ELT_unchecked(*self, i) }
    }

    #[inline]
    unsafe fn set_vector_elt_unchecked(&self, i: isize, val: SEXP) {
        unsafe {
            SET_VECTOR_ELT_unchecked(*self, i, val);
        }
    }

    #[inline]
    unsafe fn get_attr_unchecked(&self, name: SEXP) -> SEXP {
        unsafe { Rf_getAttrib_unchecked(*self, name) }
    }

    #[inline]
    unsafe fn set_attr_unchecked(&self, name: SEXP, val: SEXP) {
        unsafe {
            Rf_setAttrib_unchecked(*self, name, val);
        }
    }

    #[inline]
    unsafe fn r_char_unchecked(&self) -> *const ::std::os::raw::c_char {
        unsafe { R_CHAR_unchecked(*self) }
    }

    // endregion
}

/// Extension trait for SEXP providing pairlist (cons cell) operations.
///
/// Pairlist nodes have three slots: CAR (value), CDR (next), and TAG (name).
/// This trait encapsulates the raw C functions behind method calls.
#[allow(dead_code)]
pub(crate) trait PairListExt {
    /// Create a cons cell with this SEXP as CAR and `cdr` as CDR.
    fn cons(self, cdr: SEXP) -> SEXP;

    /// Create a language cons cell with this SEXP as CAR and `cdr` as CDR.
    fn lcons(self, cdr: SEXP) -> SEXP;

    /// Get the CAR (head/value) of this pairlist node.
    fn car(&self) -> SEXP;

    /// Get the CDR (tail/rest) of this pairlist node.
    fn cdr(&self) -> SEXP;

    /// Get the TAG (name symbol) of this pairlist node.
    fn tag(&self) -> SEXP;

    /// Set the TAG (name symbol) of this pairlist node.
    fn set_tag(&self, tag: SEXP);

    /// Set the CAR (value) of this pairlist node.
    fn set_car(&self, value: SEXP) -> SEXP;

    /// Set the CDR (tail) of this pairlist node.
    fn set_cdr(&self, tail: SEXP) -> SEXP;

    /// Create a cons cell (no thread check).
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn cons_unchecked(self, cdr: SEXP) -> SEXP;

    /// Get the CAR (no thread check).
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn car_unchecked(&self) -> SEXP;

    /// Get the CDR (no thread check).
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn cdr_unchecked(&self) -> SEXP;

    /// Set the TAG (no thread check).
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn set_tag_unchecked(&self, tag: SEXP);

    /// Set the CAR (no thread check).
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn set_car_unchecked(&self, value: SEXP) -> SEXP;

    /// Set the CDR (no thread check).
    /// # Safety
    /// Must be called from R's main thread.
    unsafe fn set_cdr_unchecked(&self, tail: SEXP) -> SEXP;
}

impl PairListExt for SEXP {
    #[inline]
    fn cons(self, cdr: SEXP) -> SEXP {
        unsafe { Rf_cons(self, cdr) }
    }

    #[inline]
    fn lcons(self, cdr: SEXP) -> SEXP {
        unsafe { Rf_lcons(self, cdr) }
    }

    #[inline]
    fn car(&self) -> SEXP {
        unsafe { CAR(*self) }
    }

    #[inline]
    fn cdr(&self) -> SEXP {
        unsafe { CDR(*self) }
    }

    #[inline]
    fn tag(&self) -> SEXP {
        unsafe { TAG(*self) }
    }

    #[inline]
    fn set_tag(&self, tag: SEXP) {
        unsafe { SET_TAG(*self, tag) }
    }

    #[inline]
    fn set_car(&self, value: SEXP) -> SEXP {
        unsafe { SETCAR(*self, value) }
    }

    #[inline]
    fn set_cdr(&self, tail: SEXP) -> SEXP {
        unsafe { SETCDR(*self, tail) }
    }

    #[inline]
    unsafe fn cons_unchecked(self, cdr: SEXP) -> SEXP {
        unsafe { Rf_cons_unchecked(self, cdr) }
    }

    #[inline]
    unsafe fn car_unchecked(&self) -> SEXP {
        unsafe { CAR_unchecked(*self) }
    }

    #[inline]
    unsafe fn cdr_unchecked(&self) -> SEXP {
        unsafe { CDR_unchecked(*self) }
    }

    #[inline]
    unsafe fn set_tag_unchecked(&self, tag: SEXP) {
        unsafe { SET_TAG_unchecked(*self, tag) }
    }

    #[inline]
    unsafe fn set_car_unchecked(&self, value: SEXP) -> SEXP {
        unsafe { SETCAR_unchecked(*self, value) }
    }

    #[inline]
    unsafe fn set_cdr_unchecked(&self, tail: SEXP) -> SEXP {
        unsafe { SETCDR_unchecked(*self, tail) }
    }
}
