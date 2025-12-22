//! ALTREP benchmarks.
//!
//! Focus on ALTREP class behavior and callback costs for each vector type.
//!
//! Planned groups:
//! 1) `elt_access`
//!    - ALTINTEGER / ALTREAL / ALTLOGICAL / ALTRAW / ALTSTRING / ALTLIST
//!    - Compare elt() vs get_region() vs dataptr() where applicable
//!
//! 2) `get_region`
//!    - varying region sizes: 1, 8, 64, 1024
//!    - contiguous vs random access patterns
//!
//! 3) `summary_methods`
//!    - sum/min/max (numeric), no_na, is_sorted
//!    - compare ALTREP overrides vs materialized vectors
//!
//! 4) `duplicate`
//!    - shallow vs deep duplicate cost
//!
//! 5) `coerce`
//!    - ALTREP Coerce method vs R's default coercion
//!
//! 6) `dataptr_or_null`
//!    - cost and behavior for lazy vs materialized data
//!
//! Parameters:
//! - size matrix, NA density matrix
//! - materialized vs non-materialized ALTREP
