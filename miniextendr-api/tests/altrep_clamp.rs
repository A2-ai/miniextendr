//! Live-SEXP coverage for the ALTREP negative-index clamps (#1190).
//!
//! Several of the hand-written ALTREP bridge macros in
//! `miniextendr-api/src/altrep_impl/macros.rs` defensively clamp
//! caller-supplied indices before using them as `usize` offsets, because R's
//! own C API does **not** bounds-check ALTREP callback arguments the way it
//! does for regular vectors — a negative `i`/`start` reaches our callback
//! as-is. Before this file, nothing exercised those clamps with a live SEXP;
//! `altrep_from_data_matrix.rs` / `altrep_extract.rs` only prove the macros
//! *compile* for a given fixture, and the guard also silently prevents an
//! `as usize` underflow (which would otherwise panic on out-of-bounds
//! indexing) — so a real R session is needed to prove the clamp, not just
//! the type-check, holds.
//!
//! Anchors covered here (verified against `macros.rs` as of this PR):
//! - `:494` (`i.max(0) as usize`) inside `__impl_alt_elt!` — shared by the
//!   integer/real/raw/complex families. Exercised via `Vec<i32>` (builtin
//!   ALTINTEGER fixture from `altrep_impl/builtins.rs`).
//! - `:515` (`if start < 0 || len <= 0 { return 0; }`) inside
//!   `__impl_alt_get_region!` — shared by integer/real/logical/raw/complex.
//!   Exercised via the same `Vec<i32>` fixture.
//! - `:944` (`i.max(0) as usize`) inside `__impl_altlogical_methods!` — the
//!   logical family's own `elt`, separate from the shared macro above
//!   because of the `Logical -> i32` conversion. Exercised via `Vec<bool>`.
//! - `:1100` (`idx = i.max(0) as usize`) inside the cached ALTSTRING `elt`.
//!   Exercised via `Vec<String>`.
//!
//! Not covered: the ALTLIST clamp (`:1186`). No builtin type is registered
//! for the list (`VECSXP`) family in `altrep_impl/builtins.rs` (only
//! integer/real/logical/raw/string/complex have `Vec<T>`/`Box<[T]>`
//! fixtures) — reaching it live would mean hand-rolling a new
//! `AltListData` fixture plus its own SEXP-materializing `elt()` body,
//! which is materially more than reusing an existing registration path.
//! Left for a follow-up if list-family ALTREP gets a builtin fixture of its
//! own for other reasons.
//!
//! All three fixtures below reuse the exact production registration path
//! (`IntoRAltrep::into_sexp_altrep`, i.e. `Altrep(vec).into_sexp()`) that
//! `altrep_thread.rs` already exercises for the (feature-gated) `arrow`
//! fixtures — no new registration machinery is introduced.
//!
//! Why the trait functions are called directly (not through R's own
//! `INTEGER_ELT`/`LOGICAL_ELT`/`STRING_ELT`): as of R's current
//! `Rinlinedfuns.h`, those C-level accessors run `CHECK_VECTOR_*_ELT` /
//! `CHECK_BOUNDS_ELT` (`if (i < 0 || i > XLENGTH(x)) error("subscript out
//! of bounds")`) *before* dispatching to the registered ALTREP `Elt`
//! method — so a negative index raised through them never reaches our
//! macro-generated clamp at all (verified: it raises an uncaught R
//! condition and aborts the test process instead). R's raw
//! `ALTINTEGER_ELT`/`ALTSTRING_ELT`/… dispatch functions (in `altrep.c`)
//! have no such guard, which is exactly why our own clamp exists —
//! calling the bridge trait function directly is the accurate simulation
//! of that unchecked dispatch path, not a shortcut around it.

mod r_test_utils;

use miniextendr_api::IntoRAltrep;
use miniextendr_api::altrep_traits::{AltInteger, AltLogical, AltString};
use miniextendr_api::prelude::SEXP;
use miniextendr_api::sys;

