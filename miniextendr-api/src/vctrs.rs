//! Optional vctrs C API support.
//!
//! Provides access to vctrs' maturing C API functions via `R_GetCCallable`.
//! This is an optional dependency - if vctrs is not available, all calls will
//! return errors (no fallback behavior).
//!
//! # Available Functions
//!
//! | Function | Purpose |
//! |----------|---------|
//! | `obj_is_vector` | Check if an object is a vector |
//! | `short_vec_size` | Get the size of a short vector |
//! | `short_vec_recycle` | Recycle a vector to a given size |
//!
//! # Initialization
//!
//! Call [`init_vctrs`] from `R_init_<pkg>` to load the C-callable function
//! pointers. This is optional - packages that don't use vctrs don't need to
//! call it.
//!
//! ```ignore
//! #[unsafe(no_mangle)]
//! pub extern "C" fn R_init_mypackage(info: *mut DllInfo) {
//!     miniextendr_worker_init();
//!
//!     // Optional: initialize vctrs support
//!     if let Err(e) = init_vctrs() {
//!         // vctrs not available - that's OK if we don't need it
//!     }
//! }
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::vctrs::{init_vctrs, obj_is_vector, short_vec_size};
//!
//! // In R_init_<pkg>:
//! init_vctrs()?;
//!
//! // Later, in a function:
//! let is_vec = obj_is_vector(sexp)?;
//! let size = short_vec_size(sexp)?;
//! ```
//!
//! # Thread Safety
//!
//! - [`init_vctrs`] must be called from R's main thread
//! - All wrapper functions must be called from R's main thread
//! - Function pointers are stored in static `OnceLock` for thread-safe init
//!
//! # R Package Configuration
//!
//! To use vctrs support, add to your DESCRIPTION:
//!
//! ```text
//! Imports: vctrs
//! ```
//!
//! And to NAMESPACE:
//!
//! ```text
//! importFrom(vctrs, obj_is_vector)
//! ```
//!
//! This ensures vctrs is loaded before your package's `.onLoad` runs.

use crate::ffi::SEXP;
use std::sync::OnceLock;

// =============================================================================
// Type aliases
// =============================================================================

/// R's short vector length type (32-bit signed integer).
///
/// This corresponds to `R_len_t` in R's C API. For long vector support,
/// use `R_xlen_t` instead.
#[allow(non_camel_case_types)]
pub type R_len_t = i32;

/// Function pointer type for `obj_is_vector`.
type ObjIsVectorFn = unsafe extern "C" fn(SEXP) -> bool;

/// Function pointer type for `short_vec_size`.
type ShortVecSizeFn = unsafe extern "C" fn(SEXP) -> R_len_t;

/// Function pointer type for `short_vec_recycle`.
type ShortVecRecycleFn = unsafe extern "C" fn(SEXP, R_len_t) -> SEXP;

// =============================================================================
// Global function pointers (loaded once)
// =============================================================================

/// Loaded `obj_is_vector` function pointer.
static P_OBJ_IS_VECTOR: OnceLock<ObjIsVectorFn> = OnceLock::new();

/// Loaded `short_vec_size` function pointer.
static P_SHORT_VEC_SIZE: OnceLock<ShortVecSizeFn> = OnceLock::new();

/// Loaded `short_vec_recycle` function pointer.
static P_SHORT_VEC_RECYCLE: OnceLock<ShortVecRecycleFn> = OnceLock::new();

// =============================================================================
// Error types
// =============================================================================

/// Error type for vctrs operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VctrsError {
    /// vctrs support has not been initialized.
    ///
    /// Call [`init_vctrs`] from `R_init_<pkg>` first.
    NotInitialized,

    /// A required vctrs callable was not found.
    ///
    /// This usually means vctrs is not installed or not loaded.
    NotAvailable {
        /// The name of the callable that was not found.
        name: &'static str,
    },

    /// [`init_vctrs`] was called multiple times.
    ///
    /// This is not necessarily an error - the second call is a no-op.
    AlreadyInitialized,

    /// [`init_vctrs`] was called from a non-main thread.
    NotMainThread,
}

impl std::fmt::Display for VctrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VctrsError::NotInitialized => {
                write!(f, "vctrs not initialized - call init_vctrs() first")
            }
            VctrsError::NotAvailable { name } => {
                write!(
                    f,
                    "vctrs callable '{}' not found - is vctrs installed and loaded?",
                    name
                )
            }
            VctrsError::AlreadyInitialized => {
                write!(f, "vctrs already initialized")
            }
            VctrsError::NotMainThread => {
                write!(f, "init_vctrs must be called from R's main thread")
            }
        }
    }
}

impl std::error::Error for VctrsError {}

// =============================================================================
// Initialization
// =============================================================================

