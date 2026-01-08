//! Trait ABI (mx_erased) benchmarks.
//!
//! Planned groups:
//! - `query_vtable`: `mx_base_vtable.query` hit and miss paths
//! - `view_construct`: compute data pointer + `TraitView::from_raw_parts`
//! - `dispatch`: method calls via `<Trait>View` vs direct concrete calls
//! - `end_to_end`: query + view + dispatch (represents downcast call sites)
//!
//! Parameters:
//! - tiny vs medium traits (1 method vs several)
//! - `&self` vs `&mut self` methods
//! - repeated dispatch on the same erased object (cache-hot behavior)
