//! Trait ABI (mx_erased) benchmarks.
//!
//! Implemented groups:
//! - `query_vtable`: hit path + miss path
//! - `view_construct`: implicit in view_value_only, query_view_value
//! - `dispatch`: &self (value) vs &mut self (increment), repeated-hot (10x)
//! - `end_to_end`: query + view + call (query_view_value)
//! - `baseline`: direct concrete calls for comparison
//!
//! Remaining gap:
//! - Multi-method trait variant (current trait has only 2 methods)