#[test]
fn integer_elt_clamps_negative_index_to_zero() {
    r_test_utils::with_r_thread(|| {
        let sexp: SEXP = vec![10i32, 20, 30].into_sexp_altrep();
        unsafe { sys::Rf_protect(sexp) };

        // Direct bridge-level call — macros.rs:494's `i.max(0)` clamp.
        // Without the clamp, `-1_i32 as usize` underflows to `usize::MAX`
        // and `self[huge_index]` panics.
        assert_eq!(<Vec<i32> as AltInteger>::elt(sexp, -1), 10);
        assert_eq!(<Vec<i32> as AltInteger>::elt(sexp, -100), 10);
        assert_eq!(<Vec<i32> as AltInteger>::elt(sexp, 1), 20);

        unsafe { sys::Rf_unprotect(1) };
    });
}

#[test]
fn integer_get_region_returns_zero_for_negative_start() {
    r_test_utils::with_r_thread(|| {
        let sexp: SEXP = vec![10i32, 20, 30].into_sexp_altrep();
        unsafe { sys::Rf_protect(sexp) };

        // macros.rs:515 -- `if start < 0 || len <= 0 { return 0; }` guard
        // inside `__impl_alt_get_region!`. The negative-start arm.
        let mut buf = [-1i32; 3];
        let written = <Vec<i32> as AltInteger>::get_region(sexp, -1, 3, &mut buf);
        assert_eq!(written, 0);
        assert_eq!(
            buf,
            [-1, -1, -1],
            "buf must be left untouched when the negative-start guard fires"
        );

        // Same guard's `len <= 0` arm.
        let mut buf_zero_len = [-1i32; 3];
        let written_zero_len = <Vec<i32> as AltInteger>::get_region(sexp, 0, 0, &mut buf_zero_len);
        assert_eq!(written_zero_len, 0);

        // Contrast case: a valid, non-negative region still works normally.
        let mut buf_valid = [0i32; 3];
        let written_valid = <Vec<i32> as AltInteger>::get_region(sexp, 0, 3, &mut buf_valid);
        assert_eq!(written_valid, 3);
        assert_eq!(buf_valid, [10, 20, 30]);

        unsafe { sys::Rf_unprotect(1) };
    });
}

#[test]
fn logical_elt_clamps_negative_index_to_zero() {
    r_test_utils::with_r_thread(|| {
        let sexp: SEXP = vec![true, false, true].into_sexp_altrep();
        unsafe { sys::Rf_protect(sexp) };

        // macros.rs:944 -- the logical family's own `i.max(0)` clamp inside
        // `__impl_altlogical_methods!` (kept separate from `__impl_alt_elt!`
        // because of the `Logical -> i32` conversion via `to_r_int()`).
        assert_eq!(<Vec<bool> as AltLogical>::elt(sexp, -1), 1); // TRUE
        assert_eq!(<Vec<bool> as AltLogical>::elt(sexp, -5), 1);
        assert_eq!(<Vec<bool> as AltLogical>::elt(sexp, 1), 0); // FALSE

        unsafe { sys::Rf_unprotect(1) };
    });
}

#[test]
fn string_elt_clamps_negative_index_to_zero() {
    r_test_utils::with_r_thread(|| {
        let sexp: SEXP =
            vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()].into_sexp_altrep();
        unsafe { sys::Rf_protect(sexp) };

        // macros.rs:1100 -- `idx = i.max(0) as usize` inside the cached
        // ALTSTRING `elt` (the per-element CHARSXP materializing cache).
        let charsxp = <Vec<String> as AltString>::elt(sexp, -1);
        let s = unsafe { std::ffi::CStr::from_ptr(sys::R_CHAR(charsxp)) }
            .to_str()
            .unwrap();
        assert_eq!(s, "alpha");

        unsafe { sys::Rf_unprotect(1) };
    });
}
