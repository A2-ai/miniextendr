//! Benchmark fixtures: the R-side preconditions against the Rust conversion,
//! and candidate places a moved check could run (#1661). The scripts in
//! `rpkg/dev/bench-preconditions/` drive them; the results are in
//! `miniextendr-bench/BENCH_RESULTS_2026-09-29-preconditions.md`.
//!
//! Naming: `pcb_<case>` keeps the default R guards, `pcb_<case>_u` is the same
//! function under `no_preconditions`. The `C_pcb_*` functions are raw
//! `extern "C-unwind"` entry points (no guard, no conversion) that run one
//! check in the body, so the raw floor `C_pcb_raw_floor` subtracted from them
//! isolates the check.

use std::ffi::c_int;
use std::hint::black_box;

use miniextendr_api::altrep_sexp::AltrepSexp;
#[cfg(feature = "either")]
use miniextendr_api::either_impl::Either;
use miniextendr_api::{DataFrame, IntoR, List, MatchArg, SEXP, SEXPTYPE, SexpExt, miniextendr};

// The plain C API symbols, called the way R's own `anyNA()` calls them.
// `miniextendr_api::sys` binds neither `*_NO_NA` nor `*_GET_REGION`, and its
// `*_OR_NULL` bindings route through the thread-checked wrappers, which the
// measured scans do not.
unsafe extern "C-unwind" {
    fn REAL_NO_NA(x: SEXP) -> c_int;
    fn INTEGER_NO_NA(x: SEXP) -> c_int;
    fn REAL_GET_REGION(x: SEXP, i: isize, n: isize, buf: *mut f64) -> isize;
    fn INTEGER_GET_REGION(x: SEXP, i: isize, n: isize, buf: *mut c_int) -> isize;
    fn REAL_OR_NULL(x: SEXP) -> *const f64;
    fn INTEGER_OR_NULL(x: SEXP) -> *const c_int;
}

const NA_INT: i32 = i32::MIN;

fn len_i32(n: usize) -> i32 {
    i32::try_from(n).expect("benchmark inputs are shorter than i32::MAX")
}

// region: scalars

/// i32, R guards.
/// @param x Integer scalar.
#[miniextendr(noexport)]
pub fn pcb_i32(x: i32) -> i32 {
    x
}

/// i32, no R guards.
/// @param x Integer scalar.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_i32_u(x: i32) -> i32 {
    x
}

/// f64, R guards.
/// @param x Double scalar.
#[miniextendr(noexport)]
pub fn pcb_f64(x: f64) -> f64 {
    x
}

/// f64, no R guards.
/// @param x Double scalar.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_f64_u(x: f64) -> f64 {
    x
}

/// bool, R guards.
/// @param x Logical scalar.
#[miniextendr(noexport)]
pub fn pcb_bool(x: bool) -> bool {
    x
}

/// bool, no R guards.
/// @param x Logical scalar.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_bool_u(x: bool) -> bool {
    x
}

/// &str, R guards.
/// @param x String scalar.
#[miniextendr(noexport)]
pub fn pcb_str(x: &str) -> i32 {
    len_i32(x.len())
}

/// &str, no R guards.
/// @param x String scalar.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_str_u(x: &str) -> i32 {
    len_i32(x.len())
}

/// String, R guards.
/// @param x String scalar.
#[miniextendr(noexport)]
pub fn pcb_string(x: String) -> i32 {
    len_i32(black_box(x).len())
}

/// String, no R guards.
/// @param x String scalar.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_string_u(x: String) -> i32 {
    len_i32(black_box(x).len())
}

/// Option<i32>, R guards.
/// @param x NULL or integer scalar.
#[miniextendr(noexport)]
pub fn pcb_opt_i32(x: Option<i32>) -> i32 {
    x.unwrap_or(0)
}

/// Option<i32>, no R guards.
/// @param x NULL or integer scalar.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_opt_i32_u(x: Option<i32>) -> i32 {
    x.unwrap_or(0)
}

/// Two params, R guards (reporting order when both are bad).
/// @param a Integer scalar.
/// @param b Double scalar.
#[miniextendr(noexport)]
pub fn pcb_two(a: i32, b: f64) -> f64 {
    f64::from(a) + b
}

