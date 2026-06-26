//! Soundness guard for the R-borrowing view types (`StrVec<'_>`, `RCow<'_, T>`).
//!
//! These views hand out borrows into R-owned vector/CHARSXP data. Every such
//! borrow is leashed to the view's `'a` — the window in which R keeps the source
//! SEXP reachable (`.Call` protects arguments for the call's duration). A borrow
//! that escaped to `'static` (stored in an `ExternalPtr`, a global, or sent to
//! another thread) would be a use-after-GC once R collects the source. This file
//! must FAIL to compile: if it ever builds, the leash is broken — most likely an
//! accessor's `&'a` return type was reverted to `&'static`, or a view's
//! `TryFromSexp` impl was re-pinned to `'static`. See the static-str soundness work.

use miniextendr_api::{RCow, StrVec};

// The original use-after-GC PoC, now a compile error: a `&str` extracted from a
// leashed `StrVec<'_>` view cannot be stowed in a `'static` field.
struct StrKeeper {
    s: &'static str,
}

fn keep_extracted_str(v: StrVec<'_>) -> StrKeeper {
    StrKeeper {
        s: v.get_str(0).unwrap(),
    }
}

// The whole borrowed `RCow<'_, T>` view is leashed the same way: it cannot be
// stored past the call that produced it.
struct CowKeeper {
    c: RCow<'static, i32>,
}

fn keep_rcow(c: RCow<'_, i32>) -> CowKeeper {
    CowKeeper { c }
}

fn main() {}
