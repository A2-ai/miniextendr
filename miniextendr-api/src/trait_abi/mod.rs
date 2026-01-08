//! # Trait ABI Runtime Support
//!
//! This module provides runtime support for cross-package trait dispatch.
//! It bridges between R's external pointer system and Rust's trait objects
//! using a stable C ABI.
//!
//! ## Overview
//!
//! The trait ABI system enables:
//!
//! 1. **Cross-package dispatch**: Package A can call trait methods on objects
//!    created by Package B, without compile-time knowledge of the concrete type.
//!
//! 2. **Type safety**: Runtime type checking via [`mx_tag`] ensures safe downcasts.
//!
//! 3. **Memory safety**: R's garbage collector manages object lifetime via
//!    external pointer finalizers.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
//! │ R Code          │     │ C-callables      │     │ Rust Runtime    │
//! │                 │     │ (rpkg)           │     │ (miniextendr)   │
//! │ .Call("method", │────►│ mx_query()       │────►│ vtable lookup   │
//! │       obj, ...) │     │ mx_wrap()        │     │ method shim     │
//! │                 │◄────│ mx_get()         │◄────│ type conversion │
//! └─────────────────┘     └──────────────────┘     └─────────────────┘
//! ```
//!
//! ## Submodules
//!
//! - [`ccall`]: C-callable function pointers loaded via `R_GetCCallable`
//! - [`conv`]: Type conversion helpers for method shims
//!
//! ## Integration with ExternalPtr / TypedExternal
//!
//! Trait ABI support is integrated with the [`ExternalPtr`] and [`TypedExternal`]
//! system. `ExternalPtr<T>` serves as the "traitless" case (equivalent to `Any`
//! in dynamic typing). To enable trait dispatch wrappers, list trait impls in
//! `miniextendr_module!`:
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! struct Circle { radius: f64 }
//!
//! #[miniextendr]
//! impl Shape for Circle { /* ... */ }
//!
//! miniextendr_module! {
//!     mod shapes;
//!     impl Shape for Circle;
//! }
//! ```
//!
//! This generates the wrapper structures, vtable references, and query
//! implementations for cross-package trait dispatch.
//!
//! ## Exporting traits you don't own
//!
//! You cannot apply `#[miniextendr]` to external traits. Instead, define a
//! **local adapter trait** that exposes the subset you want in R, then provide
//! a blanket impl for any type that implements the external trait:
//!
//! ```ignore
//! use num_traits::Num;
//!
//! #[miniextendr]
//! pub trait RNum {
//!     fn add(&self, other: &Self) -> Self;
//!     fn to_string(&self) -> String;
//! }
//!
//! impl<T: Num + Clone + ToString> RNum for T {
//!     fn add(&self, other: &Self) -> Self { self.clone() + other.clone() }
//!     fn to_string(&self) -> String { ToString::to_string(self) }
//! }
//! ```
//!
//! This keeps the ABI stable while avoiding generics in the trait itself.
//!
//! ## Initialization
//!
//! Packages using the trait ABI must call [`init_ccallables`] during
//! `R_init_<pkg>`:
//!
//! ```ignore
//! #[unsafe(no_mangle)]
//! pub extern "C" fn R_init_mypackage(info: *mut DllInfo) {
//!     miniextendr_worker_init();
//!     miniextendr_api::trait_abi::init_ccallables();  // Required!
//!     // ... register routines ...
//! }
//! ```
//!
//! ## Thread Safety
//!
//! All trait ABI operations are **main-thread only**:
//!
//! - R invokes `.Call` on the main thread
//! - Method shims do not route through `with_r_thread`
//! - C-callables must be loaded from main thread (`R_init_*`)
//!
//! ## Example Usage
//!
//! ### Defining a trait (provider package)
//!
//! ```ignore
//! // In package "shapes"
//! #[miniextendr]
//! pub trait Shape {
//!     fn area(&self) -> f64;
//!     fn perimeter(&self) -> f64;
//! }
//!
//! #[derive(ExternalPtr)]
//! pub struct Circle { radius: f64 }
//!
//! #[miniextendr]
//! impl Shape for Circle {
//!     fn area(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }
//!     fn perimeter(&self) -> f64 { 2.0 * std::f64::consts::PI * self.radius }
//! }
//!
//! miniextendr_module! {
//!     mod shapes;
//!     impl Shape for Circle;
//! }
//! ```
//!
//! ### Using across packages (consumer package)
//!
//! ```ignore
//! // In package "geometry" (depends on "shapes")
//! use shapes::{TAG_SHAPE, ShapeView};
//!
//! fn calculate_area(obj: SEXP) -> f64 {
//!     unsafe {
//!         let view = mx_query_as::<ShapeView>(obj, TAG_SHAPE)
//!             .expect("object does not implement Shape");
//!         // Call method through vtable
//!         view.area()
//!     }
//! }
//! ```
//!
//! [`ccall`]: crate::trait_abi::ccall
//! [`conv`]: crate::trait_abi::conv
//! [`init_ccallables`]: crate::trait_abi::init_ccallables
//! [`mx_tag`]: crate::abi::mx_tag
//! [`ExternalPtr`]: crate::externalptr::ExternalPtr
//! [`TypedExternal`]: crate::externalptr::TypedExternal