/// Two params, no R guards.
/// @param a Integer scalar.
/// @param b Double scalar.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_two_u(a: i32, b: f64) -> f64 {
    f64::from(a) + b
}

// endregion

// region: vectors

/// &[f64], R guards.
/// @param x Double vector.
#[miniextendr(noexport)]
pub fn pcb_slice_f64(x: &[f64]) -> i32 {
    len_i32(black_box(x).len())
}

/// &[f64], no R guards.
/// @param x Double vector.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_slice_f64_u(x: &[f64]) -> i32 {
    len_i32(black_box(x).len())
}

/// Vec<f64>, R guards.
/// @param x Double vector.
#[miniextendr(noexport)]
pub fn pcb_vec_f64(x: Vec<f64>) -> i32 {
    len_i32(black_box(x).len())
}

/// Vec<f64>, no R guards.
/// @param x Double vector.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_vec_f64_u(x: Vec<f64>) -> i32 {
    len_i32(black_box(x).len())
}

/// &[i32], R guards.
/// @param x Integer vector.
#[miniextendr(noexport)]
pub fn pcb_slice_i32(x: &[i32]) -> i32 {
    len_i32(black_box(x).len())
}

/// &[i32], no R guards.
/// @param x Integer vector.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_slice_i32_u(x: &[i32]) -> i32 {
    len_i32(black_box(x).len())
}

/// Vec<i32>, R guards.
/// @param x Integer vector.
#[miniextendr(noexport)]
pub fn pcb_vec_i32(x: Vec<i32>) -> i32 {
    len_i32(black_box(x).len())
}

/// Vec<i32>, no R guards.
/// @param x Integer vector.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_vec_i32_u(x: Vec<i32>) -> i32 {
    len_i32(black_box(x).len())
}

// endregion

// region: no_na

/// no_na &[f64], R guards (is.double + anyNA).
/// @param x Double vector without NA.
#[miniextendr(noexport)]
pub fn pcb_nona_slice_f64(#[miniextendr(no_na)] x: &[f64]) -> i32 {
    len_i32(black_box(x).len())
}

/// no_na &[f64] under no_preconditions (anyNA guard stays).
/// @param x Double vector without NA.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_nona_slice_f64_u(#[miniextendr(no_na)] x: &[f64]) -> i32 {
    len_i32(black_box(x).len())
}

/// no_na &[i32], R guards (is.integer + anyNA).
/// @param x Integer vector without NA.
#[miniextendr(noexport)]
pub fn pcb_nona_slice_i32(#[miniextendr(no_na)] x: &[i32]) -> i32 {
    len_i32(black_box(x).len())
}

/// no_na &[i32] under no_preconditions (anyNA guard stays).
/// @param x Integer vector without NA.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_nona_slice_i32_u(#[miniextendr(no_na)] x: &[i32]) -> i32 {
    len_i32(black_box(x).len())
}

/// Either<&[f64], String>: no R guard, no NA check (baseline for the Rust NA check).
/// @param x Double vector or string.
#[cfg(feature = "either")]
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_either_f64(x: Either<&[f64], String>) -> i32 {
    match black_box(x) {
        Either::Left(v) => len_i32(v.len()),
        Either::Right(s) => len_i32(s.len()),
    }
}

/// Either<&[f64], String> with no_na: the framework's Rust NA check, no R guard.
/// @param x Double vector without NA, or a string.
#[cfg(feature = "either")]
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_either_f64_nona(#[miniextendr(no_na)] x: Either<&[f64], String>) -> i32 {
    match black_box(x) {
        Either::Left(v) => len_i32(v.len()),
        Either::Right(s) => len_i32(s.len()),
    }
}

/// Either<&[i32], String>: no R guard, no NA check.
/// @param x Integer vector or string.
#[cfg(feature = "either")]
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_either_i32(x: Either<&[i32], String>) -> i32 {
    match black_box(x) {
        Either::Left(v) => len_i32(v.len()),
        Either::Right(s) => len_i32(s.len()),
    }
}

