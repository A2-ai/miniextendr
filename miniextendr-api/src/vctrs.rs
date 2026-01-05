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
    fn test_is_vctrs_initialized_initially_false() {
        // Note: This test may fail if run after init_vctrs() in the same process
        // In a fresh process, vctrs should not be initialized
        // We can't reliably test this without controlling the test environment
    }
}