/// Initialize vctrs C-callable function pointers.
///
/// Loads `obj_is_vector`, `short_vec_size`, and `short_vec_recycle` from
/// vctrs' callable table via `R_GetCCallable("vctrs", ...)`.
///
/// # Errors
///
/// Returns an error if:
/// - Called from a non-main thread ([`VctrsError::NotMainThread`])
/// - Any callable is not found ([`VctrsError::NotAvailable`])
/// - Called multiple times ([`VctrsError::AlreadyInitialized`])
///
/// # Thread Safety
///
/// Must be called from R's main thread during package initialization.
///
/// # Example
///
/// ```ignore
/// // In R_init_<pkg> or .onLoad:
/// match init_vctrs() {
///     Ok(()) => println!("vctrs support enabled"),
///     Err(VctrsError::NotAvailable { .. }) => {
///         // vctrs not available - OK if we don't need it
///     }
///     Err(e) => panic!("vctrs init failed: {}", e),
/// }
/// ```
pub fn init_vctrs() -> Result<(), VctrsError> {
    // Check we're on main thread
    if !crate::worker::is_r_main_thread() {
        return Err(VctrsError::NotMainThread);
    }

    // Load obj_is_vector
    let obj_is_vector_ptr =
        unsafe { crate::ffi::R_GetCCallable(c"vctrs".as_ptr(), c"obj_is_vector".as_ptr()) };
    if obj_is_vector_ptr.is_none() {
        return Err(VctrsError::NotAvailable {
            name: "obj_is_vector",
        });
    }
    let obj_is_vector_fn: ObjIsVectorFn = unsafe { std::mem::transmute(obj_is_vector_ptr) };
    if P_OBJ_IS_VECTOR.set(obj_is_vector_fn).is_err() {
        return Err(VctrsError::AlreadyInitialized);
    }

    // Load short_vec_size
    let short_vec_size_ptr =
        unsafe { crate::ffi::R_GetCCallable(c"vctrs".as_ptr(), c"short_vec_size".as_ptr()) };
    if short_vec_size_ptr.is_none() {
        return Err(VctrsError::NotAvailable {
            name: "short_vec_size",
        });
    }
    let short_vec_size_fn: ShortVecSizeFn = unsafe { std::mem::transmute(short_vec_size_ptr) };
    // Note: We don't check set() here because if obj_is_vector succeeded,
    // this should too (same init call). If it fails, it's a logic error.
    let _ = P_SHORT_VEC_SIZE.set(short_vec_size_fn);

    // Load short_vec_recycle
    let short_vec_recycle_ptr =
        unsafe { crate::ffi::R_GetCCallable(c"vctrs".as_ptr(), c"short_vec_recycle".as_ptr()) };
    if short_vec_recycle_ptr.is_none() {
        return Err(VctrsError::NotAvailable {
            name: "short_vec_recycle",
        });
    }
    let short_vec_recycle_fn: ShortVecRecycleFn =
        unsafe { std::mem::transmute(short_vec_recycle_ptr) };
    let _ = P_SHORT_VEC_RECYCLE.set(short_vec_recycle_fn);

    Ok(())
}

/// Check if vctrs support has been initialized.
///
/// Returns `true` if [`init_vctrs`] has been successfully called.
#[inline]
pub fn is_vctrs_initialized() -> bool {
    P_OBJ_IS_VECTOR.get().is_some()
}

// =============================================================================
// Wrapper functions
// =============================================================================

/// Check if an R object is a vector according to vctrs.
///
/// This is vctrs' definition of "vector", which is broader than base R's
/// `is.vector()`. It includes:
/// - Atomic vectors (logical, integer, double, character, raw, complex)
/// - Lists (including data frames)
/// - S3/S4 objects with a `vec_proxy()` method
///
/// # Arguments
///
/// * `sexp` - The R object to check
///
/// # Returns
///
/// `true` if the object is a vector, `false` otherwise.
///
/// # Errors
///
/// Returns [`VctrsError::NotInitialized`] if [`init_vctrs`] hasn't been called.
///
/// # Example
///
/// ```ignore
/// let x = some_r_object();
/// if obj_is_vector(x)? {
///     let size = short_vec_size(x)?;
///     println!("Vector of size {}", size);
/// }
/// ```
#[inline]
pub fn obj_is_vector(sexp: SEXP) -> Result<bool, VctrsError> {
    let f = P_OBJ_IS_VECTOR.get().ok_or(VctrsError::NotInitialized)?;
    Ok(unsafe { f(sexp) })
}

/// Get the size (length) of a short vector.
///
/// Returns the number of observations in the vector. For data frames,
/// this is the number of rows. For atomic vectors, this is the length.
///
/// # Arguments
///
/// * `sexp` - The R vector (must be a vector according to [`obj_is_vector`])
///
/// # Returns
///
/// The size of the vector as an `R_len_t` (32-bit integer).
///
/// # Errors
///
/// Returns [`VctrsError::NotInitialized`] if [`init_vctrs`] hasn't been called.
///
/// # Panics
///
/// The underlying vctrs function may error if `sexp` is not a vector.
///
/// # Example
///
/// ```ignore
/// let df = create_data_frame();
/// let nrow = short_vec_size(df)?;
/// ```
#[inline]
pub fn short_vec_size(sexp: SEXP) -> Result<R_len_t, VctrsError> {
    let f = P_SHORT_VEC_SIZE.get().ok_or(VctrsError::NotInitialized)?;
    Ok(unsafe { f(sexp) })
}

/// Recycle a vector to a specified size.
///
/// Implements vctrs' recycling rules:
/// - Size 1 vectors are recycled to any size
/// - Other vectors must match the target size exactly
///
/// # Arguments
///
/// * `sexp` - The R vector to recycle
/// * `size` - The target size
///
/// # Returns
///
/// The recycled vector. May return the original vector if no recycling
/// was needed, or a new vector if recycling occurred.
///
/// # Errors
///
/// Returns [`VctrsError::NotInitialized`] if [`init_vctrs`] hasn't been called.
///
/// # Safety
///
/// The returned SEXP must be protected by the caller if it will be used
/// across potential R allocations. Use [`OwnedProtect`] or similar.
///
/// # Panics
///
/// The underlying vctrs function may error if:
/// - `sexp` is not a vector
/// - The vector cannot be recycled to the target size
///
/// # Example
///
/// ```ignore
/// // Recycle a length-1 vector to length 10
/// let x = Rf_ScalarInteger(42);
/// let recycled = short_vec_recycle(x, 10)?;
/// // recycled is now an integer vector of length 10, all 42s
/// ```
///
/// [`OwnedProtect`]: crate::gc_protect::OwnedProtect
#[inline]
pub fn short_vec_recycle(sexp: SEXP, size: R_len_t) -> Result<SEXP, VctrsError> {
    let f = P_SHORT_VEC_RECYCLE
        .get()
        .ok_or(VctrsError::NotInitialized)?;
    Ok(unsafe { f(sexp, size) })
}