/// Either<&[i32], String> with no_na: the framework's Rust NA check.
/// @param x Integer vector without NA, or a string.
#[cfg(feature = "either")]
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_either_i32_nona(#[miniextendr(no_na)] x: Either<&[i32], String>) -> i32 {
    match black_box(x) {
        Either::Left(v) => len_i32(v.len()),
        Either::Right(s) => len_i32(s.len()),
    }
}

// endregion

// region: bare-SEXP floor and check-location candidates
//
// A `#[miniextendr]` `SEXP` parameter is materialised by its conversion
// (`TryFromSexp for SEXP` calls `ensure_materialized`), and so is every native
// slice / Vec conversion (`DATAPTR_RO`). The `C_pcb_*` functions are raw
// `extern "C-unwind"` entry points (no conversion at all): the place a check
// "in the C wrapper before conversion" would run. `AltrepSexp` keeps an ALTREP
// argument compact, for the R guard on ALTREP input.

/// Floor: a bare SEXP (materialises ALTREP), no guard.
/// @param x Anything.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_sexp_floor(x: SEXP) -> bool {
    black_box(x);
    false
}

/// R `!anyNA(x)` guard on a bare SEXP.
/// @param x Anything.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_sexp_rguard_nona(#[miniextendr(no_na)] x: SEXP) -> bool {
    black_box(x);
    false
}

/// R `inherits(x, "mx_cls")` guard on a bare SEXP.
/// @param x Anything.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_sexp_rguard_inherits(#[miniextendr(inherits = "mx_cls")] x: SEXP) -> bool {
    black_box(x);
    false
}

/// Floor for ALTREP input: `AltrepSexp` does not materialise.
/// @param x An ALTREP vector.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_altrep_floor(x: AltrepSexp) -> bool {
    black_box(&x);
    false
}

/// R `!anyNA(x)` guard on ALTREP input, no materialisation after it.
/// @param x An ALTREP vector.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_altrep_rguard_nona(#[miniextendr(no_na)] x: AltrepSexp) -> bool {
    black_box(&x);
    false
}

/// Raw floor: `.Call` into Rust, nothing else.
/// @param x Anything.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_raw_floor(x: SEXP) -> SEXP {
    black_box(x);
    false.into_sexp()
}

/// Raw: NA scan through the data pointer (DATAPTR_RO: expands a compact sequence).
/// @param x Double or integer vector.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_na_slice(x: SEXP) -> SEXP {
    let r = match x.type_of() {
        SEXPTYPE::REALSXP => unsafe { x.as_slice::<f64>() }.iter().any(|v| v.is_nan()),
        SEXPTYPE::INTSXP => unsafe { x.as_slice::<i32>() }.contains(&NA_INT),
        _ => false,
    };
    r.into_sexp()
}

/// Raw: NA scan as `from_r::any_na` does it today (`*_ELT` per element on
/// ALTREP, the data pointer otherwise, no `*_NO_NA` shortcut).
/// @param x Double or integer vector.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_na_framework(x: SEXP) -> SEXP {
    let n = x.xlength();
    let altrep = x.is_altrep();
    let r = match x.type_of() {
        SEXPTYPE::REALSXP if altrep => (0..n).any(|i| x.real_elt(i).is_nan()),
        SEXPTYPE::INTSXP if altrep => (0..n).any(|i| x.integer_elt(i) == NA_INT),
        SEXPTYPE::REALSXP => unsafe { x.as_slice::<f64>() }.iter().any(|v| v.is_nan()),
        SEXPTYPE::INTSXP => unsafe { x.as_slice::<i32>() }.contains(&NA_INT),
        _ => false,
    };
    r.into_sexp()
}

