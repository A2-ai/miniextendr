//! FFI call overhead benchmarks.
//!
//! Measure the cost of calling R C-API functions through:
//! - checked wrappers (thread assertions)
//! - unchecked wrappers (no assertions)
//! - direct raw FFI functions (where available)
//!
//! Planned benchmark groups:
//!
//! 1) `alloc_vector`
//!    - `Rf_allocVector` vs `Rf_allocVector_unchecked`
//!    - Types: INTSXP, REALSXP, LGLSXP, STRSXP
//!    - Sizes: tiny -> large
//!
//! 2) `scalar_creation`
//!    - `Rf_ScalarInteger`, `Rf_ScalarReal`, `Rf_ScalarLogical`
//!    - Checked vs unchecked
//!
//! 3) `data_access`
//!    - `INTEGER`, `REAL`, `LOGICAL`, `RAW`, `DATAPTR_RO`
//!    - Pointer acquisition only (no copy)
//!
//! 4) `protect_unprotect`
//!    - `Rf_protect` / `Rf_unprotect` vs preserve list (see preserve module)
//!    - Measure cost per protect/unprotect pair
//!
//! Metrics:
//! - ns/op for each call
//! - throughput for large vector allocations (alloc/sec)
