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
//! in dynamic typing), while types that implement traits for cross-package
//! dispatch use extended attributes:
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! #[externalptr(traits = [Shape, Display])]
//! struct Circle { radius: f64 }
//! ```
//!
//! This will generate the necessary wrapper structures, vtable references,
//! and query implementations for cross-package trait dispatch.
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
//! #[externalptr(traits = [Shape])]  // Enables trait ABI support
//! pub struct Circle { radius: f64 }
//!
//! #[miniextendr]
//! impl Shape for Circle {
//!     fn area(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }
//!     fn perimeter(&self) -> f64 { 2.0 * std::f64::consts::PI * self.radius }
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
//! [`mx_tag`]: crate::abi::mx_tag
//! [`ExternalPtr`]: crate::externalptr::ExternalPtr
//! [`TypedExternal`]: crate::externalptr::TypedExternal

pub mod ccall;
pub mod conv;

// Re-export commonly used items
pub use ccall::init_ccallables;
pub use conv::{check_arity, extract_arg, from_sexp, nil, rf_error, to_sexp, try_from_sexp};