/// ALTREP-aware NA scan through R's C API, as `anyNA()` does it: `*_NO_NA`
/// shortcut, then the `*_OR_NULL` data pointer (never expands), else
/// `*_GET_REGION` in 512-element chunks.
fn na_capi(x: SEXP) -> bool {
    let n = x.xlength();
    unsafe {
        match x.type_of() {
            SEXPTYPE::REALSXP => {
                if REAL_NO_NA(x) != 0 {
                    return false;
                }
                let p = REAL_OR_NULL(x);
                if !p.is_null() {
                    let s = std::slice::from_raw_parts(p, usize::try_from(n).unwrap_or(0));
                    return s.iter().any(|v| v.is_nan());
                }
                let mut buf = [0f64; 512];
                let mut i = 0isize;
                while i < n {
                    let got = REAL_GET_REGION(x, i, (n - i).min(512), buf.as_mut_ptr());
                    let got_u = usize::try_from(got).unwrap_or(0);
                    if buf[..got_u].iter().any(|v| v.is_nan()) {
                        return true;
                    }
                    i += got;
                }
                false
            }
            SEXPTYPE::INTSXP => {
                if INTEGER_NO_NA(x) != 0 {
                    return false;
                }
                let p = INTEGER_OR_NULL(x);
                if !p.is_null() {
                    let s = std::slice::from_raw_parts(p, usize::try_from(n).unwrap_or(0));
                    return s.contains(&NA_INT);
                }
                let mut buf: [c_int; 512] = [0; 512];
                let mut i = 0isize;
                while i < n {
                    let got = INTEGER_GET_REGION(x, i, (n - i).min(512), buf.as_mut_ptr());
                    let got_u = usize::try_from(got).unwrap_or(0);
                    if buf[..got_u].contains(&NA_INT) {
                        return true;
                    }
                    i += got;
                }
                false
            }
            _ => false,
        }
    }
}

/// Raw: ALTREP-aware C-API NA scan.
/// @param x Double or integer vector.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_na_capi(x: SEXP) -> SEXP {
    na_capi(x).into_sexp()
}

/// Raw: whole-number scan through the data pointer (TRUE when every element
/// is NA or whole), no allocation.
/// @param x Double vector.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_whole_slice(x: SEXP) -> SEXP {
    let r = match x.type_of() {
        SEXPTYPE::REALSXP => unsafe { x.as_slice::<f64>() }
            .iter()
            .all(|v| v.is_nan() || v.fract() == 0.0),
        _ => true,
    };
    r.into_sexp()
}

/// Raw: ALTREP-aware whole-number scan (`REAL_OR_NULL`, else `REAL_GET_REGION`).
/// @param x Double vector.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_whole_capi(x: SEXP) -> SEXP {
    if x.type_of() != SEXPTYPE::REALSXP {
        return true.into_sexp();
    }
    let n = x.xlength();
    let whole = |v: &f64| v.is_nan() || v.fract() == 0.0;
    let r = unsafe {
        let p = REAL_OR_NULL(x);
        if !p.is_null() {
            let s = std::slice::from_raw_parts(p, usize::try_from(n).unwrap_or(0));
            s.iter().all(whole)
        } else {
            let mut buf = [0f64; 512];
            let mut i = 0isize;
            let mut ok = true;
            while i < n {
                let got = REAL_GET_REGION(x, i, (n - i).min(512), buf.as_mut_ptr());
                let got_u = usize::try_from(got).unwrap_or(0);
                if !buf[..got_u].iter().all(whole) {
                    ok = false;
                    break;
                }
                i += got;
            }
            ok
        }
    };
    r.into_sexp()
}

/// Branch-free per chunk: any NaN in 64-element chunks (vectorises), early
/// exit between chunks.
fn chunked_any_nan(s: &[f64]) -> bool {
    let (chunks, rest) = s.as_chunks::<64>();
    for c in chunks {
        if c.iter().fold(false, |a, v| a | v.is_nan()) {
            return true;
        }
    }
    rest.iter().any(|v| v.is_nan())
}

/// Branch-free per chunk: any `NA_integer_` in 64-element chunks.
fn chunked_any_na_int(s: &[i32]) -> bool {
    let (chunks, rest) = s.as_chunks::<64>();
    for c in chunks {
        if c.iter().fold(false, |a, v| a | (*v == NA_INT)) {
            return true;
        }
    }
    rest.contains(&NA_INT)
}