// =============================================================================
// SexpExt extension trait
// =============================================================================

/// Extension trait for vctrs operations on SEXP values.
///
/// This trait provides convenient methods for calling vctrs functions
/// on R objects. All methods require [`init_vctrs`] to have been called first.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::vctrs::{init_vctrs, VctrsSexpExt};
///
/// // In R_init_<pkg>:
/// init_vctrs()?;
///
/// // Later, in a function:
/// fn process_vector(x: SEXP) -> Result<(), VctrsError> {
///     if x.vctrs_is_vector()? {
///         let size = x.vctrs_size()?;
///         println!("Vector of size {}", size);
///     }
///     Ok(())
/// }
/// ```
pub trait VctrsSexpExt {
    /// Check if this object is a vector according to vctrs.
    ///
    /// See [`obj_is_vector`] for details.
    fn vctrs_is_vector(&self) -> Result<bool, VctrsError>;

    /// Get the size of this vector.
    ///
    /// See [`short_vec_size`] for details.
    fn vctrs_size(&self) -> Result<R_len_t, VctrsError>;

    /// Recycle this vector to a target size.
    ///
    /// See [`short_vec_recycle`] for details.
    ///
    /// # Safety
    ///
    /// The returned SEXP must be protected by the caller if it will be used
    /// across potential R allocations.
    fn vctrs_recycle_to(&self, size: R_len_t) -> Result<SEXP, VctrsError>;
}

impl VctrsSexpExt for SEXP {
    #[inline]
    fn vctrs_is_vector(&self) -> Result<bool, VctrsError> {
        obj_is_vector(*self)
    }

    #[inline]
    fn vctrs_size(&self) -> Result<R_len_t, VctrsError> {
        short_vec_size(*self)
    }

    #[inline]
    fn vctrs_recycle_to(&self, size: R_len_t) -> Result<SEXP, VctrsError> {
        short_vec_recycle(*self, size)
    }
}

// =============================================================================
// C-callable initialization shim
// =============================================================================

/// C-callable shim for initializing vctrs support.
///
/// This function is intended to be called from C code (e.g., `R_init_<pkg>`).
///
/// # Returns
///
/// - `0`: Success
/// - `1`: vctrs not available (callable not found)
/// - `2`: Not on main thread
/// - `3`: Already initialized
///
/// # Example (C)
///
/// ```c
/// extern int miniextendr_init_vctrs(void);
///
/// void R_init_mypackage(DllInfo *info) {
///     miniextendr_worker_init();
///     int status = miniextendr_init_vctrs();
///     if (status == 1) {
///         // vctrs not available - OK if optional
///     }
/// }
/// ```
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_init_vctrs() -> i32 {
    match init_vctrs() {
        Ok(()) => 0,
        Err(VctrsError::NotAvailable { .. }) => 1,
        Err(VctrsError::NotMainThread) => 2,
        Err(VctrsError::AlreadyInitialized) => 3,
        Err(VctrsError::NotInitialized) => unreachable!(),
    }
}

// =============================================================================
// Construction helpers (Phase A)
// =============================================================================

use crate::ffi::{
    R_BlankString, R_NaString, R_NamesSymbol, R_NilValue, R_xlen_t, Rf_allocVector, Rf_getAttrib,
    Rf_install, Rf_setAttrib, Rf_type2char, Rf_xlength, SET_STRING_ELT, SEXPTYPE, STRING_ELT,
    TYPEOF, VECTOR_ELT,
};
use crate::gc_protect::OwnedProtect;
use crate::list::List;

/// Error type for vctrs object construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VctrsBuildError {
    /// vctrs support has not been initialized.
    VctrsNotInitialized,

    /// The data is not a vector according to vctrs.
    NotAVector,

    /// List data requires `inherit_base_type = true`.
    ///
    /// When constructing a vctr from a list, `inherit_base_type` must be `true`
    /// (or `None` to use the default) so that "list" appears in the class vector.
    ListRequiresInheritBaseType,

    /// Record fields must all have the same length.
    FieldLengthMismatch {
        /// Name of the field with mismatched length.
        field: String,
        /// Expected length (from first field).
        expected: isize,
        /// Actual length of the mismatched field.
        actual: isize,
    },

    /// Record must have at least one field.
    EmptyRecord,

    /// Record field names must be unique.
    DuplicateFieldName {
        /// The duplicate field name.
        name: String,
    },

    /// Record fields must be named.
    UnnamedFields,

    /// list_of requires at least one of ptype or size.
    MissingPtypeOrSize,

    /// Invalid size (must be non-negative).
    InvalidSize {
        /// The invalid size value.
        size: i32,
    },

    /// Class vector must not be empty.
    EmptyClass,
}

impl std::fmt::Display for VctrsBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VctrsBuildError::VctrsNotInitialized => {
                write!(f, "vctrs not initialized - call init_vctrs() first")
            }
            VctrsBuildError::NotAVector => {
                write!(f, "data is not a vector according to vctrs")
            }
            VctrsBuildError::ListRequiresInheritBaseType => {
                write!(
                    f,
                    "list data requires inherit_base_type = true (or None for default)"
                )
            }
            VctrsBuildError::FieldLengthMismatch {
                field,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "field '{}' has length {} but expected {}",
                    field, actual, expected
                )
            }
            VctrsBuildError::EmptyRecord => {
                write!(f, "record must have at least one field")
            }
            VctrsBuildError::DuplicateFieldName { name } => {
                write!(f, "duplicate field name: '{}'", name)
            }
            VctrsBuildError::UnnamedFields => {
                write!(f, "record fields must be named")
            }
            VctrsBuildError::MissingPtypeOrSize => {
                write!(f, "list_of requires at least one of ptype or size")
            }
            VctrsBuildError::InvalidSize { size } => {
                write!(f, "invalid size: {} (must be non-negative)", size)
            }
            VctrsBuildError::EmptyClass => {
                write!(f, "class vector must not be empty")
            }
        }
    }
}

