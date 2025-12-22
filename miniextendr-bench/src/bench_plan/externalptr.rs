//! ExternalPtr benchmarks.
//!
//! Planned groups:
//! - `create` (ExternalPtr::new) vs from_raw
//! - `access` (as_ref/as_mut) and pointer checks
//! - `tag_lookup` (tag, stored_type_id, type comparisons)
//! - `set_protected` (user-protected slot updates)
//! - `try_from_sexp` success/failure paths
//! - `into_raw` and reclaim cost
//!
//! Parameters:
//! - small vs large payload types (e.g., i32 vs `Vec<i32>`)
//! - type-erased vs typed external pointers