/// Branch-free per chunk: every element NA/NaN or whole.
fn chunked_all_whole(s: &[f64]) -> bool {
    let whole = |v: &f64| v.is_nan() | (v.trunc() == *v);
    let (chunks, rest) = s.as_chunks::<64>();
    for c in chunks {
        if !c.iter().fold(true, |a, v| a & whole(v)) {
            return false;
        }
    }
    rest.iter().all(whole)
}

/// Raw: ALTREP-aware C-API NA scan with the chunked (vectorised) inner loop:
/// `*_NO_NA` shortcut, `*_OR_NULL` data pointer, else `*_GET_REGION`.
/// @param x Double or integer vector.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_na_chunked(x: SEXP) -> SEXP {
    let n = x.xlength();
    let r = unsafe {
        match x.type_of() {
            SEXPTYPE::REALSXP => {
                if REAL_NO_NA(x) != 0 {
                    false
                } else {
                    let p = REAL_OR_NULL(x);
                    if !p.is_null() {
                        chunked_any_nan(std::slice::from_raw_parts(
                            p,
                            usize::try_from(n).unwrap_or(0),
                        ))
                    } else {
                        let mut buf = [0f64; 512];
                        let mut i = 0isize;
                        let mut found = false;
                        while i < n && !found {
                            let got = REAL_GET_REGION(x, i, (n - i).min(512), buf.as_mut_ptr());
                            found = chunked_any_nan(&buf[..usize::try_from(got).unwrap_or(0)]);
                            i += got;
                        }
                        found
                    }
                }
            }
            SEXPTYPE::INTSXP => {
                if INTEGER_NO_NA(x) != 0 {
                    false
                } else {
                    let p = INTEGER_OR_NULL(x);
                    if !p.is_null() {
                        chunked_any_na_int(std::slice::from_raw_parts(
                            p,
                            usize::try_from(n).unwrap_or(0),
                        ))
                    } else {
                        let mut buf: [c_int; 512] = [0; 512];
                        let mut i = 0isize;
                        let mut found = false;
                        while i < n && !found {
                            let got = INTEGER_GET_REGION(x, i, (n - i).min(512), buf.as_mut_ptr());
                            found = chunked_any_na_int(&buf[..usize::try_from(got).unwrap_or(0)]);
                            i += got;
                        }
                        found
                    }
                }
            }
            _ => false,
        }
    };
    r.into_sexp()
}

/// Raw: ALTREP-aware whole-number scan with the chunked (vectorised) inner loop.
/// @param x Double vector.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_whole_chunked(x: SEXP) -> SEXP {
    if x.type_of() != SEXPTYPE::REALSXP {
        return true.into_sexp();
    }
    let n = x.xlength();
    let r = unsafe {
        let p = REAL_OR_NULL(x);
        if !p.is_null() {
            chunked_all_whole(std::slice::from_raw_parts(
                p,
                usize::try_from(n).unwrap_or(0),
            ))
        } else {
            let mut buf = [0f64; 512];
            let mut i = 0isize;
            let mut ok = true;
            while i < n && ok {
                let got = REAL_GET_REGION(x, i, (n - i).min(512), buf.as_mut_ptr());
                ok = chunked_all_whole(&buf[..usize::try_from(got).unwrap_or(0)]);
                i += got;
            }
            ok
        }
    };
    r.into_sexp()
}

/// Raw: `TYPEOF` + `XLENGTH` check, the C-API form of `is.double(x) && length(x) == 1L`.
/// @param x Anything.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_type_len(x: SEXP) -> SEXP {
    (x.type_of() == SEXPTYPE::REALSXP && x.xlength() == 1).into_sexp()
}

/// Raw: Rust class check (`Rf_inherits`).
/// @param x Anything.
#[miniextendr(noexport)]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_pcb_inherits(x: SEXP) -> SEXP {
    x.inherits_class(c"mx_cls").into_sexp()
}

/// Rust class check for an arbitrary class name (for semantics comparison).
/// @param x Anything.
/// @param cls Class name.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_rust_inherits(x: SEXP, cls: &str) -> bool {
    let c = std::ffi::CString::new(cls).expect("class has no NUL");
    x.inherits_class(&c)
}

// endregion

// region: coerce Vec<i32> from doubles