impl std::error::Error for VctrsBuildError {}

// =============================================================================
// Helper functions
// =============================================================================

/// Build a class vector (STRSXP) from a slice of class names.
///
/// # Safety
///
/// Must be called from R's main thread.
unsafe fn build_class_vector(classes: &[&str]) -> OwnedProtect {
    let n = classes.len() as R_xlen_t;
    let class_sexp = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, n)) };

    for (i, class_name) in classes.iter().enumerate() {
        // Use Rf_mkCharLenCE with UTF-8 encoding since Rust strings are always UTF-8
        let charsxp = unsafe {
            crate::ffi::Rf_mkCharLenCE(
                class_name.as_ptr() as *const i8,
                class_name.len() as i32,
                crate::ffi::cetype_t::CE_UTF8,
            )
        };
        unsafe { SET_STRING_ELT(class_sexp.get(), i as R_xlen_t, charsxp) };
    }

    class_sexp
}

/// Create an R symbol from a Rust string.
///
/// Uses Rf_installChar to create a symbol from a CHARSXP, which properly
/// handles non-null-terminated strings.
///
/// # Safety
///
/// Must be called from R's main thread.
unsafe fn install_symbol(name: &str) -> SEXP {
    // Create a CHARSXP with proper length
    let charsxp = unsafe {
        crate::ffi::Rf_mkCharLenCE(
            name.as_ptr() as *const i8,
            name.len() as i32,
            crate::ffi::cetype_t::CE_UTF8,
        )
    };
    // Convert to symbol
    unsafe { crate::ffi::Rf_installChar(charsxp) }
}

/// Get the R typeof name for a SEXP (e.g., "double", "integer", "list").
///
/// # Safety
///
/// Must be called from R's main thread with a valid SEXP.
unsafe fn get_typeof_name(sexp: SEXP) -> &'static str {
    let sexptype = unsafe { TYPEOF(sexp) };
    let cstr = unsafe { Rf_type2char(sexptype) };
    let cstr = unsafe { std::ffi::CStr::from_ptr(cstr) };
    // SAFETY: R's type names are ASCII strings
    cstr.to_str().unwrap_or("unknown")
}

/// Repair NA names by replacing them with empty strings.
///
/// Returns a new names SEXP with NA values replaced by "", or the original
/// if no NA values were found.
///
/// # Safety
///
/// Must be called from R's main thread with a valid STRSXP.
unsafe fn repair_na_names(names: SEXP) -> SEXP {
    let n = unsafe { Rf_xlength(names) };
    let mut has_na = false;

    // First pass: check if any NA
    for i in 0..n {
        if unsafe { STRING_ELT(names, i) == R_NaString } {
            has_na = true;
            break;
        }
    }

    if !has_na {
        return names;
    }

    // Second pass: create repaired copy
    let repaired = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, n)) };
    for i in 0..n {
        let elem = unsafe { STRING_ELT(names, i) };
        let new_elem = if elem == unsafe { R_NaString } {
            unsafe { R_BlankString }
        } else {
            elem
        };
        unsafe { SET_STRING_ELT(repaired.get(), i, new_elem) };
    }

    // Return the SEXP - guard drops and unprotects
    repaired.get()
}

// =============================================================================
// new_vctr
// =============================================================================

/// Create a new vctrs vector object.
///
/// This mirrors `vctrs::new_vctr()` in R, creating an object with the
/// appropriate class structure for vctrs compatibility.
///
/// # Arguments
///
/// * `data` - The underlying data (must be a vector according to vctrs)
/// * `class` - User class names (will be prepended to "vctrs_vctr")
/// * `attrs` - Additional attributes as (name, value) pairs
/// * `inherit_base_type` - Whether to include the base type in the class vector.
///   - `None`: Use default (true for lists, false otherwise)
///   - `Some(true)`: Include base type (e.g., "double", "list")
///   - `Some(false)`: Don't include base type (error for lists)
///
/// # Class Structure
///
/// The resulting class vector will be:
/// - `c(class..., "vctrs_vctr")` if `inherit_base_type` is false
/// - `c(class..., "vctrs_vctr", typeof(data))` if `inherit_base_type` is true
///
/// # Names Repair
///
/// If `data` has a names attribute with NA values, they are replaced with "".
///
/// # Example
///
/// ```ignore
/// // Create a "percent" class backed by doubles
/// let data = vec![0.1, 0.2, 0.3].into_sexp();
/// let obj = new_vctr(data, &["vctrs_percent"], &[], None)?;
/// // class(obj) == c("vctrs_percent", "vctrs_vctr", "double")
/// ```
pub fn new_vctr(
    data: SEXP,
    class: &[&str],
    attrs: &[(&str, SEXP)],
    inherit_base_type: Option<bool>,
) -> Result<SEXP, VctrsBuildError> {
    // Check vctrs is initialized
    if !is_vctrs_initialized() {
        return Err(VctrsBuildError::VctrsNotInitialized);
    }

    // Validate: data must be a vector
    let is_vector = obj_is_vector(data).map_err(|_| VctrsBuildError::VctrsNotInitialized)?;
    if !is_vector {
        return Err(VctrsBuildError::NotAVector);
    }

    // Determine inherit_base_type
    let data_type = unsafe { TYPEOF(data) };
    let is_list = data_type == SEXPTYPE::VECSXP;

    let inherit = match inherit_base_type {
        Some(false) if is_list => return Err(VctrsBuildError::ListRequiresInheritBaseType),
        Some(v) => v,
        None => is_list, // Default: true for lists, false otherwise
    };

    // Build class vector
    let base_type_name = if inherit {
        Some(unsafe { get_typeof_name(data) })
    } else {
        None
    };

    // Count total class elements: user classes + "vctrs_vctr" + optional base type
    let mut class_parts: Vec<&str> = class.to_vec();
    class_parts.push("vctrs_vctr");
    if let Some(base_name) = base_type_name {
        class_parts.push(base_name);
    }

    // Build and set class
    let class_sexp = unsafe { build_class_vector(&class_parts) };
    unsafe { Rf_setAttrib(data, crate::ffi::R_ClassSymbol, class_sexp.get()) };

    // Repair NA names if present
    let names = unsafe { Rf_getAttrib(data, R_NamesSymbol) };
    if names != unsafe { R_NilValue } {
        let repaired = unsafe { repair_na_names(names) };
        if repaired != names {
            unsafe { Rf_setAttrib(data, R_NamesSymbol, repaired) };
        }
    }

    // Set additional attributes
    for (name, value) in attrs {
        let name_sym = unsafe { install_symbol(name) };
        unsafe { Rf_setAttrib(data, name_sym, *value) };
    }

    Ok(data)
}

