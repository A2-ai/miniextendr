//! GC protection benchmarks.
//!
//! Implemented groups:
//! - `protect_scope`: ProtectScope vs raw Rf_protect/Rf_unprotect
//! - `owned_protect`: OwnedProtect vs ProtectScope::protect
//! - `reprotect`: ReprotectSlot::set vs re-protect patterns
//! - `list_builders`: List::set_elt, set_elt_unchecked, ListBuilder, ListAccumulator, collect_list
//! - `strvec_builders`: StrVec::set_elt, StrVecBuilder, collect patterns
//! - `named_list`: ListBuilder with names vs manual allocation
//! - `preserve`: preserve::insert/release vs R_PreserveObject/R_ReleaseObject
//!
//! Comprehensive coverage of all GC protection and builder APIs.