pub mod ccall;
pub mod conv;

// Re-export commonly used items
pub use ccall::init_ccallables;
pub use conv::{check_arity, extract_arg, from_sexp, nil, rf_error, to_sexp, try_from_sexp};

/// Initialize C-callables from C code.
///
/// This is a C-callable wrapper around [`init_ccallables`] for use from
/// `R_init_<pkg>` in entrypoint.c.
///
/// # Safety
///
/// Must be called from R's main thread during package initialization.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_init_ccallables() {
    init_ccallables();
}

use crate::abi::mx_tag;
use crate::ffi::SEXP;
use std::os::raw::c_void;

// =============================================================================
// TraitView - Trait for macro-generated View structs
// =============================================================================

/// Trait for view types that can be created from SEXP via trait dispatch.
///
/// This trait is implemented by the macro-generated `<Trait>View` structs.
/// It provides a common interface for:
/// - Querying whether an object implements a trait
/// - Creating a view from an SEXP
///
/// # Generated by `#[miniextendr]` on traits
///
/// When you write:
/// ```ignore
/// #[miniextendr]
/// pub trait Counter {
///     fn value(&self) -> i32;
///     fn increment(&mut self);
/// }
/// ```
///
/// The macro generates `CounterView` that implements `TraitView`:
/// ```ignore
/// impl TraitView for CounterView {
///     const TAG: mx_tag = TAG_COUNTER;
///
///     unsafe fn from_raw_parts(data: *mut c_void, vtable: *const c_void) -> Self {
///         Self {
///             data,
///             vtable: vtable as *const CounterVTable,
///         }
///     }
/// }
/// ```
///
/// # Usage
///
/// ```ignore
/// // Try to get a Counter view from an R object
/// let view = CounterView::try_from_sexp(obj)?;
///
/// // Call methods through the view
/// view.increment();
/// let val = view.value();
/// ```
///
/// # Safety
///
/// The `from_raw_parts` method is unsafe because:
/// - `data` must be a valid pointer to the concrete object
/// - `vtable` must be a valid pointer to the trait's vtable
/// - The pointers must remain valid for the lifetime of the view
pub trait TraitView: Sized {
    /// The type tag for this trait.
    ///
    /// This is a compile-time constant generated by `#[miniextendr]` on the trait.
    const TAG: mx_tag;

    /// Create a view from raw data and vtable pointers.
    ///
    /// # Safety
    ///
    /// - `data` must be a valid, non-null pointer to the concrete object
    /// - `vtable` must be a valid, non-null pointer to the trait's vtable
    /// - Both pointers must remain valid for the lifetime of the view
    unsafe fn from_raw_parts(data: *mut c_void, vtable: *const c_void) -> Self;

    /// Try to create a view from an R SEXP.
    ///
    /// Queries the object for this trait's vtable using `mx_query`. If the
    /// object implements the trait, returns `Some(view)`. Otherwise returns `None`.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid R external pointer (EXTPTRSXP)
    /// - Must be called on R's main thread
    /// - Must call `init_ccallables()` first
    ///
    /// # Returns
    ///
    /// - `Some(Self)` if the object implements the trait
    /// - `None` if the object does not implement the trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// let counter = unsafe { CounterView::try_from_sexp(obj) }
    ///     .expect("Object does not implement Counter");
    /// ```
    #[inline]
    unsafe fn try_from_sexp(sexp: SEXP) -> Option<Self> {
        // SAFETY: Caller guarantees sexp is valid and we're on main thread
        unsafe {
            // Get the vtable for this trait
            let vtable = ccall::mx_query(sexp, Self::TAG);
            if vtable.is_null() {
                return None;
            }

            // Get the erased pointer (points to the wrapper struct header)
            let erased_ptr = crate::ffi::R_ExternalPtrAddr(sexp);
            if erased_ptr.is_null() {
                return None;
            }

            // The wrapper struct layout is:
            //   struct __MxWrapper<T> {
            //       erased: mx_erased,  // offset 0
            //       data: T,            // offset = sizeof(mx_erased)
            //   }
            // We need to calculate the data pointer by adding the erased header size.
            // mx_erased is just a single pointer (*const mx_base_vtable).
            let data_offset = std::mem::size_of::<crate::abi::mx_erased>();
            let data = (erased_ptr as *mut u8).add(data_offset) as *mut c_void;

            Some(Self::from_raw_parts(data, vtable))
        }
    }

    /// Try to create a view, returning an error message on failure.
    ///
    /// Similar to `try_from_sexp` but returns an error string suitable for
    /// use with `r_stop` if the object does not implement the trait.
    ///
    /// # Safety
    ///
    /// Same as `try_from_sexp`.
    #[inline]
    unsafe fn try_from_sexp_or_error(sexp: SEXP, trait_name: &str) -> Result<Self, String> {
        // SAFETY: Delegated to try_from_sexp
        unsafe {
            Self::try_from_sexp(sexp)
                .ok_or_else(|| format!("Object does not implement {} trait", trait_name))
        }
    }
}