// =============================================================================
// new_rcrd
// =============================================================================

/// Create a new vctrs record object.
///
/// This mirrors `vctrs::new_rcrd()` in R, creating a record type where
/// each element is a collection of fields (like a row in a data frame).
///
/// # Arguments
///
/// * `fields` - A named list where all elements have the same length
/// * `class` - User class names (will be prepended to "vctrs_rcrd")
/// * `attrs` - Additional attributes as (name, value) pairs
///
/// # Class Structure
///
/// The resulting class vector will be:
/// - `c(class..., "vctrs_rcrd", "vctrs_vctr")`
///
/// # Requirements
///
/// - `fields` must be a named list
/// - All fields must have the same length
/// - Field names must be unique
///
/// # Example
///
/// ```ignore
/// // Create a "rational" record with numerator and denominator
/// let fields = list!(n = vec![1, 2, 3], d = vec![2, 3, 4]);
/// let obj = new_rcrd(fields, &["vctrs_rational"], &[])?;
/// // class(obj) == c("vctrs_rational", "vctrs_rcrd", "vctrs_vctr")
/// ```
pub fn new_rcrd(
    fields: List,
    class: &[&str],
    attrs: &[(&str, SEXP)],
) -> Result<SEXP, VctrsBuildError> {
    let n_fields = fields.len();

    // Validate: must have at least one field
    if n_fields == 0 {
        return Err(VctrsBuildError::EmptyRecord);
    }

    // Validate: fields must be named
    let names_sexp = fields.names().ok_or(VctrsBuildError::UnnamedFields)?;

    // Validate: check for duplicate names and get expected length from first field
    let mut seen_names = std::collections::HashSet::new();
    let first_field = fields.get(0).expect("n_fields > 0");
    let expected_len = unsafe { Rf_xlength(first_field) };

    for i in 0..n_fields {
        // Check name
        let name_charsxp = unsafe { STRING_ELT(names_sexp, i) };
        if name_charsxp == unsafe { R_NaString } || name_charsxp == unsafe { R_NilValue } {
            return Err(VctrsBuildError::UnnamedFields);
        }

        let name_cstr = unsafe { std::ffi::CStr::from_ptr(crate::ffi::R_CHAR(name_charsxp)) };
        let name = name_cstr.to_str().unwrap_or("");
        if name.is_empty() {
            return Err(VctrsBuildError::UnnamedFields);
        }

        if !seen_names.insert(name.to_string()) {
            return Err(VctrsBuildError::DuplicateFieldName {
                name: name.to_string(),
            });
        }

        // Check length (skip first, already used for expected_len)
        if i > 0 {
            let field = unsafe { VECTOR_ELT(fields.as_sexp(), i) };
            let field_len = unsafe { Rf_xlength(field) };
            if field_len != expected_len {
                return Err(VctrsBuildError::FieldLengthMismatch {
                    field: name.to_string(),
                    expected: expected_len,
                    actual: field_len,
                });
            }
        }
    }

    // Build class vector: c(user_class..., "vctrs_rcrd", "vctrs_vctr")
    let mut class_parts: Vec<&str> = class.to_vec();
    class_parts.push("vctrs_rcrd");
    class_parts.push("vctrs_vctr");

    let class_sexp = unsafe { build_class_vector(&class_parts) };
    let data = fields.as_sexp();
    unsafe { Rf_setAttrib(data, crate::ffi::R_ClassSymbol, class_sexp.get()) };

    // Set additional attributes
    for (name, value) in attrs {
        let name_sym = unsafe { install_symbol(name) };
        unsafe { Rf_setAttrib(data, name_sym, *value) };
    }

    Ok(data)
}

// =============================================================================
// new_list_of
// =============================================================================