/// coerce Vec<i32>: R whole-number guard, then the Rust element check.
/// @param x Integer or whole-number double vector.
#[miniextendr(noexport, coerce)]
pub fn pcb_coerce_vec_i32(x: Vec<i32>) -> i32 {
    len_i32(black_box(x).len())
}

/// coerce Vec<i32> under no_preconditions: the Rust element check only.
/// @param x Integer or whole-number double vector.
#[miniextendr(noexport, coerce, no_preconditions)]
pub fn pcb_coerce_vec_i32_u(x: Vec<i32>) -> i32 {
    len_i32(black_box(x).len())
}

// endregion

// region: inherits / data.frame

/// inherits + List, R guards (inherits + is.list).
/// @param x A list of class mx_cls.
#[miniextendr(noexport)]
pub fn pcb_inherits_list(#[miniextendr(inherits = "mx_cls")] x: List) -> i32 {
    len_i32(usize::try_from(black_box(x).len()).unwrap_or(0))
}

/// inherits + List under no_preconditions (inherits guard stays).
/// @param x A list of class mx_cls.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_inherits_list_u(#[miniextendr(inherits = "mx_cls")] x: List) -> i32 {
    len_i32(usize::try_from(black_box(x).len()).unwrap_or(0))
}

/// List, no guards at all (baseline).
/// @param x A list.
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_list_u(x: List) -> i32 {
    len_i32(usize::try_from(black_box(x).len()).unwrap_or(0))
}

/// List, default guards (is.list only).
/// @param x A list.
#[miniextendr(noexport)]
pub fn pcb_list(x: List) -> i32 {
    len_i32(usize::try_from(black_box(x).len()).unwrap_or(0))
}

/// DataFrame, default (no R guard exists for this type).
/// @param x A data frame.
#[miniextendr(noexport)]
pub fn pcb_df(x: DataFrame) -> i32 {
    len_i32(black_box(x).nrow())
}

/// DataFrame with an R `inherits(x, "data.frame")` guard.
/// @param x A data frame.
#[miniextendr(noexport)]
pub fn pcb_df_inherits(#[miniextendr(inherits = "data.frame")] x: DataFrame) -> i32 {
    len_i32(black_box(x).nrow())
}

/// DataFrame with no_na: R `anyNA()` on a data frame (dispatches anyNA.data.frame).
/// @param x A data frame without NA cells.
#[miniextendr(noexport)]
pub fn pcb_df_nona(#[miniextendr(no_na)] x: DataFrame) -> i32 {
    len_i32(black_box(x).nrow())
}

// endregion

// region: match_arg / choices

/// Choice enum for the match_arg fixtures.
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
#[match_arg(rename_all = "snake_case")]
pub enum PcbMode {
    /// fast
    Fast,
    /// slow
    Slow,
    /// debug
    Debug,
}

/// choices on &str: R `.miniextendr_match_arg` helper.
/// @param mode One of "fast", "slow", "debug".
#[miniextendr(noexport)]
pub fn pcb_choices(#[miniextendr(choices("fast", "slow", "debug"))] mode: &str) -> i32 {
    len_i32(mode.len())
}

/// choices on &str under no_preconditions (the helper stays).
/// @param mode One of "fast", "slow", "debug".
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_choices_u(#[miniextendr(choices("fast", "slow", "debug"))] mode: &str) -> i32 {
    len_i32(mode.len())
}

/// match_arg on an enum: R helper, then Rust `match_choice`.
/// @param mode One of "fast", "slow", "debug".
#[miniextendr(noexport)]
pub fn pcb_match_arg(#[miniextendr(match_arg)] mode: PcbMode) -> i32 {
    match mode {
        PcbMode::Fast => 1,
        PcbMode::Slow => 2,
        PcbMode::Debug => 3,
    }
}

/// The enum without match_arg: Rust matching only (exact or unique prefix).
/// @param mode One of "fast", "slow", "debug".
#[miniextendr(noexport, no_preconditions)]
pub fn pcb_mode_rust(mode: PcbMode) -> i32 {
    match mode {
        PcbMode::Fast => 1,
        PcbMode::Slow => 2,
        PcbMode::Debug => 3,
    }
}

// endregion
