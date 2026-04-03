//! Test fixtures for refcount_protect (RefCountedArena).

use miniextendr_api::ffi::SEXP;
use miniextendr_api::prelude::*;
use miniextendr_api::refcount_protect::RefCountedArena;

/// Test Arena protect/is_protected/unprotect cycle.
#[miniextendr]
pub fn refcount_arena_roundtrip() -> bool {
    unsafe {
        let arena = RefCountedArena::new();
        let sexp = SEXP::scalar_integer(42);

        // Protect it
        let protected = arena.protect(sexp);

        // Should be protected
        if !arena.is_protected(protected) {
            return false;
        }

        // Ref count should be 1
        if arena.ref_count(protected) != 1 {
            return false;
        }

        // Protect again — ref count goes to 2
        arena.protect(protected);
        if arena.ref_count(protected) != 2 {
            return false;
        }

        // Unprotect once — ref count goes to 1
        arena.unprotect(protected);
        if arena.ref_count(protected) != 1 {
            return false;
        }

        // Unprotect again — removed
        arena.unprotect(protected);
        !arena.is_protected(protected)
    }
}