/// Create a new vctrs list_of object.
///
/// This mirrors `vctrs::new_list_of()` in R, creating a list where each
/// element is expected to be of a consistent type (the prototype).
///
/// # Arguments
///
/// * `x` - A list of elements
/// * `ptype` - The prototype (empty vector defining the element type)
/// * `size` - Optional fixed size for elements
/// * `class` - User class names (will be prepended to "vctrs_list_of")
/// * `attrs` - Additional attributes as (name, value) pairs
///
/// # Class Structure
///
/// The resulting class vector will be:
/// - `c(class..., "vctrs_list_of", "vctrs_vctr", "list")`
///
/// # Requirements
///
/// - At least one of `ptype` or `size` must be provided
/// - `size` must be non-negative if provided
///
/// # Example
///
/// ```ignore
/// // Create a list_of<integer>
/// let x = list!(vec![1, 2], vec![3, 4, 5]);
/// let ptype = integer(0).into_sexp();
/// let obj = new_list_of(x, Some(ptype), None, &[], &[])?;
/// // class(obj) == c("vctrs_list_of", "vctrs_vctr", "list")
/// ```
pub fn new_list_of(
    x: List,
    ptype: Option<SEXP>,
    size: Option<i32>,
    class: &[&str],
    attrs: &[(&str, SEXP)],
) -> Result<SEXP, VctrsBuildError> {
    // Validate: at least one of ptype or size
    if ptype.is_none() && size.is_none() {
        return Err(VctrsBuildError::MissingPtypeOrSize);
    }

    // Validate size if provided
    if let Some(s) = size {
        if s < 0 {
            return Err(VctrsBuildError::InvalidSize { size: s });
        }
    }

    // Build class vector: c(user_class..., "vctrs_list_of", "vctrs_vctr", "list")
    let mut class_parts: Vec<&str> = class.to_vec();
    class_parts.push("vctrs_list_of");
    class_parts.push("vctrs_vctr");
    class_parts.push("list");

    let class_sexp = unsafe { build_class_vector(&class_parts) };
    let data = x.as_sexp();
    unsafe { Rf_setAttrib(data, crate::ffi::R_ClassSymbol, class_sexp.get()) };

    // Set ptype attribute if provided
    if let Some(p) = ptype {
        let ptype_sym = unsafe { Rf_install(c"ptype".as_ptr()) };
        unsafe { Rf_setAttrib(data, ptype_sym, p) };
    }

    // Set size attribute if provided
    if let Some(s) = size {
        let size_sym = unsafe { Rf_install(c"size".as_ptr()) };
        let size_sexp = unsafe { crate::ffi::Rf_ScalarInteger(s) };
        unsafe { Rf_setAttrib(data, size_sym, size_sexp) };
    }

    // Set additional attributes
    for (name, value) in attrs {
        let name_sym = unsafe { install_symbol(name) };
        unsafe { Rf_setAttrib(data, name_sym, *value) };
    }

    Ok(data)
}

// =============================================================================
// Phase C: Traits for ergonomic vctrs type creation
// =============================================================================

/// The kind of vctrs class being created.
///
/// This corresponds to the different vctrs constructors:
/// - [`Vctr`](VctrsKind::Vctr): Simple vector backed by a base type (`vctrs::new_vctr`)
/// - [`Rcrd`](VctrsKind::Rcrd): Record type with named fields (`vctrs::new_rcrd`)
/// - [`ListOf`](VctrsKind::ListOf): Homogeneous list with prototype (`vctrs::new_list_of`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VctrsKind {
    /// Simple vctr backed by a base vector (double, integer, character, etc.).
    ///
    /// Created with [`new_vctr`]. The class structure is:
    /// `c(user_class, "vctrs_vctr", base_type?)`
    #[default]
    Vctr,

    /// Record type with named fields of equal length.
    ///
    /// Created with [`new_rcrd`]. The class structure is:
    /// `c(user_class, "vctrs_rcrd", "vctrs_vctr")`
    ///
    /// Record types are useful for compound objects like rational numbers,
    /// date-times with timezone, or any data with multiple parallel fields.
    Rcrd,

    /// Homogeneous list where all elements share a common prototype.
    ///
    /// Created with [`new_list_of`]. The class structure is:
    /// `c(user_class, "vctrs_list_of", "vctrs_vctr", "list")`
    ///
    /// Useful for storing lists of vectors of the same type.
    ListOf,
}

/// Trait for types that can describe their vctrs class metadata.
///
/// Implement this trait to define how a Rust type should be represented
/// as a vctrs-compatible R object.
///
/// # Example
///
/// ```ignore
/// struct Percent(Vec<f64>);
///
/// impl VctrsClass for Percent {
///     const CLASS_NAME: &'static str = "vctrs_percent";
///     const KIND: VctrsKind = VctrsKind::Vctr;
///     const BASE_TYPE: Option<SEXPTYPE> = Some(SEXPTYPE::REALSXP);
///     const INHERIT_BASE_TYPE: bool = false;
/// }
/// ```
pub trait VctrsClass {
    /// The primary class name for this type.
    ///
    /// This becomes the first element in the R class vector.
    /// Convention: use snake_case with a "vctrs_" prefix for custom classes.
    const CLASS_NAME: &'static str;

    /// The kind of vctrs class (vctr, rcrd, or list_of).
    const KIND: VctrsKind;

    /// The base R SEXP type for vctr kinds.
    ///
    /// - For `Vctr`: The underlying vector type (e.g., `REALSXP` for doubles)
    /// - For `Rcrd` and `ListOf`: Usually `None` (they use list internally)
    const BASE_TYPE: Option<SEXPTYPE> = None;

    /// Whether to include the base type in the class vector.
    ///
    /// - `true`: Class is `c("my_class", "vctrs_vctr", "double")`
    /// - `false`: Class is `c("my_class", "vctrs_vctr")`
    ///
    /// For list-backed types, this must be `true`.
    const INHERIT_BASE_TYPE: bool = false;

    /// Optional abbreviation for `vec_ptype_abbr` (used in printing).
    ///
    /// If `None`, vctrs will use a default based on the class name.
    const ABBR: Option<&'static str> = None;

    /// Additional class names to include (after the primary class).
    ///
    /// Useful for inheritance hierarchies. These appear between the
    /// primary class and "vctrs_vctr" in the class vector.
    fn additional_classes() -> &'static [&'static str] {
        &[]
    }

    /// Additional attributes to set on the object.
    ///
    /// Override this to add custom attributes like "digits", "units", etc.
    /// The default implementation returns an empty slice.
    fn attrs(&self) -> Vec<(&'static str, SEXP)> {
        Vec::new()
    }
}

/// Trait for converting Rust types into vctrs-compatible R objects.
///
/// This trait provides the `into_vctrs()` method which converts a Rust
/// value into an R SEXP with proper vctrs class structure.
///
/// # Implementation
///
/// Types implementing this trait should:
/// 1. Convert their data to the appropriate R SEXP type
/// 2. Apply the vctrs class structure using [`new_vctr`], [`new_rcrd`], or [`new_list_of`]
///
/// # Example
///
/// ```ignore
/// struct Percent(Vec<f64>);
///
/// impl VctrsClass for Percent {
///     const CLASS_NAME: &'static str = "vctrs_percent";
///     const KIND: VctrsKind = VctrsKind::Vctr;
///     const BASE_TYPE: Option<SEXPTYPE> = Some(SEXPTYPE::REALSXP);
/// }
///
/// impl IntoVctrs for Percent {
///     fn into_vctrs(self) -> Result<SEXP, VctrsBuildError> {
///         use miniextendr_api::IntoR;
///         let data = self.0.into_r();
///         new_vctr(
///             data,
///             &[Self::CLASS_NAME],
///             &self.attrs(),
///             Some(Self::INHERIT_BASE_TYPE),
///         )
///     }
/// }
/// ```
pub trait IntoVctrs: VctrsClass {
    /// Convert this value into a vctrs-compatible R object.
    ///
    /// # Errors
    ///
    /// Returns [`VctrsBuildError`] if:
    /// - vctrs is not initialized
    /// - The data is not a valid vector
    /// - Other construction errors occur
    fn into_vctrs(self) -> Result<SEXP, VctrsBuildError>;
}

/// Marker trait for vctrs record types.
///
/// Record types are vctrs classes backed by named lists where all fields
/// have equal length. Each "element" of the record is a row across all fields.
///
/// # Example
///
/// ```ignore
/// /// A rational number represented as numerator/denominator
/// struct Rational {
///     n: Vec<i32>,  // numerators
///     d: Vec<i32>,  // denominators
/// }
///
/// impl VctrsClass for Rational {
///     const CLASS_NAME: &'static str = "vctrs_rational";
///     const KIND: VctrsKind = VctrsKind::Rcrd;
/// }
///
/// impl VctrsRecord for Rational {
///     fn field_names() -> &'static [&'static str] {
///         &["n", "d"]
///     }
/// }
/// ```
pub trait VctrsRecord: VctrsClass {
    /// The names of the record fields.
    ///
    /// These must match the order in which fields are added to the
    /// underlying list when implementing [`IntoVctrs`].
    fn field_names() -> &'static [&'static str];
}

/// Marker trait for vctrs list_of types.
///
/// list_of types are lists where all elements are expected to share
/// a common prototype (element type).
///
/// # Example
///
/// ```ignore
/// /// A list of integer vectors
/// struct IntVecList(Vec<Vec<i32>>);
///
/// impl VctrsClass for IntVecList {
///     const CLASS_NAME: &'static str = "vctrs_int_list";
///     const KIND: VctrsKind = VctrsKind::ListOf;
/// }
///
/// impl VctrsListOf for IntVecList {
///     fn ptype_expr() -> &'static str {
///         "integer()"
///     }
/// }
/// ```
pub trait VctrsListOf: VctrsClass {
    /// An R expression that evaluates to the prototype.
    ///
    /// This is used in generated R code for `vec_ptype2` and `vec_cast`.
    /// Common values: "integer()", "double()", "character()", etc.
    fn ptype_expr() -> &'static str;

    /// Optional fixed size for list elements.
    ///
    /// If `Some(n)`, all list elements are expected to have exactly `n` elements.
    fn fixed_size() -> Option<i32> {
        None
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vctrs_error_display() {
        assert_eq!(
            VctrsError::NotInitialized.to_string(),
            "vctrs not initialized - call init_vctrs() first"
        );
        assert_eq!(
            VctrsError::NotAvailable {
                name: "obj_is_vector"
            }
            .to_string(),
            "vctrs callable 'obj_is_vector' not found - is vctrs installed and loaded?"
        );
        assert_eq!(
            VctrsError::AlreadyInitialized.to_string(),
            "vctrs already initialized"
        );
        assert_eq!(
            VctrsError::NotMainThread.to_string(),
            "init_vctrs must be called from R's main thread"
        );
    }

    #[test]
    fn test_vctrs_build_error_display() {
        assert_eq!(
            VctrsBuildError::VctrsNotInitialized.to_string(),
            "vctrs not initialized - call init_vctrs() first"
        );
        assert_eq!(
            VctrsBuildError::NotAVector.to_string(),
            "data is not a vector according to vctrs"
        );
        assert_eq!(
            VctrsBuildError::ListRequiresInheritBaseType.to_string(),
            "list data requires inherit_base_type = true (or None for default)"
        );
        assert_eq!(
            VctrsBuildError::FieldLengthMismatch {
                field: "x".to_string(),
                expected: 3,
                actual: 5
            }
            .to_string(),
            "field 'x' has length 5 but expected 3"
        );
        assert_eq!(
            VctrsBuildError::EmptyRecord.to_string(),
            "record must have at least one field"
        );
        assert_eq!(
            VctrsBuildError::DuplicateFieldName {
                name: "x".to_string()
            }
            .to_string(),
            "duplicate field name: 'x'"
        );
        assert_eq!(
            VctrsBuildError::UnnamedFields.to_string(),
            "record fields must be named"
        );
        assert_eq!(
            VctrsBuildError::MissingPtypeOrSize.to_string(),
            "list_of requires at least one of ptype or size"
        );
        assert_eq!(
            VctrsBuildError::InvalidSize { size: -1 }.to_string(),
            "invalid size: -1 (must be non-negative)"
        );
        assert_eq!(
            VctrsBuildError::EmptyClass.to_string(),
            "class vector must not be empty"
        );
    }

    #[test]
    fn test_is_vctrs_initialized_initially_false() {
        // Note: This test may fail if run after init_vctrs() in the same process
        // In a fresh process, vctrs should not be initialized
        // We can't reliably test this without controlling the test environment
    }

    // =========================================================================
    // Phase C: VctrsKind tests
    // =========================================================================

    #[test]
    fn test_vctrs_kind_default() {
        assert_eq!(VctrsKind::default(), VctrsKind::Vctr);
    }

    #[test]
    fn test_vctrs_kind_eq() {
        assert_eq!(VctrsKind::Vctr, VctrsKind::Vctr);
        assert_eq!(VctrsKind::Rcrd, VctrsKind::Rcrd);
        assert_eq!(VctrsKind::ListOf, VctrsKind::ListOf);
        assert_ne!(VctrsKind::Vctr, VctrsKind::Rcrd);
        assert_ne!(VctrsKind::Rcrd, VctrsKind::ListOf);
    }

    #[test]
    fn test_vctrs_kind_clone() {
        let kind = VctrsKind::Rcrd;
        let cloned = kind;
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_vctrs_kind_debug() {
        assert_eq!(format!("{:?}", VctrsKind::Vctr), "Vctr");
        assert_eq!(format!("{:?}", VctrsKind::Rcrd), "Rcrd");
        assert_eq!(format!("{:?}", VctrsKind::ListOf), "ListOf");
    }

    // =========================================================================
    // Phase C: VctrsClass trait tests (compile-time verification)
    // =========================================================================

    // Test struct implementing VctrsClass
    struct TestPercent;

    impl VctrsClass for TestPercent {
        const CLASS_NAME: &'static str = "test_percent";
        const KIND: VctrsKind = VctrsKind::Vctr;
        const BASE_TYPE: Option<SEXPTYPE> = Some(SEXPTYPE::REALSXP);
        const INHERIT_BASE_TYPE: bool = false;
        const ABBR: Option<&'static str> = Some("pct");
    }

    #[test]
    fn test_vctrs_class_constants() {
        assert_eq!(TestPercent::CLASS_NAME, "test_percent");
        assert_eq!(TestPercent::KIND, VctrsKind::Vctr);
        assert_eq!(TestPercent::BASE_TYPE, Some(SEXPTYPE::REALSXP));
        const { assert!(!TestPercent::INHERIT_BASE_TYPE) };
        assert_eq!(TestPercent::ABBR, Some("pct"));
    }

    #[test]
    fn test_vctrs_class_default_methods() {
        // Test default implementations
        assert!(TestPercent::additional_classes().is_empty());
        let instance = TestPercent;
        assert!(instance.attrs().is_empty());
    }

    // Test struct implementing VctrsRecord
    struct TestRational;

    impl VctrsClass for TestRational {
        const CLASS_NAME: &'static str = "test_rational";
        const KIND: VctrsKind = VctrsKind::Rcrd;
    }

    impl VctrsRecord for TestRational {
        fn field_names() -> &'static [&'static str] {
            &["n", "d"]
        }
    }

    #[test]
    fn test_vctrs_record_trait() {
        assert_eq!(TestRational::CLASS_NAME, "test_rational");
        assert_eq!(TestRational::KIND, VctrsKind::Rcrd);
        assert_eq!(TestRational::field_names(), &["n", "d"]);
    }

    // Test struct implementing VctrsListOf
    struct TestIntList;

    impl VctrsClass for TestIntList {
        const CLASS_NAME: &'static str = "test_int_list";
        const KIND: VctrsKind = VctrsKind::ListOf;
    }

    impl VctrsListOf for TestIntList {
        fn ptype_expr() -> &'static str {
            "integer()"
        }

        fn fixed_size() -> Option<i32> {
            Some(3)
        }
    }

    #[test]
    fn test_vctrs_list_of_trait() {
        assert_eq!(TestIntList::CLASS_NAME, "test_int_list");
        assert_eq!(TestIntList::KIND, VctrsKind::ListOf);
        assert_eq!(TestIntList::ptype_expr(), "integer()");
        assert_eq!(TestIntList::fixed_size(), Some(3));
    }

    // Test VctrsListOf with default fixed_size
    struct TestStringList;

    impl VctrsClass for TestStringList {
        const CLASS_NAME: &'static str = "test_string_list";
        const KIND: VctrsKind = VctrsKind::ListOf;
    }

    impl VctrsListOf for TestStringList {
        fn ptype_expr() -> &'static str {
            "character()"
        }
    }

    #[test]
    fn test_vctrs_list_of_default_size() {
        assert_eq!(TestStringList::fixed_size(), None);
    }

    // Test VctrsClass with custom additional_classes
    struct TestSubPercent;

    impl VctrsClass for TestSubPercent {
        const CLASS_NAME: &'static str = "test_sub_percent";
        const KIND: VctrsKind = VctrsKind::Vctr;

        fn additional_classes() -> &'static [&'static str] {
            &["test_percent"]
        }
    }

    #[test]
    fn test_vctrs_class_with_additional_classes() {
        assert_eq!(TestSubPercent::CLASS_NAME, "test_sub_percent");
        assert_eq!(TestSubPercent::additional_classes(), &["test_percent"]);
    }
}
